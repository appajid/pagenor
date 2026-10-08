//! macOS registry discovery through safe Core Text wrappers; no font bytes are loaded here.

use std::path::PathBuf;

/// Available descriptors include fonts activated by Font Book or other font managers, even
/// when their files are outside standard directories. Fonts without a readable SFNT file
/// cannot be used by the existing Rust shaping engine and are left out of the catalogue.
pub(crate) struct ActivatedFonts {
    pub paths: Vec<PathBuf>,
    pub pathless_faces: usize,
}

pub(crate) fn activated_fonts() -> ActivatedFonts {
    let mut paths = Vec::new();
    let mut names = std::collections::HashSet::new();
    let mut pathless_faces = 0;
    if let Some(descriptors) = core_text::font_collection::create_for_all_families().get_descriptors() {
        for descriptor in descriptors.iter() {
            names.insert(descriptor.font_name());
            match descriptor.font_path() {
                Some(path) => paths.push(path),
                None => pathless_faces += 1,
            }
        }
    }
    // Query registered PostScript names as a complementary source.
    // Keep the collection paths (including variants sharing a PostScript name); only resolve
    // additional names here. These safe wrappers request descriptors, not font file bytes.
    for name in core_text::font_collection::get_postscript_names().iter() {
        if names.insert(name.to_string()) {
            let descriptor = core_text::font_descriptor::new_from_postscript_name(&name);
            match descriptor.font_path() {
                Some(path) => paths.push(path),
                None => pathless_faces += 1,
            }
        }
    }
    paths.sort();
    paths.dedup();
    ActivatedFonts { paths, pathless_faces }
}
