//! Header-only colour metadata. Never infer source colour from decoded RGB pixels.

use image::ImageDecoder;
use std::io::Cursor;

const MAX_PROFILE: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorInfo {
    pub mode: &'static str,
    pub profile: String,
}

fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at.checked_add(2)?)?.try_into().ok()?))
}
fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

/// Stored raster mode. PDF/SVG/EPS may contain several colour spaces.
pub fn color_mode(b: &[u8]) -> &'static str {
    if super::is_psd(b) {
        if !matches!(be16(b, 4), Some(1 | 2)) {
            return "Unknown";
        }
        return match be16(b, 24) {
            Some(0) => "Bitmap",
            Some(1) => "Grayscale",
            Some(2) => "Indexed",
            Some(3) => "RGB",
            Some(4) => "CMYK",
            Some(7) => "Multichannel",
            Some(8) => "Duotone",
            Some(9) => "Lab",
            _ => "Unknown",
        };
    }
    if b.starts_with(b"\x89PNG\r\n\x1a\n") && b.get(12..16) == Some(b"IHDR") {
        return match b.get(25) {
            Some(0 | 4) => "Grayscale",
            Some(2 | 6) => "RGB",
            Some(3) => "Indexed",
            _ => "Unknown",
        };
    }
    if b.starts_with(b"\xff\xd8") {
        return jpeg_segments(b)
            .and_then(|segments| {
                segments.into_iter().find_map(|(marker, s)| {
                    if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
                        Some(match s.get(5) {
                            Some(1) => "Grayscale",
                            Some(3) => "RGB",
                            Some(4) => "CMYK",
                            _ => "Unknown",
                        })
                    } else {
                        None
                    }
                })
            })
            .unwrap_or("Unknown");
    }
    if let Some(mut d) = tiff(b) {
        return match d.colortype() {
            Ok(tiff::ColorType::Gray(_) | tiff::ColorType::GrayA(_)) => "Grayscale",
            Ok(tiff::ColorType::RGB(_) | tiff::ColorType::RGBA(_)) => "RGB",
            Ok(tiff::ColorType::CMYK(_) | tiff::ColorType::CMYKA(_)) => "CMYK",
            Ok(tiff::ColorType::Palette(_)) => "Indexed",
            Ok(tiff::ColorType::YCbCr(_)) => "YCbCr",
            _ => "Unknown",
        };
    }
    if super::is_pdf(b) || super::is_svg(b) || super::is_eps(b) {
        return "Vector / mixed";
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return "Indexed";
    }
    if b.starts_with(b"BM") {
        let bits = match b.get(14..18).and_then(|s| <[u8; 4]>::try_from(s).ok()).map(u32::from_le_bytes) {
            Some(12) => b.get(24..26),
            Some(40 | 52 | 56 | 108 | 124) => b.get(28..30),
            _ => None,
        }
        .and_then(|s| <[u8; 2]>::try_from(s).ok())
        .map(u16::from_le_bytes);
        if matches!(bits, Some(1 | 2 | 4 | 8)) {
            return "Indexed";
        }
    }
    let reader = image::ImageReader::new(Cursor::new(b)).with_guessed_format();
    let Some(d) = reader.ok().and_then(|r| r.into_decoder().ok()) else { return "Unknown" };
    match d.color_type() {
        image::ColorType::L8 | image::ColorType::La8 | image::ColorType::L16 | image::ColorType::La16 => "Grayscale",
        image::ColorType::Rgb8
        | image::ColorType::Rgba8
        | image::ColorType::Rgb16
        | image::ColorType::Rgba16
        | image::ColorType::Rgb32F
        | image::ColorType::Rgba32F => "RGB",
        _ => "Unknown",
    }
}

fn tiff(b: &[u8]) -> Option<tiff::decoder::Decoder<Cursor<&[u8]>>> {
    if ![b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"].iter().any(|m| b.starts_with(*m)) {
        return None;
    }
    let mut limits = tiff::decoder::Limits::default();
    limits.ifd_value_size = MAX_PROFILE;
    limits.intermediate_buffer_size = MAX_PROFILE;
    tiff::decoder::Decoder::new(Cursor::new(b)).ok().map(|d| d.with_limits(limits))
}

