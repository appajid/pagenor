//! The native `.pgnr` format (previously `.pagenor`): a zip archive holding
//!
//! - `mimetype` (stored first, uncompressed): `application/vnd.pagenor+zip`
//! - `document.json`: the [`Document`] (pretty JSON, documented by the serde model in `pagenor-doc`)
//! - `assets/<id>.<ext>`: embedded placed files, byte-for-byte
//! - `meta.json`: format version and generator
//!
//! Older single-file JSON documents (assets as base64 in `assetData`) still open. Documents
//! from older format versions are upgraded on load so they lay out as they did.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;

use pagenor_doc::{AssetId, Document, ParaAttrs, Story};
use serde_json::{Value, json};

pub use pagenor_doc::file_type::MIME;
pub const VERSION: u32 = 2;

const MAX_ARCHIVE_BYTES: usize = 512 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 4_096;
const MAX_CENTRAL_DIRECTORY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_EXPANDED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_META_BYTES: usize = 1024 * 1024;
const MAX_DOCUMENT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
struct ArchiveLimits {
    entries: usize,
    entry_bytes: u64,
    expanded_bytes: u64,
}

const ARCHIVE_LIMITS: ArchiveLimits =
    ArchiveLimits { entries: MAX_ARCHIVE_ENTRIES, entry_bytes: MAX_ENTRY_BYTES, expanded_bytes: MAX_EXPANDED_BYTES };

const PREFLIGHT_LIMITS: pagenor_archive::Limits =
    pagenor_archive::Limits { max_entries: MAX_ARCHIVE_ENTRIES as u64, max_central_directory_bytes: MAX_CENTRAL_DIRECTORY_BYTES };

#[derive(Clone, Copy)]
struct SaveLimits {
    entries: usize,
    entry_bytes: u64,
    expanded_bytes: u64,
    meta_bytes: usize,
    document_bytes: usize,
}

const SAVE_LIMITS: SaveLimits = SaveLimits {
    entries: MAX_ARCHIVE_ENTRIES,
    entry_bytes: MAX_ENTRY_BYTES,
    expanded_bytes: MAX_EXPANDED_BYTES,
    meta_bytes: MAX_META_BYTES,
    document_bytes: MAX_DOCUMENT_BYTES,
};

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("not a Pagenor document: {0}")]
    NotOurs(String),
    #[error("document is from a newer Pagenor (format {0}); please update")]
    TooNew(u64),
    #[error("{0}")]
    Io(String),
}

fn ext_for(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/tiff" => "tif",
        "application/pdf" => "pdf",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}

/// Serialize a document to `.pgnr` bytes.
pub fn save(doc: &Document) -> Result<Vec<u8>, FormatError> {
    let io = |e: zip::result::ZipError| FormatError::Io(e.to_string());
    let meta = json!({"format": "pagenor", "version": VERSION, "generator": concat!("Pagenor ", env!("CARGO_PKG_VERSION"))}).to_string();
    let json = serde_json::to_vec_pretty(doc).map_err(|e| FormatError::Io(e.to_string()))?;
    validate_save_parts(json.len(), meta.len(), doc.assets.values().filter(|a| !a.data.is_empty()).map(|a| a.data.len()), SAVE_LIMITS)?;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let deflate = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        z.start_file("mimetype", stored).map_err(io)?;
        z.write_all(MIME.as_bytes()).map_err(|e| FormatError::Io(e.to_string()))?;
        z.start_file("meta.json", deflate).map_err(io)?;
        z.write_all(meta.as_bytes()).map_err(|e| FormatError::Io(e.to_string()))?;
        z.start_file("document.json", deflate).map_err(io)?;
        z.write_all(&json).map_err(|e| FormatError::Io(e.to_string()))?;
        for (id, a) in &doc.assets {
            if a.data.is_empty() {
                continue;
            }
            // Already-compressed images are stored as-is.
            let opts = if matches!(a.mime.as_str(), "image/png" | "image/jpeg" | "image/gif" | "image/webp") { stored } else { deflate };
            z.start_file(format!("assets/{}.{}", id.0, ext_for(&a.mime)), opts).map_err(io)?;
            z.write_all(&a.data).map_err(|e| FormatError::Io(e.to_string()))?;
        }
        z.finish().map_err(io)?;
    }
    let bytes = buf.into_inner();
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(FormatError::Io(format!("saved file exceeds the {MAX_ARCHIVE_BYTES}-byte size limit")));
    }
    pagenor_archive::preflight(&bytes, PREFLIGHT_LIMITS).map_err(|e| FormatError::Io(format!("saved ZIP is outside format limits: {e}")))?;
    Ok(bytes)
}

