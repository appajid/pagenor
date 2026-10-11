//! Native per-process recovery namespaces. Lock ownership, rather than a PID guess,
//! determines whether another run's checkpoint files can be reclaimed.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::{EngineError, Result};

const LOCK: &str = "lock";
const ROOT_LOCK: &str = ".pagenor-runs.lock";

struct Run {
    root: PathBuf,
    dir: PathBuf,
    lock: File,
}

static RUNS: Mutex<Vec<Run>> = Mutex::new(Vec::new());

fn error(path: &Path, e: impl std::fmt::Display) -> EngineError {
    EngineError::Other(format!("{}: {e}", path.display()))
}

fn link_like(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn validate_dir(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| error(path, e))?;
    if link_like(&metadata) || !metadata.is_dir() {
        return Err(error(path, "recovery requires a directory, without a link or reparse-point leaf"));
    }
    Ok(())
}

fn create_dir(path: &Path) -> Result<()> {
    if !path.try_exists().map_err(|e| error(path, e))? {
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(path).map_err(|e| error(path, e))?;
    }
    validate_dir(path)
}

fn regular_lock(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| error(path, e))?;
    if link_like(&metadata) || !metadata.is_file() {
        return Err(error(path, "recovery lock must be a regular file"));
    }
    Ok(())
}

fn root_lock(root: &Path) -> Result<File> {
    validate_dir(root)?;
    let path = root.join(ROOT_LOCK);
    if path.try_exists().map_err(|e| error(&path, e))? {
        regular_lock(&path)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path).map_err(|e| error(&path, e))?;
    regular_lock(&path)?;
    // Coordinate creation and takeover. A simultaneous launch must not mistake a new,
    // not-yet-locked run for a terminated one. A bounded contention failure disables
    // automatic recovery for that launch instead of falling back to a shared namespace.
    for attempt in 0..5 {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) if attempt < 4 => {
                std::thread::sleep(std::time::Duration::from_millis(10 << attempt));
            }
            Err(e) => return Err(error(&path, e)),
        }
    }
    Err(error(&path, "recovery coordinator contention budget exhausted"))
}

/// A locked recovery folder unique to this process and root. The lock lives until exit.
/// Existing folders are never reused, even if their PID matches the current process.
pub fn run_dir(root: &Path) -> Result<PathBuf> {
    create_dir(root)?;
    let root = std::fs::canonicalize(root).map_err(|e| error(root, e))?;
    let mut runs = RUNS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(run) = runs.iter().find(|run| run.root == root) {
        validate_dir(&run.dir)?;
        regular_lock(&run.dir.join(LOCK))?;
        return Ok(run.dir.clone());
    }
    let _coordinator = root_lock(&root)?;
    for n in 0..64 {
        let dir = root.join(format!("run-{}-{n}", std::process::id()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(error(&dir, e)),
        }
        let path = dir.join(LOCK);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(&path).map_err(|e| error(&path, e))?;
        lock.try_lock().map_err(|e| error(&path, e))?;
        runs.push(Run { root, dir: dir.clone(), lock });
        return Ok(dir);
    }
    Err(error(&root, "no free recovery folder within 64 candidates"))
}

/// Fail closed if this process's namespace or lock has been removed/replaced.
pub(crate) fn validate_own(dir: &Path) -> Result<()> {
    let runs = RUNS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(run) = runs.iter().find(|run| run.dir == dir) else { return Ok(()) };
    validate_dir(dir)?;
    regular_lock(&dir.join(LOCK))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let held = run.lock.metadata().map_err(|e| error(dir, e))?;
        let named = std::fs::metadata(dir.join(LOCK)).map_err(|e| error(dir, e))?;
        if held.dev() != named.dev() || held.ino() != named.ino() {
            return Err(error(dir, "recovery lock was replaced; checkpoint writes refused"));
        }
    }
    #[cfg(not(unix))]
    let _ = &run.lock;
    Ok(())
}

fn run_name(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str().and_then(|s| s.strip_prefix("run-")) else { return false };
    let mut parts = name.split('-');
    parts.next().is_some_and(|s| s.parse::<u32>().is_ok()) && parts.next().is_some_and(|s| s.parse::<u32>().is_ok()) && parts.next().is_none()
}

