//! macOS registry discovery through safe Core Text wrappers; no font bytes are loaded here.

use std::path::PathBuf;

/// Available descriptors include fonts activated by Font Book or other font managers, even
/// when their files are outside standard directories. Fonts without a readable SFNT file
/// cannot be used by the existing Rust shaping engine and are left out of the catalogue.
pub(crate) fn activated_font_paths() -> Vec<PathBuf> {
    let collection = core_text::font_collection::create_for_all_families();
    let Some(descriptors) = collection.get_descriptors() else { return Vec::new() };
    let mut paths: Vec<_> = descriptors.iter().filter_map(|descriptor| descriptor.font_path()).collect();
    paths.sort();
    paths.dedup();
    paths
}
