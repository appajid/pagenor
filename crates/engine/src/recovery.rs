//! Crash recovery (InDesign's automatic recovery data): unsaved changes of every open document are
//! written to a recovery folder (`file.recovery.save`, called on a timer by the app); saving or
//! closing a document removes its file; after a crash `file.recovery.open` reopens what was left
//! there as unsaved documents that remember where they were saved.
//!
//! A recovery entry is `<key>.pagenor` (the document) plus `<key>.json` (`{"path", "title",
//! "saved"}`: the original file, the title, and when it was written).
//! Keys are owned by open documents, independently of process-local document UIDs. Reopening
//! keeps the last checkpoint intact until the document is saved, reverted or explicitly closed.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Value, json};

use crate::{DocState, EngineError, Result, Session};

/// Native session state only; never serialized into documents. Canonical directories avoid
/// opening the same checkpoint twice through different spellings of its path.
#[derive(Debug, Default)]
pub(crate) struct Entries(Mutex<BTreeMap<PathBuf, u64>>);

fn io_error(path: &Path, error: impl std::fmt::Display) -> EngineError {
    EngineError::Other(format!("{}: {error}", path.display()))
}

/// The platform's per-user recovery folder.
pub fn default_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        return home.map(|h| h.join("Library/Application Support/Pagenor/Recovery"));
    }
    if cfg!(target_os = "windows") {
        return std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Pagenor").join("Recovery"));
    }
    std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| home.map(|h| h.join(".local/share"))).map(|d| d.join("pagenor/recovery"))
}

fn files(dir: &Path, uid: u64) -> (PathBuf, PathBuf) {
    (dir.join(format!("{uid}.pagenor")), dir.join(format!("{uid}.json")))
}

/// Remove a document's recovery entry (it was saved or closed).
pub fn discard(dir: &Path, uid: u64) {
    remove_entry(dir, uid);
}

fn remove_entry(dir: &Path, uid: u64) -> bool {
    let (d, m) = files(dir, uid);
    if let Err(e) = std::fs::remove_file(&d)
        && e.kind() != io::ErrorKind::NotFound
    {
        log::warn!("could not remove recovery entry {}: {e}", d.display());
        return false;
    }
    let _ = std::fs::remove_file(m);
    true
}

/// Saving/closing must remove only this document's entries, including explicitly supplied
/// recovery directories. A clean document with a reused UID owns no old recovery files.
pub(crate) fn discard_document(d: &DocState) {
    d.recovery_entries.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).retain(|dir, uid| !remove_entry(dir, *uid));
}

fn owns(d: &DocState, dir: &Path, uid: u64) -> bool {
    d.recovery_entries.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(dir) == Some(&uid)
}

fn exists(path: &Path) -> Result<bool> {
    match path.symlink_metadata() {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(io_error(path, e)),
    }
}

/// Reserve a free key atomically. Pending (even corrupt) documents and orphan metadata are
/// never overwritten. Bound collision probes; wrapping treats all u64 filenames as valid keys
/// without advancing or overflowing the runtime UID counter.
fn reserve(dir: &Path, hint: u64) -> Result<u64> {
    for offset in 0..4096 {
        let uid = hint.wrapping_add(offset);
        let (doc, meta) = files(dir, uid);
        if exists(&meta)? {
            continue;
        }
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&doc) {
            Ok(_) => {
                // An entry's metadata could have appeared during the reservation.
                let occupied = exists(&meta);
                if !matches!(occupied, Ok(false)) {
                    let _ = std::fs::remove_file(&doc);
                    occupied?;
                    continue;
                }
                return Ok(uid);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_error(&doc, e)),
        }
    }
    Err(io_error(dir, "no free recovery key within 4096 candidates"))
}

/// Stage and flush without truncating a preexisting temp file (including a stale file after
/// a crash). File handles are closed before rename, also on Windows.
fn stage(dest: &Path, bytes: &[u8]) -> Result<PathBuf> {
    for suffix in 0..64 {
        let tmp = dest.with_extension(format!("{}.tmp-{suffix}", dest.extension().and_then(|e| e.to_str()).unwrap_or("recovery")));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp) {
            Ok(mut f) => {
                let result = f.write_all(bytes).and_then(|_| f.sync_all());
                drop(f);
                if let Err(e) = result {
                    let _ = std::fs::remove_file(&tmp);
                    return Err(io_error(&tmp, e));
                }
                return Ok(tmp);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_error(&tmp, e)),
        }
    }
    Err(io_error(dest, "no free recovery staging file within 64 candidates"))
}

