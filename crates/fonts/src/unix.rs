//! Fontconfig file locations plus bounded, freshly read configuration directories.
//! Directory reads discover additions even while the process-wide Fontconfig cache is stale.

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

use fontconfig_parser::{ConfigPart, DirPrefix};

use crate::platform::Discovery;

struct Paths {
    home: Option<PathBuf>,
    data: Option<PathBuf>,
    config: Option<PathBuf>,
    search: Vec<PathBuf>,
    cwd: PathBuf,
}

impl Paths {
    fn resolve(&self, text: &str, prefix: DirPrefix, file: &Path, include: bool) -> Option<PathBuf> {
        let path = Path::new(text);
        if let Ok(tail) = path.strip_prefix("~") {
            return self.home.as_ref().map(|h| h.join(tail));
        }
        if path.is_absolute() {
            return Some(path.into());
        }
        match prefix {
            DirPrefix::Relative => file.parent().map(|p| p.join(path)),
            DirPrefix::Xdg => if include { &self.config } else { &self.data }.as_ref().map(|p| p.join(path)),
            DirPrefix::Default if include => {
                self.search.iter().map(|p| p.join(path)).find(|p| p.exists()).or_else(|| self.search.first().map(|p| p.join(path)))
            }
            _ => Some(self.cwd.join(path)),
        }
    }
}