/// Reclaim released namespaces only. Old flat checkpoints have no liveness lock:
/// retain them for explicit recovery, since an older app may still be writing there.
pub(crate) fn take_over(own: &Path) -> Result<()> {
    let root = {
        let runs = RUNS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(run) = runs.iter().find(|run| run.dir == own) else { return Ok(()) };
        run.root.clone()
    };
    validate_own(own)?;
    let _coordinator = root_lock(&root)?;
    let entries = std::fs::read_dir(&root).map_err(|e| error(&root, e))?;
    for entry in entries.take(4096) {
        let entry = entry.map_err(|e| error(&root, e))?;
        let dir = entry.path();
        if dir == own || !run_name(&entry.file_name()) || validate_dir(&dir).is_err() {
            continue;
        }
        let path = dir.join(LOCK);
        if regular_lock(&path).is_err() {
            continue;
        }
        let Ok(lock) = OpenOptions::new().write(true).open(&path) else { continue };
        if lock.try_lock().is_err() {
            continue;
        }
        // Keep the stale lock/folder. Unknown files and failed moves stay intact; another
        // launcher cannot race through an unlinked lock file referring to the old inode.
        for (uid, _) in crate::recovery::list(&dir) {
            crate::recovery::move_entry(&dir, uid, own)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Session, recovery};
    use serde_json::json;

    fn dirty(text: &str) -> Session {
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        session.execute("frame.create", &json!({"rect": [10,10,100,100], "content":"text", "text":text})).unwrap();
        session
    }

    fn stale(root: &Path, name: &str, text: &str) -> PathBuf {
        let dir = root.join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join(LOCK), b"").unwrap();
        let mut session = dirty(text);
        session.docs[0].uid = 1;
        recovery::save(&session, &dir).unwrap();
        dir
    }

    #[test]
    fn run_is_private_locked_and_reused_only_by_this_process() {
        let root = tempfile::tempdir().unwrap();
        let dir = run_dir(root.path()).unwrap();
        assert_eq!(dir.parent(), Some(std::fs::canonicalize(root.path()).unwrap().as_path()));
        assert_eq!(run_dir(root.path()).unwrap(), dir);
        let other = OpenOptions::new().write(true).open(dir.join(LOCK)).unwrap();
        assert!(matches!(other.try_lock(), Err(std::fs::TryLockError::WouldBlock)));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o700);
            assert_eq!(std::fs::metadata(dir.join(LOCK)).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn reclaiming_released_runs_preserves_collisions_corrupt_entries_and_unknown_files() {
        let root = tempfile::tempdir().unwrap();
        let a = stale(root.path(), "run-7-0", "First run Telugu తెలుగు");
        let b = stale(root.path(), "run-8-0", "Second run");
        std::fs::write(b.join("999.pagenor"), b"corrupt document kept for diagnosis").unwrap();
        std::fs::write(a.join("owner-notes.txt"), b"keep").unwrap();
        let own = run_dir(root.path()).unwrap();
        let mut current = dirty("Already open");
        current.docs[0].uid = 1;
        recovery::save(&current, &own).unwrap();
        let old = recovery::list(&own);
        let previous = std::fs::read(own.join(format!("{}.pagenor", old[0].0))).unwrap();
        let mut recovered = Session::new();
        assert_eq!(recovery::open(&mut recovered, &own).unwrap().len(), 3);
        assert_eq!(recovery::list(&own).len(), 4);
        assert_eq!(std::fs::read(own.join(format!("{}.pagenor", old[0].0))).unwrap(), previous);
        assert_eq!(std::fs::read(a.join("owner-notes.txt")).unwrap(), b"keep");
        assert!(recovery::open(&mut recovered, &own).unwrap().is_empty());
        assert!(recovery::list(&a).is_empty() && recovery::list(&b).is_empty());
        assert!(
            recovery::list(&own)
                .iter()
                .any(|(id, _)| { std::fs::read(own.join(format!("{id}.pagenor"))).unwrap() == b"corrupt document kept for diagnosis" })
        );
    }

    #[test]
    fn unprotected_legacy_checkpoints_and_missing_locks_are_not_reclaimed() {
        let root = tempfile::tempdir().unwrap();
        recovery::save(&dirty("Older app may still be open"), root.path()).unwrap();
        let unlocked = root.path().join("run-8-0");
        std::fs::create_dir(&unlocked).unwrap();
        recovery::save(&dirty("Unknown run without lock"), &unlocked).unwrap();
        let own = run_dir(root.path()).unwrap();
        assert!(recovery::open(&mut Session::new(), &own).unwrap().is_empty());
        assert_eq!(recovery::list(root.path()).len(), 1);
        assert_eq!(recovery::list(&unlocked).len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn linked_roots_and_replaced_live_locks_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(root.path(), &link).unwrap();
        assert!(run_dir(&link).is_err());
        let own = run_dir(root.path()).unwrap();
        std::fs::remove_file(own.join(LOCK)).unwrap();
        std::fs::write(own.join(LOCK), b"replacement").unwrap();
        assert!(recovery::save(&dirty("Do not write under a replacement lock"), &own).is_err());
        assert!(recovery::list(&own).is_empty());
    }

    // The parent test launches this exact harness test in a separate OS process.
    #[test]
    fn lock_child() {
        let Some(root) = std::env::var_os("PAGENOR_RECOVERY_CHILD_ROOT").map(PathBuf::from) else { return };
        let own = run_dir(&root).unwrap();
        recovery::save(&dirty("Child process live checkpoint"), &own).unwrap();
        std::fs::write(root.join("child-ready"), b"ready").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !root.join("child-stop").exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    fn actual_second_process_is_skipped_until_its_lock_is_released() {
        let root = tempfile::tempdir().unwrap();
        let mut child = Child(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "recovery_runs::tests::lock_child", "--nocapture"])
                .env("PAGENOR_RECOVERY_CHILD_ROOT", root.path())
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !root.path().join("child-ready").exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(root.path().join("child-ready").exists(), "child did not start");
        let own = run_dir(root.path()).unwrap();
        let mut session = Session::new();
        assert!(recovery::open(&mut session, &own).unwrap().is_empty());
        std::fs::write(root.path().join("child-stop"), b"stop").unwrap();
        assert!(child.0.wait().unwrap().success());
        assert_eq!(recovery::open(&mut session, &own).unwrap().len(), 1);
        assert!(session.documents().iter().flat_map(|d| d.doc.stories.values()).any(|story| story.text == "Child process live checkpoint"));
        assert!(recovery::open(&mut session, &own).unwrap().is_empty());
    }
}
