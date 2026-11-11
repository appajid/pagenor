//! Atomic replacement for native documents, exports and other user files.
//! Before replacement, failed writes/metadata/flushes preserve previous bytes. Existing links,
//! ownership, permissions, ACLs and bounded extended attributes keep their current semantics.
//! Directory flushes are best effort after commit; this is not a power-loss transaction.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::path::Path;

pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(path, |file| file.write_all(bytes))
}

pub fn replace_with(path: &Path, prepare: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    replace_with_metadata(path, prepare, preserve_metadata)
}

fn replace_with_metadata(
    path: &Path,
    prepare: impl FnOnce(&mut File) -> io::Result<()>,
    metadata: impl FnOnce(&Path, &tempfile::NamedTempFile, Permissions) -> io::Result<()>,
) -> io::Result<()> {
    if path.file_name().is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "document path has no filename"));
    }
    // Follow an existing file link, as fs::write did, without replacing the link itself.
    // In particular, a dangling link must fail rather than become a new document file.
    let target = match fs::symlink_metadata(path) {
        Ok(_) => fs::canonicalize(path)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => path.to_path_buf(),
        Err(e) => return Err(e),
    };
    let permissions = writable_file(&target)?;
    let parent = target.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut builder = tempfile::Builder::new();
    builder.prefix(".pagenor-save-").suffix(".tmp");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Existing documents stage privately until their prior mode is applied. New
        // documents retain fs::write's 0666, restricted by the process umask.
        if permissions.is_none() {
            builder.permissions(Permissions::from_mode(0o666));
        }
    }
    let mut file = builder.tempfile_in(parent)?;
    prepare(file.as_file_mut())?;
    if let Some(permissions) = permissions {
        metadata(&target, &file, permissions)
            .map_err(|e| io::Error::new(e.kind(), format!("could not preserve document file metadata; previous save retained: {e}")))?;
    }
    // A write, permission or sync failure only affects staging; its RAII owner removes it.
    file.as_file().sync_all()?;
    // Recheck writability before replacement. Concurrent hostile directory/entry changes
    // and full power-loss durability require stronger platform-specific guarantees.
    writable_file(&target)?;
    persist(file, &target)?;
    #[cfg(unix)]
    if let Err(e) = File::open(parent).and_then(|dir| dir.sync_all()) {
        // The complete replacement is already visible. Report the durability limitation
        // without pretending a retry is required to publish the new file.
        log::warn!("{} was replaced, but its directory could not be flushed: {e}", target.display());
    }
    Ok(())
}

fn persist(file: tempfile::NamedTempFile, target: &Path) -> io::Result<()> {
    #[cfg(not(windows))]
    {
        file.persist(target).map(|_| ()).map_err(|e| e.error)
    }
    #[cfg(windows)]
    {
        let mut file = file;
        for attempt in 0..5 {
            match file.persist(target) {
                Ok(_) => return Ok(()),
                Err(e) if e.error.kind() == io::ErrorKind::PermissionDenied && attempt < 4 => {
                    file = e.file;
                    std::thread::sleep(std::time::Duration::from_millis(10 << attempt));
                }
                Err(e) => return Err(e.error),
            }
        }
        Err(io::Error::other("atomic replacement retry budget exhausted"))
    }
}