/// Bounded JPEG marker traversal, stopping before compressed pixels.
fn jpeg_segments(b: &[u8]) -> Option<Vec<(u8, &[u8])>> {
    let mut at = 2usize;
    let mut out = Vec::new();
    for _ in 0..4096 {
        if b.get(at) != Some(&0xff) {
            return None;
        }
        while b.get(at) == Some(&0xff) {
            at = at.checked_add(1)?;
        }
        let marker = *b.get(at)?;
        at = at.checked_add(1)?;
        if marker == 0xda || marker == 0xd9 {
            return Some(out);
        }
        if matches!(marker, 0x01 | 0xd0..=0xd8) {
            continue;
        }
        let len = usize::from(be16(b, at)?);
        if len < 2 {
            return None;
        }
        let end = at.checked_add(len)?;
        out.push((marker, b.get(at.checked_add(2)?..end)?));
        at = end;
    }
    None
}

fn jpeg_profile(b: &[u8]) -> Option<Option<Vec<u8>>> {
    let mut chunks = std::collections::BTreeMap::new();
    let mut count = None;
    let mut total = 0usize;
    for (marker, s) in jpeg_segments(b)? {
        if marker != 0xe2 || !s.starts_with(b"ICC_PROFILE\0") {
            continue;
        }
        let seq = *s.get(12)?;
        let n = *s.get(13)?;
        if seq == 0 || seq > n || count.is_some_and(|c| c != n) {
            return None;
        }
        count = Some(n);
        let data = s.get(14..)?;
        total = total.checked_add(data.len())?;
        if total > MAX_PROFILE || chunks.insert(seq, data).is_some() {
            return None;
        }
    }
    let Some(n) = count else { return Some(None) };
    if chunks.len() != usize::from(n) {
        return None;
    }
    Some(Some(chunks.values().flat_map(|s| s.iter().copied()).collect()))
}

fn psd_profile(b: &[u8]) -> Option<Option<Vec<u8>>> {
    let color_len = usize::try_from(be32(b, 26)?).ok()?;
    let at = 30usize.checked_add(color_len)?;
    let size = usize::try_from(be32(b, at)?).ok()?;
    let start = at.checked_add(4)?;
    let resources = b.get(start..start.checked_add(size)?)?;
    let mut at = 0usize;
    for _ in 0..4096 {
        if at == resources.len() {
            return Some(None);
        }
        if resources.get(at..at.checked_add(4)?)? != b"8BIM" {
            return None;
        }
        let id = be16(resources, at.checked_add(4)?)?;
        let name_at = at.checked_add(6)?;
        let name_size = usize::from(*resources.get(name_at)?).checked_add(1)?;
        let size_at = name_at.checked_add(name_size.checked_add(1)? & !1)?;
        let len = usize::try_from(be32(resources, size_at)?).ok()?;
        let data_at = size_at.checked_add(4)?;
        let data = resources.get(data_at..data_at.checked_add(len)?)?;
        if id == 1039 {
            return (len <= MAX_PROFILE).then(|| Some(data.to_vec()));
        }
        at = data_at.checked_add(len.checked_add(1)? & !1)?;
    }
    None
}

fn profile_bytes(b: &[u8]) -> Option<Option<Vec<u8>>> {
    if super::is_psd(b) {
        return psd_profile(b);
    }
    if b.starts_with(b"\xff\xd8") {
        return jpeg_profile(b);
    }
    if let Some(mut d) = tiff(b) {
        return match d.find_tag(tiff::tags::Tag::IccProfile).ok()? {
            None => Some(None),
            Some(_) => d.get_tag_u8_vec(tiff::tags::Tag::IccProfile).ok().map(Some),
        };
    }
    if super::is_pdf(b) || super::is_svg(b) || super::is_eps(b) {
        return None;
    }
    // These decoders don't expose source profiles. Absence of support is not absence of a profile.
    if b.starts_with(b"BM") || b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return None;
    }
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP") {
        let mut at = 12usize;
        let declared = b.get(4..8).and_then(|s| <[u8; 4]>::try_from(s).ok()).map(u32::from_le_bytes)?;
        let end = usize::try_from(declared).ok()?.checked_add(8)?;
        let b = b.get(..end)?;
        for _ in 0..4096 {
            if at == b.len() {
                return Some(None);
            }
            let kind = b.get(at..at.checked_add(4)?)?;
            let len_at = at.checked_add(4)?;
            let len = u32::from_le_bytes(b.get(len_at..len_at.checked_add(4)?)?.try_into().ok()?);
            let len = usize::try_from(len).ok()?;
            let start = at.checked_add(8)?;
            let data = b.get(start..start.checked_add(len)?)?;
            if kind == b"ICCP" {
                return (len <= MAX_PROFILE).then(|| Some(data.to_vec()));
            }
            at = start.checked_add(len.checked_add(1)? & !1)?;
        }
        return None;
    }
    let mut r = image::ImageReader::new(Cursor::new(b)).with_guessed_format().ok()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_PROFILE as u64);
    r.limits(limits);
    r.into_decoder().ok()?.icc_profile().ok()
}