fn validate_save_parts(
    document_bytes: usize,
    meta_bytes: usize,
    asset_sizes: impl Iterator<Item = usize>,
    limits: SaveLimits,
) -> Result<(), FormatError> {
    if document_bytes > limits.document_bytes {
        return Err(FormatError::Io(format!("document JSON exceeds the {}-byte size limit", limits.document_bytes)));
    }
    if meta_bytes > limits.meta_bytes {
        return Err(FormatError::Io(format!("metadata exceeds the {}-byte size limit", limits.meta_bytes)));
    }
    let mut entries = 3usize;
    if entries > limits.entries {
        return Err(FormatError::Io(format!("saved document contains more than {} entries", limits.entries)));
    }
    let mut expanded = MIME
        .len()
        .checked_add(meta_bytes)
        .and_then(|n| n.checked_add(document_bytes))
        .ok_or_else(|| FormatError::Io("saved document size overflow".into()))?;
    for size in asset_sizes {
        entries = entries.checked_add(1).ok_or_else(|| FormatError::Io("saved document entry count overflow".into()))?;
        if entries > limits.entries {
            return Err(FormatError::Io(format!("saved document contains more than {} entries", limits.entries)));
        }
        let size_u64 = u64::try_from(size).map_err(|_| FormatError::Io("saved asset size does not fit the file format".into()))?;
        if size_u64 > limits.entry_bytes {
            return Err(FormatError::Io(format!("saved asset exceeds the {}-byte entry limit", limits.entry_bytes)));
        }
        expanded = expanded.checked_add(size).ok_or_else(|| FormatError::Io("saved document expanded size overflow".into()))?;
    }
    let expanded = u64::try_from(expanded).map_err(|_| FormatError::Io("saved document expanded size does not fit the file format".into()))?;
    if expanded > limits.expanded_bytes {
        return Err(FormatError::Io(format!("saved document expands beyond the {}-byte limit", limits.expanded_bytes)));
    }
    Ok(())
}

/// Read `.pgnr` / `.pagenor` bytes (zip, or the legacy single JSON file).
pub fn load(bytes: &[u8]) -> Result<Document, FormatError> {
    if bytes.starts_with(b"PK") {
        if bytes.len() > MAX_ARCHIVE_BYTES {
            return Err(FormatError::NotOurs(format!("file exceeds the {MAX_ARCHIVE_BYTES}-byte size limit")));
        }
        return load_zip(bytes);
    }
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(FormatError::NotOurs(format!("legacy document exceeds the {MAX_DOCUMENT_BYTES}-byte size limit")));
    }
    load_legacy_json(bytes)
}

fn validate_archive(z: &mut zip::ZipArchive<Cursor<&[u8]>>, limits: ArchiveLimits) -> Result<(), FormatError> {
    if z.len() > limits.entries {
        return Err(FormatError::NotOurs(format!("archive contains more than {} entries", limits.entries)));
    }
    let mut expanded = 0u64;
    for i in 0..z.len() {
        let f = z.by_index(i).map_err(|e| FormatError::NotOurs(e.to_string()))?;
        let size = f.size();
        if size > limits.entry_bytes {
            return Err(FormatError::NotOurs(format!("archive entry exceeds the {}-byte limit", limits.entry_bytes)));
        }
        expanded = expanded.checked_add(size).ok_or_else(|| FormatError::NotOurs("archive expanded size is too large".into()))?;
        if expanded > limits.expanded_bytes {
            return Err(FormatError::NotOurs(format!("archive expands beyond the {}-byte limit", limits.expanded_bytes)));
        }
    }
    Ok(())
}

