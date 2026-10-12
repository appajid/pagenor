//! Placed graphics: format sniffing, sizes and decoding for the renderer and PDF export.
//! Rasters go through `image` (PNG, JPEG, GIF, WebP, TIFF, BMP) or [`psd`] (Photoshop's merged
//! composite); SVG is parsed with usvg (text set in document, bundled and installed fonts) and rasterised with resvg
//! for the screen — PDF export draws the same tree as vectors.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::sync::Arc;

use pagenor_fonts::{FontDb, FontFace, ScopedFonts};

pub use resvg::usvg;

mod eps;
mod metadata;
mod psd;
pub use eps::{bounding_box as eps_bounding_box, eps_pixel_size, eps_png, eps_proxy, eps_size, is_eps};
pub use metadata::{ColorInfo, color_info, color_mode};

/// CSS pixels (SVG user units) → points.
pub const PT_PER_PX: f64 = 0.75;

pub fn is_pdf(bytes: &[u8]) -> bool {
    bytes.starts_with(b"%PDF")
}

pub fn is_psd(bytes: &[u8]) -> bool {
    bytes.starts_with(b"8BPS")
}

/// An SVG document (optionally after an XML declaration, comments or a doctype).
pub fn is_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(4096)];
    let s = String::from_utf8_lossy(head);
    let t = s.trim_start_matches('\u{feff}').trim_start();
    t.starts_with('<') && t.contains("<svg")
}

/// MIME type of a placed graphic.
pub fn mime(bytes: &[u8]) -> &'static str {
    if is_eps(bytes) {
        return "application/postscript";
    }
    if is_pdf(bytes) {
        return "application/pdf";
    }
    if is_psd(bytes) {
        return "image/vnd.adobe.photoshop";
    }
    if is_svg(bytes) {
        return "image/svg+xml";
    }
    match image::guess_format(bytes) {
        Ok(image::ImageFormat::Png) => "image/png",
        Ok(image::ImageFormat::Jpeg) => "image/jpeg",
        Ok(image::ImageFormat::Gif) => "image/gif",
        Ok(image::ImageFormat::WebP) => "image/webp",
        Ok(image::ImageFormat::Tiff) => "image/tiff",
        Ok(image::ImageFormat::Bmp) => "image/bmp",
        _ => "application/octet-stream",
    }
}

/// Share only the font data needed by this SVG. Start with the document's default faces,
/// so a document replacement takes precedence even over a bundled family of the same name.
fn svg_options(fonts: ScopedFonts<'_>) -> usvg::Options<'_> {
    let mut db = Arc::new(usvg::fontdb::Database::new());
    for family in ["Source Serif 4", "Source Sans 3", "JetBrains Mono"] {
        load_family(&mut db, &fonts, family);
    }
    let fallback_fonts = fonts.clone();
    let loaded_fallback = usvg::FontResolver::default_fallback_selector();
    let font_resolver = usvg::FontResolver {
        select_font: Box::new(move |font, db| {
            // Respect family-list order; a PostScript name already selects a face's style.
            for family in font.families() {
                let name = match family {
                    usvg::FontFamily::Named(name) => name.as_str(),
                    usvg::FontFamily::Monospace => "JetBrains Mono",
                    usvg::FontFamily::SansSerif => "Source Sans 3",
                    _ => "Source Serif 4",
                };
                if let Some(canonical) = fonts.canonical_family(name) {
                    load_family(db, &fonts, &canonical);
                    if let Some(id) = db.query(&usvg::fontdb::Query {
                        families: &[usvg::fontdb::Family::Name(&canonical)],
                        weight: usvg::fontdb::Weight(font.weight()),
                        stretch: font.stretch().into(),
                        style: font.style().into(),
                    }) {
                        return Some(id);
                    }
                } else if let Some((family, style)) = fonts.postscript_face(name)
                    && let Some(id) = add_face(db, fonts.face(&family, &style))
                {
                    return Some(id);
                }
            }
            load_family(db, &fonts, "Source Serif 4");
            db.query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::Name("Source Serif 4")],
                weight: usvg::fontdb::Weight(font.weight()),
                stretch: font.stretch().into(),
                style: font.style().into(),
            })
        }),
        select_fallback: Box::new(move |c, exclude, db| {
            loaded_fallback(c, exclude, db).or_else(|| {
                let id = add_face(db, fallback_fonts.fallback_for(c, 0, None)?)?;
                (!exclude.contains(&id)).then_some(id)
            })
        }),
    };
    usvg::Options { fontdb: db, font_family: "Source Sans 3".into(), font_resolver, ..Default::default() }
}

