//! Native discovery returns file locations and OS font identities for lazy metadata/data access.

#[cfg(any(windows, target_os = "linux", target_os = "freebsd", test))]
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeFont {
    pub name: String,
    pub path: Option<PathBuf>,
}

#[derive(Default)]
pub(crate) struct Discovery {
    pub files: Vec<PathBuf>,
    pub native_fonts: Vec<NativeFont>,
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
        (Some("Core Text"), Discovery { files: host.paths, native_fonts: host.fonts, ..Default::default() }, host.pathless_faces)
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

pub(crate) fn native_font_metadata(name: &str) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    {
        crate::macos::font_data(name, true)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(format!("Native memory font metadata is unsupported on this platform: {name}"))
    }
}

pub(crate) fn native_font_data(name: &str) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    {
        crate::macos::font_data(name, false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(format!("Native memory font data is unsupported on this platform: {name}"))
    }
}