fn checkpoint(dir: &Path, uid: u64, bytes: &[u8], metadata: &[u8]) -> Result<()> {
    let (doc, meta) = files(dir, uid);
    let doc_tmp = stage(&doc, bytes)?;
    let meta_tmp = match stage(&meta, metadata) {
        Ok(path) => path,
        Err(e) => {
            let _ = std::fs::remove_file(doc_tmp);
            return Err(e);
        }
    };
    // The existing document remains valid if staging or metadata replacement fails. Original
    // path is stable between checkpoints; title/timestamp may be newer if the last rename fails.
    let result = std::fs::rename(&meta_tmp, &meta)
        .map_err(|e| io_error(&meta, e))
        .and_then(|_| std::fs::rename(&doc_tmp, &doc).map_err(|e| io_error(&doc, e)));
    let _ = std::fs::remove_file(doc_tmp);
    let _ = std::fs::remove_file(meta_tmp);
    result
}

/// Write recovery entries for the dirty documents (and drop those of clean ones). Returns how
/// many were written.
pub fn save(s: &Session, dir: &Path) -> Result<usize> {
    std::fs::create_dir_all(dir).map_err(|e| EngineError::Other(format!("{}: {e}", dir.display())))?;
    let dir = std::fs::canonicalize(dir).map_err(|e| io_error(dir, e))?;
    let mut n = 0;
    for d in &s.docs {
        if !d.is_dirty() {
            discard_document(d);
            continue;
        }
        let bytes = pagenor_format::save(d.preview_stash.as_deref().unwrap_or(d.doc.as_ref())).map_err(|e| io_error(&dir, e))?;
        let m = json!({"path": d.path, "title": d.title(), "saved": pagenor_doc::vars::now()});
        let mut entries = d.recovery_entries.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let existing = entries.get(&dir).copied();
        let uid = match existing {
            Some(uid) => uid,
            None => reserve(&dir, d.uid)?,
        };
        if let Err(e) = checkpoint(&dir, uid, &bytes, m.to_string().as_bytes()) {
            if existing.is_none() {
                discard(&dir, uid);
            }
            return Err(e);
        }
        entries.insert(dir.clone(), uid);
        n += 1;
    }
    Ok(n)
}

/// Recovery entries in `dir`: (uid, metadata).
pub fn list(dir: &Path) -> Vec<(u64, Value)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("pagenor") {
            continue;
        }
        let Some(uid) = p.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse::<u64>().ok()) else { continue };
        // Metadata is a file on disk: anything but an object (corrupt, hand-edited) counts as none.
        let meta = std::fs::read_to_string(p.with_extension("json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .filter(Value::is_object)
            .unwrap_or(json!({}));
        out.push((uid, meta));
    }
    out.sort_by_key(|e| e.0);
    out
}

