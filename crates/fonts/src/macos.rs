//! macOS discovery and lazy table access through safe Core Text wrappers.

use std::path::PathBuf;

/// Available descriptors include fonts activated by Font Book or other font managers, even
/// when their files are outside standard directories. OS identities also support fonts
/// without a readable SFNT file, when Core Text permits table access.
pub(crate) struct ActivatedFonts {
    pub paths: Vec<PathBuf>,
    pub fonts: Vec<crate::platform::NativeFont>,
    pub pathless_faces: usize,
}

pub(crate) fn activated_fonts() -> ActivatedFonts {
    let mut paths = Vec::new();
    let mut fonts = Vec::new();
    let mut names = std::collections::HashSet::new();
    let mut pathless_faces = 0;
    if let Some(descriptors) = core_text::font_collection::create_for_all_families().get_descriptors() {
        for descriptor in descriptors.iter() {
            names.insert(descriptor.font_name());
            let path = descriptor.font_path();
            fonts.push(crate::platform::NativeFont { name: descriptor.font_name(), path: path.clone() });
            match path {
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
            let path = descriptor.font_path();
            fonts.push(crate::platform::NativeFont { name: descriptor.font_name(), path: path.clone() });
            match path {
                Some(path) => paths.push(path),
                None => pathless_faces += 1,
            }
        }
    }
    fonts.sort();
    fonts.dedup();
    paths.sort();
    paths.dedup();
    ActivatedFonts { paths, fonts, pathless_faces }
}

/// Re-query by identity at use time. Never substitute an unrelated font when activation ended.
pub(crate) fn font_data(name: &str, metadata: bool) -> Result<Vec<u8>, String> {
    let font = core_text::font::new_from_name(name, 16.0).map_err(|()| format!("Core Text cannot resolve {name}"))?;
    if font.postscript_name() != name {
        return Err(format!("Native font is no longer available: {name}"));
    }
    copy_font_data(&font, metadata)
}

/// No file or persistent cache is created. Metadata reads exclude glyph/outline tables.
pub(crate) fn copy_font_data(font: &core_text::font::CTFont, metadata: bool) -> Result<Vec<u8>, String> {
    const META_TAGS: &[[u8; 4]] = &[*b"name", *b"OS/2", *b"meta", *b"fvar", *b"cmap"];
    let tags = font.get_available_font_tables().ok_or("Core Text exposes no font tables")?;
    if tags.len() > 256 {
        return Err("Native font has more than 256 tables".into());
    }
    let mut total = 0usize;
    let mut tables = Vec::new();
    for tag in tags.iter() {
        let tag_bytes = tag.to_be_bytes();
        if tag_bytes == *b"DSIG" || (metadata && !META_TAGS.contains(&tag_bytes)) {
            continue;
        }
        let data = font.get_font_table(*tag).ok_or_else(|| format!("Core Text cannot expose table {}", String::from_utf8_lossy(&tag_bytes)))?;
        let bytes = data.bytes();
        let limit = if metadata { 1 << 20 } else { 64 << 20 };
        total = total.checked_add(bytes.len()).ok_or("Native font size overflow")?;
        if bytes.len() > limit || total > if metadata { 4 << 20 } else { 64 << 20 } {
            return Err("Native font tables exceed the supported memory limit".into());
        }
        let mut data = bytes.to_vec();
        if tag_bytes == *b"head" {
            data.get_mut(8..12).ok_or("Native head table is truncated")?.fill(0);
        }
        tables.push((tag_bytes, data));
    }
    // The existing metadata assembler supplies checked offsets and four-byte alignment.
    // Real font data also needs the CFF signature, table checksums and head adjustment.
    let cff = tables.iter().any(|(tag, _)| tag == b"CFF " || tag == b"CFF2");
    tables.sort_by_key(|t| t.0);
    let records: Vec<_> = tables.iter().map(|(tag, data)| (*tag, checksum(data))).collect();
    let mut out = crate::group::sfnt(tables).ok_or("Cannot assemble native font tables")?;
    if cff {
        out.get_mut(..4).ok_or("Native font header is truncated")?.copy_from_slice(b"OTTO");
    }
    let mut head = None;
    for (i, (tag, sum)) in records.iter().enumerate() {
        let record = 12 + 16 * i;
        out.get_mut(record + 4..record + 8).ok_or("Native table record is truncated")?.copy_from_slice(&sum.to_be_bytes());
        if tag == b"head" {
            let offset =
                u32::from_be_bytes(out.get(record + 8..record + 12).and_then(|s| s.try_into().ok()).ok_or("Native table offset is truncated")?);
            head = Some(offset as usize);
        }
    }
    if let Some(offset) = head {
        let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&out));
        out.get_mut(offset + 8..offset + 12).ok_or("Native head adjustment is truncated")?.copy_from_slice(&adjustment.to_be_bytes());
    }
    Ok(out)
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0; 4];
        for (out, input) in word.iter_mut().zip(chunk) {
            *out = *input;
        }
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_table_data_has_valid_checksum_and_never_substitutes_a_missing_font() {
        let native = core_text::font::new_from_buffer(crate::bundled()[0]).unwrap();
        let data = copy_font_data(&native, false).unwrap();
        assert_eq!(checksum(&data), 0xb1b0_afba);
        assert!(skrifa::FontRef::new(&data).is_ok());
        assert!(font_data("Pagenor-Definitely-Unregistered-Font-Regular", false).is_err());
    }
}
