use super::*;

// ---------- embedded resolution (#1991) ----------

/// A real encoder-written 600x900 JPEG with an APPn segment inserted after SOI (decoders skip
/// APPn, so the file stays fully valid and dimension readers exercise the real path).
fn jpeg_with(app: &[u8]) -> Vec<u8> {
    use image::codecs::jpeg::JpegEncoder;
    let img = image::RgbaImage::from_pixel(600, 900, image::Rgba([200, 30, 30, 255]));
    let mut b = Vec::new();
    JpegEncoder::new_with_quality(&mut b, 90).encode_image(&img).unwrap();
    assert_eq!(&b[..2], &[0xFF, 0xD8]);
    let mut out = b[..2].to_vec();
    out.extend_from_slice(app);
    out.extend_from_slice(&b[2..]);
    out
}

/// JFIF APP0 with the given units (0 aspect, 1 dpi, 2 per cm) and densities.
fn jfif_app(units: u8, x: u16, y: u16) -> Vec<u8> {
    let mut a = vec![0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x02, units];
    a.extend_from_slice(&x.to_be_bytes());
    a.extend_from_slice(&y.to_be_bytes());
    a.extend_from_slice(&[0x00, 0x00]);
    a
}

/// Minimal little-endian TIFF with X/YResolution rationals and an optional ResolutionUnit.
fn tiff_bytes(x: (u32, u32), y: (u32, u32), unit: Option<u16>) -> Vec<u8> {
    let mut t = vec![b'I', b'I', 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    let n = if unit.is_some() { 3 } else { 2 };
    t.extend_from_slice(&(n as u16).to_le_bytes());
    let data = 8 + 2 + n * 12 + 4;
    let mut entry = |tag: u16, typ: u16, val: u32| {
        t.extend_from_slice(&tag.to_le_bytes());
        t.extend_from_slice(&typ.to_le_bytes());
        t.extend_from_slice(&1u32.to_le_bytes());
        t.extend_from_slice(&val.to_le_bytes());
    };
    entry(0x011A, 5, data as u32);
    entry(0x011B, 5, (data + 8) as u32);
    if let Some(u) = unit {
        // SHORT count 1: the value sits inline in the low bytes.
        t.extend_from_slice(&0x0128u16.to_le_bytes());
        t.extend_from_slice(&3u16.to_le_bytes());
        t.extend_from_slice(&1u32.to_le_bytes());
        t.extend_from_slice(&u.to_le_bytes());
        t.extend_from_slice(&[0, 0]);
    }
    t.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(t.len(), data);
    for (num, den) in [x, y] {
        t.extend_from_slice(&num.to_le_bytes());
        t.extend_from_slice(&den.to_le_bytes());
    }
    t
}

/// EXIF APP1 wrapping a TIFF header.
fn exif_app(tiff: &[u8]) -> Vec<u8> {
    let mut a = vec![0xFF, 0xE1];
    a.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
    a.extend_from_slice(b"Exif\x00\x00");
    a.extend_from_slice(tiff);
    a
}

/// #1991: a 600x900 px photo saved at 600 dpi places 1 x 1.5 in, not 600 x 900 pt.
#[test]
fn jpeg_jfif_density_sets_natural_size() {
    let jpg = jpeg_with(&jfif_app(1, 600, 600));
    assert_eq!(pixel_size(&jpg), Some((600, 900)));
    assert_eq!(raster_resolution(&jpg), Some((600.0, 600.0)));
    assert_eq!(natural_size(&jpg), Some((72.0, 108.0)));
    // Dots per cm convert to dpi; aspect-only JFIF (units 0) carries no density.
    let cm = jpeg_with(&jfif_app(2, 100, 100));
    assert_eq!(raster_resolution(&cm), Some((254.0, 254.0)));
    let aspect = jpeg_with(&jfif_app(0, 600, 600));
    assert_eq!(raster_resolution(&aspect), None);
    assert_eq!(natural_size(&aspect), Some((600.0, 900.0)));
}

#[test]
fn jpeg_exif_resolution_beats_jfif() {
    // EXIF wins over a conflicting JFIF density.
    let mut app = jfif_app(1, 72, 72);
    app.extend_from_slice(&exif_app(&tiff_bytes((300, 1), (300, 1), Some(2))));
    let jpg = jpeg_with(&app);
    assert_eq!(raster_resolution(&jpg), Some((300.0, 300.0)));
    assert_eq!(natural_size(&jpg), Some((600.0 * 72.0 / 300.0, 900.0 * 72.0 / 300.0)));
    // A missing unit defaults to inch; per-cm scales by 2.54.
    let no_unit = jpeg_with(&exif_app(&tiff_bytes((300, 1), (300, 1), None)));
    assert_eq!(raster_resolution(&no_unit), Some((300.0, 300.0)));
    // A non-EXIF APP1 (XMP) before the EXIF one does not stop the scan.
    let mut xmp = vec![0xFF, 0xE1];
    let xmp_payload = b"http://ns.adobe.com/xap/1.0/\x00<x:xmpmeta/>";
    xmp.extend_from_slice(&((xmp_payload.len() + 2) as u16).to_be_bytes());
    xmp.extend_from_slice(xmp_payload);
    xmp.extend_from_slice(&exif_app(&tiff_bytes((300, 1), (300, 1), Some(2))));
    assert_eq!(raster_resolution(&jpeg_with(&xmp)), Some((300.0, 300.0)));
    let cm = jpeg_with(&exif_app(&tiff_bytes((100, 1), (100, 1), Some(3))));
    assert_eq!(raster_resolution(&cm), Some((254.0, 254.0)));
    // A zero denominator or a missing axis is unusable: back to a pixel per point.
    let zero = jpeg_with(&exif_app(&tiff_bytes((300, 0), (300, 1), Some(2))));
    assert_eq!(raster_resolution(&zero), None);
}

/// Big-endian EXIF parses too.
#[test]
fn jpeg_exif_big_endian_parses() {
    let mut t = vec![b'M', b'M', 0x00, 0x2A, 0x00, 0x00, 0x00, 0x08];
    t.extend_from_slice(&[0x00, 0x02]);
    let mut entry = |tag: u16, val: u32| {
        t.extend_from_slice(&tag.to_be_bytes());
        t.extend_from_slice(&5u16.to_be_bytes());
        t.extend_from_slice(&1u32.to_be_bytes());
        t.extend_from_slice(&val.to_be_bytes());
    };
    entry(0x011A, 38);
    entry(0x011B, 46);
    t.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(t.len(), 38);
    for (num, den) in [(600u32, 2u32), (600u32, 2u32)] {
        t.extend_from_slice(&num.to_be_bytes());
        t.extend_from_slice(&den.to_be_bytes());
    }
    let jpg = jpeg_with(&exif_app(&t));
    assert_eq!(raster_resolution(&jpg), Some((300.0, 300.0)));
    assert_eq!(natural_size(&jpg), Some((144.0, 216.0)));
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// A real encoder-written PNG with a pHYs chunk spliced in after IHDR (valid CRC, so the
/// file stays fully decodable and exercises the real dimension path).
fn png_with_phys(ppu: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(600, 900, image::Rgba([30, 200, 30, 255]));
    let mut b = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Png).unwrap();
    // Signature (8) + IHDR chunk (length 4 + type 4 + data 13 + CRC 4).
    assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&b[12..16], b"IHDR");
    let mut out = b[..33].to_vec();
    let mut phys = Vec::new();
    phys.extend_from_slice(&ppu.to_be_bytes());
    phys.extend_from_slice(&ppu.to_be_bytes());
    phys.push(1);
    out.extend_from_slice(&(phys.len() as u32).to_be_bytes());
    out.extend_from_slice(b"pHYs");
    out.extend_from_slice(&phys);
    let mut c = b"pHYs".to_vec();
    c.extend_from_slice(&phys);
    out.extend_from_slice(&crc32(&c).to_be_bytes());
    out.extend_from_slice(&b[33..]);
    out
}

