//! Portable ICNS packing: classic RGB/mask entries for small icons, PNG for Retina/large sizes.

fn entry(entries: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) -> Result<(), String> {
    let length = data.len().checked_add(8).and_then(|n| u32::try_from(n).ok()).ok_or("icon entry too large")?;
    entries.extend_from_slice(tag);
    entries.extend_from_slice(&length.to_be_bytes());
    entries.extend_from_slice(data);
    Ok(())
}

fn small_icon(entries: &mut Vec<u8>, png: &[u8], width: u32) -> Result<(), String> {
    let (rgb_tag, mask_tag) = match width {
        16 => (b"is32", b"s8mk"),
        32 => (b"il32", b"l8mk"),
        _ => return Err("classic icons must be 16 or 32 pixels".into()),
    };
    let image = resvg::tiny_skia::Pixmap::decode_png(png).map_err(|e| format!("decode small icon: {e}"))?;
    let pixels: Vec<_> = image.pixels().iter().map(|p| p.demultiply()).collect();
    let mut rgb = Vec::new();
    // Classic ICNS accepts uncompressed 32-bit RGB with an unused first byte
    // and a separate alpha mask. PNG in the classic icp4/icp5 slots instead
    // corrupts colours in Apple's small-icon reader.
    for p in &pixels {
        rgb.extend_from_slice(&[0, p.red(), p.green(), p.blue()]);
    }
    let alpha: Vec<_> = pixels.iter().map(|p| p.alpha()).collect();
    entry(entries, rgb_tag, &rgb)?;
    entry(entries, mask_tag, &alpha)
}

pub fn run(args: &[&str]) -> Result<(), String> {
    let [output, inputs @ ..] = args else { return Err("usage: icns <out.icns> <in.png>...".into()) };
    if inputs.is_empty() || inputs.len() > 7 {
        return Err("need 1..=7 icon sizes".into());
    }
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for input in inputs {
        let png = std::fs::read(input).map_err(|e| format!("read {input}: {e}"))?;
        let (width, height) = crate::ico::png_size(&png)?;
        if width != height || !seen.insert(width) {
            return Err("icons must be square and have distinct sizes".into());
        }
        if matches!(width, 16 | 32) {
            small_icon(&mut entries, &png, width)?;
            if width == 32 {
                entry(&mut entries, b"ic11", &png)?; // 16-point Retina icon.
            }
            continue;
        }
        let tag = match width {
            64 => b"ic12", // 32-point Retina icon (icp6 is a classic RGB slot).
            128 => b"ic07",
            256 => b"ic08",
            512 => b"ic09",
            1024 => b"ic10",
            _ => return Err(format!("unsupported ICNS size: {width}")),
        };
        entry(&mut entries, tag, &png)?;
    }
    let length = entries.len().checked_add(8).and_then(|n| u32::try_from(n).ok()).ok_or("ICNS too large")?;
    let mut output_bytes = b"icns".to_vec();
    output_bytes.extend_from_slice(&length.to_be_bytes());
    output_bytes.extend(entries);
    std::fs::write(output, output_bytes).map_err(|e| format!("write {output}: {e}"))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    #[test]
    fn native_reader_preserves_small_icon_colours() {
        // Apple reads these sizes in Login Items, Open With and file-info headers.
        // Extracting the PNG bytes directly did not reveal the native ICNS corruption.
        let dir = tempfile::tempdir().unwrap();
        for (size, name) in [(16, "icon_16x16.png"), (32, "icon_32x32.png"), (64, "icon_32x32@2x.png")] {
            for alpha in [0, 128, 255] {
                let mut source = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
                source.fill(resvg::tiny_skia::Color::from_rgba8(16, 160, 245, alpha));
                let png = dir.path().join(format!("source-{size}-{alpha}.png"));
                source.save_png(&png).unwrap();
                let icns = dir.path().join(format!("icon-{size}-{alpha}.icns"));
                super::run(&[icns.to_str().unwrap(), png.to_str().unwrap()]).unwrap();
                let iconset = dir.path().join(format!("decoded-{size}-{alpha}.iconset"));
                let status = std::process::Command::new("iconutil")
                    .args(["-c", "iconset", icns.to_str().unwrap(), "-o", iconset.to_str().unwrap()])
                    .status()
                    .unwrap();
                assert!(status.success());
                let image = std::fs::read(iconset.join(name)).unwrap();
                let decoded = resvg::tiny_skia::Pixmap::decode_png(&image).unwrap();
                assert_eq!(decoded.data(), source.data(), "native {size}-pixel icon colours must match the source PNG");
            }
        }
    }
}
