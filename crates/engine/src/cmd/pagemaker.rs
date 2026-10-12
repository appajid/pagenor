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
use pagenor_doc::{Align, CharAttrs, Document, ParaAttrs, ParaFormat, SpreadRef};
#[cfg(not(target_arch = "wasm32"))]
use pagenor_geom::Rect;
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
const WARNING: &str = "Experimental PageMaker import: pages and editable text frames are recovered through the optional pmd2raw helper. Legacy font codes are retained; install the original fonts. Unicode conversion, original text threading, layers, master items, images and exact typography are not yet preserved. Verify every page before publishing and save as .pgnr.";

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
        let raw = native::convert(&bytes)?;
        let title = std::path::Path::new(name).file_stem().and_then(|n| n.to_str()).unwrap_or("PageMaker document");
        let (mut d, mut warnings, frames) = parse(&raw, title)?;
        let (fonts, faces, font_warnings) = path.map_or((None, 0, Vec::new()), |path| super::file::load_document_fonts(&mut d, path));
        warnings.extend(font_warnings);
        let db = pagenor_fonts::FontDb::global().scoped(d.font_scope);
        resolve_single_styles(&mut d, |family| db.styles(family))?;
        if p.get("recoverRepeatedStories").and_then(Value::as_bool).unwrap_or(true) {
            warnings.extend(recover_repeated_stories(&mut d)?);
        }
        let pages: usize = d.spreads.iter().map(|s| s.pages.len()).sum();
        // Import is always a new native document: Save can never overwrite the original.
        let mut st = DocState::new(d, None);
        st.fonts = fonts;
        let index = s.add_document(st);
        Ok(json!({"index":index,"pages":pages,"textFrames":frames,"documentFonts":faces,"importWarnings":warnings}))
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

    fn helper() -> std::ffi::OsString {
        if let Some(path) = std::env::var_os("PAGENOR_PAGEMAKER_CONVERTER") {
            return path;
        }
        #[cfg(target_os = "macos")]
        for path in ["/opt/homebrew/bin/pmd2raw", "/usr/local/bin/pmd2raw"] {
            if std::path::Path::new(path).is_file() {
                return path.into();
            }
        }
        if cfg!(target_os = "windows") { "pmd2raw.exe".into() } else { "pmd2raw".into() }
    }

    fn read_pipe(reader: impl Read, limit: usize, exceeded: &AtomicBool) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        reader.take(limit as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            exceeded.store(true, Ordering::Release);
        }
        Ok(bytes)
    }

    pub(super) fn convert(bytes: &[u8]) -> Result<String> {
        let mut input = tempfile::NamedTempFile::new().map_err(|e| bad("file.open", e.to_string()))?;
        input.write_all(bytes).map_err(|e| bad("file.open", e.to_string()))?;
        input.flush().map_err(|e| bad("file.open", e.to_string()))?;
        // No shell, no source-file-controlled executable or switches. The helper only sees
        // our bounded temporary copy, even if the original is modified during conversion.
        let mut child = Command::new(helper()).arg(input.path()).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
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
            String::from_utf8(output).map_err(|_| bad("file.open", "PageMaker helper returned invalid UTF-8"))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Frame {
    rect: Rect,
    text: String,
    chars: Vec<(Range<usize>, CharAttrs)>,
    paras: Vec<(Range<usize>, ParaAttrs)>,
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
fn parse(raw: &str, title: &str) -> Result<(Document, Vec<String>, usize)> {
    if raw.len() > MAX_BYTES {
        return Err(bad("file.open", "PageMaker event stream exceeds 64 MiB"));
    }
    let mut pages: Vec<(f64, f64, Vec<Frame>)> = Vec::new();
    let mut frame: Option<Frame> = None;
    let mut para: Option<(usize, ParaAttrs)> = None;
    let mut span: Option<(usize, CharAttrs)> = None;
    let mut colors = BTreeSet::new();
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
                if count > MAX_FRAMES {
                    return Err(bad("file.open", "too many PageMaker frames"));
                }
                let a = attrs(e)?;
                let (x, y, w, h) = (required(&a, "svg:x")?, required(&a, "svg:y")?, required(&a, "svg:width")?, required(&a, "svg:height")?);
                if w <= 0. || h <= 0. {
                    return Err(bad("file.open", "invalid PageMaker frame dimensions"));
                }
                if optional(&a, "librevenge:rotate")?.is_some_and(|v| v != 0.) {
                    *skipped.entry("frame rotation".into()).or_default() += 1;
                }
                frame = Some(Frame { rect: Rect::new(x, y, x + w, y + h), ..Default::default() });
            }
            "endTextObject" if frame.is_some() && para.is_none() => {
                let f = frame.take().ok_or_else(|| bad("file.open", "missing PageMaker frame"))?;
                pages.last_mut().ok_or_else(|| bad("file.open", "missing PageMaker page"))?.2.push(f);
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
    for (page, (_, _, frames)) in pages.into_iter().enumerate() {
        for f in frames {
            let (_, sid) = d.add_text_frame(SpreadRef::Doc(page), f.rect, layer, &f.text, ParaFormat::default())?;
            let story = d.story_mut(sid).ok_or_else(|| bad("file.open", "missing imported story"))?;
            for (range, attr) in f.chars {
                story.format_chars(range, |format| format.over = attr.clone());
            }
            // format_paras includes its end byte; do not modify the next paragraph.
            for (range, attr) in f.paras {
                story.format_paras(range.start..range.end.saturating_sub(1), |format| format.para = attr.clone());
            }
        }
    }
    let mut warnings = vec![WARNING.into()];
    warnings.extend(skipped.into_iter().map(|(kind, count)| format!("PageMaker: {count} unsupported {kind} event(s) omitted.")));
    Ok((d, warnings, count))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    const EVENTS: &str = "startDocument()\n  startPage(svg:height: 11in, svg:width: 8in)\n    startTextObject (svg:height: 3in, svg:width: 4in, svg:x: 1in, svg:y: 2in)\n      openParagraph (fo:text-align: center)\n        openSpan(fo:color: #123456, fo:font-size: 18pt, style:font-name: Legacy Medium)\n          insertText (a (b) తెలుగు)\n          insertLineBreak\n          insertTab\n          insertSpace\n          insertText (z)\n        closeSpan\n      closeParagraph\n    endTextObject\n  endPage\nendDocument\n";
    #[test]
    fn editable_text_geometry_styles_and_native_round_trip() {
        let (d, w, count) = parse(EVENTS, "test").unwrap();
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
        let (mut d, _, _) = parse(EVENTS, "test").unwrap();
        resolve_single_styles(&mut d, |_| vec!["Medium".into()]).unwrap();
        assert_eq!(d.stories.values().next().unwrap().chars[0].format.over.font_style.as_deref(), Some("Medium"));
        for choices in [vec![], vec!["Medium".into(), "Black".into()]] {
            let (mut d, _, _) = parse(EVENTS, "test").unwrap();
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
            assert!(parse(&raw, "x").is_err());
        }
        assert!(parse(&"x".repeat(1024 * 1024 + 1), "x").is_err());
    }
    #[test]
    fn maintains_multiple_pages_and_warns_about_omitted_artwork() {
        let raw =
            EVENTS.replace("  endPage\n", "    drawRectangle (svg:x: 0in)\n  endPage\n  startPage(svg:height: 11in, svg:width: 8in)\n  endPage\n");
        let (d, w, _) = parse(&raw, "x").unwrap();
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