#[test]
fn png_phys_sets_natural_size() {
    // 23622 px/m = 600 dpi: the photo places 1 x 1.5 in.
    let png = png_with_phys(23622);
    assert_eq!(pixel_size(&png), Some((600, 900)));
    let (dx, _) = raster_resolution(&png).unwrap();
    assert!((dx - 600.0).abs() < 0.05, "{dx}");
    let (w, h) = natural_size(&png).unwrap();
    assert!((w - 72.0).abs() < 0.01 && (h - 108.0).abs() < 0.02, "{w} x {h}");
    // No pHYs: a pixel is a point.
    let plain_img = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 255]));
    let mut plain = Vec::new();
    plain_img.write_to(&mut std::io::Cursor::new(&mut plain), image::ImageFormat::Png).unwrap();
    assert_eq!(raster_resolution(&plain), None);
    assert_eq!(natural_size(&plain), Some((4.0, 4.0)));
}

/// A real encoder-written BMP with the header pixels-per-meter patched (compression and image
/// size stay zero, as the encoder writes them).
fn bmp_with_ppm(ppm: i32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(600, 900, image::Rgba([30, 30, 200, 255]));
    let mut b = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Bmp).unwrap();
    assert_eq!(&b[..2], b"BM");
    b[38..42].copy_from_slice(&ppm.to_le_bytes());
    b[42..46].copy_from_slice(&ppm.to_le_bytes());
    b
}

