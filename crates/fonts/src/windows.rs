//! Read-only installed-font registry discovery. No font data or host settings are changed.

/// REG_EXPAND_SZ expansion in UTF-16, preserving paths with unpaired surrogates.
/// Expansion is single-pass, like Windows; the resolver is injectable for isolated tests.
fn expand_path(value: &[u16], lookup: impl Fn(&[u16]) -> Option<Vec<u16>>) -> Result<Vec<u16>, String> {
    const MAX_UNITS: usize = 32768;
    if value.len() > MAX_UNITS {
        return Err("Font path exceeds 32768 UTF-16 units".into());
    }
    let mut out = Vec::new();
    let mut at = 0;
    while at < value.len() {
        if value.get(at) == Some(&u16::from(b'%'))
            && let Some(end) = value.get(at + 1..).and_then(|s| s.iter().position(|c| *c == u16::from(b'%'))).map(|n| at + 1 + n)
            && end > at + 1
        {
            let name = value.get(at + 1..end).ok_or("Invalid environment variable span")?;
            let expanded = lookup(name).ok_or_else(|| format!("Unknown font-path environment variable: {}", String::from_utf16_lossy(name)))?;
            if out.len().saturating_add(expanded.len()) > MAX_UNITS {
                return Err("Expanded font path exceeds 32768 UTF-16 units".into());
            }
            out.extend(expanded);
            at = end + 1;
        } else {
            if out.len() == MAX_UNITS {
                return Err("Expanded font path exceeds 32768 UTF-16 units".into());
            }
            if let Some(c) = value.get(at) {
                out.push(*c);
            }
            at += 1;
        }
    }
    Ok(out)
}

#[cfg(windows)]
pub(crate) fn discover() -> crate::platform::Discovery {
    use std::path::PathBuf;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
    let system = PathBuf::from(std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into())).join("Fonts");
    let user = std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Fonts"));
    let mut found = crate::platform::Discovery::default();
    registry_fonts(&RegKey::predef(HKEY_LOCAL_MACHINE), KEY, &system, &mut found);
    if let Some(user) = user {
        registry_fonts(&RegKey::predef(HKEY_CURRENT_USER), KEY, &user, &mut found);
    } else {
        found.note(std::path::Path::new("HKCU/Fonts"), "LOCALAPPDATA is unavailable; relative user font paths cannot be resolved".into());
    }
    found.files.sort();
    found.files.dedup();
    found
}

#[cfg(windows)]
fn registry_fonts(root: &winreg::RegKey, key: &str, folder: &std::path::Path, out: &mut crate::platform::Discovery) {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Component, Path, PathBuf};
    use winreg::enums::{KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_EXPAND_SZ, REG_SZ};

    for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
        let registry = match root.open_subkey_with_flags(key, KEY_READ | view) {
            Ok(k) => k,
            // Older systems may have no per-user key or alternate view.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                out.note(Path::new(key), format!("Cannot enumerate font registry: {e}"));
                continue;
            }
        };
        for (i, entry) in registry.enum_values().enumerate() {
            if i == 65536 {
                out.note(Path::new(key), "Font registry limit reached (65536 entries per view)".into());
                break;
            }
            let (name, value) = match entry {
                Ok(v) => v,
                Err(e) => {
                    out.note(Path::new(key), format!("Cannot read font registry value: {e}"));
                    continue;
                }
            };
            let decode = || -> Result<PathBuf, String> {
                if !matches!(value.vtype, REG_SZ | REG_EXPAND_SZ) {
                    return Err("Font registry value is not a string".into());
                }
                if value.bytes.len() > 65536 || value.bytes.len() % 2 != 0 {
                    return Err("Malformed or oversized UTF-16 font path".into());
                }
                let mut units: Vec<_> = value.bytes.as_chunks::<2>().0.iter().map(|b| u16::from_le_bytes(*b)).collect();
                while units.last() == Some(&0) {
                    units.pop();
                }
                if units.is_empty() || units.contains(&0) {
                    return Err("Empty font path or embedded NUL".into());
                }
                if value.vtype == REG_EXPAND_SZ {
                    units = expand_path(&units, |name| std::env::var_os(OsString::from_wide(name)).map(|v| v.encode_wide().collect()))?;
                }
                let path = PathBuf::from(OsString::from_wide(&units));
                if path.is_absolute() {
                    Ok(path)
                } else if path.has_root() || matches!(path.components().next(), Some(Component::Prefix(_))) {
                    Err("Ambiguous drive-relative font path".into())
                } else {
                    Ok(folder.join(path))
                }
            };
            match decode() {
                Ok(path) => out.files.push(path),
                Err(e) => out.note(Path::new(key), format!("{name}: {e}")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_paths_preserve_unicode_and_expand_once() {
        let value: Vec<_> = r"%HOME%\ಕನ್ನಡ\font.otc".encode_utf16().collect();
        let out = expand_path(&value, |name| (String::from_utf16_lossy(name) == "HOME").then(|| r"C:\%literal%".encode_utf16().collect())).unwrap();
        assert_eq!(String::from_utf16(&out).unwrap(), r"C:\%literal%\ಕನ್ನಡ\font.otc");
        assert!(expand_path(&value, |_| None).is_err());
        assert_eq!(expand_path(&[0xd800, 37, 37], |_| None).unwrap(), [0xd800, 37, 37]);
        assert!(expand_path(&vec![1; 32769], |_| None).is_err());
        assert!(expand_path(&value, |_| Some(vec![1; 32768])).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn isolated_registry_discovers_relative_and_external_collection_paths() {
        use std::path::{Path, PathBuf};
        use winreg::enums::{HKEY_CURRENT_USER, REG_EXPAND_SZ};
        use winreg::{RegKey, RegValue};
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let name = format!(r"Software\PagenorFontTests-{}", std::process::id());
        let (key, _) = root.create_subkey(&name).unwrap();
        key.set_value("Relative", &"font.otf").unwrap();
        key.set_value("External", &r"C:\Pagenor\ಕನ್ನಡ\collection.otc").unwrap();
        let bytes = "%WINDIR%\\Fonts\\variable.ttf\0".encode_utf16().flat_map(u16::to_le_bytes).collect();
        key.set_raw_value("Expanded", &RegValue { bytes, vtype: REG_EXPAND_SZ }).unwrap();
        key.set_value("Wrong type", &123u32).unwrap();
        let mut found = crate::platform::Discovery::default();
        registry_fonts(&root, &name, Path::new(r"C:\TestFonts"), &mut found);
        drop(key);
        root.delete_subkey_all(&name).unwrap();
        assert!(found.files.contains(&PathBuf::from(r"C:\TestFonts\font.otf")));
        assert!(found.files.contains(&PathBuf::from(r"C:\Pagenor\ಕನ್ನಡ\collection.otc")));
        let windows = PathBuf::from(std::env::var_os("WINDIR").unwrap()).join(r"Fonts\variable.ttf");
        assert!(found.files.contains(&windows));
        assert!(found.issues.iter().any(|(_, r)| r.contains("Wrong type")));
    }
}