fn read_entry(z: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str, max_bytes: usize, remaining: &mut usize) -> Result<Option<Vec<u8>>, FormatError> {
    let mut f = match z.by_name(name) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(FormatError::NotOurs(e.to_string())),
    };
    let max_bytes = max_bytes.min(*remaining);
    if f.size() > max_bytes as u64 {
        return Err(FormatError::NotOurs(format!("{name} exceeds the {max_bytes}-byte limit")));
    }
    let capacity = usize::try_from(f.size()).unwrap_or(max_bytes).min(max_bytes);
    let mut v = Vec::with_capacity(capacity);
    (&mut f).take(max_bytes.saturating_add(1) as u64).read_to_end(&mut v).map_err(|e| FormatError::NotOurs(format!("can't read {name}: {e}")))?;
    if v.len() > max_bytes {
        return Err(FormatError::NotOurs(format!("{name} exceeds the {max_bytes}-byte limit")));
    }
    *remaining = remaining.checked_sub(v.len()).ok_or_else(|| FormatError::NotOurs("archive expanded size overflow".into()))?;
    Ok(Some(v))
}

fn load_zip(bytes: &[u8]) -> Result<Document, FormatError> {
    let directory = pagenor_archive::preflight(bytes, PREFLIGHT_LIMITS).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    if directory.entries != z.len() as u64 {
        return Err(FormatError::NotOurs("archive contains duplicate or inconsistent entries".into()));
    }
    let mut remaining = MAX_EXPANDED_BYTES as usize;
    validate_archive(&mut z, ARCHIVE_LIMITS)?;
    let version = read_entry(&mut z, "meta.json", MAX_META_BYTES, &mut remaining)?
        .and_then(|meta| serde_json::from_slice::<Value>(&meta).ok())
        .and_then(|m| m.get("version").and_then(Value::as_u64))
        .unwrap_or(1);
    if version > u64::from(VERSION) {
        return Err(FormatError::TooNew(version));
    }
    let json = read_entry(&mut z, "document.json", MAX_DOCUMENT_BYTES, &mut remaining)?
        .ok_or_else(|| FormatError::NotOurs("missing document.json".into()))?;
    let mut v: Value = serde_json::from_slice(&json).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    upgrade_json(&mut v);
    let mut doc: Document = serde_json::from_value(v).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    let names: HashSet<String> = z.file_names().filter(|name| name.starts_with("assets/")).map(str::to_string).collect();
    let mut loaded_assets = HashSet::new();
    for name in names {
        let Some(stem) = name.strip_prefix("assets/") else { continue };
        let Some(id) = stem.split('.').next().and_then(|s| s.parse::<u64>().ok()) else { continue };
        let Some(a) = doc.assets.get_mut(&AssetId(id)) else { continue };
        if !loaded_assets.insert(id) {
            return Err(FormatError::NotOurs(format!("multiple embedded files for asset {id}")));
        }
        if let Some(data) = read_entry(&mut z, &name, MAX_ENTRY_BYTES as usize, &mut remaining)? {
            Arc::make_mut(a).data = Arc::new(data);
        }
    }
    upgrade(&mut doc, version);
    doc.check().map_err(|e| FormatError::NotOurs(e.to_string()))?;
    Ok(doc)
}

/// Bring a document saved in format `from` up to [`VERSION`].
fn upgrade(doc: &mut Document, from: u64) {
    if from < 2 {
        // Format 1 laid out an unset Keep Lines Together mode as all lines in paragraph, and drop
        // caps without Align Left Edge.
        pin_para_default(doc, |a| &mut a.keep_all_lines, true);
        pin_para_default(doc, |a| &mut a.drop_cap_align_left, false);
    }
}