fn clean_name(s: String) -> Option<String> {
    let s: String = s.chars().filter(|c| !c.is_control()).take(256).collect();
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// ICC v2 `desc` or v4 `mluc`, bounded and validated against the declared profile.
fn profile_name(b: &[u8]) -> Option<String> {
    let size = usize::try_from(be32(b, 0)?).ok()?;
    let b = b.get(..size)?;
    if b.get(36..40)? != b"acsp" {
        return None;
    }
    let count = usize::try_from(be32(b, 128)?).ok()?;
    if count > 4096 {
        return None;
    }
    for i in 0..count {
        let at = 132usize.checked_add(i.checked_mul(12)?)?;
        if b.get(at..at.checked_add(4)?)? != b"desc" {
            continue;
        }
        let offset = usize::try_from(be32(b, at.checked_add(4)?)?).ok()?;
        let len = usize::try_from(be32(b, at.checked_add(8)?)?).ok()?;
        let tag = b.get(offset..offset.checked_add(len)?)?;
        match tag.get(..4)? {
            b"desc" => {
                let len = usize::try_from(be32(tag, 8)?).ok()?;
                if len > 4096 {
                    return None;
                }
                return clean_name(String::from_utf8_lossy(tag.get(12..12usize.checked_add(len)?)?).into_owned());
            }
            b"mluc" => {
                let n = usize::try_from(be32(tag, 8)?).ok()?;
                let record = usize::try_from(be32(tag, 12)?).ok()?;
                if n > 256 || record < 12 {
                    return None;
                }
                let mut fallback = None;
                for i in 0..n {
                    let at = 16usize.checked_add(i.checked_mul(record)?)?;
                    let len = usize::try_from(be32(tag, at.checked_add(4)?)?).ok()?;
                    let offset = usize::try_from(be32(tag, at.checked_add(8)?)?).ok()?;
                    if len > 4096 || len % 2 != 0 {
                        return None;
                    }
                    let bytes = tag.get(offset..offset.checked_add(len)?)?;
                    let units: Vec<_> = bytes.as_chunks::<2>().0.iter().map(|s| u16::from_be_bytes(*s)).collect();
                    let name = clean_name(String::from_utf16(&units).ok()?);
                    if tag.get(at..at.checked_add(2)?) == Some(b"en") {
                        return name;
                    }
                    if fallback.is_none() {
                        fallback = name;
                    }
                }
                return fallback;
            }
            _ => return None,
        }
    }
    None
}

/// Metadata from stored asset bytes, without decoding any pixel buffer or changing the image.
pub fn color_info(b: &[u8]) -> ColorInfo {
    let mode = color_mode(b);
    let profile = match profile_bytes(b) {
        Some(Some(p)) => profile_name(&p).unwrap_or_else(|| "Embedded (unnamed or unreadable)".into()),
        Some(None) => "Not embedded".into(),
        None => "Unknown".into(),
    };
    ColorInfo { mode, profile }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder;

    fn icc(tag: &[u8]) -> Vec<u8> {
        let mut b = vec![0; 144];
        b[36..40].copy_from_slice(b"acsp");
        b[128..132].copy_from_slice(&1u32.to_be_bytes());
        b[132..136].copy_from_slice(b"desc");
        b[136..140].copy_from_slice(&144u32.to_be_bytes());
        b[140..144].copy_from_slice(&(tag.len() as u32).to_be_bytes());
        b.extend_from_slice(tag);
        let len = b.len() as u32;
        b[..4].copy_from_slice(&len.to_be_bytes());
        b
    }
    fn named_icc() -> Vec<u8> {
        let mut tag = b"desc\0\0\0\0".to_vec();
        tag.extend_from_slice(&13u32.to_be_bytes());
        tag.extend_from_slice(b"Pagenor Test\0");
        icc(&tag)
    }
    fn segment(marker: u8, bytes: &[u8], out: &mut Vec<u8>) {
        out.extend_from_slice(&[0xff, marker]);
        out.extend_from_slice(&((bytes.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(bytes);
    }
    fn jpeg(channels: u8, profile: Option<&[u8]>) -> Vec<u8> {
        let mut b = vec![0xff, 0xd8];
        if let Some(p) = profile {
            // Reversed segment order must still reassemble by sequence number.
            for (seq, data) in [(2, &p[p.len() / 2..]), (1, &p[..p.len() / 2])] {
                let mut payload = b"ICC_PROFILE\0".to_vec();
                payload.extend_from_slice(&[seq, 2]);
                payload.extend_from_slice(data);
                segment(0xe2, &payload, &mut b);
            }
        }
        segment(0xc0, &[8, 0, 1, 0, 1, channels], &mut b);
        b.extend_from_slice(&[0xff, 0xd9]);
        b
    }

    #[test]
    fn jpeg_source_modes_and_segmented_profile() {
        for (n, mode) in [(1, "Grayscale"), (3, "RGB"), (4, "CMYK")] {
            assert_eq!(color_info(&jpeg(n, None)), ColorInfo { mode, profile: "Not embedded".into() });
        }
        assert_eq!(color_info(&jpeg(4, Some(&named_icc()))), ColorInfo { mode: "CMYK", profile: "Pagenor Test".into() });
        let mut broken = jpeg(4, Some(&named_icc()));
        broken[18] = 1; // Duplicate ICC sequence number.
        assert_eq!(color_info(&broken).profile, "Unknown");
    }

    #[test]
    fn png_metadata_retains_grayscale_and_embedded_profile() {
        let mut b = Vec::new();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut b);
        encoder.set_icc_profile(named_icc()).unwrap();
        encoder.write_image(&[40], 1, 1, image::ExtendedColorType::L8).unwrap();
        assert_eq!(color_info(&b), ColorInfo { mode: "Grayscale", profile: "Pagenor Test".into() });
        let mut rgb = Vec::new();
        image::codecs::png::PngEncoder::new(&mut rgb).write_image(&[0, 0, 0, 255], 1, 1, image::ExtendedColorType::Rgba8).unwrap();
        assert_eq!(color_info(&rgb), ColorInfo { mode: "RGB", profile: "Not embedded".into() });
    }

    #[test]
    fn cmyk_tiff_and_psd_are_not_labeled_as_screen_rgb() {
        let mut b = Cursor::new(Vec::new());
        let mut encoder = tiff::encoder::TiffEncoder::new(&mut b).unwrap();
        let mut img = encoder.new_image::<tiff::encoder::colortype::CMYK8>(1, 1).unwrap();
        img.encoder().write_tag(tiff::tags::Tag::IccProfile, named_icc().as_slice()).unwrap();
        img.write_data(&[0, 0, 0, 255]).unwrap();
        assert_eq!(color_info(b.get_ref()), ColorInfo { mode: "CMYK", profile: "Pagenor Test".into() });
        let mut psd = b"8BPS\0\x01".to_vec();
        psd.resize(26, 0);
        psd[24..26].copy_from_slice(&4u16.to_be_bytes());
        psd.extend_from_slice(&0u32.to_be_bytes());
        let profile = named_icc();
        let mut resource = b"8BIM\x04\x0f\0\0".to_vec();
        resource.extend_from_slice(&(profile.len() as u32).to_be_bytes());
        resource.extend_from_slice(&profile);
        if resource.len() % 2 == 1 {
            resource.push(0);
        }
        psd.extend_from_slice(&(resource.len() as u32).to_be_bytes());
        psd.extend_from_slice(&resource);
        assert_eq!(color_info(&psd), ColorInfo { mode: "CMYK", profile: "Pagenor Test".into() });
    }

    #[test]
    fn unicode_icc_description_and_hostile_offsets() {
        let text: Vec<u8> = "తెలుగు Profile".encode_utf16().flat_map(u16::to_be_bytes).collect();
        let mut tag = b"mluc\0\0\0\0".to_vec();
        tag.extend_from_slice(&1u32.to_be_bytes());
        tag.extend_from_slice(&12u32.to_be_bytes());
        tag.extend_from_slice(b"teIN");
        tag.extend_from_slice(&(text.len() as u32).to_be_bytes());
        tag.extend_from_slice(&28u32.to_be_bytes());
        tag.extend_from_slice(&text);
        assert_eq!(profile_name(&icc(&tag)).as_deref(), Some("తెలుగు Profile"));
        let mut bad = icc(&tag);
        bad[136..140].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(profile_name(&bad), None);
        assert_eq!(color_info(&jpeg(3, Some(b"bad profile"))).profile, "Embedded (unnamed or unreadable)");
    }

    #[test]
    fn truncated_metadata_is_bounded_and_vector_modes_are_not_guessed() {
        for b in [jpeg(4, Some(&named_icc())), named_icc()] {
            for n in 0..b.len() {
                let _ = color_info(&b[..n]);
                let _ = profile_name(&b[..n]);
            }
        }
        assert_eq!(color_info(b"%PDF-1.7\n").mode, "Vector / mixed");
        assert_eq!(color_info(b"%PDF-1.7\n").profile, "Unknown");
        assert_eq!(color_info(&[]).mode, "Unknown");
    }
}
