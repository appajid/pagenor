//! Installed system fonts: found by family name by every lookup, whatever ran before it (#22).

use std::path::{Path, PathBuf};

use super::*;

const FAMILY: &str = "Sysfont Sans3";

/// A bundled Source Sans 3 file renamed [`FAMILY`] (as long as the original name).
fn renamed(file: &str) -> Vec<u8> {
    let mut data = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts").join(file)).unwrap();
    let utf16 = |s: &str| s.encode_utf16().flat_map(u16::to_be_bytes).collect::<Vec<u8>>();
    for (from, to) in [(utf16("Source Sans 3"), utf16(FAMILY)), (b"Source Sans 3".to_vec(), FAMILY.as_bytes().to_vec())] {
        let mut i = 0;
        while let Some(at) = data[i..].windows(from.len()).position(|w| w == from) {
            data[i + at..i + at + to.len()].copy_from_slice(&to);
            i += at + to.len();
        }
    }
    data
}

/// The fonts in `faces` as one collection (TTC) file.
fn collection(faces: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"ttcf".to_vec();
    out.extend_from_slice(&0x0001_0000_u32.to_be_bytes());
    out.extend_from_slice(&(faces.len() as u32).to_be_bytes());
    let mut base = 12 + 4 * faces.len();
    for f in faces {
        out.extend_from_slice(&(base as u32).to_be_bytes());
        base += f.len();
    }
    for f in faces {
        // Table offsets count from the start of the collection.
        let start = out.len() as u32;
        let mut f = f.clone();
        let tables = u16::from_be_bytes([f[4], f[5]]) as usize;
        for r in 0..tables {
            let at = 12 + r * 16 + 8;
            let off = u32::from_be_bytes(f[at..at + 4].try_into().unwrap()) + start;
            f[at..at + 4].copy_from_slice(&off.to_be_bytes());
        }
        out.extend_from_slice(&f);
    }
    out
}

/// A font folder (with a subfolder, a damaged font and a file that isn't a font) holding
/// [`FAMILY`] Regular and Bold.
fn font_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dc-sysfonts-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Sub")).unwrap();
    std::fs::write(dir.join("Sysfont-Regular.ttf"), renamed("SourceSans3-Regular.ttf")).unwrap();
    std::fs::write(dir.join("Sub/Sysfont-Bold.TTF"), renamed("SourceSans3-Bold.ttf")).unwrap();
    std::fs::write(dir.join("damaged.ttf"), b"ttcf\0\x01\0\0\xff\xff\xff\xff").unwrap();
    std::fs::write(dir.join("readme.txt"), FAMILY).unwrap();
    dir
}

fn has(list: &[String], family: &str) -> bool {
    list.iter().any(|f| f == family)
}

#[test]
fn every_lookup_by_name_finds_installed_fonts_in_a_fresh_database() {
    let dir = font_dir("fresh");
    // Each lookup is the first thing a new database is asked. A folder listed twice is read once.
    let db = || FontDb::with_font_dirs(vec![dir.clone(), dir.join("Sub/..")]);
    let f = db().face("sysfont sans3", "Bold");
    assert_eq!((f.family.as_str(), f.style.as_str()), (FAMILY, "Bold"));
    assert!(db().has_family(FAMILY));
    assert!(has(&db().families(), FAMILY));
    assert_eq!(db().styles(FAMILY), ["Regular", "Bold"]);
    assert_eq!(db().load_system_fonts(), 2);
    // Missing fonts stay missing.
    let db = db();
    assert!(!db.has_family("No Such Font"));
    assert_eq!(db.face("No Such Font", "Regular").family, FALLBACK_FAMILY);
}

#[test]
fn the_scan_reads_collections_and_loads_fonts_only_when_used() {
    let dir = font_dir("collection");
    std::fs::remove_file(dir.join("Sysfont-Regular.ttf")).unwrap();
    std::fs::remove_file(dir.join("Sub/Sysfont-Bold.TTF")).unwrap();
    std::fs::write(dir.join("Sysfont.ttc"), collection(&[renamed("SourceSans3-Regular.ttf"), renamed("SourceSans3-Bold.ttf")])).unwrap();
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    assert_eq!(db.styles(FAMILY), ["Regular", "Bold"]);
    // Cataloged, not loaded.
    assert!(!db.is_loaded(FAMILY));
    let bold = db.face(FAMILY, "Bold");
    assert_eq!((bold.style.as_str(), bold.index()), ("Bold", 1));
    assert_eq!(db.face(FAMILY, "Regular").index(), 0, "the collection's faces load together");
}

