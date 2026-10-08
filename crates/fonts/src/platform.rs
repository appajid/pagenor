//! Native discovery returns paths only; the shared catalogue reads their bounded metadata.

#[cfg(any(windows, target_os = "linux", target_os = "freebsd", test))]
use std::path::Path;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Discovery {
    pub files: Vec<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub issues: Vec<(PathBuf, String)>,
    pub omitted: usize,
}

impl Discovery {
    #[cfg(any(windows, target_os = "linux", target_os = "freebsd", test))]
    pub fn note(&mut self, path: &Path, reason: String) {
        if self.issues.len() < 256 {
            self.issues.push((path.into(), reason));
        } else {
            self.omitted += 1;
        }
    }
}

pub(crate) fn discover() -> (Option<&'static str>, Discovery, usize) {
    #[cfg(target_os = "macos")]
    {
        let host = crate::macos::activated_fonts();
        (Some("Core Text"), Discovery { files: host.paths, ..Default::default() }, host.pathless_faces)
    }
    #[cfg(windows)]
    {
        (Some("Windows font registry"), crate::windows::discover(), 0)
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        (Some("Fontconfig"), crate::unix::discover(), 0)
    }
    #[cfg(not(any(target_os = "macos", windows, target_os = "linux", target_os = "freebsd")))]
    {
        (None, Discovery::default(), 0)
    }
}