fn preserve_metadata(target: &Path, staging: &tempfile::NamedTempFile, permissions: Permissions) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, fchown};
        let old = fs::metadata(target)?;
        let new = staging.as_file().metadata()?;
        // Only change IDs that differ: needless chown can fail for an unprivileged
        // owner and can clear mode/security bits. Failure must precede replacement.
        let uid = (old.uid() != new.uid()).then_some(old.uid());
        let gid = (old.gid() != new.gid()).then_some(old.gid());
        if uid.is_some() || gid.is_some() {
            fchown(staging.as_file(), uid, gid)?;
        }
    }
    staging.as_file().set_permissions(permissions)?;
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    copy_attributes(target, staging.as_file())?;
    #[cfg(target_os = "macos")]
    {
        // macOS ACLs are separate from xattrs and Unix mode bits. An empty ACL
        // deliberately removes any access rules inherited by the staging file.
        let acl = exacl::getfacl(target, None)?;
        exacl::setfacl(&[staging.path()], &acl, None)?;
    }
    #[cfg(not(unix))]
    let _ = target;
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn attribute_names(file: &File) -> io::Result<Vec<std::ffi::CString>> {
    use rustix::fs::flistxattr;
    let mut list = vec![0; 64 * 1024];
    let count = match flistxattr(file, list.as_mut_slice()) {
        Ok(count) => count,
        // Volumes without xattr support have no such metadata to preserve.
        Err(rustix::io::Errno::NOTSUP) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let bytes = list.get(..count).ok_or_else(|| io::Error::other("invalid extended attribute list length"))?;
    bytes
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| std::ffi::CString::new(name).map_err(|_| io::Error::other("invalid extended attribute name")))
        .collect()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn copy_attributes(target: &Path, staging: &File) -> io::Result<()> {
    use rustix::fs::{XattrFlags, fgetxattr, fremovexattr, fsetxattr};
    // Inspect metadata through a write-capable descriptor without reading document
    // bytes. The OS can independently deny attribute access; then Save is refused.
    let source = OpenOptions::new().write(true).open(target)?;
    let names = attribute_names(&source)?;
    for name in attribute_names(staging)? {
        if !names.contains(&name) {
            fremovexattr(staging, name.as_c_str())?;
        }
    }
    // Copy one value at a time with fixed allocation and a total budget. Large
    // resource forks or unsupported/security attributes refuse Save, never vanish
    // silently or cause an unbounded allocation. Document bytes are not reread.
    let mut value = vec![0; 1024 * 1024];
    let mut remaining: usize = 8 * 1024 * 1024;
    for name in names {
        let count = fgetxattr(&source, name.as_c_str(), value.as_mut_slice())?;
        remaining = remaining.checked_sub(count).ok_or_else(|| io::Error::other("extended document metadata exceeds 8 MiB"))?;
        let bytes = value.get(..count).ok_or_else(|| io::Error::other("invalid extended attribute length"))?;
        fsetxattr(staging, name.as_c_str(), bytes, XattrFlags::empty())?;
    }
    Ok(())
}

fn writable_file(path: &Path) -> io::Result<Option<Permissions>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "document destination is not a regular file"));
            }
            let permissions = metadata.permissions();
            if permissions.readonly() {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "document destination is read-only"));
            }
            // Check file permissions/ACLs without truncating. Close before the Windows rename.
            drop(OpenOptions::new().write(true).open(path)?);
            Ok(Some(permissions))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_failure_keeps_previous_bytes_and_cleans_staging_then_retry_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Metadata failure.pgnr");
        fs::write(&path, b"old complete document").unwrap();
        let result = replace_with_metadata(
            &path,
            |file| file.write_all(b"new complete document"),
            |_, staging, _| {
                assert_eq!(fs::read(staging.path()).unwrap(), b"new complete document");
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "synthetic metadata access denial"))
            },
        );
        let error = result.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("previous save retained"));
        assert_eq!(fs::read(&path).unwrap(), b"old complete document");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        write(&path, b"successful retry").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"successful retry");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn replacement_retains_extended_attributes_and_ownership() {
        use rustix::fs::{XattrFlags, fgetxattr, fsetxattr};
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Tagged తెలుగు.pgnr");
        fs::write(&path, b"previous bytes").unwrap();
        let original = OpenOptions::new().write(true).open(&path).unwrap();
        #[cfg(target_os = "macos")]
        let name = "com.apple.metadata:_kMDItemUserTags";
        #[cfg(target_os = "linux")]
        let name = "user.pagenor.test-tags";
        let tags = br#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><array><string>Pagenor QA</string></array></plist>"#;
        fsetxattr(&original, name, tags, XattrFlags::empty()).unwrap();
        #[cfg(target_os = "macos")]
        let custom = "org.pagenor.qa.binary";
        #[cfg(target_os = "linux")]
        let custom = "user.pagenor.binary";
        fsetxattr(&original, custom, &[0, 255, 1, 0], XattrFlags::empty()).unwrap();
        original.set_permissions(Permissions::from_mode(0o640)).unwrap();
        let before = original.metadata().unwrap();
        drop(original);
        for bytes in [b"first replacement".as_slice(), b"second replacement".as_slice()] {
            write(&path, bytes).unwrap();
            assert_eq!(fs::read(&path).unwrap(), bytes);
            let saved = File::open(&path).unwrap();
            let after = saved.metadata().unwrap();
            assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
            assert_eq!(after.permissions().mode() & 0o777, 0o640);
            let mut buffer = vec![0; 1024];
            let count = fgetxattr(&saved, name, buffer.as_mut_slice()).unwrap();
            assert_eq!(&buffer[..count], tags);
            let count = fgetxattr(&saved, custom, buffer.as_mut_slice()).unwrap();
            assert_eq!(&buffer[..count], &[0, 255, 1, 0]);
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn replacement_preserves_a_different_existing_group() {
        use std::os::unix::fs::{MetadataExt, fchown};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Group.pgnr");
        fs::write(&path, b"old").unwrap();
        // Every macOS user belongs to the OS's everyone group (GID 12).
        // On this host newly staged files use staff (20), so this exercises
        // actual fchown rather than just the unchanged-owner fast path.
        fchown(File::open(&path).unwrap(), None, Some(12)).unwrap();
        write(&path, b"new group-preserving save").unwrap();
        assert_eq!(fs::metadata(&path).unwrap().gid(), 12);
        assert_eq!(fs::read(&path).unwrap(), b"new group-preserving save");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn replacement_retains_macos_extended_acl_and_removes_inherited_rules() {
        use exacl::{AclEntry, Perm};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACL.pgnr");
        fs::write(&path, b"old").unwrap();
        let rule = AclEntry::deny_user("nobody", Perm::READ, None);
        exacl::setfacl(&[&path], &[rule], None).unwrap();
        let expected = exacl::getfacl(&path, None).unwrap();
        assert_eq!(expected.len(), 1);
        write(&path, b"new").unwrap();
        assert_eq!(exacl::getfacl(&path, None).unwrap(), expected);
        // A staged file can have rules inherited from its parent. The source
        // file's empty ACL must clear these rather than broadening access.
        exacl::setfacl(&[&path], &[], None).unwrap();
        replace_with_metadata(
            &path,
            |file| file.write_all(b"no inherited rules"),
            |target, staging, permissions| {
                exacl::setfacl(&[staging.path()], &expected, None)?;
                preserve_metadata(target, staging, permissions)
            },
        )
        .unwrap();
        assert!(exacl::getfacl(&path, None).unwrap().is_empty());
        assert_eq!(fs::read(&path).unwrap(), b"no inherited rules");
    }

    #[test]
    fn partial_write_failure_preserves_the_previous_document_and_allows_retry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Saved తెలుగు.pgnr");
        let mut doc = pagenor_doc::Document::new(&pagenor_doc::build::NewDocument::default());
        let old = pagenor_format::save(&doc).unwrap();
        std::fs::write(&path, &old).unwrap();
        doc.title = "New edit ఉదయం".into();
        let new = pagenor_format::save(&doc).unwrap();
        let result = replace_with(&path, |file| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(file.metadata()?.permissions().mode() & 0o777, 0o600, "replacement staging stays private while writing");
            }
            file.write_all(&new[..32])?;
            Err(io::Error::new(io::ErrorKind::StorageFull, "synthetic full-disk failure after partial write"))
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), old, "failed writes must leave the last saved document intact");
        assert_eq!(pagenor_format::load(&std::fs::read(&path).unwrap()).unwrap().title, "Untitled-1");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1, "failed staging is cleaned up");
        write(&path, &new).unwrap();
        assert_eq!(pagenor_format::load(&std::fs::read(&path).unwrap()).unwrap().title, doc.title);
    }

    #[test]
    fn failed_new_document_write_publishes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("New.pgnr");
        assert!(
            replace_with(&path, |file| {
                file.write_all(b"incomplete archive")?;
                Err(io::Error::new(io::ErrorKind::StorageFull, "synthetic write failure"))
            })
            .is_err()
        );
        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        write(&path, b"complete replacement").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"complete replacement");
    }

    #[cfg(unix)]
    #[test]
    fn save_follows_file_links_and_preserves_existing_modes() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("Saved.pgnr");
        let link = dir.path().join("Linked.pgnr");
        fs::write(&target, b"previous bytes").unwrap();
        fs::set_permissions(&target, Permissions::from_mode(0o640)).unwrap();
        symlink("Saved.pgnr", &link).unwrap();
        write(&link, b"new bytes").unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"new bytes");
        assert_eq!(fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o640);
        let dangling = dir.path().join("Dangling.pgnr");
        symlink("Missing.pgnr", &dangling).unwrap();
        assert!(write(&dangling, b"must not replace link").is_err());
        assert!(fs::symlink_metadata(&dangling).unwrap().file_type().is_symlink());
        assert!(!dir.path().join("Missing.pgnr").exists());
    }
}