#[test]
fn rescanning_finds_fonts_installed_since() {
    let dir = font_dir("rescan");
    let later = dir.join("Later");
    let db = FontDb::with_font_dirs(vec![later.clone()]);
    assert!(!db.has_family(FAMILY));
    std::fs::create_dir_all(&later).unwrap();
    std::fs::copy(dir.join("Sysfont-Regular.ttf"), later.join("Sysfont-Regular.ttf")).unwrap();
    assert!(!db.has_family(FAMILY), "the folders are scanned once, not on every miss");
    assert_eq!(db.load_system_fonts(), 1);
    assert!(db.has_family(FAMILY) && has(&db.families(), FAMILY));
}

#[test]
fn rescanning_retries_a_previously_missing_glyph() {
    let dir = font_dir("fallback-rescan");
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    let latin = db.face(crate::DEFAULT_FAMILY, "Regular");
    let ch = '\u{e123}';
    assert!(db.fallback_for(ch, latin.id(), None).is_none());
    std::fs::write(dir.join("new.ttf"), crate::testing::font_with("Pagenor Rescan", &[ch]).unwrap()).unwrap();
    db.load_system_fonts();
    assert_eq!(db.fallback_for(ch, latin.id(), None).unwrap().family, "Pagenor Rescan");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scan_diagnostics_explain_rejections_and_duplicates_without_loading() {
    let dir = font_dir("diagnostics");
    std::fs::copy(dir.join("Sysfont-Regular.ttf"), dir.join("duplicate.otf")).unwrap();
    let missing = dir.join("absent");
    let db = FontDb::with_font_dirs(vec![dir.clone(), missing.clone(), dir.join("Sub/..")]);
    let report = db.scan_report();
    assert_eq!((report.files_scanned, report.families, report.faces), (4, 1, 2));
    assert_eq!((report.rejected_files, report.duplicate_faces, report.unavailable_paths), (1, 1, 1));
    assert!(report.issues.iter().any(|(p, reason)| p.ends_with("damaged.ttf") && reason.contains("metadata")));
    assert!(report.issues.iter().any(|(p, reason)| p == &missing && reason.contains("directory")));
    assert!(report.issues.iter().any(|(_, reason)| reason.contains("Duplicate") && reason.contains(FAMILY)));
    assert!(!db.is_loaded(FAMILY));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scan_diagnostics_cap_records_but_count_every_rejection() {
    let dir = font_dir("diagnostics-cap");
    for n in 0..300 {
        std::fs::write(dir.join(format!("broken-{n}.ttf")), b"broken").unwrap();
    }
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    let report = db.scan_report();
    assert_eq!(report.rejected_files, 301);
    assert_eq!(report.issues.len(), 256);
    assert_eq!(report.issues_omitted, 45);
    assert_eq!(report.faces, 2);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn automatic_refresh_finds_added_files_and_preserves_document_fonts() {
    let dir = font_dir("dynamic");
    let installed = dir.join("installed");
    let own = dir.join("document");
    std::fs::create_dir_all(&installed).unwrap();
    std::fs::create_dir_all(&own).unwrap();
    let family = "Pagenor Dynamic";
    std::fs::write(own.join("own.ttf"), crate::testing::font_with(family, &['a']).unwrap()).unwrap();
    let db = FontDb::with_font_dirs(vec![installed.clone()]);
    let document = db.load_document_fonts(&own);
    let face = db.scoped(document.scope).face(family, "Regular");
    db.scan_report();
    let before = db.scan_generation();
    assert!(!db.refresh_catalog(false).0);
    assert_eq!(db.scan_generation(), before, "an unchanged check does not rescan metadata");
    std::fs::write(installed.join("new.otf"), crate::testing::font_with(family, &['b']).unwrap()).unwrap();
    assert!(db.refresh_catalog(false).0);
    assert!(db.scan_generation() > before);
    assert!(db.has_family(family));
    assert!(!db.is_loaded(family), "automatic discovery retains the lazy catalogue");
    assert!(Arc::ptr_eq(&face, &db.scoped(document.scope).face(family, "Regular")));
    std::fs::remove_file(installed.join("new.otf")).unwrap();
    assert!(db.refresh_catalog(false).0);
    assert!(!db.has_family(family), "an unused installed face leaves the catalogue when removed");
    assert!(db.scoped(document.scope).has_family(family));
    assert!(Arc::ptr_eq(&face, &db.scoped(document.scope).face(family, "Regular")));
    db.add_font(crate::testing::font_with("Pagenor Memory", &['m']).unwrap());
    assert!(db.has_family("Pagenor Memory"));
    assert!(db.scoped(document.scope).has_family("Pagenor Memory"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn parallel_rescans_publish_complete_catalogues() {
    let dir = font_dir("parallel-rescan");
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    db.scan_report();
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..3 {
                    assert_eq!(db.load_system_fonts(), 2);
                    let r = db.scan_report();
                    assert_eq!((r.faces, r.families), (2, 1));
                    assert_eq!(db.styles(FAMILY), ["Regular", "Bold"]);
                }
            });
        }
    });
    assert!(!db.is_loaded(FAMILY));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_background_scan_serves_the_first_lookup() {
    let dir = font_dir("background");
    let db: &'static FontDb = Box::leak(Box::new(FontDb::with_font_dirs(vec![dir])));
    db.scan_in_background();
    // Waits for the scan when it is still running.
    assert!(has(&db.families(), FAMILY));
    assert_eq!(db.load_system_fonts(), 2, "a rescan catalogs the two faces again");
}

#[cfg(unix)]
#[test]
fn a_link_back_to_a_parent_folder_ends_the_scan() {
    let dir = font_dir("loop");
    std::os::unix::fs::symlink(&dir, dir.join("Sub/loop")).unwrap();
    let db = FontDb::with_font_dirs(vec![dir]);
    assert_eq!(db.load_system_fonts(), 2);
}

#[test]
fn the_scan_reads_names_without_loading_the_font() {
    let dir = font_dir("names");
    let names: Vec<(String, String)> = file_face_names(&dir.join("Sysfont-Regular.ttf")).into_iter().map(|f| (f.family, f.style)).collect();
    assert_eq!(names, [(FAMILY.to_string(), "Regular".to_string())]);
    assert!(file_face_names(&dir.join("damaged.ttf")).is_empty());
    assert!(file_face_names(&dir.join("readme.txt")).is_empty());
    assert!(file_face_names(&dir.join("missing.ttf")).is_empty());
}

#[test]
fn font_menus_group_installed_and_loaded_fonts_without_loading_them() {
    use crate::testing::{font_with, with_code_pages, with_names};
    use crate::{FamilyInfo, FontGroup};
    let dir = std::env::temp_dir().join(format!("dc-sysfonts-groups-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let names = |family: &str, lang: u16, native: &str| {
        let font = font_with(family, &['a']).unwrap();
        with_names(&font, &[(3, 1, 0x409, 1, family), (3, 1, 0x409, 2, "Regular"), (3, 1, lang, 1, native)]).unwrap()
    };
    std::fs::write(dir.join("jp.ttf"), names("Groups Mincho", 0x0411, "グループ明朝")).unwrap();
    // Declares nothing (no code pages): its kana make it Japanese, read from the cmap.
    std::fs::write(dir.join("kana.otf"), with_code_pages(font_with("Groups Kana", &['a', 'あ']).unwrap(), 0, 0).unwrap()).unwrap();
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    db.set_system_fallback(false);
    db.add_font(with_code_pages(font_with("Groups Ming", &['a']).unwrap(), 1 << 20, 0).unwrap());
    let infos = db.scoped(0).family_infos();
    let find = |f: &str| infos.iter().find(|i| i.family == f).cloned().unwrap_or_else(|| panic!("{f} in {infos:?}"));
    assert_eq!(find("Groups Mincho"), FamilyInfo { family: "Groups Mincho".into(), group: FontGroup::Japanese, native: Some("グループ明朝".into()) });
    assert_eq!(find("Groups Kana").group, FontGroup::Japanese);
    assert_eq!(find("Groups Ming").group, FontGroup::ChineseTraditional);
    assert_eq!(find(crate::DEFAULT_FAMILY).group, FontGroup::Western);
    assert!(!db.is_loaded("Groups Mincho"), "the scan read its tables, not the font");
    // Families are listed once.
    assert_eq!(infos.iter().filter(|i| i.family == "Groups Mincho").count(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn all_sfnt_extensions_and_duplicate_files_remain_lazy() {
    let dir = font_dir("extensions");
    let regular = renamed("SourceSans3-Regular.ttf");
    for ext in ["otf", "OTF", "ttc", "OTC"] {
        let data = if ext.eq_ignore_ascii_case("ttc") || ext.eq_ignore_ascii_case("otc") {
            collection(std::slice::from_ref(&regular))
        } else {
            regular.clone()
        };
        std::fs::write(dir.join(format!("duplicate.{ext}")), data).unwrap();
    }
    let db = FontDb::with_font_dirs(vec![dir.clone(), dir.join("Sub")]);
    assert_eq!(db.load_system_fonts(), 2);
    assert_eq!(db.styles(FAMILY), ["Regular", "Bold"]);
    assert!(!db.is_loaded(FAMILY));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn named_instances_are_cataloged_before_loading() {
    use crate::testing::{with_names, with_tables};
    let dir = font_dir("variable-catalog");
    let font = with_names(
        &renamed("SourceSans3-Regular.ttf"),
        &[
            (3, 1, 0x409, 1, FAMILY),
            (3, 1, 0x409, 2, "Regular"),
            (3, 1, 0x409, 256, "Weight"),
            (3, 1, 0x409, 257, "Light"),
            (3, 1, 0x409, 258, "Heavy"),
        ],
    )
    .unwrap();
    let mut fvar = Vec::new();
    for value in [1u16, 0, 16, 2, 1, 20, 2, 8] {
        fvar.extend_from_slice(&value.to_be_bytes());
    }
    fvar.extend_from_slice(b"wght");
    for value in [100i32, 400, 900] {
        fvar.extend_from_slice(&(value << 16).to_be_bytes());
    }
    fvar.extend_from_slice(&0u16.to_be_bytes());
    fvar.extend_from_slice(&256u16.to_be_bytes());
    for (id, weight) in [(257u16, 300i32), (258, 900)] {
        fvar.extend_from_slice(&id.to_be_bytes());
        fvar.extend_from_slice(&0u16.to_be_bytes());
        fvar.extend_from_slice(&(weight << 16).to_be_bytes());
    }
    let font = with_tables(&font, vec![(*b"fvar", fvar)]).unwrap();
    std::fs::remove_file(dir.join("Sysfont-Regular.ttf")).unwrap();
    std::fs::remove_file(dir.join("Sub/Sysfont-Bold.TTF")).unwrap();
    std::fs::write(dir.join("variable.otf"), font).unwrap();
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    assert_eq!(db.styles(FAMILY), ["Light", "Heavy"]);
    assert!(!db.is_loaded(FAMILY));
    assert_eq!(db.face(FAMILY, "Heavy").style, "Heavy");
    assert!(db.face(FAMILY, "Heavy").is_variable());
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(target_os = "macos")]
#[test]
fn host_registry_fonts_are_cataloged_without_loading() {
    let paths = crate::macos::activated_font_paths();
    assert!(!paths.is_empty(), "Core Text should report installed fonts on macOS");
    let folder_families = FontDb::with_font_dirs(system_font_dirs()).families();
    let bundled_families = FontDb::with_font_dirs(vec![]).families();
    let db = FontDb::with_system_fonts();
    // Verify every readable SFNT face in the host registry reaches the menu catalogue.
    let infos = db.scoped(0).family_infos();
    let added: Vec<_> = infos.iter().filter(|info| !folder_families.contains(&info.family)).map(|info| &info.family).collect();
    eprintln!("Core Text reports {} files; registry adds {} families beyond directory scanning: {added:?}", paths.len(), added.len());
    let mut usable = 0;
    for path in paths {
        let faces = file_face_names(&path);
        if faces.is_empty() {
            eprintln!("No usable SFNT metadata: {}", path.display());
        }
        for face in faces {
            usable += 1;
            assert!(infos.iter().any(|info| info.family.eq_ignore_ascii_case(&face.family)), "{}: {}", path.display(), face.family);
            assert!(db.styles(&face.family).iter().any(|style| style.eq_ignore_ascii_case(&face.style)));
            assert!(!db.is_loaded(&face.family) || bundled_families.contains(&face.family));
        }
    }
    assert!(usable > 0);
}
