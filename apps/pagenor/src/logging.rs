//! The desktop app's logger: `log::` records go to standard error and to a log file.
//!
//! Without a logger every `log::warn!`/`log::error!` in the workspace (the crash guard's panic
//! report, a failed crash-recovery save, font folders that can't be read) vanished. A launch from
//! a desktop menu has no terminal, so the file is what a bug report can attach:
//! `<settings dir>/logs/pagenor.log` (the directory beside `ui.json` and `prefs.json`, see
//! `config_dir` in `main.rs`). Each start moves the previous log to `pagenor.1.log` (and that
//! one to `.2`), so the log of a run that crashed survives the next launch.
//!
//! Levels: `info` for Pagenor's own crates, `warn` for everything else (wgpu and naga are
//! chatty). `RUST_LOG` replaces that with env_logger-style directives: `debug`,
//! `warn,pagenor_render=trace`, `wgpu_core=info`. A directive ending in `*` matches every
//! target that starts with it (`pagenor*=debug`).
//!
//! Records logged before the settings directory is known are kept (up to [`MAX_PENDING`]
//! records and [`MAX_PENDING_BYTES`] bytes) and
//! written once the file is attached. Individual records are capped at [`MAX_RECORD_BYTES`]. Writing never panics: a file that can't be created or
//! written leaves standard error as the only sink.

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::SystemTime;

use log::LevelFilter;

/// The current log file's name inside the log directory.
pub const LOG_FILE: &str = "pagenor.log";
/// How many previous logs are kept (`pagenor.1.log` … `pagenor.<KEEP>.log`).
pub const KEEP: usize = 2;
/// The log file stops growing past this size (a runaway warning can't fill the disk).
pub const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
/// Records kept in memory until the log file is attached.
pub const MAX_PENDING: usize = 512;
/// Aggregate buffered bytes and maximum formatted record bytes, including the newline.
pub const MAX_PENDING_BYTES: usize = 1024 * 1024;
pub const MAX_RECORD_BYTES: usize = 64 * 1024;
const MAX_FILTER_BYTES: usize = 64 * 1024;
const MAX_FILTER_PARTS: usize = 256;
/// The built-in filter when `RUST_LOG` is unset or empty.
pub const DEFAULT_FILTER: &str = "warn,pagenor*=info";

/// Per-target level filter parsed from env_logger-style directives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    default: LevelFilter,
    directives: Vec<(String, LevelFilter)>,
}

impl Filter {
    /// Parse `spec`; unknown levels and empty parts are skipped. Without a bare level, targets no
    /// directive names log errors only.
    pub fn parse(spec: &str) -> Filter {
        let mut f = Filter { default: LevelFilter::Error, directives: Vec::new() };
        let spec = if spec.len() > MAX_FILTER_BYTES { DEFAULT_FILTER } else { spec };
        for part in spec.split(',').take(MAX_FILTER_PARTS).map(str::trim).filter(|p| !p.is_empty()) {
            match part.split_once('=') {
                Some((name, level)) => {
                    let name = name.trim();
                    if let (false, Some(level)) = (name.is_empty(), parse_level(level)) {
                        f.directives.push((name.to_owned(), level));
                    }
                }
                None => match parse_level(part) {
                    Some(level) => f.default = level,
                    // A bare target name: everything from it (env_logger does the same).
                    None => f.directives.push((part.to_owned(), LevelFilter::Trace)),
                },
            }
        }
        f
    }

    /// The level that applies to `target` (the most specific matching directive wins).
    pub fn level_for(&self, target: &str) -> LevelFilter {
        self.directives.iter().filter(|(name, _)| matches(name, target)).max_by_key(|(name, _)| name.len()).map_or(self.default, |(_, level)| *level)
    }

    /// The most verbose level any target can reach (for `log::set_max_level`).
    pub fn max(&self) -> LevelFilter {
        self.directives.iter().map(|(_, level)| *level).fold(self.default, Ord::max)
    }
}

fn parse_level(s: &str) -> Option<LevelFilter> {
    s.trim().parse().ok()
}

/// `name` matches `target` itself and its submodules (`a` matches `a` and `a::b`, not `ab`);
/// `name*` matches every target starting with `name`.
fn matches(name: &str, target: &str) -> bool {
    match name.strip_suffix('*') {
        Some(prefix) => target.starts_with(prefix),
        None => target.strip_prefix(name).is_some_and(|rest| rest.is_empty() || rest.starts_with("::")),
    }
}