/// Reopen every unclaimed recovery entry as an unsaved document, retaining its checkpoint.
/// Returns the opened documents' indices; repeated calls in the same session skip owned entries.
pub fn open(s: &mut Session, dir: &Path) -> Result<Vec<usize>> {
    let mut opened = Vec::new();
    let pending = list(dir);
    if pending.is_empty() {
        return Ok(opened);
    }
    let dir = std::fs::canonicalize(dir).map_err(|e| io_error(dir, e))?;
    for (uid, meta) in pending {
        if s.docs.iter().any(|d| owns(d, &dir, uid)) {
            continue;
        }
        let (doc, _) = files(&dir, uid);
        let Ok(bytes) = std::fs::read(&doc) else { continue };
        let mut d = match crate::cmd::from_bytes(&bytes) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let path = meta.get("path").and_then(Value::as_str).map(str::to_string);
        // Skipped font files are logged; the document opens without them.
        let fonts = path.as_deref().and_then(|p| crate::cmd::load_document_fonts(&mut d, p).0);
        let mut st = DocState::new(d, path);
        st.fonts = fonts;
        // Unsaved: the copy on disk (if any) is older than what was recovered.
        st.saved_doc = std::sync::Arc::new((*st.doc).clone());
        st.revision += 1;
        st.recovery_entries.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(dir.clone(), uid);
        opened.push(s.add_document(st));
    }
    Ok(opened)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pagenor-recovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn dirty_document(s: &mut Session) {
        s.execute("file.new", &json!({})).unwrap();
        s.execute("frame.create", &json!({"rect": [10, 10, 100, 100], "content": "text", "text": "Recovery हिन्दी தமிழ்"})).unwrap();
    }

    #[test]
    fn immediate_second_restart_keeps_checkpoint_and_does_not_open_twice() {
        let dir = test_dir("second-restart");
        let mut s = Session::new();
        dirty_document(&mut s);
        save(&s, &dir).unwrap();
        let uid = list(&dir)[0].0;
        let before = std::fs::read(files(&dir, uid).0).unwrap();
        drop(s);
        let mut first = Session::new();
        assert_eq!(open(&mut first, &dir).unwrap().len(), 1);
        assert_eq!(list(&dir).len(), 1, "reopening must retain the checkpoint");
        assert_eq!(std::fs::read(files(&dir, uid).0).unwrap(), before);
        assert!(open(&mut first, &dir).unwrap().is_empty(), "already open in this session");
        let recovered = (*first.doc().unwrap().doc).clone();
        drop(first); // No recovery save between restarts.
        let mut second = Session::new();
        assert_eq!(open(&mut second, &dir).unwrap().len(), 1);
        assert_eq!(*second.doc().unwrap().doc, recovered);
        assert!(second.doc().unwrap().is_dirty());
        second.close_document(0);
        assert!(list(&dir).is_empty(), "explicit close removes the owned entry even with no configured folder");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reset_uid_cannot_overwrite_or_discard_unclaimed_entries() {
        let dir = test_dir("uid-collision");
        let mut s = Session::new();
        dirty_document(&mut s);
        let uid = s.doc().unwrap().uid;
        let (doc, meta) = files(&dir, uid);
        std::fs::write(&doc, b"pending unreadable snapshot").unwrap();
        std::fs::write(&meta, b"{\"title\":\"pending\"}").unwrap();
        let orphan = files(&dir, uid.wrapping_add(1)).1;
        std::fs::write(&orphan, b"pending metadata").unwrap();
        std::fs::write(files(&dir, u64::MAX).0, b"hostile identifier").unwrap();
        save(&s, &dir).unwrap();
        assert_eq!(std::fs::read(&doc).unwrap(), b"pending unreadable snapshot");
        assert_eq!(std::fs::read(&meta).unwrap(), b"{\"title\":\"pending\"}");
        assert_eq!(std::fs::read(&orphan).unwrap(), b"pending metadata");
        assert_eq!(open(&mut s, &dir).unwrap().len(), 0, "skip corrupt entries and this session's own document");
        s.close_document(0);
        assert_eq!(list(&dir).len(), 2, "close removes only the owned checkpoint");
        let mut clean = Session::new();
        clean.execute("file.new", &json!({})).unwrap();
        clean.doc_mut().unwrap().uid = uid;
        save(&clean, &dir).unwrap();
        assert_eq!(std::fs::read(&doc).unwrap(), b"pending unreadable snapshot");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_checkpoint_keeps_previous_document_and_can_retry() {
        let dir = test_dir("failed-checkpoint");
        let mut s = Session::new();
        dirty_document(&mut s);
        save(&s, &dir).unwrap();
        drop(s);
        let mut recovered = Session::new();
        open(&mut recovered, &dir).unwrap();
        let uid = list(&dir)[0].0;
        let (doc, meta) = files(&dir, uid);
        let before = std::fs::read(&doc).unwrap();
        let metadata = std::fs::read(&meta).unwrap();
        // A directory at the metadata destination makes replacement fail on every platform.
        std::fs::remove_file(&meta).unwrap();
        std::fs::create_dir(&meta).unwrap();
        recovered.execute("frame.create", &json!({"rect": [0, 0, 20, 20]})).unwrap();
        assert!(save(&recovered, &dir).is_err());
        assert_eq!(std::fs::read(&doc).unwrap(), before, "last valid snapshot must survive a failed write");
        assert!(recovered.doc().unwrap().is_dirty());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2, "failed staging files are cleaned up");
        let mut restart = Session::new();
        assert_eq!(open(&mut restart, &dir).unwrap().len(), 1);
        assert_eq!(restart.doc().unwrap().doc.spreads[0].items.len(), 1, "previous checkpoint remains usable");
        drop(restart);
        std::fs::remove_dir(&meta).unwrap();
        std::fs::write(&meta, metadata).unwrap();
        assert_eq!(save(&recovered, &dir).unwrap(), 1);
        assert_eq!(list(&dir).len(), 1, "retry uses the original key");
        assert_eq!(crate::cmd::from_bytes(&std::fs::read(doc).unwrap()).unwrap(), *recovered.doc().unwrap().doc);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn multiple_recovered_documents_preserve_paths_and_clean_up_only_owned_entries() {
        let dir = test_dir("multiple");
        let mut original = Session::new();
        dirty_document(&mut original);
        let path = dir.join("original.pagenor");
        original.execute("file.saveAs", &json!({"path": path})).unwrap();
        original.execute("frame.create", &json!({"rect": [0, 0, 20, 20]})).unwrap();
        dirty_document(&mut original);
        save(&original, &dir).unwrap();
        std::fs::write(files(&dir, u64::MAX).0, b"corrupt pending entry").unwrap();
        drop(original);
        let mut s = Session::new();
        assert_eq!(open(&mut s, &dir).unwrap(), vec![0, 1]);
        let keys: Vec<_> = list(&dir).into_iter().map(|e| e.0).collect();
        assert_eq!(save(&s, &dir).unwrap(), 2);
        assert_eq!(list(&dir).into_iter().map(|e| e.0).collect::<Vec<_>>(), keys);
        s.set_active(0);
        assert_eq!(s.doc().unwrap().path.as_deref(), path.to_str());
        let copy = dir.join("copy.pagenor");
        s.execute("file.saveACopy", &json!({"path": copy})).unwrap();
        assert!(s.doc().unwrap().is_dirty());
        assert_eq!(list(&dir).len(), 3, "copy-save keeps the checkpoint");
        std::fs::remove_file(&path).unwrap();
        assert!(s.execute("file.revert", &json!({})).is_err());
        assert_eq!(list(&dir).len(), 3, "failed revert keeps the checkpoint");
        std::fs::copy(copy, &path).unwrap();
        s.execute("file.revert", &json!({})).unwrap();
        assert!(!s.doc().unwrap().is_dirty());
        assert_eq!(list(&dir).len(), 2);
        s.set_active(1);
        let missing = dir.join("missing/failed.pagenor");
        assert!(s.execute("file.saveAs", &json!({"path": missing})).is_err());
        assert!(s.doc().unwrap().is_dirty());
        assert_eq!(list(&dir).len(), 2, "failed save keeps the checkpoint");
        s.execute("file.saveAs", &json!({"path": dir.join("saved.pagenor")})).unwrap();
        assert_eq!(list(&dir).len(), 1);
        assert_eq!(std::fs::read(files(&dir, u64::MAX).0).unwrap(), b"corrupt pending entry");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn saving_cleans_entries_in_explicit_directories_and_undo_gets_a_new_checkpoint() {
        let dir = test_dir("explicit-directories");
        let other = dir.join("other");
        let mut s = Session::new();
        dirty_document(&mut s);
        assert_eq!(save(&s, &dir).unwrap(), 1);
        assert_eq!(save(&s, &other).unwrap(), 1);
        assert!(open(&mut s, &dir.join(".")).unwrap().is_empty(), "canonical path owns the same entry");
        s.execute("file.saveAs", &json!({"path": dir.join("saved.pagenor")})).unwrap();
        assert!(list(&dir).is_empty());
        assert!(list(&other).is_empty());
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(s.doc().unwrap().is_dirty());
        save(&s, &dir).unwrap();
        assert_eq!(list(&dir).len(), 1);
        s.close_document(0);
        assert!(list(&dir).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn stale_staging_files_and_extreme_keys_are_preserved() {
        let dir = test_dir("staging");
        std::fs::write(files(&dir, u64::MAX).0, b"pending").unwrap();
        assert_eq!(reserve(&dir, u64::MAX).unwrap(), 0);
        discard(&dir, 0);
        let mut s = Session::new();
        dirty_document(&mut s);
        let tmp = files(&dir, s.doc().unwrap().uid).0.with_extension("pagenor.tmp-0");
        std::fs::write(&tmp, b"previous interrupted write").unwrap();
        save(&s, &dir).unwrap();
        assert_eq!(std::fs::read(tmp).unwrap(), b"previous interrupted write");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_first_checkpoint_releases_reservation_without_touching_stale_files() {
        let dir = test_dir("first-failure");
        let mut s = Session::new();
        dirty_document(&mut s);
        let uid = s.doc().unwrap().uid;
        let (doc, _) = files(&dir, uid);
        for suffix in 0..64 {
            std::fs::write(doc.with_extension(format!("pagenor.tmp-{suffix}")), b"pending staging data").unwrap();
        }
        assert!(save(&s, &dir).is_err(), "bounded staging exhaustion returns an error");
        assert!(list(&dir).is_empty(), "no empty reserved document after a failed first write");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 64);
        assert!(s.doc().unwrap().is_dirty());
        std::fs::remove_file(doc.with_extension("pagenor.tmp-0")).unwrap();
        assert_eq!(save(&s, &dir).unwrap(), 1);
        assert_eq!(list(&dir).len(), 1);
        assert_eq!(std::fs::read(doc.with_extension("pagenor.tmp-1")).unwrap(), b"pending staging data");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn recovers_unsaved_documents() {
        let dir = std::env::temp_dir().join(format!("pagenor-recovery-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.new", &json!({})).unwrap();
        // Only the edited document is written.
        s.execute("frame.create", &json!({"rect": [10, 10, 100, 100]})).unwrap();
        assert_eq!(save(&s, &dir).unwrap(), 1);
        assert_eq!(list(&dir).len(), 1);
        drop(s);
        // "Crash": a new session reopens it, unsaved, with the item.
        let mut s2 = Session::new();
        let opened = open(&mut s2, &dir).unwrap();
        assert_eq!(opened.len(), 1);
        let st = s2.doc().unwrap();
        assert!(st.is_dirty());
        assert_eq!(st.doc.spreads[0].items.len(), 1);
        assert_eq!(list(&dir).len(), 1, "checkpoint retained until save/close");
        // Saving a document drops its entry.
        s2.recovery_dir = Some(dir.clone());
        assert_eq!(save(&s2, &dir).unwrap(), 1);
        let path = dir.join("saved.pagenor");
        s2.execute("file.saveAs", &json!({"path": path.to_string_lossy()})).unwrap();
        assert!(list(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod file_tests {
    use super::*;

    /// `file.recovery.list` adds `uid` to each entry's metadata: metadata that wasn't an object
    /// (a corrupt `.json` beside the recovery file) panicked on `m["uid"] = …`.
    #[test]
    fn corrupt_recovery_metadata_is_ignored() {
        let dir = std::env::temp_dir().join(format!("dc-recovery-corrupt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("7.pagenor"), b"not a document").unwrap();
        std::fs::write(dir.join("7.json"), b"[1, 2]").unwrap();
        let mut s = Session::new();
        s.recovery_dir = Some(dir.clone());
        let r = s.execute("file.recovery.list", &json!({})).unwrap();
        assert_eq!(r[0]["uid"], json!(7));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn revert_and_save_a_copy() {
        let dir = std::env::temp_dir().join(format!("dc-revert-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (a, copy) = (dir.join("a.pagenor"), dir.join("copy.pagenor"));
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        assert!(s.execute("file.revert", &json!({})).is_err(), "never saved");
        s.execute("file.saveAs", &json!({"path": a.to_string_lossy()})).unwrap();
        s.execute("frame.create", &json!({"rect": [10, 10, 100, 100]})).unwrap();
        // A copy keeps the document unsaved and on its own file.
        s.execute("file.saveACopy", &json!({"path": copy.to_string_lossy()})).unwrap();
        assert!(copy.exists());
        assert!(s.doc().unwrap().is_dirty());
        assert_eq!(s.doc().unwrap().path.as_deref(), Some(a.to_string_lossy().as_ref()));
        // Revert drops the frame.
        s.execute("file.revert", &json!({})).unwrap();
        assert!(s.doc().unwrap().doc.spreads[0].items.is_empty());
        assert!(!s.doc().unwrap().is_dirty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
