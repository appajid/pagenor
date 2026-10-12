//! Optional PageMaker compatibility import. The external helper is used as a black box;
//! no libpagemaker code is linked or copied. Its diagnostic event stream is deliberately
//! accepted only as a strict, bounded subset, not treated as a lossless interchange format.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::{BTreeMap, BTreeSet};
#[cfg(not(target_arch = "wasm32"))]
use std::ops::Range;

#[cfg(not(target_arch = "wasm32"))]
use pagenor_color::{Color, Swatch};
#[cfg(not(target_arch = "wasm32"))]
use pagenor_doc::build::NewDocument;
#[cfg(not(target_arch = "wasm32"))]
use pagenor_doc::{Align, Asset, AssetId, CharAttrs, Content, Document, Graphic, Item, ItemId, ParaAttrs, ParaFormat, Shape, SpreadRef, StoryId};
#[cfg(not(target_arch = "wasm32"))]
use pagenor_geom::{Affine, Rect, shapes};
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;

use super::bad;
#[cfg(not(target_arch = "wasm32"))]
use super::str_param;
#[cfg(not(target_arch = "wasm32"))]
use crate::DocState;
use crate::{Result, Session};

#[cfg(not(target_arch = "wasm32"))]
const MAX_BYTES: usize = 64 * 1024 * 1024;
#[cfg(not(target_arch = "wasm32"))]
const MAX_EVENTS: usize = 200_000;
#[cfg(not(target_arch = "wasm32"))]
const MAX_FRAMES: usize = 10_000;
#[cfg(not(target_arch = "wasm32"))]
const WARNING: &str = "Experimental PageMaker import: pages and editable text frames are recovered through the optional pmd2raw helper. Legacy font codes are retained; install the original fonts. Unicode conversion, exact original flow order, layers, master items and exact typography are not yet preserved. Embedded raster images and shared stories are recovered when exposed by the helper. Verify every page before publishing and save as .pgnr.";

pub fn is_name(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| ["p65", "t65", "pmd", "pmt", "pm7"].iter().any(|e| ext.eq_ignore_ascii_case(e)))
}

pub(crate) fn open(s: &mut Session, p: &Value) -> Result<Value> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, p);
        Err(bad("file.open", "PageMaker import requires the desktop application and the optional pmd2raw helper"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = str_param(p, "path");
        let name = path.or_else(|| str_param(p, "name")).ok_or_else(|| bad("file.open", "missing PageMaker name"))?;
        let bytes = if let Some(path) = path {
            let limit = p.get("maxBytes").and_then(Value::as_u64).unwrap_or(MAX_BYTES as u64).min(MAX_BYTES as u64);
            super::file::read_asset_file(path, limit)?
        } else {
            let b = str_param(p, "base64").ok_or_else(|| bad("file.openBytes", "missing `base64`"))?;
            if b.len() > MAX_BYTES.div_ceil(3).saturating_mul(4) {
                return Err(bad("file.openBytes", "PageMaker input exceeds 64 MiB"));
            }
            super::base64_decode(b)
        };
        if bytes.len() > MAX_BYTES || !bytes.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
            return Err(bad("file.open", "expected a PageMaker 6.5/7 compound document (64 MiB maximum)"));
        }
        let (raw, byte_output) = native::convert(&bytes)?;
        let title = std::path::Path::new(name).file_stem().and_then(|n| n.to_str()).unwrap_or("PageMaker document");
        let recover = p.get("recoverRepeatedStories").and_then(Value::as_bool).unwrap_or(true);
        let (mut d, mut warnings, frames) = parse(&raw, title, recover)?;
        let (fonts, faces, font_warnings) = path.map_or((None, 0, Vec::new()), |path| super::file::load_document_fonts(&mut d, path));
        warnings.extend(font_warnings);
        if byte_output {
            // Wait only for the initial metadata scan, not a rescan or full font loads.
            // A partial catalogue can make the first import infer a different sole style.
            let _ = pagenor_fonts::FontDb::global().scan_report();
        }
        let db = pagenor_fonts::FontDb::global().scoped(d.font_scope);
        resolve_single_styles(&mut d, |family| db.styles(family))?;
        if byte_output {
            let mut byte_faces = BTreeMap::new();
            resolve_legacy_bytes(&mut d, |family, style, byte| {
                let face = byte_faces.entry((family.to_string(), style.to_string())).or_insert_with(|| db.face(family, style));
                legacy_char(byte, |c| face.covers(c))
            })?;
            warnings.push("PageMaker 8-bit text bytes preserved. Font character maps select ordinary or private glyph slots; this is not Telugu Unicode conversion. Confirm the original legacy font version and every page.".into());
        } else {
            warnings.push("The standard pmd2raw helper may discard bytes 128–255 on platforms with signed char. Use Pagenor's byte-preserving helper for legacy fonts; changing fonts cannot restore discarded characters.".into());
        }
        let images = d.spreads.iter().flat_map(|s| &s.items).filter(|i| matches!(i.content, Content::Graphic(_))).count();
        let pages: usize = d.spreads.iter().map(|s| s.pages.len()).sum();
        // Import is always a new native document: Save can never overwrite the original.
        let mut st = DocState::new(d, None);
        st.fonts = fonts;
        let index = s.add_document(st);
        Ok(json!({"index":index,"pages":pages,"textFrames":frames,"images":images,"documentFonts":faces,"importWarnings":warnings}))
    }
}