/// `2026-10-08T07:59:17.728Z` (UTC, milliseconds). A clock before 1970 reads as the epoch.
pub fn timestamp(t: SystemTime) -> String {
    let since = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = since.as_secs();
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z", rem / 3600, rem % 3600 / 60, rem % 60, since.subsec_millis())
}

/// Days since 1970-01-01 to a proleptic Gregorian (year, month, day) (Howard Hinnant's
/// `civil_from_days`, unsigned because `timestamp` clamps at the epoch).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}

fn numbered(dir: &Path, n: usize) -> PathBuf {
    if n == 0 { dir.join(LOG_FILE) } else { dir.join(format!("pagenor.{n}.log")) }
}

/// Shift the previous logs up one (`.log` → `.1.log` → … → `.<KEEP>.log`, the oldest dropped)
/// and create a fresh, empty log file in `dir` (created if missing).
pub fn rotate(dir: &Path) -> std::io::Result<(PathBuf, File)> {
    let mut directory = std::fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(dir)?;
    for n in (1..=KEEP).rev() {
        match std::fs::rename(numbered(dir, n - 1), numbered(dir, n)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    let path = numbered(dir, 0);
    let mut options = File::options();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    Ok((path, file))
}

/// One formatted log line: `<timestamp> <LEVEL> [<thread>] <target>: <message>\n`.
#[cfg(test)]
pub fn format_line(ts: &str, level: log::Level, thread: &str, target: &str, message: &str) -> String {
    format_record(ts, level, thread, target, format_args!("{message}"))
}

/// Stop formatting at a UTF-8 boundary instead of first allocating the complete message.
struct BoundedLine {
    text: String,
    max: usize,
}
impl fmt::Write for BoundedLine {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let available = self.max.saturating_sub(self.text.len());
        if text.len() <= available {
            self.text.push_str(text);
            return Ok(());
        }
        let mut end = available;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        self.text.push_str(text.get(..end).ok_or(fmt::Error)?);
        Err(fmt::Error)
    }
}

fn format_record(ts: &str, level: log::Level, thread: &str, target: &str, args: fmt::Arguments<'_>) -> String {
    const NOTE: &str = " [record truncated]";
    let mut line = BoundedLine { text: String::new(), max: MAX_RECORD_BYTES - NOTE.len() - 1 };
    let truncated = fmt::write(&mut line, format_args!("{ts} {level:<5} [{thread}] {target}: {args}")).is_err();
    if line.text.ends_with('\n') {
        line.text.pop();
    }
    if truncated {
        line.text.push_str(NOTE);
    }
    line.text.push('\n');
    line.text
}

/// Where formatted lines go besides standard error: kept in memory until a file is attached,
/// then appended to it until it reaches its size cap.
pub struct Sink {
    file: Option<File>,
    pending: Vec<String>,
    pending_bytes: usize,
    dropped: usize,
    written: u64,
    max_bytes: u64,
    capped: bool,
}

impl Sink {
    pub fn new(max_bytes: u64) -> Sink {
        Sink { file: None, pending: Vec::new(), pending_bytes: 0, dropped: 0, written: 0, max_bytes, capped: false }
    }

    /// Append `line` to the file, or keep it until [`Sink::attach`].
    pub fn write(&mut self, line: &str) {
        if self.capped {
            return;
        }
        if self.file.is_none() {
            if self.pending.len() < MAX_PENDING
                && line.len() <= MAX_RECORD_BYTES
                && line.len() <= MAX_PENDING_BYTES.saturating_sub(self.pending_bytes)
            {
                self.pending.push(line.to_owned());
                self.pending_bytes += line.len();
            } else {
                self.dropped = self.dropped.saturating_add(1);
            }
            return;
        }
        self.append(line);
    }

    /// Start writing to `file`, first the lines kept so far.
    pub fn attach(&mut self, file: File) {
        self.file = Some(file);
        self.written = 0;
        self.capped = false;
        self.pending_bytes = 0;
        for line in std::mem::take(&mut self.pending) {
            self.append(&line);
        }
        if self.dropped > 0 {
            let note = format!("{} earlier log lines were dropped before the log file was opened\n", self.dropped);
            self.dropped = 0;
            self.append(&note);
        }
    }

    fn disable(&mut self) {
        self.file = None;
        self.pending.clear();
        self.pending_bytes = 0;
        self.dropped = 0;
        self.capped = true;
    }

    fn flush(&mut self) {
        if self.file.as_mut().is_some_and(|file| file.flush().is_err()) {
            self.disable();
        }
    }

    fn append(&mut self, line: &str) {
        if self.capped {
            return;
        }
        let Some(file) = self.file.as_mut() else { return };
        let remaining = self.max_bytes.saturating_sub(self.written);
        let len = u64::try_from(line.len()).unwrap_or(u64::MAX);
        let note;
        let text = if len > remaining {
            self.capped = true;
            note = format!("log file reached {} bytes; later records go to standard error only\n", self.max_bytes);
            // The notice is ASCII; even a very small residual budget cannot exceed the cap.
            let keep = usize::try_from(remaining).unwrap_or(usize::MAX).min(note.len());
            note.get(..keep).unwrap_or("")
        } else {
            line
        };
        // A failed write drops the file and pending buffer, leaving stderr as the sole sink.
        if file.write_all(text.as_bytes()).is_err() {
            self.disable();
        } else {
            self.written = self.written.saturating_add(u64::try_from(text.len()).unwrap_or(u64::MAX));
        }
    }
}

/// The installed logger: a [`Filter`], standard error and a [`Sink`].
pub struct AppLogger {
    filter: Filter,
    stderr: bool,
    sink: Mutex<Sink>,
}

impl AppLogger {
    pub fn disable_file(&self) {
        self.sink.lock().unwrap_or_else(PoisonError::into_inner).disable();
    }

    pub fn new(filter: Filter, stderr: bool) -> AppLogger {
        AppLogger { filter, stderr, sink: Mutex::new(Sink::new(MAX_FILE_BYTES)) }
    }

    /// Rotate the logs in `dir` and send records to the fresh file; returns its path.
    pub fn attach_dir(&self, dir: &Path) -> Result<PathBuf, String> {
        let (path, file) = rotate(dir).map_err(|e| {
            self.disable_file();
            format!("{}: {e}", dir.display())
        })?;
        self.sink.lock().unwrap_or_else(PoisonError::into_inner).attach(file);
        Ok(path)
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.filter.level_for(metadata.target())
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let thread = std::thread::current();
        let line = format_record(&timestamp(SystemTime::now()), record.level(), thread.name().unwrap_or("?"), record.target(), *record.args());
        if self.stderr {
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
        self.sink.lock().unwrap_or_else(PoisonError::into_inner).write(&line);
    }

    fn flush(&self) {
        self.sink.lock().unwrap_or_else(PoisonError::into_inner).flush();
    }
}

/// Install the logger (filter from `RUST_LOG`, else [`DEFAULT_FILTER`]); `None` when another
/// logger was installed first. Call [`AppLogger::attach_dir`] once the settings directory is known.
pub fn install() -> Option<&'static AppLogger> {
    static LOGGER: OnceLock<AppLogger> = OnceLock::new();
    let spec = std::env::var("RUST_LOG").ok().filter(|s| !s.trim().is_empty());
    let filter = Filter::parse(spec.as_deref().unwrap_or(DEFAULT_FILTER));
    let max = filter.max();
    let logger = LOGGER.get_or_init(|| AppLogger::new(filter, true));
    log::set_logger(logger).ok()?;
    log::set_max_level(max);
    Some(logger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let d = std::env::temp_dir().join(format!("pagenor-logging-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn the_default_filter_shows_pagenor_info_and_other_crates_warnings() {
        let f = Filter::parse(DEFAULT_FILTER);
        assert_eq!(f.level_for("pagenor"), LevelFilter::Info);
        assert_eq!(f.level_for("pagenor::control_server"), LevelFilter::Info);
        assert_eq!(f.level_for("pagenor_render::tile"), LevelFilter::Info);
        assert_eq!(f.level_for("wgpu_core::device"), LevelFilter::Warn);
        assert_eq!(f.level_for("naga"), LevelFilter::Warn);
        assert_eq!(f.max(), LevelFilter::Info);
    }

    #[test]
    fn directives_follow_env_logger_and_the_most_specific_one_wins() {
        let f = Filter::parse("info,wgpu_core=error,pagenor_render=trace,pagenor_render::tile=off");
        assert_eq!(f.level_for("eframe"), LevelFilter::Info);
        assert_eq!(f.level_for("wgpu_core::instance"), LevelFilter::Error);
        assert_eq!(f.level_for("pagenor_render::raster"), LevelFilter::Trace);
        assert_eq!(f.level_for("pagenor_render::tile"), LevelFilter::Off);
        assert_eq!(f.max(), LevelFilter::Trace);
        // A module name is matched at `::` boundaries, not as a bare prefix.
        assert_eq!(f.level_for("wgpu_core_extra"), LevelFilter::Info);
        // A bare target name sets that target to the most verbose level, as env_logger does.
        assert_eq!(Filter::parse("naga").level_for("naga::front"), LevelFilter::Trace);
        assert_eq!(Filter::parse("naga").level_for("eframe"), LevelFilter::Error);
    }

    #[test]
    fn hostile_specs_never_panic_and_fall_back_sensibly() {
        for spec in ["", ",,,", "=", "=debug", "pagenor=", "nonsense=loud", "🦀=info", "*", "a=b=c", "warn,,pagenor=DEBUG"] {
            let f = Filter::parse(spec);
            let _ = f.level_for("pagenor");
            let _ = f.max();
        }
        assert_eq!(Filter::parse("warn,,pagenor=DEBUG").level_for("pagenor"), LevelFilter::Debug);
        assert_eq!(Filter::parse("nonsense=loud").level_for("nonsense"), LevelFilter::Error);
    }

    #[test]
    fn timestamps_are_utc_with_milliseconds() {
        assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
        // 2026-10-08T07:59:17.728Z
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_millis(1_791_446_357_728)), "2026-10-08T07:59:17.728Z");
        // Leap days, including the century rule (2000 is a leap year, 2100 is not).
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_secs(1_709_164_800)), "2024-02-29T00:00:00.000Z");
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_secs(951_782_400)), "2000-02-29T00:00:00.000Z");
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_secs(4_107_542_400)), "2100-03-01T00:00:00.000Z");
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_millis(1_704_067_199_999)), "2023-12-31T23:59:59.999Z");
        // Before the epoch (a clock set wrong) clamps instead of panicking.
        assert_eq!(timestamp(UNIX_EPOCH - Duration::from_secs(5)), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn rotation_keeps_the_previous_logs_and_starts_an_empty_file() {
        let dir = temp_dir("rotate");
        for run in 1..=4 {
            let (path, _file) = rotate(&dir).expect("rotate");
            assert_eq!(path, dir.join(LOG_FILE));
            assert_eq!(std::fs::read_to_string(&path).expect("read"), "");
            std::fs::write(&path, format!("run {run}")).expect("write");
        }
        assert_eq!(std::fs::read_to_string(dir.join(LOG_FILE)).expect("current"), "run 4");
        assert_eq!(std::fs::read_to_string(dir.join("pagenor.1.log")).expect("1"), "run 3");
        assert_eq!(std::fs::read_to_string(dir.join("pagenor.2.log")).expect("2"), "run 2");
        assert!(!dir.join("pagenor.3.log").exists(), "only {KEEP} old logs are kept");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn newly_created_logs_are_private_and_rotation_preserves_existing_targets() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let dir = temp_dir("private");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("untouched.txt");
        std::fs::write(&target, "preserve me").unwrap();
        symlink(&target, dir.join(LOG_FILE)).unwrap();
        let (path, file) = rotate(&dir).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
        assert_eq!(read(&target), "preserve me");
        drop(file);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rotation_into_an_unusable_directory_is_an_error_not_a_panic() {
        let dir = temp_dir("blocked");
        std::fs::create_dir_all(dir.parent().expect("parent")).expect("tmp");
        std::fs::write(&dir, "a file where the directory should be").expect("block");
        assert!(rotate(&dir).is_err());
        let _ = std::fs::remove_file(&dir);
    }
    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).expect("read log")
    }

    #[test]
    fn a_line_carries_time_level_thread_target_and_message() {
        assert_eq!(
            format_line("2026-10-08T07:59:17.728Z", log::Level::Error, "main", "pagenor_engine::guard", "command `frame.create` panicked: boom"),
            "2026-10-08T07:59:17.728Z ERROR [main] pagenor_engine::guard: command `frame.create` panicked: boom\n"
        );
        // A multi-line message (a panic with a backtrace) keeps its lines; the record still ends in one newline.
        assert!(format_line("t", log::Level::Error, "w", "x", "a\nb\n").ends_with("x: a\nb\n"));
    }

    #[test]
    fn lines_logged_before_the_file_exists_are_written_when_it_is_attached() {
        let dir = temp_dir("pending");
        let (path, file) = rotate(&dir).expect("rotate");
        let mut sink = Sink::new(MAX_FILE_BYTES);
        sink.write("early 1\n");
        sink.write("early 2\n");
        sink.attach(file);
        sink.write("late\n");
        assert_eq!(read(&path), "early 1\nearly 2\nlate\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_pending_buffer_is_bounded_and_says_how_much_it_dropped() {
        let dir = temp_dir("bounded");
        let (path, file) = rotate(&dir).expect("rotate");
        let mut sink = Sink::new(MAX_FILE_BYTES);
        for i in 0..MAX_PENDING + 5 {
            sink.write(&format!("line {i}\n"));
        }
        sink.attach(file);
        let text = read(&path);
        assert_eq!(text.lines().filter(|l| l.starts_with("line ")).count(), MAX_PENDING);
        assert!(text.contains("5 earlier log lines were dropped"), "{text}");
        assert!(text.contains("line 0\n") && !text.contains(&format!("line {MAX_PENDING}\n")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_file_stops_at_its_size_cap_with_one_note() {
        let dir = temp_dir("cap");
        let (path, file) = rotate(&dir).expect("rotate");
        let mut sink = Sink::new(100);
        sink.attach(file);
        for i in 0..50 {
            sink.write(&format!("line {i:02} with some padding\n"));
        }
        let text = read(&path);
        assert!(text.len() <= 100, "the cap notice must fit too: {} bytes", text.len());
        assert!(text.lines().filter(|l| l.starts_with("line ")).count() < 5, "{text}");
        assert_eq!(text.matches("log file reached").count(), 1, "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_logger_filters_records_and_writes_them_to_the_attached_file() {
        use log::Log;
        let dir = temp_dir("logger");
        let logger = AppLogger::new(Filter::parse(DEFAULT_FILTER), false);
        let record = |level, target: &'static str, msg: &'static str| {
            logger.log(&log::Record::builder().level(level).target(target).args(format_args!("{msg}")).build());
        };
        record(log::Level::Info, "pagenor::prefs", "settings directory");
        record(log::Level::Info, "wgpu_core::device", "too chatty");
        let path = logger.attach_dir(&dir).expect("attach");
        assert_eq!(path, dir.join(LOG_FILE));
        record(log::Level::Warn, "wgpu_hal::vulkan", "a real warning");
        record(log::Level::Debug, "pagenor", "below info");
        let text = read(&path);
        assert!(text.contains(" INFO  [") && text.contains("pagenor::prefs: settings directory\n"), "{text}");
        assert!(text.contains("wgpu_hal::vulkan: a real warning\n"), "{text}");
        assert!(!text.contains("too chatty") && !text.contains("below info"), "{text}");
        assert!(logger.enabled(&log::Metadata::builder().level(log::Level::Info).target("pagenor_render").build()));
        assert!(!logger.enabled(&log::Metadata::builder().level(log::Level::Info).target("naga").build()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn attaching_to_an_unusable_directory_reports_an_error_and_keeps_logging() {
        use log::Log;
        let dir = temp_dir("unusable");
        std::fs::write(&dir, "not a directory").expect("block");
        let logger = AppLogger::new(Filter::parse("info"), false);
        assert!(logger.attach_dir(&dir).is_err());
        logger.log(&log::Record::builder().level(log::Level::Error).target("x").args(format_args!("still fine")).build());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn pending_bytes_and_oversized_raw_records_are_bounded() {
        let mut sink = Sink::new(MAX_FILE_BYTES);
        let line = "x".repeat(MAX_RECORD_BYTES);
        for _ in 0..MAX_PENDING {
            sink.write(&line);
        }
        sink.write(&"x".repeat(MAX_RECORD_BYTES + 1));
        assert!(sink.pending.len() <= MAX_PENDING);
        assert!(sink.pending.iter().map(String::len).sum::<usize>() <= MAX_PENDING_BYTES);
        assert_eq!(sink.pending_bytes, MAX_PENDING_BYTES);
        assert!(sink.dropped > 0);
        sink.disable();
        assert!(sink.pending.is_empty());
        assert_eq!(sink.pending_bytes, 0);
    }

    #[test]
    fn failed_file_writes_do_not_resume_buffering() {
        let dir = temp_dir("write-failed");
        let (path, file) = rotate(&dir).unwrap();
        drop(file);
        let mut sink = Sink::new(MAX_FILE_BYTES);
        sink.attach(File::open(&path).unwrap()); // Read-only handle deterministically rejects writing.
        sink.write("failed write\n");
        for _ in 0..MAX_PENDING {
            sink.write("later\n");
        }
        assert!(sink.capped && sink.file.is_none());
        assert!(sink.pending.is_empty());
        assert_eq!(sink.pending_bytes, 0);
        assert_eq!(read(&path), "");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn formatting_stops_at_a_utf8_boundary_without_building_the_whole_message() {
        struct Flood<'a>(&'a AtomicUsize);
        impl fmt::Display for Flood<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for _ in 0..1_000_000 {
                    self.0.fetch_add(1, Ordering::Relaxed);
                    f.write_str("é")?;
                }
                Ok(())
            }
        }
        let calls = AtomicUsize::new(0);
        let text = format_record("t", log::Level::Error, "main", "pagenor", format_args!("{}", Flood(&calls)));
        assert!(text.len() <= MAX_RECORD_BYTES);
        assert!(text.ends_with(" [record truncated]\n"));
        assert!(calls.load(Ordering::Relaxed) < MAX_RECORD_BYTES);
        assert!(std::str::from_utf8(text.as_bytes()).is_ok());
    }

    #[test]
    fn filter_resources_are_bounded() {
        let repeated = "pagenor=trace,".repeat(MAX_FILTER_PARTS + 1);
        assert!(Filter::parse(&repeated).directives.len() <= MAX_FILTER_PARTS);
        assert_eq!(Filter::parse(&"x".repeat(MAX_FILTER_BYTES + 1)), Filter::parse(DEFAULT_FILTER));
    }

    /// Runs only as a fresh child process, so the global logger and panic hook cannot affect
    /// the rest of the test harness. All files and control credentials are synthetic.
    #[test]
    fn isolated_logger_helper() {
        let Some(dir) = std::env::var_os("PAGENOR_TEST_LOG_DIR") else { return };
        let mut signal = [0_u8; 1];
        std::io::Read::read_exact(&mut std::io::stdin().lock(), &mut signal).unwrap();
        let logger = install().unwrap();
        log::warn!("early isolated diagnostic");
        let _control = crate::control_server::start(0, egui::Context::default()).unwrap();
        logger.attach_dir(Path::new(&dir)).unwrap();
        log::error!("late isolated diagnostic");
        pagenor_engine::guard::install_panic_hook();
        assert!(std::panic::catch_unwind(|| panic!("isolated diagnostic panic")).is_err());
        log::logger().flush();
    }

    #[test]
    fn fresh_process_logging_survives_closed_stderr_and_keeps_tokens_out_of_files() {
        use std::process::{Command, Stdio};
        const TOKEN: &str = "synthetic-control-token-with-at-least-32-bytes";
        let dir = temp_dir("process");
        for configured in [true, false] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "logging::tests::isolated_logger_helper", "--nocapture"])
                .env("PAGENOR_TEST_LOG_DIR", &dir)
                .env("RUST_LOG", DEFAULT_FILTER)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped());
            if configured {
                command.env("PAGENOR_CONTROL_TOKEN", TOKEN);
            } else {
                command.env_remove("PAGENOR_CONTROL_TOKEN");
            }
            let mut child = command.spawn().unwrap();
            // Close the reader before signalling the child, making stderr a broken pipe.
            drop(child.stderr.take());
            child.stdin.take().unwrap().write_all(b"x").unwrap();
            assert!(child.wait().unwrap().success());
            let text = read(&dir.join(LOG_FILE));
            for expected in ["early isolated diagnostic", "late isolated diagnostic", "authenticated control server", "isolated diagnostic panic"] {
                assert!(text.contains(expected), "missing {expected}: {text}");
            }
            assert!(!text.contains(TOKEN));
            assert!(!text.contains("control token:"));
        }
        assert!(read(&dir.join("pagenor.1.log")).contains("early isolated diagnostic"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