/// Pin a paragraph attribute to the value an older format resolved it to where nothing set it:
/// at the start of each paragraph style chain that leaves it unset, and on paragraphs whose style
/// is missing.
fn pin_para_default<T: Clone>(doc: &mut Document, field: fn(&mut ParaAttrs) -> &mut Option<T>, old: T) {
    let unset = |a: &ParaAttrs| field(&mut a.clone()).is_none();
    // Match style-chain resolution's 32-step/cycle bound, with indexed names. Walking
    // para_chain for every style would repeatedly scan the entire hostile style list.
    let mut indices = std::collections::HashMap::new();
    for (i, style) in doc.styles.paragraph.iter().enumerate() {
        indices.entry(style.name.clone()).or_insert(i);
    }
    let mut roots = HashSet::new();
    for start in 0..doc.styles.paragraph.len() {
        let mut chain = Vec::new();
        let mut next = Some(start);
        let mut all_unset = true;
        while let Some(i) = next {
            if chain.len() >= 32 || chain.contains(&i) {
                break;
            }
            let Some(style) = doc.styles.paragraph.get(i) else { break };
            chain.push(i);
            if !unset(&style.para) {
                all_unset = false;
                break;
            }
            next = style.based_on.as_ref().and_then(|name| indices.get(name)).copied();
        }
        if all_unset && let Some(i) = chain.last() {
            roots.insert(*i);
        }
    }
    if !roots.is_empty() {
        for (i, style) in doc.styles_mut().paragraph.iter_mut().enumerate() {
            if roots.contains(&i) {
                *field(&mut style.para) = Some(old.clone());
            }
        }
    }
    let mut pin = |st: &mut Story| {
        for p in st.paras.iter_mut().filter(|p| !indices.contains_key(&p.style)) {
            let v = field(&mut p.para);
            if v.is_none() {
                *v = Some(old.clone());
            }
        }
    };
    for st in doc.stories.values_mut() {
        let st = Arc::make_mut(st);
        st.for_each_text_mut(&mut pin);
    }
}

fn load_legacy_json(bytes: &[u8]) -> Result<Document, FormatError> {
    let mut v: Value = serde_json::from_slice(bytes).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    upgrade_json(&mut v);
    let mut doc: Document = serde_json::from_value(v.clone()).map_err(|e| FormatError::NotOurs(e.to_string()))?;
    if let Some(data) = v.get("assetData").and_then(Value::as_object) {
        for (k, s) in data {
            if let (Ok(id), Some(s)) = (k.parse::<u64>(), s.as_str())
                && let Some(a) = doc.assets.get_mut(&AssetId(id))
            {
                Arc::make_mut(a).data = Arc::new(base64_decode(s));
            }
        }
    }
    upgrade(&mut doc, 1);
    doc.check().map_err(|e| FormatError::NotOurs(e.to_string()))?;
    Ok(doc)
}

/// Bring older documents' JSON up to the current model. Vertical Type was a text frame option
/// (`content.options.vertical`); it is the story's direction, which a story takes from its first
/// frame.
fn upgrade_json(doc: &mut Value) {
    let mut vertical = HashSet::new();
    vertical_frames(doc, &mut vertical, 0);
    if vertical.is_empty() {
        return;
    }
    let Some(stories) = doc.get_mut("stories").and_then(Value::as_object_mut) else { return };
    for st in stories.values_mut() {
        let first = st.get("frames").and_then(|f| f.get(0)).and_then(Value::as_u64);
        if let (Some(f), Some(st)) = (first, st.as_object_mut())
            && vertical.contains(&f)
            && !st.contains_key("vertical")
        {
            st.insert("vertical".into(), Value::Bool(true));
        }
    }
}

/// Ids of the text frames in `v` (any depth: groups, anchored objects) whose options say vertical.
fn vertical_frames(v: &Value, out: &mut HashSet<u64>, depth: usize) {
    // serde_json already refuses JSON nested deeper than 128 levels.
    if depth > 256 {
        return;
    }
    match v {
        Value::Object(m) => {
            if let (Some(id), Some(c)) = (m.get("id").and_then(Value::as_u64), m.get("content"))
                && c.get("type").and_then(Value::as_str) == Some("text")
                && c.get("options").and_then(|o| o.get("vertical")).and_then(Value::as_bool) == Some(true)
            {
                out.insert(id);
            }
            for c in m.values() {
                vertical_frames(c, out, depth + 1);
            }
        }
        Value::Array(a) => {
            for c in a {
                vertical_frames(c, out, depth + 1);
            }
        }
        _ => {}
    }
}