#[test]
fn bmp_density_sets_natural_size() {
    // 2835 px/m = 72 dpi: the bitmap keeps its pixel size in points.
    let bmp = bmp_with_ppm(2835);
    assert_eq!(pixel_size(&bmp), Some((600, 900)));
    let (dx, _) = raster_resolution(&bmp).unwrap();
    assert!((dx - 72.0).abs() < 0.05, "{dx}");
    // 600 dpi (23622 px/m): the bitmap places 1 x 1.5 in.
    let dense = bmp_with_ppm(23622);
    let (w, h) = natural_size(&dense).unwrap();
    assert!((w - 72.0).abs() < 0.01 && (h - 108.0).abs() < 0.02, "{w} x {h}");
    // Zero density: no metadata, a pixel is a point.
    let flat = bmp_with_ppm(0);
    assert_eq!(raster_resolution(&flat), None);
    assert_eq!(natural_size(&flat), Some((600.0, 900.0)));
}

#[test]
fn tiff_resolution_tags_set_natural_size() {
    use tiff::encoder::colortype::Gray8;
    // A TIFF the encoder writes carries no density: pixels are points.
    let mut plain = Vec::new();
    {
        let mut enc = tiff::encoder::TiffEncoder::new(std::io::Cursor::new(&mut plain)).unwrap();
        enc.new_image::<Gray8>(6, 9).unwrap().write_data(&[0u8; 54]).unwrap();
    }
    assert_eq!(pixel_size(&plain), Some((6, 9)));
    assert_eq!(natural_size(&plain), Some((6.0, 9.0)));
    // The hand-built tag body parses (300 dpi, inch default and explicit).
    for unit in [None, Some(2)] {
        let t = tiff_bytes((300, 1), (300, 1), unit);
        assert_eq!(resolution::tiff_resolution(&t), Some((300.0, 300.0)));
    }
}

/// A 1-bit gray PNG of `side` x `side` blank pixels, written row by row so the test stays small.
fn blank_png(side: u32) -> Vec<u8> {
    use std::io::Write;
    let chunk = |out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]| {
        out.extend((data.len() as u32).to_be_bytes());
        let mut crc = flate2::Crc::new();
        crc.update(kind);
        crc.update(data);
        out.extend(kind);
        out.extend(data);
        out.extend(crc.sum().to_be_bytes());
    };
    let row = vec![0u8; 1 + side.div_ceil(8) as usize];
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    for _ in 0..side {
        z.write_all(&row).unwrap();
    }
    let mut ihdr = [side.to_be_bytes(), side.to_be_bytes()].concat();
    ihdr.extend([1, 0, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z.finish().unwrap());
    chunk(&mut png, b"IEND", &[]);
    png
}

/// A small file that expands past 1 GiB of RGBA is refused from its header (#1586).
#[test]
fn rasters_past_the_rgba_budget_are_refused_before_decoding() {
    let png = blank_png(16_385);
    assert!(png.len() < 1 << 20, "{} bytes", png.len());
    assert_eq!(pixel_size(&png), Some((16_385, 16_385)));
    assert_eq!(natural_size(&png), Some((16_385.0, 16_385.0)));
    assert!(too_large_to_decode(&png));
    assert!(decode_rgba(&png).is_none());
    assert!(!too_large_to_decode(&blank_png(16_384)));
    let small = decode_rgba(&blank_png(9)).expect("decodes");
    assert_eq!(small.get_pixel(8, 8).0, [0, 0, 0, 255]);
}
