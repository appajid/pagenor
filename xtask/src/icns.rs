//! Portable ICNS packing: PNG representations for macOS 11+ (the app's minimum OS).

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
        let tag = match width {
            16 => b"icp4",
            32 => b"icp5",
            64 => b"icp6",
            128 => b"ic07",
            256 => b"ic08",
            512 => b"ic09",
            1024 => b"ic10",
            _ => return Err(format!("unsupported ICNS size: {width}")),
        };
        let length = png.len().checked_add(8).and_then(|n| u32::try_from(n).ok()).ok_or("PNG too large")?;
        entries.extend_from_slice(tag);
        entries.extend_from_slice(&length.to_be_bytes());
        entries.extend_from_slice(&png);
    }
    let length = entries.len().checked_add(8).and_then(|n| u32::try_from(n).ok()).ok_or("ICNS too large")?;
    let mut output_bytes = b"icns".to_vec();
    output_bytes.extend_from_slice(&length.to_be_bytes());
    output_bytes.extend(entries);
    std::fs::write(output, output_bytes).map_err(|e| format!("write {output}: {e}"))
}
