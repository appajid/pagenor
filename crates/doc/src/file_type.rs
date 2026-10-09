//! Native document naming shared by hosts, commands and document variables.

/// Preferred extension for newly created Pagenor documents (without the dot).
pub const EXTENSION: &str = "pgnr";
/// The previous extension stays readable, including through file pickers/drop.
pub const EXTENSIONS: &[&str] = &[EXTENSION, "pagenor"];
/// Stable format identity: changing the filename does not change the ZIP format.
pub const MIME: &str = "application/vnd.pagenor+zip";

pub fn is_native_name(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, extension)| EXTENSIONS.iter().any(|e| extension.eq_ignore_ascii_case(e)))
}