fn load_family(db: &mut Arc<usvg::fontdb::Database>, fonts: &ScopedFonts<'_>, family: &str) {
    if !db.faces().any(|face| face.families.iter().any(|(name, _)| name == family)) && fonts.has_family(family) {
        for style in fonts.styles(family) {
            add_face(db, fonts.face(family, &style));
        }
    }
}

/// Own the Pagenor face rather than copying a file or native-service font's bytes.
struct FaceBytes(Arc<FontFace>);
impl AsRef<[u8]> for FaceBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.data()
    }
}

fn add_face(db: &mut Arc<usvg::fontdb::Database>, face: Arc<FontFace>) -> Option<usvg::fontdb::ID> {
    let same = |f: &usvg::fontdb::FaceInfo| {
        f.index == face.index()
            && matches!(&f.source, usvg::fontdb::Source::Binary(b)
            if std::ptr::eq(b.as_ref().as_ref().as_ptr(), face.data().as_ptr()))
    };
    if let Some(f) = db.faces().find(|f| same(f)) {
        return Some(f.id);
    }
    let index = face.index();
    // Parse collections privately: unrelated faces must not defeat a document replacement.
    // The SVG database only receives the face selected by Pagenor; mutation stays additive.
    let mut parsed = usvg::fontdb::Database::new();
    let ids = parsed.load_font_source(usvg::fontdb::Source::Binary(Arc::new(FaceBytes(face))));
    let info = ids.into_iter().find_map(|id| parsed.face(id).filter(|f| f.index == index).cloned())?;
    Some(Arc::make_mut(db).push_face_info(info))
}

/// Parse with the shared font database. Other open documents' fonts are excluded.
pub fn svg_tree(bytes: &[u8]) -> Option<usvg::Tree> {
    svg_tree_with_fonts(bytes, FontDb::global().scoped(0))
}

/// Parse with one document's font scope. usvg applies SVG variation settings to the original
/// variable font; Pagenor's named-instance coordinates are not substituted for SVG CSS.
pub fn svg_tree_with_fonts(bytes: &[u8], fonts: ScopedFonts<'_>) -> Option<usvg::Tree> {
    usvg::Tree::from_data(bytes, &svg_options(fonts)).ok()
}

/// Natural size of a placed graphic in points: rasters at 72 ppi (one pixel per point), PSDs
/// likewise, SVGs from their width/height (CSS pixels), PDFs are the caller's.
pub fn natural_size(bytes: &[u8]) -> Option<(f64, f64)> {
    if is_svg(bytes) {
        let s = svg_tree(bytes)?.size();
        return Some((s.width() as f64 * PT_PER_PX, s.height() as f64 * PT_PER_PX));
    }
    let (w, h) = pixel_size(bytes)?;
    Some((w as f64, h as f64))
}

/// Pixel dimensions of a raster.
pub fn pixel_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if is_psd(bytes) && bytes.len() >= 26 {
        // Header: signature, version, 6 reserved, channels, height, width (big-endian).
        let be = |i: usize| u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        return Some((be(18), be(14)));
    }
    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().ok()?.into_dimensions().ok()
}

/// Straight (non-premultiplied) RGBA8 of a raster.
pub fn decode_rgba(bytes: &[u8]) -> Option<image::RgbaImage> {
    if is_psd(bytes) {
        return psd::decode(bytes);
    }
    Some(image::load_from_memory(bytes).ok()?.to_rgba8())
}