/// Non-recursive traversal preserves include/reset ordering, handles cycles, and caps input.
fn configured_dirs(primary: PathBuf, paths: &Paths, found: &mut Discovery) {
    enum Work {
        File(PathBuf, bool),
        Part(PathBuf, ConfigPart),
    }
    let mut work = vec![Work::File(primary, false)];
    let mut visited = HashSet::new();
    let mut bytes = 0;
    while let Some(item) = work.pop() {
        match item {
            Work::File(file, optional) => {
                let canonical = std::fs::canonicalize(&file).unwrap_or_else(|_| file.clone());
                if !visited.insert(canonical) {
                    continue;
                }
                if visited.len() > 256 {
                    found.note(&file, "Fontconfig limit reached (256 configuration paths)".into());
                    break;
                }
                if file.is_dir() {
                    let entries = match std::fs::read_dir(&file) {
                        Ok(e) => e,
                        Err(e) => {
                            found.note(&file, format!("Cannot read Fontconfig include directory: {e}"));
                            continue;
                        }
                    };
                    let mut configs = Vec::new();
                    for (i, entry) in entries.enumerate() {
                        if i == 4096 {
                            found.note(&file, "Fontconfig include limit reached (4096 entries)".into());
                            break;
                        }
                        match entry {
                            Ok(e) => {
                                // Fontconfig includes only digit-prefixed *.conf files in directories.
                                let name = e.file_name();
                                let name = name.to_string_lossy();
                                if name.as_bytes().first().is_some_and(u8::is_ascii_digit) && name.ends_with(".conf") {
                                    configs.push(e.path());
                                }
                            }
                            Err(e) => found.note(&file, format!("Cannot read Fontconfig include entry: {e}")),
                        }
                    }
                    configs.sort();
                    if configs.len().saturating_add(work.len()) > 16384 {
                        configs.truncate(16384usize.saturating_sub(work.len()));
                        found.note(&file, "Fontconfig configuration exceeds the 16384 pending elements limit".into());
                    }
                    work.extend(configs.into_iter().rev().map(|p| Work::File(p, false)));
                    continue;
                }
                let read = || -> std::io::Result<Vec<u8>> {
                    let mut data = Vec::new();
                    std::fs::File::open(&file)?.take(1024 * 1024 + 1).read_to_end(&mut data)?;
                    Ok(data)
                };
                let data = match read() {
                    Ok(d) => d,
                    Err(e) if optional && e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => {
                        found.note(&file, format!("Cannot read Fontconfig configuration: {e}"));
                        continue;
                    }
                };
                bytes += data.len();
                if data.len() > 1024 * 1024 || bytes > 8 * 1024 * 1024 {
                    found.note(&file, "Fontconfig configuration exceeds the 1 MiB/file or 8 MiB total limit".into());
                    if bytes > 8 * 1024 * 1024 {
                        break;
                    }
                    continue;
                }
                let parts = std::str::from_utf8(&data)
                    .map_err(|e| e.to_string())
                    .and_then(|s| fontconfig_parser::parse_config_parts(s).map_err(|e| e.to_string()));
                match parts {
                    Ok(parts) if parts.len().saturating_add(work.len()) <= 16384 => {
                        work.extend(parts.into_iter().rev().map(|p| Work::Part(file.clone(), p)))
                    }
                    Ok(_) => found.note(&file, "Fontconfig configuration exceeds the 16384 pending elements limit".into()),
                    Err(e) => found.note(&file, format!("Cannot parse Fontconfig configuration: {e}")),
                }
            }
            Work::Part(file, part) => {
                let (text, prefix, include, optional) = match part {
                    ConfigPart::Dir(d) => (d.path, d.prefix, false, false),
                    ConfigPart::RemapDir(d) => (d.path, d.prefix, false, false),
                    ConfigPart::Include(d) => (d.path, d.prefix, true, d.ignore_missing),
                    ConfigPart::ResetDirs => {
                        found.dirs.clear();
                        continue;
                    }
                    _ => continue,
                };
                if text.is_empty() {
                    continue;
                }
                let Some(path) = paths.resolve(&text, prefix, &file, include) else {
                    found.note(&file, format!("Cannot resolve Fontconfig path: {text}"));
                    continue;
                };
                if include {
                    work.push(Work::File(path, optional));
                } else if found.dirs.len() < 4096 {
                    found.dirs.push(path);
                } else {
                    found.note(&file, "Fontconfig directory limit reached (4096 directories)".into());
                }
            }
        }
    }
    found.dirs.sort();
    found.dirs.dedup();
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub(crate) fn discover() -> Discovery {
    let mut found = Discovery::default();
    let mut list = || -> Result<(), String> {
        let fc = fontconfig::Fontconfig::new().ok_or("Fontconfig library or configuration is unavailable; scanning font directories")?;
        let pattern = fontconfig::Pattern::new(&fc).map_err(|e| e.to_string())?;
        let mut objects = fontconfig::ObjectSet::new(&fc).map_err(|e| e.to_string())?;
        objects.add(fontconfig::FC_FILE).map_err(|e| e.to_string())?;
        let fonts = fontconfig::list_fonts(&pattern, Some(&objects)).map_err(|e| e.to_string())?;
        for (i, font) in fonts.iter().enumerate() {
            if i == 100000 {
                found.note(Path::new("Fontconfig"), "Fontconfig file limit reached (100000 paths)".into());
                break;
            }
            match font.filename() {
                Ok(file) => found.files.push(PathBuf::from(file)),
                Err(e) => found.note(Path::new("Fontconfig"), format!("Font has no readable UTF-8 file location: {e}")),
            }
        }
        Ok(())
    };
    if let Err(e) = list() {
        found.note(Path::new("Fontconfig"), e);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let xdg = |key: &str, fallback: &str| {
        std::env::var_os(key).map(PathBuf::from).filter(|p| p.is_absolute()).or_else(|| home.as_ref().map(|h| h.join(fallback)))
    };
    let default_search = if cfg!(target_os = "freebsd") { "/usr/local/etc/fonts" } else { "/etc/fonts" };
    let search = std::env::var_os("FONTCONFIG_PATH")
        .map(|p| std::env::split_paths(&p).take(128).collect())
        .unwrap_or_else(|| vec![PathBuf::from(default_search)]);
    let paths = Paths {
        data: xdg("XDG_DATA_HOME", ".local/share"),
        config: xdg("XDG_CONFIG_HOME", ".config"),
        home,
        search,
        cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
    };
    let primary = std::env::var_os("FONTCONFIG_FILE").map(PathBuf::from).unwrap_or_else(|| "fonts.conf".into());
    let primary = if primary.is_absolute() {
        primary
    } else {
        paths.search.iter().map(|p| p.join(&primary)).find(|p| p.exists()).unwrap_or_else(|| PathBuf::from(default_search).join(primary))
    };
    configured_dirs(primary, &paths, &mut found);
    found.files.sort();
    found.files.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_includes_reset_cycles_and_xdg_paths() {
        let root = std::env::temp_dir().join(format!("pagenor-fontconfig-{}", std::process::id()));
        let config = root.join("fonts.conf");
        let includes = root.join("conf.d");
        std::fs::create_dir_all(&includes).unwrap();
        std::fs::write(&config, "<fontconfig><dir>/discarded</dir><include prefix='relative'>conf.d</include><dir prefix='xdg'>fonts</dir><dir>~/private-fonts</dir></fontconfig>").unwrap();
        std::fs::write(
            includes.join("10-reset.conf"),
            "<fontconfig><reset-dirs/><dir prefix='relative'>../external</dir><include prefix='relative'>../fonts.conf</include></fontconfig>",
        )
        .unwrap();
        std::fs::write(
            includes.join("20-more.conf"),
            "<fontconfig><dir>/shared</dir><include ignore_missing='yes'>missing.conf</include></fontconfig>",
        )
        .unwrap();
        std::fs::write(includes.join("ignored.conf"), "not XML").unwrap();
        let paths = Paths {
            home: Some(root.join("home")),
            data: Some(root.join("xdg-data")),
            config: Some(root.join("xdg-config")),
            search: vec![root.clone()],
            cwd: root.clone(),
        };
        let mut found = Discovery::default();
        configured_dirs(config, &paths, &mut found);
        assert!(found.issues.is_empty(), "{:?}", found.issues);
        assert!(!found.dirs.contains(&PathBuf::from("/discarded")));
        for path in [includes.join("../external"), PathBuf::from("/shared"), root.join("xdg-data/fonts"), root.join("home/private-fonts")] {
            assert!(found.dirs.contains(&path), "{path:?}: {:?}", found.dirs);
        }
        assert_eq!(paths.resolve("custom.conf", DirPrefix::Xdg, &root, true), Some(root.join("xdg-config/custom.conf")));
        std::fs::write(root.join("huge.conf"), vec![b' '; 1024 * 1024 + 1]).unwrap();
        configured_dirs(root.join("huge.conf"), &paths, &mut found);
        assert!(found.issues.iter().any(|(_, r)| r.contains("limit")));
        std::fs::write(root.join("bad.conf"), "<fontconfig><").unwrap();
        configured_dirs(root.join("bad.conf"), &paths, &mut found);
        assert!(found.issues.iter().any(|(_, r)| r.contains("Cannot parse")));
        std::fs::remove_dir_all(root).unwrap();
    }
}