/// libpagemaker can emit the whole shared story once per page frame, without a
/// story id. Conservatively infer only long, identical, equally formatted stories
/// repeated on at least three distinct pages with equal frame widths. Short
/// running heads, same-page copies, distinct formatting and ordinary two-page
/// duplicates remain independent. This inference is disclosed and can be disabled.
#[cfg(not(target_arch = "wasm32"))]
fn recover_repeated_stories(d: &mut Document) -> Result<Vec<String>> {
    let groups = {
        let mut repeated = BTreeMap::<&str, Vec<(usize, pagenor_doc::ItemId, pagenor_doc::StoryId, f64)>>::new();
        for (page, spread) in d.spreads.iter().enumerate() {
            for item in &spread.items {
                if let Some(frame) = item.text_frame()
                    && let Some(story) = d.story(frame.story)
                    && story.text.len() >= 2048
                    && story.frames.len() == 1
                {
                    repeated.entry(&story.text).or_default().push((page, item.id, story.id, item.bounds().width()));
                }
            }
        }
        repeated
            .values()
            .filter(|frames| {
                let Some((_, _, sid, width)) = frames.first() else { return false };
                let Some(base) = d.story(*sid) else { return false };
                frames.len() >= 3
                    && frames.iter().map(|f| f.0).collect::<BTreeSet<_>>().len() == frames.len()
                    && frames.iter().all(|(_, _, sid, w)| {
                        (*w - width).abs() < 0.25 && d.story(*sid).is_some_and(|st| st.chars == base.chars && st.paras == base.paras)
                    })
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut warnings = Vec::new();
    for group in groups {
        let Some((_, mut previous, _, _)) = group.first().copied() else { continue };
        for (_, frame, sid, _) in group.iter().skip(1).copied() {
            let st = d.story_mut(sid).ok_or_else(|| bad("file.open", "missing repeated PageMaker story"))?;
            st.delete(0..st.text.len());
            d.thread(previous, frame)?;
            previous = frame;
        }
        warnings.push(format!("PageMaker shared-story recovery: identical long text was linked once across {} page frames. This is inferred from converter output; compare with the source. file.open {{recoverRepeatedStories:false}} retains separate copies.",group.len()));
    }
    Ok(warnings)
}

/// The helper supplies a font name but no independent style. Many legacy fonts
/// are separate one-face families with styles such as Medium or Black. Use the
/// sole catalogued style of that exact family; do not infer a weight or substitute
/// a different family. Preserve the source name so missing-font repair stays clear.
#[cfg(not(target_arch = "wasm32"))]
fn resolve_single_styles(d: &mut Document, mut styles: impl FnMut(&str) -> Vec<String>) -> Result<()> {
    let families: BTreeSet<String> = d.stories.values().flat_map(|s| s.chars.iter().filter_map(|c| c.format.over.font_family.clone())).collect();
    if families.len() > 1024 {
        return Err(bad("file.open", "PageMaker font budget exceeded"));
    }
    let map: BTreeMap<String, String> = families
        .into_iter()
        .filter_map(|family| {
            let choices = styles(&family);
            if choices.len() == 1 { choices.first().map(|style| (family, style.clone())) } else { None }
        })
        .collect();
    for story in d.stories.values_mut() {
        let story = std::sync::Arc::make_mut(story);
        story.format_chars(0..story.text.len(), |format| {
            if let Some(style) = format.over.font_family.as_ref().and_then(|f| map.get(f)) {
                format.over.font_style = Some(style.clone());
            }
        });
    }
    Ok(())
}

/// The byte helper uses the original library compiled with unsigned char. Only
/// insertText payloads are 8-bit data; event structure and attributes remain strict
/// UTF-8. Carry bytes in U+F000..F0FF until document-scoped fonts are available.
#[cfg(not(target_arch = "wasm32"))]
fn byte_trace(output: &[u8]) -> Result<String> {
    let mut raw = String::new();
    for (index, line) in output.split(|b| *b == b'\n').enumerate() {
        if index >= MAX_EVENTS || line.len() > 1024 * 1024 {
            return Err(bad("file.open", "PageMaker event budget exceeded"));
        }
        let start = line.iter().take_while(|b| **b == b' ').count();
        let event = line.get(start..).ok_or_else(|| bad("file.open", "invalid PageMaker event"))?;
        if let Some(text) = event.strip_prefix(b"insertText (").and_then(|t| t.strip_suffix(b")")) {
            raw.push_str(
                std::str::from_utf8(line.get(..start).ok_or_else(|| bad("file.open", "invalid PageMaker indent"))?)
                    .map_err(|_| bad("file.open", "invalid PageMaker indent"))?,
            );
            raw.push_str("insertText (");
            for &b in text {
                if b < 0x20 || b == 0x7f {
                    return Err(bad("file.open", "unsupported PageMaker control byte"));
                }
                let c = if b == b' ' {
                    ' '
                } else {
                    char::from_u32(0xf000 + u32::from(b)).ok_or_else(|| bad("file.open", "invalid PageMaker text byte"))?
                };
                raw.push(c);
            }
            raw.push(')');
        } else {
            raw.push_str(std::str::from_utf8(line).map_err(|_| bad("file.open", "PageMaker helper attributes are not UTF-8"))?);
        }
        raw.push('\n');
        if raw.len() > MAX_BYTES {
            return Err(bad("file.open", "PageMaker event stream exceeds 64 MiB"));
        }
    }
    // split includes a final empty element for the helper's terminal newline.
    if output.ends_with(b"\n") {
        raw.pop();
    }
    Ok(raw)
}

#[cfg(not(target_arch = "wasm32"))]
fn legacy_char(byte: u8, covers: impl Fn(char) -> bool) -> char {
    // Windows-1252 printable characters; undefined slots retain their source byte
    // in the private symbol range instead of disappearing or becoming U+FFFD.
    const HIGH: [u32; 32] = [
        0x20ac, 0xf081, 0x201a, 0x0192, 0x201e, 0x2026, 0x2020, 0x2021, 0x02c6, 0x2030, 0x0160, 0x2039, 0x0152, 0xf08d, 0x017d, 0xf08f, 0xf090,
        0x2018, 0x2019, 0x201c, 0x201d, 0x2022, 0x2013, 0x2014, 0x02dc, 0x2122, 0x0161, 0x203a, 0x0153, 0xf09d, 0x017e, 0x0178,
    ];
    let cp =
        if (0x80..0xa0).contains(&byte) { HIGH.get(usize::from(byte - 0x80)).copied().unwrap_or(0xf000 + u32::from(byte)) } else { u32::from(byte) };
    let ordinary = char::from_u32(cp).unwrap_or('\u{f000}');
    let private = char::from_u32(0xf000 + u32::from(byte)).unwrap_or('\u{f000}');
    if covers(ordinary) || !covers(private) { ordinary } else { private }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_legacy_bytes(d: &mut Document, mut map: impl FnMut(&str, &str, u8) -> char) -> Result<()> {
    for story in d.stories.values_mut() {
        let story = std::sync::Arc::make_mut(story);
        let mut start = 0usize;
        let mut text = String::with_capacity(story.text.len());
        for run in &mut story.chars {
            let end = start.checked_add(run.len).ok_or_else(|| bad("file.open", "PageMaker character range overflow"))?;
            let source = story.text.get(start..end).ok_or_else(|| bad("file.open", "invalid PageMaker character range"))?;
            let before = text.len();
            let family = run.format.over.font_family.as_deref().unwrap_or("");
            let style = run.format.over.font_style.as_deref().unwrap_or("Regular");
            for c in source.chars() {
                text.push(if (0xf000..=0xf0ff).contains(&u32::from(c)) { map(family, style, (u32::from(c) - 0xf000) as u8) } else { c });
            }
            run.len = text.len() - before;
            start = end;
        }
        if start != story.text.len() {
            return Err(bad("file.open", "incomplete PageMaker character ranges"));
        }
        story.text = text;
        story.rev = story.rev.wrapping_add(1);
        story.check().map_err(|e| bad("file.open", e))?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::{Duration, Instant};

    use super::*;

    fn helper() -> (std::ffi::OsString, bool) {
        if let Some(path) = std::env::var_os("PAGENOR_PAGEMAKER_CONVERTER") {
            return (path, std::env::var_os("PAGENOR_PAGEMAKER_BYTE_OUTPUT").is_some_and(|v| v == "1"));
        }
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        let data = if cfg!(target_os = "macos") {
            home.map(|p| p.join("Library/Application Support/Pagenor"))
        } else if cfg!(target_os = "windows") {
            std::env::var_os("APPDATA").map(|p| std::path::PathBuf::from(p).join("Pagenor"))
        } else {
            std::env::var_os("XDG_DATA_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| home.map(|p| p.join(".local/share")))
                .map(|p| p.join("pagenor"))
        };
        if let Some(path) = data
            .filter(|p| p.is_absolute())
            .map(|p| p.join("helpers").join(if cfg!(target_os = "windows") { "pmd2raw-byte-v1.exe" } else { "pmd2raw-byte-v1" }))
            && path.is_file()
        {
            return (path.into(), true);
        }
        #[cfg(target_os = "macos")]
        for path in ["/opt/homebrew/bin/pmd2raw", "/usr/local/bin/pmd2raw"] {
            if std::path::Path::new(path).is_file() {
                return (path.into(), false);
            }
        }
        (if cfg!(target_os = "windows") { "pmd2raw.exe".into() } else { "pmd2raw".into() }, false)
    }

    fn read_pipe(reader: impl Read, limit: usize, exceeded: &AtomicBool) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        reader.take(limit as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            exceeded.store(true, Ordering::Release);
        }
        Ok(bytes)
    }

    pub(super) fn convert(bytes: &[u8]) -> Result<(String, bool)> {
        let (helper, byte_output) = helper();
        let mut input = tempfile::NamedTempFile::new().map_err(|e| bad("file.open", e.to_string()))?;
        input.write_all(bytes).map_err(|e| bad("file.open", e.to_string()))?;
        input.flush().map_err(|e| bad("file.open", e.to_string()))?;
        // No shell, no source-file-controlled executable or switches. The helper only sees
        // our bounded temporary copy, even if the original is modified during conversion.
        let mut child = Command::new(helper).arg(input.path()).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| bad("file.open", format!("PageMaker helper unavailable: {e}. Install pmd2raw (libpagemaker), or set PAGENOR_PAGEMAKER_CONVERTER to its executable path.")))?;
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(bad("file.open", "cannot read PageMaker helper output"));
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(bad("file.open", "cannot read PageMaker helper diagnostics"));
        };
        let exceeded = Arc::new(AtomicBool::new(false));
        // Scoped readers cannot outlive this operation and drain both pipes concurrently.
        std::thread::scope(|scope| {
            let out = scope.spawn(|| read_pipe(stdout, MAX_BYTES, &exceeded));
            let err = scope.spawn(|| read_pipe(stderr, 16 * 1024, &exceeded));
            let deadline = Instant::now() + Duration::from_secs(30);
            let status = loop {
                if exceeded.load(Ordering::Acquire) || Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(bad("file.open", "PageMaker helper exceeded its output or 30-second time limit"));
                }
                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                    Err(e) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        break Err(bad("file.open", e.to_string()));
                    }
                }
            };
            let output = out.join().map_err(|_| bad("file.open", "PageMaker output reader failed"))?.map_err(|e| bad("file.open", e.to_string()))?;
            let diagnostics =
                err.join().map_err(|_| bad("file.open", "PageMaker diagnostic reader failed"))?.map_err(|e| bad("file.open", e.to_string()))?;
            if exceeded.load(Ordering::Acquire) {
                return Err(bad("file.open", "PageMaker helper output exceeds its limit"));
            }
            if !status?.success() {
                return Err(bad("file.open", format!("PageMaker helper could not read this file: {}", String::from_utf8_lossy(&diagnostics))));
            }
            // An unrecognised diagnostic must not silently accompany a partial import.
            if !diagnostics.is_empty() {
                return Err(bad("file.open", format!("PageMaker helper reported: {}", String::from_utf8_lossy(&diagnostics))));
            }
            let raw = if byte_output {
                byte_trace(&output)?
            } else {
                String::from_utf8(output).map_err(|_| {
                    bad("file.open", "PageMaker helper returned invalid UTF-8; use the explicit byte-preserving helper profile for 8-bit output")
                })?
            };
            Ok((raw, byte_output))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Frame {
    rect: Rect,
    source_story: Option<u32>,
    rotation: f64,
    text: String,
    chars: Vec<(Range<usize>, CharAttrs)>,
    paras: Vec<(Range<usize>, ParaAttrs)>,
}

#[cfg(not(target_arch = "wasm32"))]
struct Picture {
    rect: Rect,
    rotation: f64,
    bytes: Vec<u8>,
    mime: String,
    pixels: (u32, u32),
}

#[cfg(not(target_arch = "wasm32"))]
enum PageItem {
    Text(Frame),
    Picture(Picture),
}

#[cfg(not(target_arch = "wasm32"))]
type SourceFrames = BTreeMap<u32, Vec<(usize, ItemId, StoryId, Rect)>>;

/// The helper exposes the original text-block identity, but not frame-chain order.
/// Keep distinct IDs independent. Flow each identical shared story once in page,
/// left-column, then vertical order, without changing item paint order.
#[cfg(not(target_arch = "wasm32"))]
fn recover_source_stories(d: &mut Document, groups: SourceFrames) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    for (source, mut frames) in groups {
        if frames.len() < 2 {
            continue;
        }
        frames.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.3.x0.total_cmp(&b.3.x0)).then_with(|| a.3.y0.total_cmp(&b.3.y0)).then(a.1.cmp(&b.1)));
        let Some((_, mut previous, first, _)) = frames.first().copied() else {
            continue;
        };
        let base = d.story(first).ok_or_else(|| bad("file.open", "missing PageMaker source story"))?;
        if frames.iter().any(|(_, _, sid, _)| d.story(*sid).is_none_or(|st| st.text != base.text || st.chars != base.chars || st.paras != base.paras))
        {
            warnings.push(format!("PageMaker source story {source} has conflicting text or formatting; its frames were retained separately."));
            continue;
        }
        for (_, frame, sid, _) in frames.iter().skip(1).copied() {
            let st = d.story_mut(sid).ok_or_else(|| bad("file.open", "missing PageMaker source story"))?;
            st.delete(0..st.text.len());
            d.thread(previous, frame)?;
            previous = frame;
        }
        warnings.push(format!("PageMaker source story {source}: {} frames linked with text stored once. Frame flow order is inferred by page and left-to-right column position; verify it against the source.", frames.len()));
    }
    Ok(warnings)
}

#[cfg(not(target_arch = "wasm32"))]
fn angle(a: &BTreeMap<&str, &str>) -> Result<f64> {
    // libpagemaker text rotations contain degree values mislabeled with the
    // default "in" suffix; bitmap rotations use unitless degrees. Never scale angles
    // as lengths. Accept only these two observed helper forms.
    let n = a
        .get("librevenge:rotate")
        .map_or(Ok(0.), |v| v.strip_suffix("in").unwrap_or(v).parse::<f64>())
        .map_err(|_| bad("file.open", "invalid PageMaker rotation"))?;
    if !n.is_finite() || n.abs() > 360_000. {
        return Err(bad("file.open", "PageMaker rotation out of range"));
    }
    Ok(n.to_radians())
}

#[cfg(not(target_arch = "wasm32"))]
fn picture(a: &BTreeMap<&str, &str>, pixel_budget: &mut u64) -> Result<Picture> {
    let (x, y, w, h) = (required(a, "svg:x")?, required(a, "svg:y")?, required(a, "svg:width")?, required(a, "svg:height")?);
    if w <= 0. || h <= 0. {
        return Err(bad("file.open", "invalid PageMaker image dimensions"));
    }
    let encoded = a.get("office:binary-data").ok_or_else(|| bad("file.open", "PageMaker image has no embedded bytes"))?;
    // The outer event limit bounds allocation. Round-trip validation rejects the
    // permissive shared decoder's ignored characters, bad padding and trailing bits.
    let bytes = super::base64_decode(encoded);
    if bytes.is_empty() || super::base64_encode(&bytes) != *encoded {
        return Err(bad("file.open", "invalid PageMaker image base64"));
    }
    let mime = a.get("librevenge:mime-type").ok_or_else(|| bad("file.open", "PageMaker image has no MIME type"))?;
    if !["image/tiff", "image/png", "image/jpeg", "image/bmp", "image/gif", "image/webp"].contains(mime) || pagenor_images::mime(&bytes) != *mime {
        return Err(bad("file.open", "unsupported or mismatched PageMaker embedded image type"));
    }
    let pixels = pagenor_images::pixel_size(&bytes).ok_or_else(|| bad("file.open", "invalid PageMaker embedded raster"))?;
    let count = u64::from(pixels.0) * u64::from(pixels.1);
    *pixel_budget = pixel_budget.saturating_add(count);
    if count == 0 || count > 4_000_000 || *pixel_budget > 16_000_000 || pagenor_images::decode_rgba_limited(&bytes, count).is_none() {
        return Err(bad("file.open", "PageMaker raster exceeds its pixel/decode budget or is corrupt"));
    }
    Ok(Picture { rect: Rect::new(x, y, x + w, y + h), rotation: angle(a)?, bytes, mime: (*mime).into(), pixels })
}

#[cfg(not(target_arch = "wasm32"))]
fn attrs(event: &str) -> Result<BTreeMap<&str, &str>> {
    let body = event.split_once('(').and_then(|(_, s)| s.strip_suffix(')')).ok_or_else(|| bad("file.open", "invalid PageMaker event attributes"))?;
    let mut out = BTreeMap::new();
    if !body.is_empty() {
        for entry in body.split(", ") {
            let (key, value) = entry.split_once(": ").ok_or_else(|| bad("file.open", "ambiguous PageMaker attributes"))?;
            if out.insert(key, value).is_some() {
                return Err(bad("file.open", "duplicate PageMaker attributes"));
            }
        }
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn measure(v: &str) -> Result<f64> {
    let (v, scale) = if let Some(v) = v.strip_suffix("in") {
        (v, 72.)
    } else if let Some(v) = v.strip_suffix("pt") {
        (v, 1.)
    } else {
        return Err(bad("file.open", "unknown PageMaker measurement unit"));
    };
    let n = v.parse::<f64>().map_err(|_| bad("file.open", "invalid PageMaker measurement"))? * scale;
    if !n.is_finite() || n.abs() > 100_000. {
        return Err(bad("file.open", "PageMaker measurement out of range"));
    }
    Ok(n)
}
#[cfg(not(target_arch = "wasm32"))]
fn required(a: &BTreeMap<&str, &str>, key: &str) -> Result<f64> {
    measure(a.get(key).ok_or_else(|| bad("file.open", format!("missing PageMaker {key}")))?)
}
#[cfg(not(target_arch = "wasm32"))]
fn optional(a: &BTreeMap<&str, &str>, key: &str) -> Result<Option<f64>> {
    a.get(key).map(|v| measure(v)).transpose()
}

#[cfg(not(target_arch = "wasm32"))]
fn paragraph(a: &BTreeMap<&str, &str>) -> Result<ParaAttrs> {
    let align = match a.get("fo:text-align").copied() {
        None | Some("left") => Align::Left,
        Some("center") => Align::Center,
        Some("right") => Align::Right,
        Some("justify") => Align::LeftJustified,
        _ => return Err(bad("file.open", "unsupported PageMaker paragraph alignment")),
    };
    Ok(ParaAttrs {
        align: Some(align),
        left_indent: optional(a, "fo:margin-left")?,
        right_indent: optional(a, "fo:margin-right")?,
        first_line_indent: optional(a, "fo:text-indent")?,
        space_before: optional(a, "fo:margin-top")?,
        space_after: optional(a, "fo:margin-bottom")?,
        hyphenate: a.get("fo:hyphenate").map(|v| *v == "true"),
        ..Default::default()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn character(a: &BTreeMap<&str, &str>, colors: &mut BTreeSet<String>) -> Result<CharAttrs> {
    let font = a.get("style:font-name").ok_or_else(|| bad("file.open", "PageMaker span has no font name"))?;
    if font.is_empty() || font.len() > 1024 {
        return Err(bad("file.open", "invalid PageMaker font name"));
    }
    let size = required(a, "fo:font-size")?;
    if size <= 0. || size > 10_000. {
        return Err(bad("file.open", "invalid PageMaker font size"));
    }
    let fill = if let Some(color) = a.get("fo:color") {
        if color.len() != 7 || !color.starts_with('#') || !color.get(1..).is_some_and(|c| c.bytes().all(|b| b.is_ascii_hexdigit())) {
            return Err(bad("file.open", "unsupported PageMaker text colour"));
        }
        if colors.len() >= 1024 {
            return Err(bad("file.open", "PageMaker colour budget exceeded"));
        }
        colors.insert(color.to_string());
        Some(format!("PageMaker {color}"))
    } else {
        None
    };
    Ok(CharAttrs { font_family: Some(font.to_string()), font_style: Some("Regular".into()), size: Some(size), fill, ..Default::default() })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse(raw: &str, title: &str, recover: bool) -> Result<(Document, Vec<String>, usize)> {
    if raw.len() > MAX_BYTES {
        return Err(bad("file.open", "PageMaker event stream exceeds 64 MiB"));
    }
    let mut pages: Vec<(f64, f64, Vec<PageItem>)> = Vec::new();
    let mut frame: Option<Frame> = None;
    let mut para: Option<(usize, ParaAttrs)> = None;
    let mut span: Option<(usize, CharAttrs)> = None;
    let mut colors = BTreeSet::new();
    let (mut graphics, mut pixel_budget, mut source_ids) = (0usize, 0u64, false);
    let (mut started, mut ended, mut in_page) = (false, false, false);
    let (mut count, mut skipped) = (0usize, BTreeMap::<String, usize>::new());
    for (line_no, line) in raw.lines().enumerate() {
        if line_no >= MAX_EVENTS || line.len() > 1024 * 1024 {
            return Err(bad("file.open", "PageMaker event budget exceeded"));
        }
        let indent = line.bytes().take_while(|c| *c == b' ').count();
        let event = line.trim_start_matches(' ');
        let expected: usize = if span.is_some() {
            10
        } else if para.is_some() {
            8
        } else if frame.is_some() {
            6
        } else if in_page {
            4
        } else if started && !ended {
            2
        } else {
            0
        };
        let close = matches!(event, "closeSpan" | "closeParagraph" | "endTextObject" | "endPage" | "endDocument");
        if indent != if close { expected.saturating_sub(2) } else { expected } || ended {
            return Err(bad("file.open", format!("ambiguous PageMaker event at line {}", line_no + 1)));
        }
        match event {
            "startDocument()" if !started => started = true,
            "endDocument" if started && !in_page => ended = true,
            e if e.starts_with("startPage(") && started && !in_page => {
                let a = attrs(e)?;
                let w = required(&a, "svg:width")?;
                let h = required(&a, "svg:height")?;
                if w <= 0. || h <= 0. || pages.len() >= 1000 {
                    return Err(bad("file.open", "invalid PageMaker page dimensions/count"));
                }
                pages.push((w, h, Vec::new()));
                in_page = true;
            }
            "endPage" if in_page && frame.is_none() => in_page = false,
            e if e.starts_with("startTextObject (") && in_page && frame.is_none() => {
                count += 1;
                if count.saturating_add(graphics) > MAX_FRAMES {
                    return Err(bad("file.open", "too many PageMaker frames"));
                }
                let a = attrs(e)?;
                let (x, y, w, h) = (required(&a, "svg:x")?, required(&a, "svg:y")?, required(&a, "svg:width")?, required(&a, "svg:height")?);
                if w <= 0. || h <= 0. {
                    return Err(bad("file.open", "invalid PageMaker frame dimensions"));
                }
                let source_story = a
                    .get("pagenor:story-id")
                    .map(|v| v.parse::<u32>().map_err(|_| bad("file.open", "invalid PageMaker source story ID")))
                    .transpose()?;
                source_ids |= source_story.is_some();
                frame = Some(Frame {
                    rect: Rect::new(x, y, x + w, y + h),
                    source_story: source_story.filter(|id| *id != 0),
                    rotation: angle(&a)?,
                    ..Default::default()
                });
            }
            "endTextObject" if frame.is_some() && para.is_none() => {
                let f = frame.take().ok_or_else(|| bad("file.open", "missing PageMaker frame"))?;
                pages.last_mut().ok_or_else(|| bad("file.open", "missing PageMaker page"))?.2.push(PageItem::Text(f));
            }
            e if e.starts_with("openParagraph (") && frame.is_some() && para.is_none() => {
                let f = frame.as_ref().ok_or_else(|| bad("file.open", "missing frame"))?;
                para = Some((f.text.len(), paragraph(&attrs(e)?)?));
            }
            "closeParagraph" if para.is_some() && span.is_none() => {
                let (start, attr) = para.take().ok_or_else(|| bad("file.open", "missing paragraph"))?;
                let f = frame.as_mut().ok_or_else(|| bad("file.open", "missing frame"))?;
                if f.paras.len() >= 2048 {
                    return Err(bad("file.open", "PageMaker paragraph budget exceeded"));
                }
                if let Some((_, attr)) = f.chars.last().cloned() {
                    f.chars.push((f.text.len()..f.text.len() + 1, attr));
                }
                f.text.push('\n');
                f.paras.push((start..f.text.len(), attr));
            }
            e if e.starts_with("openSpan(") && para.is_some() && span.is_none() => {
                let start = frame.as_ref().ok_or_else(|| bad("file.open", "missing frame"))?.text.len();
                span = Some((start, character(&attrs(e)?, &mut colors)?));
            }
            "closeSpan" if span.is_some() => {
                let (start, attr) = span.take().ok_or_else(|| bad("file.open", "missing span"))?;
                let f = frame.as_mut().ok_or_else(|| bad("file.open", "missing frame"))?;
                if f.chars.len() >= 2048 {
                    return Err(bad("file.open", "PageMaker span budget exceeded"));
                }
                f.chars.push((start..f.text.len(), attr));
            }
            e if span.is_some() => {
                let text = if let Some(text) = e.strip_prefix("insertText (").and_then(|s| s.strip_suffix(')')) {
                    text
                } else {
                    match e {
                        "insertLineBreak" => "\u{2028}",
                        "insertTab" => "\t",
                        "insertSpace" => " ",
                        _ => return Err(bad("file.open", "unsupported or ambiguous PageMaker text event")),
                    }
                };
                // Raw text events are not escaped by the helper. A multiline record or
                // control sequence is rejected rather than mistaken for another event.
                if text.chars().any(|c| c.is_control() && c != '\t') {
                    return Err(bad("file.open", "unsupported PageMaker control character"));
                }
                frame.as_mut().ok_or_else(|| bad("file.open", "missing frame"))?.text.push_str(text);
            }
            e if in_page && frame.is_none() && e.starts_with("drawGraphicObject (") => {
                graphics += 1;
                if graphics.saturating_add(count) > MAX_FRAMES {
                    return Err(bad("file.open", "too many PageMaker items"));
                }
                let image = picture(&attrs(e)?, &mut pixel_budget)?;
                pages.last_mut().ok_or_else(|| bad("file.open", "missing PageMaker page"))?.2.push(PageItem::Picture(image));
            }
            e if in_page && frame.is_none() && (e.starts_with("draw") || e.starts_with("setStyle(")) => {
                let key = e.split([' ', '(']).next().unwrap_or("graphics").to_string();
                *skipped.entry(key).or_default() += 1;
            }
            _ => return Err(bad("file.open", format!("unsupported PageMaker structure at line {}", line_no + 1))),
        }
    }
    if !ended || in_page || frame.is_some() || para.is_some() || span.is_some() || pages.is_empty() {
        return Err(bad("file.open", "incomplete PageMaker conversion"));
    }
    let (width, height, _) = pages.first().ok_or_else(|| bad("file.open", "no PageMaker pages"))?;
    if pages.iter().any(|(w, h, _)| (w - width).abs() > 0.01 || (h - height).abs() > 0.01) {
        return Err(bad("file.open", "mixed PageMaker page sizes are not yet supported"));
    }
    let mut d = Document::new(&NewDocument {
        title: title.into(),
        width: *width,
        height: *height,
        pages: pages.len(),
        facing_pages: false,
        ..Default::default()
    });
    for color in colors {
        let n = u32::from_str_radix(color.get(1..).ok_or_else(|| bad("file.open", "invalid colour"))?, 16)
            .map_err(|_| bad("file.open", "invalid colour"))?;
        d.swatches.push(Swatch::color(
            &format!("PageMaker {color}"),
            Color::rgb(((n >> 16) & 255) as f32 / 255., ((n >> 8) & 255) as f32 / 255., (n & 255) as f32 / 255.),
        ));
    }
    let layer = d.default_layer();
    let mut groups = SourceFrames::new();
    for (page, (_, _, items)) in pages.into_iter().enumerate() {
        for item in items {
            match item {
                PageItem::Text(f) => {
                    let (id, sid) = d.add_text_frame(SpreadRef::Doc(page), f.rect, layer, &f.text, ParaFormat::default())?;
                    let story = d.story_mut(sid).ok_or_else(|| bad("file.open", "missing imported story"))?;
                    for (range, attr) in f.chars {
                        story.format_chars(range, |format| format.over = attr.clone());
                    }
                    // format_paras includes its end byte; do not modify the next paragraph.
                    for (range, attr) in f.paras {
                        story.format_paras(range.start..range.end.saturating_sub(1), |format| format.para = attr.clone());
                    }
                    if let Some(source) = f.source_story {
                        groups.entry(source).or_default().push((page, id, sid, f.rect));
                    }
                    if f.rotation != 0. {
                        let center = f.rect.center();
                        d.item_mut(id).ok_or_else(|| bad("file.open", "missing imported frame"))?.xf =
                            Affine::translate(center.to_vec2()) * Affine::rotate(f.rotation) * Affine::translate(-center.to_vec2());
                    }
                }
                PageItem::Picture(image) => {
                    let fresh = AssetId(d.alloc());
                    let asset = d.add_asset(Asset {
                        id: fresh,
                        name: format!("PageMaker embedded image {}", fresh.0),
                        mime: image.mime,
                        data: std::sync::Arc::new(image.bytes),
                        pixels: Some(image.pixels),
                        ..Default::default()
                    });
                    let mut it = Item::new(ItemId(d.alloc()), layer, Shape::Rectangle, shapes::rectangle(image.rect));
                    let center = image.rect.center();
                    it.xf = Affine::translate(center.to_vec2()) * Affine::rotate(image.rotation) * Affine::translate(-center.to_vec2());
                    it.content = Content::Graphic(Graphic {
                        asset,
                        size: (image.rect.width(), image.rect.height()),
                        xf: Affine::translate((image.rect.x0, image.rect.y0)),
                        clip: None,
                        auto_fit: Default::default(),
                        fit_align: 4,
                        crop: [0.; 4],
                        wrap: Default::default(),
                    });
                    d.insert_item(SpreadRef::Doc(page), it, None)?;
                }
            }
        }
    }
    let mut warnings = vec![WARNING.into()];
    if recover {
        warnings.extend(if source_ids { recover_source_stories(&mut d, groups)? } else { recover_repeated_stories(&mut d)? });
    }
    warnings.extend(skipped.into_iter().map(|(kind, count)| format!("PageMaker: {count} unsupported {kind} event(s) omitted.")));
    Ok((d, warnings, count))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    const EVENTS: &str = "startDocument()\n  startPage(svg:height: 11in, svg:width: 8in)\n    startTextObject (librevenge:rotate: 0.0000in, svg:height: 3in, svg:width: 4in, svg:x: 1in, svg:y: 2in)\n      openParagraph (fo:text-align: center)\n        openSpan(fo:color: #123456, fo:font-size: 18pt, style:font-name: Legacy Medium)\n          insertText (a (b) తెలుగు)\n          insertLineBreak\n          insertTab\n          insertSpace\n          insertText (z)\n        closeSpan\n      closeParagraph\n    endTextObject\n  endPage\nendDocument\n";
    #[test]
    fn helper_degree_rotation_accepts_legacy_text_suffix_without_length_scaling() {
        let raw = EVENTS.replace("rotate: 0.0000in", "rotate: 90.0000in");
        let (d, _, _) = parse(&raw, "rotated", true).unwrap();
        let item = &d.spreads[0].items[0];
        let origin = item.xf * pagenor_geom::Point::new(72., 144.);
        assert!((origin.x - 324.).abs() < 1e-6 && (origin.y - 108.).abs() < 1e-6);
        assert!(parse(&EVENTS.replace("rotate: 0.0000in", "rotate: 90pt"), "bad unit", true).is_err());
    }

    #[test]
    fn byte_helper_preserves_every_high_byte_and_formatting_on_native_reopen() {
        let marker = "a (b) తెలుగు";
        let (before, after) = EVENTS.split_once(marker).unwrap();
        let bytes = (128..=255).collect::<Vec<u8>>();
        let raw = [before.as_bytes(), &bytes, after.as_bytes()].concat();
        let normalized = byte_trace(&raw).unwrap();
        let (mut d, _, _) = parse(&normalized, "byte test", true).unwrap();
        resolve_legacy_bytes(&mut d, |_, _, b| legacy_char(b, |c| (0xf000..=0xf0ff).contains(&u32::from(c)))).unwrap();
        let st = d.stories.values().next().unwrap();
        assert_eq!(st.text.chars().take(128).map(|c| u32::from(c) - 0xf000).collect::<Vec<_>>(), (128..=255).collect::<Vec<_>>());
        assert_eq!(st.chars[0].format.over.font_family.as_deref(), Some("Legacy Medium"));
        assert_eq!(st.chars[0].format.over.size, Some(18.));
        assert_eq!(st.paras[0].para.align, Some(Align::Center));
        st.check().unwrap();
        let d2 = crate::cmd::from_bytes(&crate::cmd::to_bytes(&d).unwrap()).unwrap();
        let reopened = d2.stories.values().next().unwrap();
        assert_eq!(reopened.text, st.text);
        assert_eq!(reopened.chars, st.chars);
        assert_eq!(reopened.paras, st.paras);
    }

    #[test]
    fn byte_helper_keeps_english_ordinary_and_symbol_runs_in_their_font() {
        let data = EVENTS.replace("a (b) తెలుగు", "Latin: Ph.D., FCSEPI");
        let raw = byte_trace(data.as_bytes()).unwrap();
        let (mut d, _, _) = parse(&raw, "latin", true).unwrap();
        resolve_legacy_bytes(&mut d, |_, _, b| legacy_char(b, |c| c.is_ascii())).unwrap();
        assert_eq!(d.stories.values().next().unwrap().text, "Latin: Ph.D., FCSEPI\u{2028}\t z\n");
        assert_eq!(legacy_char(0xe9, |c| c == 'é'), 'é');
        assert_eq!(legacy_char(0x8d, |_| false), '\u{f08d}');
        assert_eq!(legacy_char(b'Z', |c| c == '\u{f05a}'), '\u{f05a}');
        assert_eq!(legacy_char(b'Z', |c| c == 'Z' || c == '\u{f05a}'), 'Z');
    }

    #[test]
    fn byte_helper_rejects_metadata_and_control_corruption_without_guessing() {
        for byte in [0, 0x1b, 0x7f] {
            assert!(byte_trace(&[b"          insertText (".as_slice(), &[byte], b")\n"].concat()).is_err());
        }
        assert!(byte_trace(b"        openSpan(style:font-name: \xff)\n").is_err());
        // A genuine Unicode trace stays on the existing UTF-8 path, never byte mode.
        let (d, _, _) = parse(EVENTS, "unicode", true).unwrap();
        assert!(d.stories.values().next().unwrap().text.contains("తెలుగు"));
    }

    #[test]
    fn editable_text_geometry_styles_and_native_round_trip() {
        let (d, w, count) = parse(EVENTS, "test", true).unwrap();
        assert_eq!(count, 1);
        assert!(!w.is_empty());
        assert_eq!(d.spreads.len(), 1);
        let st = d.stories.values().next().unwrap();
        assert_eq!(st.text, "a (b) తెలుగు\u{2028}\t z\n");
        assert_eq!(st.chars[0].format.over.font_family.as_deref(), Some("Legacy Medium"));
        assert_eq!(st.paras[0].para.align, Some(Align::Center));
        assert_eq!(d.spreads[0].items[0].bounds(), Rect::new(72., 144., 360., 360.));
        let d2 = crate::cmd::from_bytes(&crate::cmd::to_bytes(&d).unwrap()).unwrap();
        assert_eq!(d2.stories.values().next().unwrap().text, st.text);
    }
    #[test]
    fn legacy_single_face_style_is_resolved_without_guessing_missing_or_multiple_faces() {
        let (mut d, _, _) = parse(EVENTS, "test", true).unwrap();
        resolve_single_styles(&mut d, |_| vec!["Medium".into()]).unwrap();
        assert_eq!(d.stories.values().next().unwrap().chars[0].format.over.font_style.as_deref(), Some("Medium"));
        for choices in [vec![], vec!["Medium".into(), "Black".into()]] {
            let (mut d, _, _) = parse(EVENTS, "test", true).unwrap();
            resolve_single_styles(&mut d, |_| choices.clone()).unwrap();
            let st = d.stories.values().next().unwrap();
            assert_eq!(st.chars[0].format.over.font_style.as_deref(), Some("Regular"));
            assert_eq!(st.chars[0].format.over.font_family.as_deref(), Some("Legacy Medium"));
        }
    }

    fn repeated_pages(text: &str, count: usize) -> Result<Document> {
        let mut d = Document::new(&NewDocument { pages: count, facing_pages: false, ..Default::default() });
        let layer = d.default_layer();
        for page in 0..count {
            d.add_text_frame(SpreadRef::Doc(page), Rect::new(36., 36., 400., 700.), layer, text, ParaFormat::default())?;
        }
        Ok(d)
    }
    #[test]
    fn repeated_long_story_flows_once_across_pages_and_native_reopen() {
        let text = "Long legacy text. ".repeat(200);
        let mut d = repeated_pages(&text, 5).unwrap();
        let warnings = recover_repeated_stories(&mut d).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(d.stories.len(), 1);
        let st = d.stories.values().next().unwrap();
        assert_eq!(st.text, text);
        assert_eq!(st.frames.len(), 5);
        st.check().unwrap();
        for spread in &d.spreads {
            assert_eq!(spread.items[0].text_frame().unwrap().story, st.id);
        }
        let reopened = crate::cmd::from_bytes(&crate::cmd::to_bytes(&d).unwrap()).unwrap();
        assert_eq!(reopened.stories.values().next().unwrap().text, text);
        assert_eq!(reopened.stories.values().next().unwrap().frames.len(), 5);
    }
    #[test]
    fn short_headers_two_page_copies_and_different_formatting_stay_independent() {
        for (text, count) in [("A running header".to_string(), 5), ("Long story ".repeat(300), 2)] {
            let mut d = repeated_pages(&text, count).unwrap();
            assert!(recover_repeated_stories(&mut d).unwrap().is_empty());
            assert_eq!(d.stories.len(), count);
        }
        let mut d = repeated_pages(&"Long story ".repeat(300), 3).unwrap();
        let sid = d.stories.keys().next().copied().unwrap();
        d.story_mut(sid).unwrap().format_chars(0..1, |f| f.over.size = Some(99.));
        assert!(recover_repeated_stories(&mut d).unwrap().is_empty());
        assert_eq!(d.stories.len(), 3);
    }

    fn column_events(id: u32, x: &str, size: &str) -> Result<String> {
        let body = EVENTS
            .split_once("    startTextObject")
            .and_then(|(_, s)| s.split_once("  endPage"))
            .map(|(s, _)| s)
            .ok_or_else(|| bad("test", "missing synthetic frame"))?;
        Ok(format!("    startTextObject{body}").replace("svg:x: 1in", &format!("pagenor:story-id: {id}, svg:x: {x}")).replace("18pt", size))
    }
    #[test]
    fn source_ids_flow_columns_once_without_merging_independent_copies() {
        // Paint order is deliberately right, left, independent; flow is left, right.
        let raw = format!(
            "startDocument()\n  startPage(svg:height: 11in, svg:width: 8in)\n{}{}{}  endPage\nendDocument\n",
            column_events(7, "4in", "18pt").unwrap(),
            column_events(7, "1in", "18pt").unwrap(),
            column_events(8, "6in", "18pt").unwrap()
        );
        let (d, w, count) = parse(&raw, "columns", true).unwrap();
        assert_eq!(count, 3);
        assert_eq!(d.stories.len(), 2);
        assert!(w.iter().any(|s| s.contains("source story 7")));
        let items = &d.spreads[0].items;
        let shared = d.story(items[0].text_frame().unwrap().story).unwrap();
        assert_eq!(shared.text, "a (b) తెలుగు\u{2028}\t z\n");
        assert_eq!(shared.frames, [items[1].id, items[0].id]);
        assert_ne!(items[2].text_frame().unwrap().story, shared.id);
        assert_eq!(items[0].bounds().x0, 288.);
        let reopened = crate::cmd::from_bytes(&crate::cmd::to_bytes(&d).unwrap()).unwrap();
        assert_eq!(reopened.story(shared.id).unwrap().frames, shared.frames);
        assert_eq!(reopened.story(shared.id).unwrap().text, shared.text);
        assert_eq!(reopened.story(shared.id).unwrap().chars, shared.chars);
        assert_eq!(reopened.story(shared.id).unwrap().paras, shared.paras);
        assert_eq!(parse(&raw, "disabled", false).unwrap().0.stories.len(), 3);
        let conflicting = raw.replacen("18pt", "19pt", 1);
        let (conflict, warnings, _) = parse(&conflicting, "conflict", true).unwrap();
        assert_eq!(conflict.stories.len(), 3);
        assert!(warnings.iter().any(|w| w.contains("conflicting")));
        assert!(parse(&raw.replacen("pagenor:story-id: 7", "pagenor:story-id: -1", 1), "bad ID", true).is_err());
    }

    fn tiny_bmp() -> Vec<u8> {
        let mut b = b"BM".to_vec();
        b.extend(58u32.to_le_bytes());
        b.extend([0; 4]);
        b.extend(54u32.to_le_bytes());
        b.extend(40u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(24u16.to_le_bytes());
        b.extend([0; 4]);
        b.extend(4u32.to_le_bytes());
        b.extend([0; 16]);
        b.extend([0, 0, 255, 0]);
        b
    }
    fn image_events(bytes: &[u8]) -> String {
        format!(
            "    drawGraphicObject (librevenge:mime-type: image/bmp, office:binary-data: {}, svg:height: 1in, svg:width: 2in, svg:x: 1in, svg:y: 2in, librevenge:rotate: 90.0000)\n",
            super::super::base64_encode(bytes)
        )
    }
    #[test]
    fn embedded_rasters_keep_bytes_placement_rotation_paint_order_and_native_reopen() {
        let bytes = tiny_bmp();
        let event = image_events(&bytes);
        let raw = EVENTS.replace("    startTextObject", &format!("{event}    startTextObject")).replace("  endPage", &format!("{event}  endPage"));
        let (d, _, frames) = parse(&raw, "images", true).unwrap();
        assert_eq!(frames, 1);
        assert_eq!(d.assets.len(), 1, "identical embedded bytes share an asset");
        let items = &d.spreads[0].items;
        assert_eq!(items.len(), 3);
        assert!(matches!(items[0].content, Content::Graphic(_)));
        assert!(matches!(items[1].content, Content::Text(_)));
        assert!(matches!(items[2].content, Content::Graphic(_)));
        let graphic = items[0].graphic().unwrap();
        assert_eq!(graphic.size, (144., 72.));
        assert_eq!(graphic.xf.translation(), pagenor_geom::Vec2::new(72., 144.));
        let transformed = items[0].xf * pagenor_geom::Point::new(72., 144.);
        assert!((transformed.x - 180.).abs() < 1e-6 && (transformed.y - 108.).abs() < 1e-6);
        let reopened = crate::cmd::from_bytes(&crate::cmd::to_bytes(&d).unwrap()).unwrap();
        let asset = reopened.assets.values().next().unwrap();
        assert_eq!(*asset.data, bytes);
        assert_eq!(asset.mime, "image/bmp");
        assert_eq!(asset.pixels, Some((1, 1)));
        assert!(asset.link.is_none());
        assert_eq!(reopened.spreads[0].items[0].xf, items[0].xf);
        assert_eq!(pagenor_images::decode_rgba_limited(&asset.data, 1).unwrap().get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert!(pagenor_images::decode_rgba_limited(&asset.data, 0).is_none());
    }
    #[test]
    fn corrupt_or_oversized_embedded_rasters_fail_without_silent_omission() {
        let good = EVENTS.replace("  endPage", &format!("{}  endPage", image_events(&tiny_bmp())));
        for raw in [
            good.replace("image/bmp", "image/png"),
            good.replace("binary-data: Q", "binary-data: !Q"),
            good.replace("90.0000", "NaN"),
            good.replace("svg:width: 2in", "svg:width: 0in"),
        ] {
            assert!(parse(&raw, "bad raster", true).is_err());
        }
        let mut huge = tiny_bmp();
        huge[18..22].copy_from_slice(&100_000u32.to_le_bytes());
        let raw = EVENTS.replace("  endPage", &format!("{}  endPage", image_events(&huge)));
        assert!(parse(&raw, "pixel budget", true).is_err());
    }

    #[test]
    fn rejects_incomplete_ambiguous_and_unbounded_events() {
        for raw in [
            EVENTS.replace("endDocument", ""),
            EVENTS.replace("insertText (z)", "insertText (z\nendDocument)"),
            EVENTS.replace("8in", "NaNin"),
            EVENTS.replace("18pt", "0pt"),
            EVENTS.replace("insertText (z)", "insertText (z)\nendDocument"),
            EVENTS.replace("svg:width: 8in", "svg:width: 8in, svg:width: 9in"),
        ] {
            assert!(parse(&raw, "x", true).is_err());
        }
        assert!(parse(&"x".repeat(1024 * 1024 + 1), "x", true).is_err());
    }
    #[test]
    fn maintains_multiple_pages_and_warns_about_omitted_artwork() {
        let raw =
            EVENTS.replace("  endPage\n", "    drawRectangle (svg:x: 0in)\n  endPage\n  startPage(svg:height: 11in, svg:width: 8in)\n  endPage\n");
        let (d, w, _) = parse(&raw, "x", true).unwrap();
        assert_eq!(d.spreads.len(), 2);
        assert!(w.iter().any(|w| w.contains("drawRectangle")));
    }
    #[test]
    fn page_maker_names_are_case_insensitive_and_specific() {
        for name in ["x.P65", "x.t65", "x.pmd", "x.PMT", "x.PM7"] {
            assert!(is_name(name));
        }
        for name in ["pmd", "x.pmd.zip", "x.indd", "x.pgnr"] {
            assert!(!is_name(name));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_import_preserves_the_open_session() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"title":"keep"})).unwrap();
        assert!(open(&mut s, &json!({"name":"bad.p65","base64":"AAAA"})).is_err());
        assert_eq!(s.doc().unwrap().doc.title, "keep");
    }
    // Owner documents stay outside the repository. Opt-in genuine-file validation uses
    // the installed black-box helper and never writes to the source document.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "requires a local PageMaker sample and pmd2raw"]
    fn local_page_maker_sample_imports_and_reopens() {
        let path = std::env::var("PAGENOR_PAGEMAKER_TEST_FILE").unwrap();
        let pages: usize = std::env::var("PAGENOR_PAGEMAKER_TEST_PAGES").unwrap().parse().unwrap();
        let frames: usize = std::env::var("PAGENOR_PAGEMAKER_TEST_FRAMES").unwrap().parse().unwrap();
        let mut s = Session::new();
        let report = s.execute("file.open", &json!({"path":path})).unwrap();
        assert_eq!(report["pages"], pages);
        assert_eq!(report["textFrames"], frames);
        assert!(s.doc().unwrap().path.is_none());
        let d = &s.doc().unwrap().doc;
        let reopened = crate::cmd::from_bytes(&crate::cmd::to_bytes(d).unwrap()).unwrap();
        assert_eq!(reopened.spreads.len(), pages);
        let stories: usize = std::env::var("PAGENOR_PAGEMAKER_TEST_STORIES").ok().map(|n| n.parse().unwrap()).unwrap_or(frames);
        assert_eq!(reopened.stories.len(), stories);
        assert_eq!(reopened.stories.values().map(|s| s.frames.len()).sum::<usize>(), frames);
        for (id, st) in &d.stories {
            assert_eq!(st.text, reopened.stories[id].text);
            assert_eq!(st.chars, reopened.stories[id].chars);
        }
    }
}