/// A CMYK raster's ink values, 8 bits per ink (0 = no ink, 255 = solid), as PDF's DeviceCMYK
/// reads them.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CmykRaster {
    pub width: u32,
    pub height: u32,
    /// C, M, Y, K for each pixel, row by row: `width * height * 4` bytes.
    pub cmyk: Vec<u8>,
    /// Straight alpha, `width * height` bytes; `None` when the image is opaque.
    pub alpha: Option<Vec<u8>>,
}

fn tiff_decoder(bytes: &[u8]) -> Option<tiff::decoder::Decoder<std::io::Cursor<&[u8]>>> {
    let tiff = [b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"].iter().any(|m| bytes.starts_with(*m));
    if !tiff {
        return None;
    }
    tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).ok()
}

/// A TIFF whose pixels are CMYK (PhotometricInterpretation Separated, four inks).
pub fn is_cmyk_tiff(bytes: &[u8]) -> bool {
    tiff_decoder(bytes).and_then(|mut d| d.colortype().ok()).is_some_and(|c| matches!(c, tiff::ColorType::CMYK(_) | tiff::ColorType::CMYKA(_)))
}

/// The ink values of a CMYK TIFF, which [`decode_rgba`] converts to RGB. 8- or 16-bit CMYK,
/// pixels interleaved, opaque or with unassociated alpha. `None` for anything else — another
/// format, another ink set (InkSet 2), planar storage or premultiplied alpha — and for files
/// over the decoder's 256 MiB limit; [`is_cmyk_tiff`] tells those apart from RGB files.
pub fn decode_cmyk_tiff(bytes: &[u8]) -> Option<CmykRaster> {
    use tiff::tags::Tag;
    let mut d = tiff_decoder(bytes)?;
    let has_alpha = match d.colortype().ok()? {
        tiff::ColorType::CMYK(8 | 16) => false,
        tiff::ColorType::CMYKA(8 | 16) => true,
        _ => return None,
    };
    // InkSet (tag 332): 1 or absent = CMYK; 2 = other inks, named in InkNames.
    if d.find_tag_unsigned::<u16>(Tag::Unknown(332)).ok()?.is_some_and(|s| s != 1) {
        return None;
    }
    // ExtraSamples 2 = unassociated alpha. Premultiplied CMYK has no meaning as ink values.
    if has_alpha && d.find_tag_unsigned_vec::<u16>(Tag::ExtraSamples).ok()?.and_then(|v| v.first().copied()) != Some(2) {
        return None;
    }
    let (w, h) = d.dimensions().ok()?;
    let pixels = usize::try_from(w).ok()?.checked_mul(usize::try_from(h).ok()?)?;
    let samples = if has_alpha { 5 } else { 4 };
    let px = match d.read_image().ok()? {
        tiff::decoder::DecodingResult::U8(v) => v,
        tiff::decoder::DecodingResult::U16(v) => v.iter().map(|&s| ((u32::from(s) * 255 + 32_767) / 65_535) as u8).collect(),
        _ => return None,
    };
    // A planar file reads back as its first plane only, and extra non-alpha samples make the
    // pixels wider: either way the length is not `pixels * samples`.
    if px.len() != pixels.checked_mul(samples)? {
        return None;
    }
    if !has_alpha {
        return Some(CmykRaster { width: w, height: h, cmyk: px, alpha: None });
    }
    let mut cmyk = Vec::with_capacity(pixels.checked_mul(4)?);
    let mut alpha = Vec::with_capacity(pixels);
    for &[c, m, y, k, a] in px.as_chunks::<5>().0 {
        cmyk.extend([c, m, y, k]);
        alpha.push(a);
    }
    let alpha = alpha.iter().any(|&a| a < 255).then_some(alpha);
    Some(CmykRaster { width: w, height: h, cmyk, alpha })
}