fn base64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut buf, mut bits) = (0u32, 0);
    for b in s.bytes() {
        let v = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => continue,
        } as u32;
        buf = buf << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pagenor_doc::build::NewDocument;
    use pagenor_doc::geom::Rect;
    use pagenor_doc::{Asset, ParaFormat, SpreadRef};

    #[test]
    fn roundtrip_with_assets() {
        let mut d = Document::new(&NewDocument { pages: 3, ..Default::default() });
        let lid = d.default_layer();
        d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 200.0, 200.0), lid, "héllo\nworld", ParaFormat::default()).unwrap();
        let aid = AssetId(d.alloc());
        d.assets.insert(
            aid,
            Arc::new(Asset {
                page: 0,
                id: aid,
                name: "x.png".into(),
                mime: "image/png".into(),
                link: None,
                data: Arc::new(vec![1, 2, 3, 4]),
                pixels: Some((1, 1)),
            }),
        );
        let bytes = save(&d).unwrap();
        assert!(bytes.starts_with(b"PK"));
        let back = load(&bytes).unwrap();
        assert_eq!(back.page_count(), 3);
        assert_eq!(*back.assets[&aid].data, vec![1, 2, 3, 4]);
        assert_eq!(back.stories, d.stories);
    }

    /// `doc` as format 1 wrote it.
    fn save_v1(doc: &Document) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
        z.start_file("meta.json", zip::write::SimpleFileOptions::default()).unwrap();
        z.write_all(br#"{"format": "pagenor", "version": 1}"#).unwrap();
        z.start_file("document.json", zip::write::SimpleFileOptions::default()).unwrap();
        z.write_all(&serde_json::to_vec(doc).unwrap()).unwrap();
        z.finish().unwrap().into_inner()
    }

    fn para_style(name: &str, based_on: Option<&str>, keep_all_lines: Option<bool>) -> pagenor_doc::ParagraphStyle {
        pagenor_doc::ParagraphStyle {
            name: name.into(),
            based_on: based_on.map(str::to_string),
            next_style: None,
            para: pagenor_doc::ParaAttrs { keep_lines_together: Some(true), keep_all_lines, ..Default::default() },
            chars: Default::default(),
            shortcut: String::new(),
        }
    }

    /// Format 1 resolved an unset Keep Lines Together mode as all lines: those documents still
    /// do after loading, wherever the mode comes from. New documents keep lines at start/end.
    #[test]
    fn format_1_keep_lines_without_mode_stays_all_lines() {
        let mut d = Document::new(&NewDocument::default());
        let st = d.styles_mut();
        st.paragraph.push(para_style("Root", None, None));
        st.paragraph.push(para_style("Child", Some("Root"), None));
        st.paragraph.push(para_style("Explicit", None, Some(false)));
        st.paragraph.push(para_style("Under explicit", Some("Explicit"), None));
        st.paragraph.push(para_style("Loop A", Some("Loop B"), None));
        st.paragraph.push(para_style("Loop B", Some("Loop A"), None));
        let lid = d.default_layer();
        let names = [pagenor_doc::story::BASIC_PARAGRAPH, "Root", "Child", "Explicit", "Under explicit", "Loop A", "Loop B", "Missing"];
        let text = names.join("\n");
        let (_, sid) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 200.0, 200.0), lid, &text, ParaFormat::default()).unwrap();
        {
            let s = d.story_mut(sid).unwrap();
            for (p, name) in s.paras.iter_mut().zip(names) {
                p.style = name.into();
                p.para.keep_lines_together = Some(true);
            }
            s.insert_note(1, "Note", ParaFormat { style: "Missing".into(), ..Default::default() });
        }
        let back = load(&save_v1(&d)).unwrap();
        let resolve = |f: &ParaFormat| back.styles.resolve_para(f).0;
        let story = back.story(sid).unwrap();
        for (p, name) in story.paras.iter().zip(names) {
            let pp = resolve(p);
            assert!(pp.keep_lines_together);
            assert_eq!(pp.keep_all_lines, !name.contains("xplicit"), "{name}");
        }
        assert!(resolve(&story.notes[0].text.paras[0]).keep_all_lines, "footnote with a missing style");
        assert!(back.styles.resolve_para_style("Loop A").0.keep_all_lines && back.styles.resolve_para_style("Loop B").0.keep_all_lines);
        // Pinned where the chains start; explicit and inheriting styles are left as they were.
        assert_eq!(back.styles.para("Root").unwrap().para.keep_all_lines, Some(true));
        assert_eq!(back.styles.para("Child").unwrap().para.keep_all_lines, None);
        assert_eq!(back.styles.para("Under explicit").unwrap().para.keep_all_lines, None);
        // The current format keeps the new default: at start/end of paragraph.
        let now = load(&save(&d).unwrap()).unwrap();
        let p = &now.story(sid).unwrap().paras[0];
        assert!(!now.styles.resolve_para(p).0.keep_all_lines);
        assert!(!pagenor_doc::ParaProps::default().keep_all_lines);
    }

    #[test]
    fn format_1_migration_handles_many_independent_style_chains_and_endnotes() {
        let mut d = Document::new(&NewDocument::default());
        for i in 0..2000 {
            d.styles_mut().paragraph.push(para_style(&format!("Root {i}"), None, None));
            d.styles_mut().paragraph.push(para_style(&format!("Child {i}"), Some(&format!("Root {i}")), None));
        }
        let (_, sid) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(0.0, 0.0, 100.0, 100.0), d.default_layer(), "x", ParaFormat::default()).unwrap();
        d.story_mut(sid).unwrap().endnotes.push(Arc::new(pagenor_doc::Footnote {
            id: 1,
            text: Story::with_text(sid, "note", ParaFormat { style: "Missing".into(), ..Default::default() }),
        }));
        upgrade(&mut d, 1);
        for i in 0..2000 {
            assert_eq!(d.styles.para(&format!("Root {i}")).unwrap().para.keep_all_lines, Some(true));
            assert_eq!(d.styles.para(&format!("Child {i}")).unwrap().para.keep_all_lines, None);
        }
        assert_eq!(d.story(sid).unwrap().endnotes[0].text.paras[0].para.keep_all_lines, Some(true));
    }

    /// Format 1 wrote paragraph rules, text frame options and numbered lists in full, so their
    /// old defaults (black rules, auto-size from the top centre, lists continuing across stories)
    /// load as saved. New documents take InDesign's.
    #[test]
    fn format_1_rules_frames_and_lists_load_as_saved() {
        let mut d = Document::new(&NewDocument::default());
        let lid = d.default_layer();
        let old_rule = pagenor_doc::Rule { on: true, color: pagenor_doc::color::swatch::BLACK.into(), ..Default::default() };
        let (fid, sid) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 200.0, 200.0), lid, "Ruled", ParaFormat::default()).unwrap();
        d.story_mut(sid).unwrap().paras[0].para.rule_above = Some(old_rule.clone());
        d.item_mut(fid).unwrap().text_frame_mut().unwrap().options.auto_size_ref = 1;
        d.settings.lists.push(pagenor_doc::NumberedList { name: "Steps".into(), continue_across_stories: true });
        let back = load(&save_v1(&d)).unwrap();
        assert_eq!(back.story(sid).unwrap().paras[0].para.rule_above, Some(old_rule));
        assert_eq!(back.item(fid).unwrap().text_frame().unwrap().options.auto_size_ref, 1);
        assert!(back.settings.lists[0].continue_across_stories);
        assert_eq!(pagenor_doc::Rule::default().color, pagenor_doc::TEXT_COLOR);
        assert_eq!(pagenor_doc::TextFrameOptions::default().auto_size_ref, 4);
        assert!(!pagenor_doc::NumberedList::default().continue_across_stories);
    }

    /// Format 1 set drop caps without Align Left Edge: those documents still do where nothing set
    /// it. New documents align the drop cap's left edge.
    #[test]
    fn format_1_drop_caps_stay_unaligned() {
        let mut d = Document::new(&NewDocument::default());
        let st = d.styles_mut();
        st.paragraph.push(para_style("Opener", None, None));
        st.paragraph.push(pagenor_doc::ParagraphStyle {
            para: pagenor_doc::ParaAttrs { drop_cap_align_left: Some(true), ..Default::default() },
            ..para_style("Aligned", None, None)
        });
        let lid = d.default_layer();
        let (_, sid) =
            d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 200.0, 200.0), lid, "One\nTwo\nThree", ParaFormat::default()).unwrap();
        {
            let s = d.story_mut(sid).unwrap();
            for (p, name) in s.paras.iter_mut().zip(["Opener", "Aligned", "Missing"]) {
                p.style = name.into();
                p.para.drop_cap_lines = Some(2);
                p.para.drop_cap_chars = Some(1);
            }
        }
        let back = load(&save_v1(&d)).unwrap();
        let aligned: Vec<bool> = back.story(sid).unwrap().paras.iter().map(|p| back.styles.resolve_para(p).0.drop_cap_align_left).collect();
        assert_eq!(aligned, [false, true, false]);
        let now = load(&save(&d).unwrap()).unwrap();
        assert!(now.styles.resolve_para(&now.story(sid).unwrap().paras[0]).0.drop_cap_align_left);
    }

    /// The object of the item `id` in a document's JSON.
    fn item_json(v: &mut Value, id: u64) -> Option<&mut Value> {
        if v.get("id").and_then(Value::as_u64) == Some(id) && v.get("content").is_some() {
            return Some(v);
        }
        match v {
            Value::Object(m) => m.values_mut().find_map(|c| item_json(c, id)),
            Value::Array(a) => a.iter_mut().find_map(|c| item_json(c, id)),
            _ => None,
        }
    }

    #[test]
    fn frame_level_vertical_text_opens_as_a_vertical_story() {
        // Documents that set Vertical on the frame: the story takes its first frame's direction.
        let mut d = Document::new(&NewDocument::default());
        let lid = d.default_layer();
        let (a, sid) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(72.0, 72.0, 200.0, 400.0), lid, "縦書き", ParaFormat::default()).unwrap();
        let (b, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(300.0, 72.0, 428.0, 400.0), lid, "", ParaFormat::default()).unwrap();
        d.thread(a, b).unwrap();
        let (c, other) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(450.0, 72.0, 550.0, 400.0), lid, "横", ParaFormat::default()).unwrap();
        let mut v = serde_json::to_value(&d).unwrap();
        item_json(&mut v, a.0).unwrap()["content"]["options"]["vertical"] = json!(true);
        item_json(&mut v, c.0).unwrap()["content"]["options"]["vertical"] = json!(false);
        let json = serde_json::to_vec(&v).unwrap();
        let mut zipped = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut zipped);
            z.start_file("document.json", zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(&json).unwrap();
            z.finish().unwrap();
        }
        for bytes in [json.clone(), zipped.into_inner()] {
            let back = load(&bytes).unwrap();
            assert!(back.stories[&sid].vertical);
            assert!(!back.stories[&other].vertical);
            // Saved again, the direction is the story's.
            let again = load(&save(&back).unwrap()).unwrap();
            assert!(again.stories[&sid].vertical && !again.stories[&other].vertical);
        }
    }

    #[test]
    fn documents_saved_before_glyph_fallback_keep_their_fallback_fonts() {
        let d = Document::new(&NewDocument::default());
        assert!(!d.settings.glyph_fallback);
        assert!(!load(&save(&d).unwrap()).unwrap().settings.glyph_fallback, "saved and read as off");
        let mut on = d.clone();
        on.settings.glyph_fallback = true;
        assert!(load(&save(&on).unwrap()).unwrap().settings.glyph_fallback);
        // Written before the setting existed: drawn from fallback fonts, as they were.
        let mut v = serde_json::to_value(&d).unwrap();
        v["settings"].as_object_mut().unwrap().remove("glyphFallback").unwrap();
        assert!(load(&serde_json::to_vec(&v).unwrap()).unwrap().settings.glyph_fallback);
    }

    #[test]
    fn rejects_garbage_and_newer_versions() {
        assert!(load(b"nope").is_err());
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            z.start_file("meta.json", zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(br#"{"version": 99}"#).unwrap();
            z.finish().unwrap();
        }
        assert!(matches!(load(&buf.into_inner()), Err(FormatError::TooNew(99))));
    }

    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            for (name, body) in entries {
                z.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
                z.write_all(body).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn archive_resource_limits_cover_count_entry_and_total_sizes() {
        let bytes = archive(&[("one", b"123"), ("two", b"456")]);
        let limits = |entries, entry_bytes, expanded_bytes| ArchiveLimits { entries, entry_bytes, expanded_bytes };

        let mut z = zip::ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        assert!(validate_archive(&mut z, limits(1, 10, 10)).is_err());
        let mut z = zip::ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        assert!(validate_archive(&mut z, limits(2, 2, 10)).is_err());
        let mut z = zip::ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        assert!(validate_archive(&mut z, limits(2, 10, 5)).is_err());
        let mut z = zip::ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        assert!(validate_archive(&mut z, limits(2, 3, 6)).is_ok());
    }

    #[test]
    fn save_enforces_the_same_entry_and_expanded_limits_as_load() {
        let base = u64::try_from(MIME.len() + 8).unwrap();
        let limits = SaveLimits { entries: 4, entry_bytes: 3, expanded_bytes: base + 3, meta_bytes: 4, document_bytes: 4 };
        assert!(validate_save_parts(4, 4, [3].into_iter(), limits).is_ok());
        assert!(validate_save_parts(5, 4, std::iter::empty(), limits).is_err());
        assert!(validate_save_parts(4, 4, [4].into_iter(), limits).is_err());
        assert!(validate_save_parts(4, 4, [1, 1].into_iter(), limits).is_err());
        assert!(validate_save_parts(4, 4, [3, 3].into_iter(), SaveLimits { entries: 5, expanded_bytes: base + 5, ..limits }).is_err());
    }

    #[test]
    fn load_rejects_excessive_entry_count_in_the_zip_preflight() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            for i in 0..=MAX_ARCHIVE_ENTRIES {
                z.start_file(format!("entry-{i}"), zip::write::SimpleFileOptions::default()).unwrap();
            }
            z.finish().unwrap();
        }
        assert!(matches!(load(&buf.into_inner()), Err(FormatError::NotOurs(message)) if message.contains("entries")));
    }
    #[test]
    fn hardening_corrupt_embedded_asset_is_rejected() {
        let mut d = Document::new(&NewDocument::default());
        let aid = AssetId(d.alloc());
        d.assets.insert(
            aid,
            Arc::new(Asset {
                page: 0,
                id: aid,
                name: "fixture.bin".into(),
                mime: "application/octet-stream".into(),
                link: None,
                data: Arc::new(vec![1, 2, 3, 4]),
                pixels: None,
            }),
        );
        let mut bytes = save(&d).unwrap();
        let name = format!("assets/{}.bin", aid.0);
        let at = bytes
            .windows(4)
            .enumerate()
            .find_map(|(at, sig)| {
                if sig != b"PK\x01\x02" {
                    return None;
                }
                let len = u16::from_le_bytes(bytes[at + 28..at + 30].try_into().unwrap()) as usize;
                (bytes.get(at + 46..at + 46 + len) == Some(name.as_bytes())).then_some(at)
            })
            .unwrap();
        // Central-directory checksum no longer describes the stored data.
        bytes[at + 16] ^= 1;
        assert!(load(&bytes).is_err());
    }

    #[test]
    fn huge_format_version_cannot_wrap_to_a_supported_version() {
        let version = u64::from(u32::MAX) + 2;
        let meta = json!({"version": version}).to_string();
        let bytes = archive(&[("meta.json", meta.as_bytes()), ("document.json", b"{}")]);
        assert!(matches!(load(&bytes), Err(FormatError::TooNew(v)) if v == version));
    }

    #[test]
    fn native_reads_share_the_actual_expanded_byte_budget() {
        let bytes = archive(&[("one", b"123"), ("two", b"456")]);
        let mut z = zip::ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        let mut remaining = 5;
        assert_eq!(read_entry(&mut z, "one", 10, &mut remaining).unwrap(), Some(b"123".to_vec()));
        assert_eq!(remaining, 2);
        assert!(read_entry(&mut z, "two", 10, &mut remaining).is_err());
        // An entry exactly at the remaining boundary is permitted.
        remaining = 3;
        assert_eq!(read_entry(&mut z, "two", 10, &mut remaining).unwrap(), Some(b"456".to_vec()));
        assert_eq!(remaining, 0);
    }

    #[test]
    fn conflicting_embedded_files_for_an_asset_are_rejected() {
        let mut d = Document::new(&NewDocument::default());
        let aid = AssetId(d.alloc());
        d.assets.insert(
            aid,
            Arc::new(Asset {
                page: 0,
                id: aid,
                name: "fixture.bin".into(),
                mime: "application/octet-stream".into(),
                link: None,
                data: Arc::new(vec![]),
                pixels: None,
            }),
        );
        let json = serde_json::to_vec(&d).unwrap();
        let first = format!("assets/{}.bin", aid.0);
        let second = format!("assets/{}.png", aid.0);
        let bytes = archive(&[("document.json", &json), (&first, b"first"), (&second, b"second")]);
        assert!(matches!(load(&bytes), Err(FormatError::NotOurs(message)) if message.contains("multiple embedded files")));
    }
}
