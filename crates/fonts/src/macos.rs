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
    let collection = core_text::font_collection::create_for_all_families();
    let Some(descriptors) = collection.get_descriptors() else { return ActivatedFonts { paths: Vec::new(), pathless_faces: 0 } };
    let mut paths: Vec<_> = descriptors.iter().filter_map(|descriptor| descriptor.font_path()).collect();
    let pathless_faces = usize::try_from(descriptors.len()).unwrap_or(0).saturating_sub(paths.len());
    paths.sort();
    paths.dedup();
    ActivatedFonts { paths, pathless_faces }
}