/// An SVG rasterised so its longer side is `max_side` pixels: premultiplied RGBA8.
pub fn render_svg(bytes: &[u8], max_side: u32) -> Option<(Vec<u8>, u32, u32)> {
    render_svg_with_fonts(bytes, max_side, FontDb::global().scoped(0))
}

/// Rasterise with the font scope used by document composition and PDF export.
pub fn render_svg_with_fonts(bytes: &[u8], max_side: u32, fonts: ScopedFonts<'_>) -> Option<(Vec<u8>, u32, u32)> {
    let tree = svg_tree_with_fonts(bytes, fonts)?;
    let s = tree.size();
    let k = max_side as f32 / s.width().max(s.height()).max(1e-3);
    let (w, h) = ((s.width() * k).round().max(1.0) as u32, (s.height() * k).round().max(1.0) as u32);
    let mut pm = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(k, k), &mut pm.as_mut());
    Some((pm.take(), w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 100">
  <rect x="0" y="0" width="100" height="100" fill="#ff0000"/>
  <text x="110" y="60" font-size="20">Hi</text>
</svg>"##;

    #[test]
    fn svg_sniff_size_and_render() {
        let b = SVG.as_bytes();
        assert!(is_svg(b));
        assert_eq!(mime(b), "image/svg+xml");
        assert_eq!(natural_size(b), Some((150.0, 75.0)));
        let (px, w, h) = render_svg(b, 400).unwrap();
        assert_eq!((w, h), (400, 200));
        // Red on the left; the text drew some ink on the right.
        assert_eq!(&px[(100 * 400 + 100) * 4..][..4], &[255, 0, 0, 255]);
        assert!(px.chunks(4).skip(220).step_by(7).any(|p| p[3] > 128 && p[0] < 100), "text pixels");
        assert!(!is_svg(b"\x89PNG"));
    }

    fn text_svg(family: &str, body: &str) -> Vec<u8> {
        format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text y="30" font-size="20" font-family="{family}">{body}</text></svg>"#).into_bytes()
    }

    fn glyph_families(tree: &usvg::Tree) -> Vec<(String, u16)> {
        fn walk(group: &usvg::Group, tree: &usvg::Tree, out: &mut Vec<(String, u16)>) {
            for node in group.children() {
                match node {
                    usvg::Node::Group(group) => walk(group, tree, out),
                    usvg::Node::Text(text) => {
                        for glyph in text.layouted().iter().flat_map(|span| &span.positioned_glyphs) {
                            let face = tree.fontdb().face(glyph.font).unwrap();
                            out.push((face.families[0].0.clone(), glyph.id.0));
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(tree.root(), tree, &mut out);
        out
    }

    #[test]
    fn svg_named_and_fallback_text_share_memory_font_bytes_without_duplicates() {
        let fonts = FontDb::with_font_dirs(Vec::new());
        assert_eq!(fonts.add_font(pagenor_fonts::testing::font_with("Pagenor SVG Hangul", &['한', '글']).unwrap()), 1);
        let svg = text_svg("pAgEnOr SvG hAnGuL", "한글");
        let tree = svg_tree_with_fonts(&svg, fonts.scoped(0)).unwrap();
        let glyphs = glyph_families(&tree);
        assert_eq!(glyphs.len(), 2);
        assert!(glyphs.iter().all(|(name, gid)| name == "Pagenor SVG Hangul" && *gid != 0));
        let face = fonts.face("Pagenor SVG Hangul", "Regular");
        let mut db = Arc::new(usvg::fontdb::Database::new());
        let id = add_face(&mut db, face.clone()).unwrap();
        assert_eq!(add_face(&mut db, face.clone()), Some(id));
        assert_eq!(db.faces().count(), 1);
        db.with_face_data(id, |data, index| {
            assert!(std::ptr::eq(data.as_ptr(), face.data().as_ptr()));
            assert_eq!(index, face.index());
        })
        .unwrap();
        let fallback = svg_tree_with_fonts(&text_svg("Source Sans 3", "한글"), fonts.scoped(0)).unwrap();
        assert!(glyph_families(&fallback).iter().all(|(_, gid)| *gid != 0));
        // The SVG retains the bytes even after the originating database is dropped.
        drop(fonts);
        assert_eq!(glyph_families(&tree), glyphs);
    }

    #[test]
    fn svg_uses_installed_fonts_and_keeps_two_document_replacements_isolated() {
        let folder = std::env::temp_dir().join(format!("pagenor-svg-scopes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("installed")).unwrap();
        for (name, glyph) in [("one", 'X'), ("two", 'W')] {
            std::fs::create_dir_all(folder.join(name)).unwrap();
            std::fs::write(folder.join(name).join("font.otf"), pagenor_fonts::testing::font_with_glyph("Source Sans 3", &['A'], glyph).unwrap())
                .unwrap();
        }
        std::fs::write(folder.join("installed/font.ttf"), pagenor_fonts::testing::font_with("Pagenor SVG Installed", &['A']).unwrap()).unwrap();
        let fonts = FontDb::with_font_dirs(vec![folder.join("installed")]);
        let installed = svg_tree_with_fonts(&text_svg("Pagenor SVG Installed", "A"), fonts.scoped(0)).unwrap();
        assert_eq!(glyph_families(&installed)[0].0, "Pagenor SVG Installed");
        let one = fonts.load_document_fonts(&folder.join("one"));
        let two = fonts.load_document_fonts(&folder.join("two"));
        let svg = text_svg("Source Sans 3", "A");
        let first = glyph_families(&svg_tree_with_fonts(&svg, fonts.scoped(one.scope)).unwrap());
        let second = glyph_families(&svg_tree_with_fonts(&svg, fonts.scoped(two.scope)).unwrap());
        let global = glyph_families(&svg_tree_with_fonts(&svg, fonts.scoped(0)).unwrap());
        assert_ne!(first[0].1, second[0].1);
        assert_ne!(first[0].1, global[0].1);
        assert_ne!(second[0].1, global[0].1);
        assert_eq!(glyph_families(&svg_tree_with_fonts(&svg, fonts.scoped(one.scope)).unwrap()), first);
        let ps = text_svg("SourceSans3-Regular", "A");
        assert_eq!(glyph_families(&svg_tree_with_fonts(&ps, fonts.scoped(one.scope)).unwrap()), first);
        assert_eq!(glyph_families(&svg_tree_with_fonts(&ps, fonts.scoped(two.scope)).unwrap()), second);
        fonts.close_scope(one.scope);
        assert_eq!(glyph_families(&svg_tree_with_fonts(&svg, fonts.scoped(one.scope)).unwrap()), global);
        assert_eq!(glyph_families(&svg_tree_with_fonts(&ps, fonts.scoped(one.scope)).unwrap()), global);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn svg_collection_selects_the_requested_face_index_once() {
        let faces = [
            pagenor_fonts::testing::font_with("Pagenor SVG Collection One", &['A']).unwrap(),
            pagenor_fonts::testing::font_with("Pagenor SVG Collection Two", &['A']).unwrap(),
        ];
        let mut bytes = b"ttcf".to_vec();
        bytes.extend_from_slice(&0x0001_0000_u32.to_be_bytes());
        bytes.extend_from_slice(&2_u32.to_be_bytes());
        let mut base = 20;
        for face in &faces {
            bytes.extend_from_slice(&(base as u32).to_be_bytes());
            base += face.len();
        }
        for face in faces {
            let start = bytes.len() as u32;
            let mut face = face;
            let tables = u16::from_be_bytes([face[4], face[5]]) as usize;
            for table in 0..tables {
                let at = 12 + table * 16 + 8;
                let offset = u32::from_be_bytes(face[at..at + 4].try_into().unwrap()) + start;
                face[at..at + 4].copy_from_slice(&offset.to_be_bytes());
            }
            bytes.extend_from_slice(&face);
        }
        let fonts = FontDb::with_font_dirs(Vec::new());
        assert_eq!(fonts.add_font(bytes), 2);
        let face = fonts.face("Pagenor SVG Collection Two", "Regular");
        assert_eq!(face.index(), 1);
        let mut db = Arc::new(usvg::fontdb::Database::new());
        let id = add_face(&mut db, face.clone()).unwrap();
        assert_eq!(db.face(id).unwrap().index, 1);
        assert_eq!(add_face(&mut db, face), Some(id));
        assert_eq!(db.faces().count(), 1);
        let tree = svg_tree_with_fonts(&text_svg("Pagenor SVG Collection Two", "A"), fonts.scoped(0)).unwrap();
        assert_eq!(glyph_families(&tree)[0].0, "Pagenor SVG Collection Two");
    }

    /// The PostScript name of the face that sets the text of `svg` (one face for all of it).
    fn text_face(svg_attrs: &str) -> String {
        let svg =
            format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text y="30" font-size="20"{svg_attrs}>ab</text></svg>"#);
        let tree = svg_tree(svg.as_bytes()).unwrap();
        let mut faces = Vec::new();
        for n in tree.root().children() {
            if let usvg::Node::Text(t) = n {
                for g in t.layouted().iter().flat_map(|s| &s.positioned_glyphs) {
                    faces.extend(tree.fontdb().face(g.font).map(|f| f.post_script_name.clone()));
                }
            }
        }
        faces.dedup();
        assert_eq!(faces.len(), 1, "{faces:?}");
        faces.remove(0)
    }

    #[test]
    fn svg_text_finds_fonts_by_postscript_name() {
        // Bundled, as Illustrator's SVG export names it; any case.
        assert_eq!(text_face(r#" font-family="'SourceSerif4-Semibold'""#), "SourceSerif4-Semibold");
        assert_eq!(text_face(r#" font-family="sourceserif4-semibold""#), "SourceSerif4-Semibold");
        // A shared font (as an installed one would be).
        let font = pagenor_fonts::testing::font_with("Pagenor Test SVG PostScript", &['a', 'b']).unwrap();
        pagenor_fonts::FontDb::global().add_font(font);
        assert_eq!(text_face(r#" font-family="PagenorTestSVGPostScript-Regular""#), "PagenorTestSVGPostScript-Regular");
        // The name picks that face: font-weight and font-style don't change it.
        assert_eq!(text_face(r#" font-family="SourceSerif4-Semibold" font-weight="bold" font-style="italic""#), "SourceSerif4-Semibold");
        // A family listed before it still comes first; one after it is a fallback.
        assert_eq!(text_face(r#" font-family="'Source Sans 3', SourceSerif4-Bold""#), "SourceSans3-Regular");
        assert_eq!(text_face(r#" font-family="NoSuch-Font, SourceSerif4-Bold, 'Source Sans 3'""#), "SourceSerif4-Bold");
        // Family names work as before.
        assert_eq!(text_face(r#" font-family="'Source Serif 4'" font-weight="bold""#), "SourceSerif4-Bold");
        // An unknown name falls back as before (the serif default).
        assert_eq!(text_face(r#" font-family="NoSuch-Font""#), text_face(r#" font-family="No Such Font""#));
        assert_eq!(text_face(r#" font-family="NoSuch-Font""#), "SourceSerif4-Regular");
    }

    /// A minimal Photoshop file: 2×1 RGB, no layers, raw merged image.
    fn tiny_psd() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(b"8BPS");
        b.extend(1u16.to_be_bytes());
        b.extend([0u8; 6]);
        b.extend(3u16.to_be_bytes()); // channels
        b.extend(1u32.to_be_bytes()); // height
        b.extend(2u32.to_be_bytes()); // width
        b.extend(8u16.to_be_bytes()); // depth
        b.extend(3u16.to_be_bytes()); // RGB
        b.extend(0u32.to_be_bytes()); // colour mode data
        b.extend(0u32.to_be_bytes()); // image resources
        b.extend(0u32.to_be_bytes()); // layer and mask info
        b.extend(0u16.to_be_bytes()); // raw
        b.extend([255, 0, 0, 255, 0, 0]); // R, G, B planes: a red pixel, then a green one
        b
    }

    #[test]
    fn psd_composite_decodes() {
        let b = tiny_psd();
        assert_eq!(mime(&b), "image/vnd.adobe.photoshop");
        assert_eq!(pixel_size(&b), Some((2, 1)));
        let img = decode_rgba(&b).unwrap();
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 255, 0, 255]);
    }

    fn tiff_of<C: tiff::encoder::colortype::ColorType>(w: u32, h: u32, px: &[C::Inner], alpha: bool) -> Vec<u8>
    where
        [C::Inner]: tiff::encoder::TiffValue,
    {
        let mut b = Vec::new();
        let mut enc = tiff::encoder::TiffEncoder::new(std::io::Cursor::new(&mut b)).unwrap();
        let mut img = enc.new_image::<C>(w, h).unwrap();
        if alpha {
            // CMYKA8 already has five samples: only the tag that says the fifth is alpha.
            img.encoder().write_tag(tiff::tags::Tag::ExtraSamples, &[2u16][..]).unwrap();
        }
        img.write_data(px).unwrap();
        b
    }

    /// CMYK TIFFs read back as their ink values; RGB ones aren't CMYK.
    #[test]
    fn cmyk_tiff_keeps_inks() {
        use tiff::encoder::colortype::{CMYK8, CMYK16, CMYKA8, RGB8};
        // 2×1: 100% K, then C0 M100 Y100 K0.
        let px = [0, 0, 0, 255, 0, 255, 255, 0];
        let b = tiff_of::<CMYK8>(2, 1, &px, false);
        assert_eq!(mime(&b), "image/tiff");
        assert!(is_cmyk_tiff(&b));
        assert_eq!(decode_cmyk_tiff(&b), Some(CmykRaster { width: 2, height: 1, cmyk: px.to_vec(), alpha: None }));
        // The screen still gets RGB.
        assert_eq!(decode_rgba(&b).unwrap().get_pixel(0, 0).0, [0, 0, 0, 255]);

        let b = tiff_of::<CMYK16>(2, 1, &[0, 0, 0, 65_535, 0, 32_896, 65_535, 0], false);
        assert_eq!(decode_cmyk_tiff(&b).unwrap().cmyk, [0, 0, 0, 255, 0, 128, 255, 0]);

        let b = tiff_of::<CMYKA8>(2, 1, &[0, 0, 0, 255, 255, 0, 255, 255, 0, 0], true);
        let r = decode_cmyk_tiff(&b).unwrap();
        assert_eq!((r.cmyk, r.alpha), (px.to_vec(), Some(vec![255, 0])));

        let rgb = tiff_of::<RGB8>(1, 1, &[255, 0, 0], false);
        assert!(!is_cmyk_tiff(&rgb));
        assert_eq!(decode_cmyk_tiff(&rgb), None);
        assert!(!is_cmyk_tiff(b"II*\0"), "truncated");
        assert_eq!(decode_cmyk_tiff(&b[..b.len() / 2]), None, "truncated");
    }

    #[test]
    fn bmp_decodes() {
        let mut bmp = Vec::new();
        image::RgbaImage::from_pixel(3, 2, image::Rgba([0, 0, 255, 255]))
            .write_to(&mut std::io::Cursor::new(&mut bmp), image::ImageFormat::Bmp)
            .unwrap();
        assert_eq!(mime(&bmp), "image/bmp");
        assert_eq!(pixel_size(&bmp), Some((3, 2)));
        assert_eq!(decode_rgba(&bmp).unwrap().get_pixel(1, 1).0, [0, 0, 255, 255]);
    }
}
