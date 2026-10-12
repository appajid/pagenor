//! File menu: new, open, save, place, export, close.

use std::sync::Arc;

use pagenor_doc::build::{NewDocument, PRESETS};
use pagenor_doc::{Asset, AssetId, Content, Document, Graphic, Item, ItemId, PdfCrop, Selection, Shape};
use pagenor_geom::{Affine, Rect, shapes};
use serde_json::{Value, json};

use super::{CommandSpec, always, bad, cmd, f64_or, has_doc, ok, str_param};
use crate::{DocState, EngineError, Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "file.new", "Document…", ["File", "New"], Some("Cmd+N"),
            "{preset?: \"Letter\"|\"A4\"|… or a saved preset (file.savePreset), intent?: print|web|mobile, units?: points|picas|millimeters|…, width?, height?, pages?, startPage?, facingPages?, columns?, gutter?, margins?: number|{top,bottom,inside,outside}, bleed?, slug?: n | [top, bottom, inside, outside], title?}",
            always, file_new),
        cmd!(noundo "file.newSample", "Sample Document", ["Help"], None, "{} — a multi-page magazine sample", always, file_sample),
        cmd!(query "file.presets", "Document Presets", [], None, "{} → the built-in presets [{name, intent, width, height, units}], then the saved ones (with `saved: true`)", always, |s, _| {
            let mut presets: Vec<Value> = PRESETS.iter().map(serde_json::to_value).collect::<std::result::Result<_, _>>().map_err(|e| EngineError::Other(e.to_string()))?;
            presets.extend(
                s.prefs
                    .document_presets
                    .iter()
                    .map(|p| json!({"name": p.title, "intent": p.intent, "width": p.width, "height": p.height, "units": p.units, "saved": true})),
            );
            Ok(Value::Array(presets))
        }),
        cmd!(noundo "file.savePreset", "Save Document Preset", [], None,
            "{name, …the settings file.new takes} — kept with the preferences; `file.new {preset: name}` uses it. A saved preset of that name is replaced; built-in names are refused",
            always, save_preset),
        cmd!(noundo "file.deletePreset", "Delete Document Preset", [], None, "{name} — a saved document preset", always, |s, p| {
            let name = str_param(p, "name").unwrap_or_default().trim().to_string();
            let before = s.prefs.document_presets.len();
            s.prefs.document_presets.retain(|x| !x.title.eq_ignore_ascii_case(&name));
            if s.prefs.document_presets.len() == before {
                return Err(bad("file.deletePreset", format!("no saved preset `{name}`")));
            }
            ok()
        }),
        cmd!(noundo "file.open", "Open…", ["File"], Some("Cmd+O"),
            "{path, maxBytes?: n, recoverRepeatedStories?: true} — .pgnr, .idml, .indd or experimental PageMaker .p65/.pmd (desktop pmd2raw required); optional actual-read limit; the fonts in a `Document Fonts` folder beside it load first → {index, documentFonts: faces loaded, warnings: skipped fonts and missing IDML package parts}",
            always, file_open),
        cmd!(noundo "file.openBytes", "Open Bytes", [], None, "{name, base64} — Pagenor document, IDML package, INDD subset or PageMaker (include .p65/.pmd in name; desktop helper required)", always, file_open_bytes),
        cmd!(noundo "file.save", "Save", ["File"], Some("Cmd+S"), "{path?}", has_doc, file_save),
        cmd!(noundo "file.saveAs", "Save As…", ["File"], Some("Cmd+Shift+S"), "{path}", has_doc, file_save),
        cmd!(noundo "file.saveACopy", "Save a Copy…", ["File"], None, "{path} — writes the document without changing which file it is or its unsaved state", has_doc, |s, p| {
            let path = str_param(p, "path").ok_or_else(|| bad("file.saveACopy", "missing `path`"))?.to_string();
            let bytes = to_bytes(&s.doc()?.doc)?;
            #[cfg(not(target_arch = "wasm32"))]
            crate::native_save::write(std::path::Path::new(&path), &bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
            Ok(json!({"path": path, "bytes": bytes.len()}))
        }),
        cmd!(noundo "file.revert", "Revert", ["File"], None, "{} — back to the last saved version (not undoable)", can_revert, file_revert),
        cmd!(query "file.recovery.list", "Recovery Data", [], None, "{dir?} → [{uid, path, title, saved}] unsaved documents left by a crash (default: the session's recovery folder)", always, |s, p| {
            let dir = recovery_dir(s, p)?;
            Ok(Value::Array(crate::recovery::list(&dir).into_iter().map(|(uid, mut m)| { m["uid"] = json!(uid); m }).collect()))
        }),
        cmd!(noundo "file.recovery.save", "Save Recovery Data", [], None, "{dir?} — write every unsaved document to the recovery folder (the app does this on a timer)", always, |s, p| {
            let dir = recovery_dir(s, p)?;
            Ok(json!({"saved": crate::recovery::save(s, &dir)?}))
        }),
        cmd!(noundo "file.recovery.open", "Recover Documents", [], None, "{dir?} — reopen documents left in the recovery folder (unsaved)", always, |s, p| {
            let dir = recovery_dir(s, p)?;
            Ok(json!({"opened": crate::recovery::open(s, &dir)?}))
        }),
        cmd!(query "file.serialize", "Serialize", [], None, "{} → {base64, bytes} the .pgnr file", has_doc, |s, _| {
            let st = s.doc()?;
            // Preview fill is session state. Serialize the unfilled template.
            let doc = st.preview_stash.as_deref().unwrap_or(st.doc.as_ref());
            let bytes = to_bytes(doc)?;
            Ok(json!({"base64": base64_encode(&bytes), "bytes": bytes.len()}))
        }),
        cmd!(noundo "file.close", "Close", ["File"], Some("Cmd+W"), "{index?}", has_doc, |s, p| {
            let i = p.get("index").and_then(Value::as_u64).map(|v| v as usize).or(s.active_index()).unwrap_or(0);
            s.close_document(i);
            ok()
        }),
        cmd!(noundo "file.activate", "Activate Document", [], None, "{index}", has_doc, |s, p| {
            s.set_active(p.get("index").and_then(Value::as_u64).unwrap_or(0) as usize);
            ok()
        }),
        cmd!(query "snippet.export", "Export Selection as Snippet", ["File", "Export"], None,
            "{path?} — the selected items (with their stories, styles and images) as a .pgnr snippet; no path: {base64}", super::has_selection, snippet_export),
        cmd!(query "place.styles", "Import Options: Styles", [], None,
        "{path | base64, name} → {paragraph: [name], character: [name], conflicts: [name]} — the styles a Word/RTF/Markdown file brings (for styleMap / styleConflicts)",
        has_doc, |s, p| {
            let (bytes, name, _) = read_source(p)?;
            let imp = pagenor_textimport::import(&name, &bytes).map_err(|e| bad("place.styles", e.to_string()))?;
            let st = &s.doc()?.doc.styles;
            let conflicts: Vec<&str> = imp
                .para_styles
                .iter()
                .filter(|x| st.para(&x.name).is_some())
                .chain(imp.char_styles.iter().filter(|x| st.char_style(&x.name).is_some()))
                .map(|x| x.name.as_str())
                .collect();
            Ok(json!({"paragraph": imp.para_styles.iter().map(|x| &x.name).collect::<Vec<_>>(), "character": imp.char_styles.iter().map(|x| &x.name).collect::<Vec<_>>(), "conflicts": conflicts}))
        }),
        cmd!(
            "snippet.place",
            "Place Snippet",
            [],
            None,
            "{path | base64, spread?, x?, y?} — items keep their positions unless x/y given (top-left)",
            has_doc,
            snippet_place
        ),
        cmd!(
            "place.load",
            "Load Place Cursor",
            [],
            None,
            "{path | base64, name?} — load a graphic into the place cursor (then click/drag with the placeGun tool)",
            has_doc,
            place_load
        ),
        cmd!("place.drop", "Place Loaded Graphic", [], None, "{spread?, x, y, rect?: [x0,y0,x1,y1], frame?: id}", has_doc, place_drop),
        cmd!(
            "file.place",
            "Place…",
            ["File"],
            Some("Cmd+D"),
            "{path?|base64?, name?, frame?: id (place into), spread?, x?, y?, width?, pdfPage?: n (1-based, Image Import Options), pdfCrop?: crop|trim|bleed|art|media, layoutPage?: n (IDML / .pgnr: that page's objects as a group)} — places an image into `frame` when given; otherwise any explicit x/y/spread/width creates a new frame; otherwise uses the selected empty or graphic frame, or creates a new frame; text files (.txt, .docx, .rtf, .md, .markdown) and Excel workbooks (.xlsx, as a table) go into the insertion point, the selected frame or a new frame on `page`/`rect` — {autoflow?: adds pages with threaded frames until the text fits, removeStyles?, styleMap?: {imported name: document style}, styleConflicts?: useExisting|redefine|autoRename}",
            has_doc,
            file_place
        ),
    ]
}

pub fn to_bytes(d: &Document) -> Result<Vec<u8>> {
    pagenor_format::save(d).map_err(|e| EngineError::Other(e.to_string()))
}

pub fn from_bytes(b: &[u8]) -> Result<Document> {
    pagenor_format::load(b).map_err(|e| EngineError::Other(e.to_string()))
}

/// The longest saved preset name, and how many presets are kept.
const MAX_PRESET_NAME: usize = 100;
const MAX_PRESETS: usize = 500;

/// A built-in document preset or a saved one, by name (case ignored).
fn find_preset(s: &Session, name: &str) -> Option<NewDocument> {
    NewDocument::from_preset(name.trim()).or_else(|| s.prefs.document_presets.iter().find(|p| p.title.eq_ignore_ascii_case(name.trim())).cloned())
}

fn file_new(s: &mut Session, p: &Value) -> Result<Value> {
    let mut nd = match str_param(p, "preset") {
        Some(name) => find_preset(s, name).ok_or_else(|| bad("file.new", format!("unknown preset `{name}`")))?,
        None => NewDocument::default(),
    };
    apply_settings(&mut nd, p);
    check_size("file.new", &nd)?;
    s.untitled = s.untitled.saturating_add(1);
    nd.title = str_param(p, "title").map(str::to_string).unwrap_or_else(|| format!("Untitled-{}", s.untitled));
    let d = Document::new(&nd);
    let i = s.add_document(DocState::new(d, None));
    Ok(json!({"index": i}))
}

fn save_preset(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").map(str::trim).filter(|n| !n.is_empty()).ok_or_else(|| bad("file.savePreset", "missing `name`"))?;
    if name.chars().count() > MAX_PRESET_NAME {
        return Err(bad("file.savePreset", format!("a preset name has at most {MAX_PRESET_NAME} characters")));
    }
    if NewDocument::from_preset(name).is_some() {
        return Err(bad("file.savePreset", format!("`{name}` is a built-in preset")));
    }
    let mut nd = NewDocument::default();
    apply_settings(&mut nd, p);
    check_size("file.savePreset", &nd)?;
    nd.title = name.to_string();
    let presets = &mut s.prefs.document_presets;
    let replacing = presets.iter().filter(|x| x.title.eq_ignore_ascii_case(name)).count();
    if presets.len().saturating_sub(replacing) >= MAX_PRESETS {
        return Err(bad("file.savePreset", format!("at most {MAX_PRESETS} saved presets")));
    }
    presets.retain(|x| !x.title.eq_ignore_ascii_case(name));
    presets.push(nd);
    Ok(json!({"name": name}))
}

fn check_size(cmd: &str, nd: &NewDocument) -> Result<()> {
    if !(nd.width > 0.0 && nd.height > 0.0 && nd.width <= 15552.0 && nd.height <= 15552.0) {
        return Err(bad(cmd, "page size out of range (0 < size ≤ 216 in)"));
    }
    if ![nd.gutter, nd.margins.top, nd.margins.bottom, nd.margins.inside, nd.margins.outside].into_iter().all(f64::is_finite) {
        return Err(bad(cmd, "gutter and margins must be finite measures"));
    }
    Ok(())
}

/// The document settings in `p` (as file.new takes them) over `nd`.
fn apply_settings(nd: &mut NewDocument, p: &Value) {
    if let Some(intent) = p.get("intent").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        nd.intent = intent;
    }
    if let Some(units) = p.get("units").and_then(|v| serde_json::from_value(v.clone()).ok()) {
        nd.units = units;
    }
    nd.width = f64_or(p, "width", nd.width);
    nd.height = f64_or(p, "height", nd.height);
    nd.pages = p.get("pages").and_then(Value::as_u64).map(|v| v.clamp(1, 9999) as usize).unwrap_or(nd.pages).clamp(1, 9999);
    nd.start_page = p.get("startPage").and_then(Value::as_u64).map(|v| v.clamp(1, 99_999) as u32).unwrap_or(nd.start_page).clamp(1, 99_999);
    nd.facing_pages = p.get("facingPages").and_then(Value::as_bool).unwrap_or(nd.facing_pages);
    nd.columns = p.get("columns").and_then(Value::as_u64).map(|v| v.clamp(1, 216) as u32).unwrap_or(nd.columns).clamp(1, 216);
    nd.gutter = f64_or(p, "gutter", nd.gutter);
    nd.primary_text_frame = p.get("primaryTextFrame").and_then(Value::as_bool).unwrap_or(nd.primary_text_frame);
    match p.get("margins") {
        Some(Value::Number(n)) => nd.margins = pagenor_doc::Margins::uniform(n.as_f64().unwrap_or(36.0)),
        Some(v @ Value::Object(_)) => {
            if let Ok(m) = serde_json::from_value(v.clone()) {
                nd.margins = m;
            }
        }
        _ => {}
    }
    if let Some(b) = edges(p, "bleed") {
        nd.bleed = b;
    }
    if let Some(b) = edges(p, "slug") {
        nd.slug = b;
    }
    // Preferences may come from older or hand-edited files: bound inherited edge widths too.
    nd.bleed = nd.bleed.map(clean_edge);
    nd.slug = nd.slug.map(clean_edge);
}

/// A bleed or slug parameter: one width for every edge, or [top, bottom, inside, outside].
fn clean_edge(v: f64) -> f64 {
    if v.is_finite() { v.clamp(0.0, 1296.0) } else { 0.0 }
}

fn edges(p: &Value, key: &str) -> Option<[f64; 4]> {
    match p.get(key)? {
        Value::Array(a) if a.len() == 4 => Some([
            clean_edge(a.first()?.as_f64()?),
            clean_edge(a.get(1)?.as_f64()?),
            clean_edge(a.get(2)?.as_f64()?),
            clean_edge(a.get(3)?.as_f64()?),
        ]),
        v => v.as_f64().map(|b| [clean_edge(b); 4]),
    }
}

fn file_sample(s: &mut Session, _p: &Value) -> Result<Value> {
    let d = crate::sample::magazine();
    let i = s.add_document(DocState::new(d, None));
    Ok(json!({"index": i}))
}

fn file_open(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").ok_or_else(|| bad("file.open", "missing `path`"))?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        if super::pagemaker::is_name(path) {
            return super::pagemaker::open(s, p);
        }
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".idml") || lower.ends_with(".indd") {
            return super::interchange::open_idml(s, p);
        }
        let limit = p.get("maxBytes").and_then(Value::as_u64).unwrap_or(512 * 1024 * 1024).min(512 * 1024 * 1024);
        let bytes = read_asset_file(path, limit)?;
        let mut d = from_bytes(&bytes)?;
        super::interchange::resolve_packaged_links(&mut d, std::path::Path::new(path).parent());
        super::datamerge::resolve_sources_on_open(&mut d, Some(std::path::Path::new(path)));
        let (fonts, faces, warnings) = load_document_fonts(&mut d, path);
        let mut st = DocState::new(d, Some(path.to_string()));
        st.fonts = fonts;
        let i = s.add_document(st);
        Ok(json!({"index": i, "documentFonts": faces, "warnings": warnings}))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, path);
        Err(EngineError::Other("use file.openBytes on the web".into()))
    }
}

/// Load the fonts in the `Document Fonts` folder beside the document file at `path` for `d` alone
/// (`d.font_scope`) → (the scope, for the document's state to hold; faces; a warning for each
/// font file skipped). Nothing on the web (no folders).
pub(crate) fn load_document_fonts(d: &mut Document, path: &str) -> (Option<Arc<crate::FontScope>>, usize, Vec<String>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use pagenor_fonts::DOCUMENT_FONTS_FOLDER;
        let Some(folder) = std::path::Path::new(path).parent().map(document_fonts_folder) else { return (None, 0, Vec::new()) };
        let r = pagenor_fonts::FontDb::global().load_document_fonts(&folder);
        d.font_scope = r.scope;
        let scope = (r.scope != 0).then(|| Arc::new(crate::FontScope(r.scope)));
        (scope, r.faces, r.skipped.into_iter().map(|s| format!("{DOCUMENT_FONTS_FOLDER}: {s}")).collect())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (d, path);
        (None, 0, Vec::new())
    }
}

/// The `Document Fonts` folder in `dir`, matched without regard to case: packages name it
/// `Document fonts`, and file systems on Linux are case-sensitive. The exact name wins.
#[cfg(not(target_arch = "wasm32"))]
fn document_fonts_folder(dir: &std::path::Path) -> std::path::PathBuf {
    use pagenor_fonts::DOCUMENT_FONTS_FOLDER;
    let exact = dir.join(DOCUMENT_FONTS_FOLDER);
    if exact.is_dir() {
        return exact;
    }
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .find(|e| e.file_name().to_str().is_some_and(|n| n.eq_ignore_ascii_case(DOCUMENT_FONTS_FOLDER)) && e.path().is_dir())
        .map_or(exact, |e| e.path())
}

fn file_open_bytes(s: &mut Session, p: &Value) -> Result<Value> {
    if str_param(p, "name").is_some_and(super::pagemaker::is_name) {
        return super::pagemaker::open(s, p);
    }
    let b = base64_decode(str_param(p, "base64").unwrap_or(""));
    // IDML packages are recognised by their stored `mimetype` first entry, InDesign documents by
    // their master page header.
    if pagenor_idml::is_idml(&b) || pagenor_indd::is_indd(&b) {
        return super::interchange::open_idml(s, p);
    }
    let mut d = from_bytes(&b)?;
    super::datamerge::resolve_sources_on_open(&mut d, None);
    if let Some(n) = str_param(p, "name") {
        d.title = n.to_string();
    }
    let i = s.add_document(DocState::new(d, None));
    Ok(json!({"index": i}))
}

fn file_save(s: &mut Session, p: &Value) -> Result<Value> {
    let st = s.doc_mut()?;
    let path = str_param(p, "path")
        .map(str::to_string)
        .or_else(|| st.path.clone())
        .ok_or_else(|| bad("file.save", "missing `path` (document has never been saved)"))?;
    // Prepare the new relative paths without changing the live document. Serialization or
    // writing can fail; only a successful save commits these paths and the saved state.
    let mut d = (*st.doc).clone();
    super::datamerge::refresh_relative_paths(&mut d, std::path::Path::new(&path));
    let bytes = to_bytes(&d)?;
    #[cfg(not(target_arch = "wasm32"))]
    crate::native_save::write(std::path::Path::new(&path), &bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
    if d.data_merge != st.doc.data_merge {
        st.doc = Arc::new(d);
        st.revision = st.revision.saturating_add(1);
    }
    st.path = Some(path.clone());
    st.saved_revision = st.revision;
    st.saved_doc = st.doc.clone();
    #[cfg(not(target_arch = "wasm32"))]
    crate::recovery::discard_document(st);
    Ok(json!({"path": path, "bytes": bytes.len()}))
}

fn file_place(s: &mut Session, p: &Value) -> Result<Value> {
    if str_param(p, "base64").is_none() && str_param(p, "path").is_none() {
        s.ui_requests.push(crate::UiRequest::Pick { purpose: "place".into(), params: json!({}) });
        return ok();
    }
    let (bytes, name, link) = read_source(p)?;
    // A page of another layout (IDML or Pagenor): its objects, as one group.
    let lname = name.to_lowercase();
    if lname.ends_with(".idml") || lname.ends_with(".indd") || pagenor_doc::file_type::is_native_name(&name) {
        let mut result = place_layout_page(s, p, &name, &bytes)?;
        if pagenor_indd::is_indd(&bytes) {
            result["importWarnings"] = json!([super::interchange::INDD_WARNING]);
        }
        return Ok(result);
    }
    // Keep the EPS source for link comparison; rendering uses its preview at the bounding box size.
    let eps_size = eps_size(&bytes, "file.place")?;
    // Video and sound: a media frame (Window › Interactive › Media).
    if let Some(mime) = pagenor_doc::media_mime(&name) {
        return super::media::place_media(s, p, name, mime, bytes, link);
    }
    // Data merge reads .xls. Place keeps .xlsx tables and does not grow a table from .xls or .xlsm.
    if lname.ends_with(".xls") || lname.ends_with(".xlsm") {
        let ext = if lname.ends_with(".xlsm") { "xlsm" } else { "xls" };
        return Err(bad("file.place", format!("{ext} workbooks are not placed as a table")));
    }
    // Text files (plain, Word, RTF) flow into frames.
    if pagenor_textimport::is_text_file(&name) {
        return super::place_text::place_text(s, p, &name, &bytes);
    }
    // Image Import Options: which page of a PDF (1-based).
    let pdf_page = match p.get("pdfPage").and_then(Value::as_u64) {
        Some(n) if pagenor_render::is_pdf(&bytes) => {
            let count = pagenor_render::pdf_page_count(&bytes).unwrap_or(1);
            let invalid_page = || bad("file.place", format!("the PDF has {count} page(s)"));
            let one_based = usize::try_from(n).map_err(|_| invalid_page())?;
            if one_based == 0 || one_based > count {
                return Err(invalid_page());
            }
            let zero_based = one_based.checked_sub(1).ok_or_else(invalid_page)?;
            u32::try_from(zero_based).map_err(|_| invalid_page())?
        }
        _ => 0,
    };
    let (pw, ph) = match pdf_page {
        0 => pagenor_render::image_size(&bytes).ok_or_else(|| bad("file.place", "unsupported or corrupt image"))?,
        n => pagenor_render::pdf_page_size(&bytes, n as usize)
            .map(|(w, h)| (w.round().max(1.0) as u32, h.round().max(1.0) as u32))
            .ok_or_else(|| bad("file.place", "can't read that PDF page"))?,
    };
    // 72 ppi by default unless the file says otherwise; scale so it fits the page when huge.
    let (nw, nh) = eps_size.or_else(|| pagenor_render::natural_size(&bytes, pdf_page as usize)).unwrap_or((pw as f64, ph as f64));
    // Image Import Options › Crop to: the graphic spans that box of the PDF page.
    let is_pdf = pagenor_render::is_pdf(&bytes);
    let pdf_crop = match str_param(p, "pdfCrop") {
        Some(k) if is_pdf => PdfCrop::from_name(k)
            .ok_or_else(|| bad("file.place", format!("unknown pdfCrop `{k}` (crop, art, trim, bleed, media, contentVisible, contentAll)")))?,
        _ => PdfCrop::Crop,
    };
    let mime = pagenor_render::image_mime(&bytes).to_string();
    let mut asset = Asset { page: pdf_page, name, mime, link, data: Arc::new(bytes), pixels: Some((pw, ph)), pdf_crop, ..Default::default() };
    // The graphic's natural size: the box in points (the whole page when the box can't be found,
    // and then the document says so).
    let (bw, bh) = match crop_pdf(&mut asset) {
        Some(size) => size,
        None => {
            asset.pdf_crop = PdfCrop::Crop;
            (nw, nh)
        }
    };
    let target_frame = super::id_param(p, "frame").or_else(|| {
        if ["x", "y", "spread", "width"].iter().any(|key| p.get(*key).is_some()) {
            return None;
        }
        let st = s.active()?;
        st.selection.items.iter().copied().find(|i| st.doc.item(*i).is_some_and(|it| matches!(it.content, Content::Unassigned | Content::Graphic(_))))
    });
    let spread = super::spread_param(p, "spread");
    let lid = s.doc()?.active_layer;
    let px = p.get("x").and_then(Value::as_f64);
    let py = p.get("y").and_then(Value::as_f64);
    let want_w = p.get("width").and_then(Value::as_f64);
    // Prepare and validate all geometry before allocating an asset or changing a frame.
    let d = &s.doc()?.doc;
    let (frame_rect, xf, auto_fit) = if let Some(fid) = target_frame {
        let r = d.item(fid).ok_or(pagenor_doc::DocError::NoItem(fid))?.inner_bounds();
        let k = (r.width() / bw).max(r.height() / bh);
        let xf = Affine::translate((r.x0 + (r.width() - bw * k) / 2.0, r.y0 + (r.height() - bh * k) / 2.0)) * Affine::scale(k);
        // Fill once; keep Auto-Fit off so resizing the frame crops the placed image.
        (r, xf, Default::default())
    } else {
        let sp = d.spread(spread).ok_or_else(|| bad("file.place", "no such spread"))?;
        let pr = sp.pages.first().map(|pg| pg.margin_rect()).unwrap_or(Rect::new(36.0, 36.0, 300.0, 300.0));
        let (x, y) = (px.unwrap_or(pr.x0), py.unwrap_or(pr.y0));
        let w = want_w.unwrap_or_else(|| nw.min(d.settings.page_width * 0.6));
        let k = w / nw;
        let r = Rect::new(x, y, x + bw * k, y + bh * k);
        let xf = Affine::translate((x, y)) * Affine::scale(k);
        (r, xf, Default::default())
    };
    validate_placement((bw, bh), frame_rect, xf, "file.place")?;
    s.edit(|d, sel| {
        let fresh = AssetId(d.alloc());
        let aid = d.add_asset(Asset { id: fresh, ..asset });
        let graphic = Content::Graphic(Graphic {
            clip: None,
            asset: aid,
            size: (bw, bh),
            xf,
            auto_fit,
            fit_align: 4,
            crop: [0.0; 4],
            wrap: Default::default(),
        });
        let id = if let Some(fid) = target_frame {
            let it = d.item_mut(fid).ok_or(pagenor_doc::DocError::NoItem(fid))?;
            it.content = graphic;
            fid
        } else {
            let id = ItemId(d.alloc());
            let mut it = Item::new(id, lid, Shape::Rectangle, shapes::rectangle(frame_rect));
            it.object_style = d.styles.default_graphic_frame.clone();
            it.content = graphic;
            d.insert_item(spread, it, None)?;
            id
        };
        *sel = Selection::items(vec![id]);
        Ok(json!({"id": id.0, "asset": aid.0}))
    })
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

pub fn base64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;
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

/// Sets `a.pdf_box` to where its `pdf_crop` box sits on its page and returns the box's size in
/// points. The crop box is the whole page and has none; nor has a page that can't be read, which
/// keeps its crop choice (a missing link relinked later gets its box then).
pub(crate) fn crop_pdf(a: &mut Asset) -> Option<(f64, f64)> {
    a.pdf_box = None;
    if a.pdf_crop == PdfCrop::Crop || !pagenor_render::is_pdf(&a.data) {
        return None;
    }
    let (frac, size) = pagenor_render::pdf_crop_box(&a.data, a.page as usize, a.pdf_crop)?;
    a.pdf_box = Some(frac);
    Some(size)
}

pub(crate) fn read_source(p: &Value) -> Result<(Vec<u8>, String, Option<String>)> {
    if let Some(b) = str_param(p, "base64") {
        if b.len() > (MAX_ASSET_FILE_BYTES as usize).div_ceil(3).saturating_mul(4) {
            return Err(bad("place", "encoded source exceeds size limit"));
        }
        let bytes = base64_decode(b);
        if bytes.len() as u64 > MAX_ASSET_FILE_BYTES {
            return Err(bad("place", "source exceeds size limit"));
        }
        return Ok((bytes, str_param(p, "name").unwrap_or("image").to_string(), None));
    }
    let path = str_param(p, "path").ok_or_else(|| bad("place", "missing `path` or `base64`"))?;
    let b = read_asset_file(path, MAX_ASSET_FILE_BYTES)?;
    let name = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.into());
    Ok((b, name, Some(path.to_string())))
}

pub(crate) const MAX_ASSET_FILE_BYTES: u64 = 256 * 1024 * 1024;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_asset_file(path: &str, limit: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
    let len = file.metadata().map_err(|e| EngineError::Other(format!("{path}: {e}")))?.len();
    if len > limit {
        return Err(bad("place", format!("{path}: source exceeds the {limit}-byte size limit")));
    }
    read_asset_bytes(file, limit)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn read_asset_file(_path: &str, _limit: u64) -> Result<Vec<u8>> {
    Err(bad("place", "local files must be supplied by the browser picker"))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_asset_bytes(reader: impl std::io::Read, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    reader.take(limit.saturating_add(1)).read_to_end(&mut bytes).map_err(|e| EngineError::Other(e.to_string()))?;
    if bytes.len() as u64 > limit {
        return Err(bad("place", "source exceeded the actual read size limit"));
    }
    Ok(bytes)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod source_limit_tests {
    #[test]
    fn actual_reads_are_limited_even_when_metadata_cannot_predict_growth() {
        assert_eq!(super::read_asset_bytes(&b"12345678"[..], 8).unwrap(), b"12345678");
        assert!(super::read_asset_bytes(&b"123456789"[..], 8).is_err());
    }
}

fn eps_size(bytes: &[u8], command: &str) -> Result<Option<(f64, f64)>> {
    if !pagenor_images::is_eps(bytes) {
        return Ok(None);
    }
    pagenor_images::eps_size(bytes)
        .map(Some)
        .ok_or_else(|| bad(command, "the EPS has a missing or invalid bounding box (dimensions must be 0.001–20,000 points)"))
}

/// Fitting can overflow even with finite inputs. Never persist NaN/Inf as JSON nulls.
pub(crate) fn validate_placement(size: (f64, f64), frame: Rect, xf: Affine, command: &str) -> Result<()> {
    let valid_rect = |r: Rect| [r.x0, r.y0, r.x1, r.y1, r.width(), r.height()].iter().all(|n| n.is_finite()) && r.width() >= 0.0 && r.height() >= 0.0;
    if !size.0.is_finite()
        || !size.1.is_finite()
        || size.0 <= 0.0
        || size.1 <= 0.0
        || !valid_rect(frame)
        || !xf.as_coeffs().iter().all(|n| n.is_finite())
        || !valid_rect(xf.transform_rect_bbox(Rect::new(0.0, 0.0, size.0, size.1)))
    {
        return Err(bad(command, "image placement exceeds finite geometry limits"));
    }
    Ok(())
}

fn place_load(s: &mut Session, p: &Value) -> Result<Value> {
    let (bytes, name, link) = read_source(p)?;
    let (pw, ph) = pagenor_render::image_size(&bytes).ok_or_else(|| bad("place.load", "unsupported or corrupt image"))?;
    let size = eps_size(&bytes, "place.load")?.or_else(|| pagenor_render::natural_size(&bytes, 0)).unwrap_or((pw as f64, ph as f64));
    let aid = s.edit(|d, _| {
        let fresh = AssetId(d.alloc());
        let mime = pagenor_render::image_mime(&bytes).to_string();
        Ok(d.add_asset(Asset {
            page: 0,
            id: fresh,
            name: name.clone(),
            mime,
            link: link.clone(),
            data: Arc::new(bytes),
            pixels: Some((pw, ph)),
            ..Default::default()
        }))
    })?;
    s.loaded = Some((aid, size));
    s.set_tool("placeGun");
    Ok(json!({"asset": aid.0, "name": name, "width": pw, "height": ph}))
}

fn place_drop(s: &mut Session, p: &Value) -> Result<Value> {
    let (aid, (nw, nh)) = s.loaded.ok_or_else(|| bad("place.drop", "nothing loaded in the place cursor"))?;
    let sr = super::spread_param(p, "spread");
    let lid = s.doc()?.active_layer;
    let frame = super::id_param(p, "frame");
    let rect = super::rect_param(p, "rect");
    let (x, y) = (super::f64_or(p, "x", 0.0), super::f64_or(p, "y", 0.0));
    let d = &s.doc()?.doc;
    let (frame_rect, xf, auto_fit) = if let Some(fid) = frame {
        let r = d.item(fid).ok_or(pagenor_doc::DocError::NoItem(fid))?.inner_bounds();
        let k = (r.width() / nw).max(r.height() / nh);
        let xf = Affine::translate((r.x0 + (r.width() - nw * k) / 2.0, r.y0 + (r.height() - nh * k) / 2.0)) * Affine::scale(k);
        // Fill once; keep Auto-Fit off so resizing the frame crops the placed image.
        (r, xf, Default::default())
    } else {
        if d.spread(sr).is_none() {
            return Err(bad("place.drop", "no such spread"));
        }
        // Drag: fit proportionally; click: actual size at the point.
        let (r, k) = match rect {
            Some(r) if r.width() > 2.0 && r.height() > 2.0 => {
                let k = (r.width() / nw).min(r.height() / nh);
                (Rect::new(r.x0, r.y0, r.x0 + nw * k, r.y0 + nh * k), k)
            }
            _ => (Rect::new(x, y, x + nw, y + nh), 1.0),
        };
        (r, Affine::translate((r.x0, r.y0)) * Affine::scale(k), Default::default())
    };
    validate_placement((nw, nh), frame_rect, xf, "place.drop")?;
    let r = s.edit(|d, sel| {
        let graphic = Content::Graphic(Graphic {
            clip: None,
            asset: aid,
            size: (nw, nh),
            xf,
            auto_fit,
            fit_align: 4,
            crop: [0.0; 4],
            wrap: Default::default(),
        });
        let id = if let Some(fid) = frame {
            d.item_mut(fid).ok_or(pagenor_doc::DocError::NoItem(fid))?.content = graphic;
            fid
        } else {
            let id = ItemId(d.alloc());
            let mut it = Item::new(id, lid, Shape::Rectangle, shapes::rectangle(frame_rect));
            it.object_style = d.styles.default_graphic_frame.clone();
            it.content = graphic;
            d.insert_item(sr, it, None)?;
            id
        };
        *sel = Selection::items(vec![id]);
        Ok(json!({"id": id.0}))
    })?;
    s.loaded = None;
    s.set_tool("selection");
    Ok(r)
}

/// The selection as `.pgnr` snippet bytes.
pub(crate) fn snippet_bytes(s: &mut Session) -> Result<Vec<u8>> {
    to_bytes(&super::edit::clip_doc(s)?)
}

fn snippet_export(s: &mut Session, p: &Value) -> Result<Value> {
    let d = super::edit::clip_doc(s)?;
    let bytes = to_bytes(&d)?;
    match str_param(p, "path") {
        Some(path) => {
            #[cfg(not(target_arch = "wasm32"))]
            crate::write_atomic(std::path::Path::new(path), &bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
            Ok(json!({"path": path, "bytes": bytes.len(), "items": d.spreads.first().map(|s| s.items.len()).unwrap_or(0)}))
        }
        None => Ok(json!({"base64": base64_encode(&bytes), "bytes": bytes.len()})),
    }
}

/// File › Place of an IDML or Pagenor document: the objects of one page (`layoutPage`,
/// 1-based) grouped, top-left at x/y (default: the source position on the first page).
fn place_layout_page(s: &mut Session, p: &Value, name: &str, bytes: &[u8]) -> Result<Value> {
    const ID: &str = "file.place";
    let src = if pagenor_indd::is_indd(bytes) {
        let idml = pagenor_indd::to_idml_named(bytes, name).map_err(|e| bad(ID, e.to_string()))?;
        super::interchange::import(&idml, None)?
    } else if name.to_lowercase().ends_with(".idml") {
        {
            let mut d = pagenor_idml::import_idml(bytes).map_err(|e| bad(ID, e.to_string()))?;
            super::interchange::resolve_pdf_crops(&mut d);
            d
        }
    } else {
        from_bytes(bytes)?
    };
    let n = src.page_count();
    let page = p.get("layoutPage").and_then(Value::as_u64).unwrap_or(1) as usize;
    if page == 0 || page > n {
        return Err(bad(ID, format!("the document has {n} page(s)")));
    }
    let (si, pi) = src.page_loc(page - 1).ok_or_else(|| bad(ID, "no such page"))?;
    let sp = &src.spreads[si];
    let pr = sp.pages[pi].bounds();
    let ids: Vec<ItemId> = sp.items.iter().filter(|it| !it.hidden && sp.page_at_x(it.bounds().center().x) == Some(pi)).map(|it| it.id).collect();
    if ids.is_empty() {
        return Err(bad(ID, format!("page {page} has no objects")));
    }
    let to = super::spread_param(p, "spread");
    let off = match (p.get("x").and_then(Value::as_f64), p.get("y").and_then(Value::as_f64)) {
        (Some(x), Some(y)) => pagenor_geom::Vec2::new(x - pr.x0, y - pr.y0),
        _ => pagenor_geom::Vec2::new(-pr.x0, -pr.y0),
    };
    let placed = s.edit(|d, sel| {
        for sw in &src.swatches {
            if d.swatch(&sw.name).is_none() {
                d.swatches.push(sw.clone());
            }
        }
        let missing_p: Vec<_> = src.styles.paragraph.iter().filter(|ps| d.styles.para(&ps.name).is_none()).cloned().collect();
        let missing_c: Vec<_> = src.styles.character.iter().filter(|cs| d.styles.char_style(&cs.name).is_none()).cloned().collect();
        if !missing_p.is_empty() || !missing_c.is_empty() {
            let st = d.styles_mut();
            st.paragraph.extend(missing_p);
            st.character.extend(missing_c);
        }
        let new = super::object::duplicate_from(d, &src, &ids, to, off)?;
        *sel = Selection::items(new.clone());
        Ok(new)
    })?;
    if placed.len() > 1 {
        let g = s.execute("object.group", &json!({"ids": placed.iter().map(|i| i.0).collect::<Vec<_>>()}))?;
        return Ok(json!({"id": g["id"], "items": placed.len()}));
    }
    Ok(json!({"id": placed[0].0, "items": 1}))
}

pub(crate) fn snippet_place(s: &mut Session, p: &Value) -> Result<Value> {
    let (bytes, _, _) = read_source(p)?;
    let src = from_bytes(&bytes)?;
    let ids: Vec<ItemId> = src.spreads.first().map(|sp| sp.items.iter().map(|i| i.id).collect()).unwrap_or_default();
    if ids.is_empty() {
        return Err(bad("snippet.place", "the snippet is empty"));
    }
    let sr = super::spread_param(p, "spread");
    let bounds = ids.iter().filter_map(|i| src.item(*i).map(|it| it.bounds())).reduce(|a, b| a.union(b)).unwrap_or(Rect::ZERO);
    let off = match (p.get("x").and_then(Value::as_f64), p.get("y").and_then(Value::as_f64)) {
        (Some(x), Some(y)) => pagenor_geom::Vec2::new(x - bounds.x0, y - bounds.y0),
        _ => pagenor_geom::Vec2::ZERO,
    };
    s.edit(|d, sel| {
        // Bring over styles and swatches the snippet uses that this document lacks.
        for sw in &src.swatches {
            if d.swatch(&sw.name).is_none() {
                d.swatches.push(sw.clone());
            }
        }
        let missing_p: Vec<_> = src.styles.paragraph.iter().filter(|ps| d.styles.para(&ps.name).is_none()).cloned().collect();
        let missing_c: Vec<_> = src.styles.character.iter().filter(|cs| d.styles.char_style(&cs.name).is_none()).cloned().collect();
        if !missing_p.is_empty() || !missing_c.is_empty() {
            let st = d.styles_mut();
            st.paragraph.extend(missing_p);
            st.character.extend(missing_c);
        }
        let new = super::object::duplicate_from(d, &src, &ids, sr, off)?;
        *sel = Selection::items(new.clone());
        Ok(json!({"ids": new.iter().map(|i| i.0).collect::<Vec<_>>()}))
    })
}

fn recovery_dir(s: &Session, p: &Value) -> Result<std::path::PathBuf> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, p);
        Err(EngineError::Other("no recovery folder on the web".into()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    str_param(p, "dir")
        .map(std::path::PathBuf::from)
        .or_else(|| s.recovery_dir.clone())
        .ok_or_else(|| bad("file.recovery", "no recovery folder (give `dir`)"))
}

fn can_revert(s: &Session) -> std::result::Result<(), String> {
    match s.active() {
        Some(d) if d.path.is_some() && d.is_dirty() => Ok(()),
        Some(d) if d.path.is_none() => Err("the document has never been saved".into()),
        Some(_) => Err("no changes since the last save".into()),
        None => Err("no document open".into()),
    }
}

fn file_revert(s: &mut Session, _: &Value) -> Result<Value> {
    let path = s.doc()?.path.clone().ok_or_else(|| bad("file.revert", "the document has never been saved"))?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut d = if path.to_ascii_lowercase().ends_with(".idml") {
            let bytes = std::fs::read(&path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
            let mut d = pagenor_idml::import_idml(&bytes).map_err(|e| EngineError::Other(e.to_string()))?;
            super::interchange::resolve_pdf_crops(&mut d);
            d
        } else {
            from_bytes(&std::fs::read(&path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?)?
        };
        // As opened again: the fonts beside it now (unchanged files are shared, not read again).
        let (fonts, _, _) = load_document_fonts(&mut d, &path);
        let uid = s.doc()?.uid;
        let st = s.doc_mut()?;
        let mut fresh = DocState::new(d, Some(path.clone()));
        fresh.fonts = fonts;
        fresh.uid = uid;
        fresh.revision = st.revision + 1;
        crate::recovery::discard_document(st);
        *st = fresh;
        Ok(json!({"path": path}))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, path);
        Err(EngineError::Other("revert isn't available on the web".into()))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod image_place_tests {
    use pagenor_doc::{Content, ItemId};
    use pagenor_geom::Rect;
    use serde_json::{Value, json};

    use crate::Session;

    fn image_file() -> std::io::Result<tempfile::NamedTempFile> {
        let file = tempfile::Builder::new().suffix(".png").tempfile()?;
        let png = pagenor_render::Rendered { width: 40, height: 20, pixels: [30, 90, 180, 255].repeat(800) }.to_png();
        std::fs::write(file.path(), png)?;
        Ok(file)
    }

    #[test]
    fn consecutive_path_placements_create_distinct_graphics() {
        let image = image_file().unwrap();
        let mut session = Session::new();
        session.execute("file.new", &json!({"pages": 3, "facingPages": false})).unwrap();
        let mut placements = Vec::new();
        let mut originals = Vec::new();

        for (spread, x, y) in [(0, 0.0, 0.0), (1, 70.0, 90.0), (2, 150.0, 210.0)] {
            let placed = session.execute("file.place", &json!({"path": image.path(), "spread": spread, "x": x, "y": y})).unwrap();
            let id = ItemId(placed["id"].as_u64().unwrap());
            assert!(
                placements.iter().all(|previous: &Value| previous["id"] != placed["id"]),
                "each explicit placement needs a distinct frame: {placed}"
            );
            let document = &session.doc().unwrap().doc;
            let item = document.item(id).unwrap();
            assert_eq!(item.bounds(), Rect::new(x, y, x + 40.0, y + 20.0));
            assert!(document.spreads[spread].items.iter().any(|item| item.id == id));
            originals.push(serde_json::to_value(item).unwrap());
            placements.push(placed);
        }

        let document = &session.doc().unwrap().doc;
        assert_eq!(document.spreads.iter().flat_map(|spread| &spread.items).filter(|item| matches!(item.content, Content::Graphic(_))).count(), 3);
        assert_eq!(document.assets.len(), 1, "identical linked placements share their asset");
        for (placed, original) in placements.iter().zip(originals) {
            assert_eq!(serde_json::to_value(document.item(ItemId(placed["id"].as_u64().unwrap())).unwrap()).unwrap(), original);
        }
        let links = session.execute("links.list", &json!({})).unwrap();
        assert_eq!(links.as_array().unwrap().len(), 1);
        for placed in placements {
            let row = links.as_array().unwrap().iter().find(|row| row["asset"] == placed["asset"]).unwrap();
            assert_eq!(row["path"], json!(image.path()));
            let uses = row["uses"].as_array().unwrap();
            assert_eq!(uses.len(), 3, "all frames using the shared asset are listed");
            assert!(uses.iter().any(|usage| usage["id"] == placed["id"]));
        }
    }

    #[test]
    fn individual_placement_parameters_bypass_selected_frames() {
        let image = image_file().unwrap();
        for graphic in [false, true] {
            for (key, value, expected) in [
                ("x", 0, Rect::new(0.0, 36.0, 40.0, 56.0)),
                ("y", 0, Rect::new(36.0, 0.0, 76.0, 20.0)),
                ("spread", 0, Rect::new(36.0, 36.0, 76.0, 56.0)),
                ("width", 0, Rect::new(36.0, 36.0, 36.0, 36.0)),
                ("width", 80, Rect::new(36.0, 36.0, 116.0, 76.0)),
            ] {
                let mut session = Session::new();
                session.execute("file.new", &json!({})).unwrap();
                let selected = session.execute("frame.create", &json!({"rect": [100, 120, 200, 220]})).unwrap();
                if graphic {
                    session.execute("file.place", &json!({"path": image.path()})).unwrap();
                }
                let id = ItemId(selected["id"].as_u64().unwrap());
                let original = serde_json::to_value(session.doc().unwrap().doc.item(id).unwrap()).unwrap();
                let mut params = json!({"path": image.path()});
                params[key] = json!(value);

                let placed = session.execute("file.place", &params).unwrap();

                assert_ne!(placed["id"], selected["id"], "{key}={value}, graphic={graphic}");
                let document = &session.doc().unwrap().doc;
                assert_eq!(serde_json::to_value(document.item(id).unwrap()).unwrap(), original);
                assert_eq!(document.item(ItemId(placed["id"].as_u64().unwrap())).unwrap().bounds(), expected);
            }
        }
    }

    #[test]
    fn explicit_frame_wins_over_placement_and_selection() {
        let image = image_file().unwrap();
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let target = session.execute("frame.create", &json!({"rect": [100, 120, 200, 220]})).unwrap();
        let selected = session.execute("frame.create", &json!({"rect": [300, 320, 400, 420]})).unwrap();

        let placed =
            session.execute("file.place", &json!({"path": image.path(), "frame": target["id"], "x": 0, "y": 0, "spread": 99, "width": 0})).unwrap();

        assert_eq!(placed["id"], target["id"]);
        let document = &session.doc().unwrap().doc;
        let item = document.item(ItemId(target["id"].as_u64().unwrap())).unwrap();
        assert_eq!(item.bounds(), Rect::new(100.0, 120.0, 200.0, 220.0));
        assert!(matches!(item.content, Content::Graphic(_)));
        assert!(matches!(document.item(ItemId(selected["id"].as_u64().unwrap())).unwrap().content, Content::Unassigned));
        assert_eq!(document.spreads[0].items.len(), 2);
    }

    #[test]
    fn implicit_placement_fills_empty_and_replaces_graphic_selection() {
        let image = image_file().unwrap();
        for graphic in [false, true] {
            let mut session = Session::new();
            session.execute("file.new", &json!({})).unwrap();
            let selected = session.execute("frame.create", &json!({"rect": [100, 120, 200, 220]})).unwrap();
            let previous = graphic.then(|| session.execute("file.place", &json!({"path": image.path()})).unwrap());

            let placed = session.execute("file.place", &json!({"path": image.path()})).unwrap();

            assert_eq!(placed["id"], selected["id"]);
            if let Some(previous) = previous {
                assert_eq!(placed["asset"], previous["asset"], "placing the same source reuses its asset");
            }
            let document = &session.doc().unwrap().doc;
            let item = document.item(ItemId(placed["id"].as_u64().unwrap())).unwrap();
            assert_eq!(item.bounds(), Rect::new(100.0, 120.0, 200.0, 220.0));
            let Content::Graphic(content) = &item.content else { panic!("placed frame must be graphic") };
            assert_eq!(json!(content.asset.0), placed["asset"]);
            assert_eq!(document.spreads[0].items.len(), 1);
        }
    }
}

#[cfg(test)]
pub(crate) mod eps_place_tests {
    use super::*;

    pub(crate) fn tiff_eps() -> Vec<u8> {
        let mut tiff = b"II".to_vec();
        tiff.extend(42u16.to_le_bytes());
        tiff.extend(8u32.to_le_bytes());
        tiff.extend(10u16.to_le_bytes());
        for (tag, ty, count, value) in [
            (256u16, 4u16, 1u32, 2u32),
            (257, 4, 1, 2),
            (258, 3, 3, 134),
            (259, 3, 1, 1),
            (262, 3, 1, 2),
            (273, 4, 1, 140),
            (277, 3, 1, 3),
            (278, 4, 1, 2),
            (279, 4, 1, 12),
            (284, 3, 1, 1),
        ] {
            tiff.extend(tag.to_le_bytes());
            tiff.extend(ty.to_le_bytes());
            tiff.extend(count.to_le_bytes());
            tiff.extend(value.to_le_bytes());
        }
        tiff.extend(0u32.to_le_bytes());
        for _ in 0..3 {
            tiff.extend(8u16.to_le_bytes());
        }
        tiff.extend([255, 0, 0].repeat(4));
        let ps = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%EOF\n";
        let mut eps = vec![0xC5, 0xD0, 0xD3, 0xC6];
        for n in [30usize, ps.len(), 0, 0, 30 + ps.len(), tiff.len()] {
            eps.extend((n as u32).to_le_bytes());
        }
        eps.extend([255, 255]);
        eps.extend(ps);
        eps.extend(tiff);
        eps
    }

    #[test]
    fn eps_degenerate_bounds_are_rejected_without_edits() {
        for bounds in ["0 0 1e-320 1e-320", "0 0 1e308 1e308", "0 0 inf 10"] {
            let source = format!("%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: {bounds}\n%%EOF\n");
            for command in ["file.place", "place.load"] {
                let mut s = Session::new();
                s.execute("file.new", &json!({})).unwrap();
                let before = s.doc().unwrap().doc.clone();
                let revision = s.doc().unwrap().revision;
                assert!(
                    s.execute(command, &json!({"base64": base64_encode(source.as_bytes()), "name": "bad.eps", "width": 100})).is_err(),
                    "{command}: {bounds}"
                );
                assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
                assert_eq!(s.doc().unwrap().revision, revision);
                assert!(s.loaded.is_none());
                pagenor_format::load(&pagenor_format::save(&s.doc().unwrap().doc).unwrap()).unwrap();
            }
        }
    }

    #[test]
    fn eps_fitting_overflow_is_rejected_before_document_edits() {
        let source = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 0.01 0.01\n%%EOF\n";
        let params = json!({"base64": base64_encode(source), "name": "small.eps"});
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let before = s.doc().unwrap().doc.clone();
        assert!(s.execute("file.place", &json!({"base64": base64_encode(source), "name": "small.eps", "width": 1e308})).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        s.execute("place.load", &params).unwrap();
        let before = s.doc().unwrap().doc.clone();
        assert!(s.execute("place.drop", &json!({"rect": [0, 0, 1e308, 1e308]})).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        s.execute("place.drop", &json!({"rect": [72, 72, 172, 122]})).unwrap();
        pagenor_format::load(&pagenor_format::save(&s.doc().unwrap().doc).unwrap()).unwrap();
    }

    #[test]
    fn eps_table_and_data_merge_reject_fitting_overflow() {
        let source = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 0.01 0.01\n%%EOF\n";
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let r = s.execute("frame.create", &json!({"rect":[72,72,372,272],"content":"text"})).unwrap();
        s.execute("text.select", &json!({"story":r["story"],"anchor":0,"focus":0})).unwrap();
        s.execute("table.insert", &json!({"rows":1,"cols":1,"width":1e308})).unwrap();
        let before = s.doc().unwrap().doc.clone();
        assert!(s.execute("table.placeGraphic", &json!({"base64":base64_encode(source),"name":"small.eps","fit":"fill"})).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        pagenor_format::load(&pagenor_format::save(&before).unwrap()).unwrap();

        let path = std::env::temp_dir().join(format!("dc-eps-fit-{}.eps", std::process::id()));
        std::fs::write(&path, source).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("data.source.select", &json!({"bytes":base64_encode(format!("@Photo\n{}\n",path.display()).as_bytes()),"name":"source.csv"}))
            .unwrap();
        let r = s.execute("frame.create", &json!({"rect":[0,0,1e308,100],"content":"graphic"})).unwrap();
        s.execute("data.placeholder.add", &json!({"field":"Photo","item":r["id"]})).unwrap();
        let result = s.execute("data.merge", &json!({"fitting":"fillProportionally"})).unwrap();
        assert_eq!(result["missingImages"].as_array().unwrap().len(), 1);
        assert!(s.doc().unwrap().doc.assets.is_empty());
        pagenor_format::load(&pagenor_format::save(&s.doc().unwrap().doc).unwrap()).unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn eps_table_and_data_merge_use_bounding_box_points() {
        let dir = std::env::temp_dir().join(format!("dc-eps-routes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (index, bytes) in [b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%EOF\n".to_vec(), tiff_eps()].into_iter().enumerate() {
            let mut s = Session::new();
            s.execute("file.new", &json!({})).unwrap();
            let r = s.execute("frame.create", &json!({"rect": [72,72,372,272], "content":"text"})).unwrap();
            s.execute("text.select", &json!({"story": r["story"], "anchor":0, "focus":0})).unwrap();
            s.execute("table.insert", &json!({"rows":1,"cols":1})).unwrap();
            s.execute("table.placeGraphic", &json!({"base64": base64_encode(&bytes), "name":"logo.eps"})).unwrap();
            let table = s.doc().unwrap().doc.stories.values().flat_map(|st| st.tables.values()).next().unwrap();
            assert_eq!(table.cells[0].graphic.as_ref().unwrap().size, (100.0, 50.0), "table {index}");
            let path = dir.join(format!("logo-{index}.eps"));
            std::fs::write(&path, &bytes).unwrap();
            let mut s = Session::new();
            s.execute("file.new", &json!({})).unwrap();
            s.execute(
                "data.source.select",
                &json!({"bytes": base64_encode(format!("@Photo\n{}\n", path.display()).as_bytes()), "name":"source.csv"}),
            )
            .unwrap();
            let r = s.execute("frame.create", &json!({"rect":[72,72,372,272],"content":"graphic"})).unwrap();
            s.execute("data.placeholder.add", &json!({"field":"Photo","item":r["id"]})).unwrap();
            let result = s.execute("data.merge", &json!({"fitting":"none"})).unwrap();
            assert!(result["missingImages"].as_array().unwrap().is_empty());
            let g = s
                .doc()
                .unwrap()
                .doc
                .spreads
                .iter()
                .flat_map(|sp| &sp.items)
                .find_map(|it| if let Content::Graphic(g) = &it.content { Some(g) } else { None })
                .unwrap();
            assert_eq!(g.size, (100.0, 50.0), "merge {index}");
            pagenor_format::load(&pagenor_format::save(&s.doc().unwrap().doc).unwrap()).unwrap();
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn eps_idml_roundtrip_preserves_source_and_resource_type() {
        use std::io::Read;
        for bytes in [b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%EOF\n".to_vec(), tiff_eps()] {
            let mut s = Session::new();
            s.execute("file.new", &json!({})).unwrap();
            s.execute("file.place", &json!({"base64":base64_encode(&bytes),"name":"logo.eps"})).unwrap();
            let source = s.doc().unwrap().doc.clone();
            let idml = pagenor_idml::export_idml(&source);
            let mut z = zip::ZipArchive::new(std::io::Cursor::new(&idml)).unwrap();
            let names: Vec<_> = z.file_names().filter(|n| n.starts_with("Spreads/")).map(str::to_string).collect();
            let mut xml = String::new();
            for name in names {
                z.by_name(&name).unwrap().read_to_string(&mut xml).unwrap();
            }
            assert!(xml.contains("<EPS "), "EPS resource element missing");
            assert!(xml.contains("Encapsulated PostScript (EPS)"));
            assert!(!xml.contains("Portable Network Graphics (PNG)"));
            let restored = pagenor_idml::import_idml(&idml).unwrap();
            let a = restored.assets.values().next().unwrap();
            assert_eq!(a.mime, "application/postscript");
            assert_eq!(a.data.as_slice(), bytes);
            assert!(pagenor_render::decode_pixmap(&a.data).is_some());
            // A linked-only EPS keeps its type even when the external file is unavailable;
            // when the resolver supplies it, sniff the raw source rather than PNG metadata.
            let mut linked = (*source).clone();
            let a = Arc::make_mut(linked.assets.values_mut().next().unwrap());
            a.link = Some("/synthetic/logo.eps".into());
            let idml = pagenor_idml::export_idml_with(&linked, &pagenor_idml::ExportOptions { embed_images: false });
            let missing = pagenor_idml::import_idml_with(&idml, &|_| None).unwrap();
            let a = missing.assets.values().next().unwrap();
            assert_eq!(a.mime, "application/postscript");
            assert!(a.data.is_empty());
            let resolved = pagenor_idml::import_idml_with(&idml, &|path| {
                assert_eq!(path, "/synthetic/logo.eps");
                Some(bytes.clone())
            })
            .unwrap();
            let a = resolved.assets.values().next().unwrap();
            assert_eq!(a.mime, "application/postscript");
            assert_eq!(a.data.as_slice(), bytes);
        }
    }

    #[test]
    fn eps_place_cursor_retains_source_and_uses_bounding_box_size() {
        let source = b"%!PS-Adobe-3.0 EPSF-3.0\n%%HiResBoundingBox: 0 0 100.5 50.25\n%%EOF\n";
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let result = s.execute("place.load", &json!({"base64": base64_encode(source), "name": "logo.eps"})).unwrap();
        let aid = AssetId(result["asset"].as_u64().unwrap());
        assert_eq!(s.doc().unwrap().doc.assets[&aid].data.as_slice(), source);
        assert_eq!(s.doc().unwrap().doc.assets[&aid].mime, "application/postscript");
        assert_eq!(s.doc().unwrap().doc.assets[&aid].pixels, Some((201, 101)));
        let result = s.execute("place.drop", &json!({"x": 72, "y": 72})).unwrap();
        let it = s.doc().unwrap().doc.item(ItemId(result["id"].as_u64().unwrap())).unwrap();
        let Content::Graphic(g) = &it.content else { panic!("expected a graphic") };
        assert_eq!(g.size, (100.5, 50.25));
        assert_eq!((it.bounds().width(), it.bounds().height()), g.size);
        let before = s.doc().unwrap().doc.clone();
        for invalid in ["%!PS-Adobe-3.0 EPSF-3.0\nno bounding box\n", "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 inf 10\n"] {
            let corrupt = json!({"base64": base64_encode(invalid.as_bytes()), "name": "bad.eps"});
            assert!(s.execute("place.load", &corrupt).is_err());
            assert!(s.execute("file.place", &corrupt).is_err());
        }
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
    }
}

#[cfg(test)]
mod new_document_tests {
    use serde_json::json;

    use crate::Session;

    #[test]
    fn saved_presets_make_documents_by_name() {
        let mut s = Session::new();
        let newsletter = json!({"name": "Newsletter", "units": "millimeters", "width": 595.2755905511812, "height": 841.8897637795277,
            "pages": 8, "facingPages": false, "columns": 3, "margins": {"top": 20, "bottom": 30, "inside": 40, "outside": 50},
            "bleed": [9, 9, 0, 9], "primaryTextFrame": true});
        assert_eq!(s.execute("file.savePreset", &newsletter).unwrap()["name"], "Newsletter");
        let listed = s.execute("file.presets", &json!({})).unwrap();
        let saved: Vec<_> = listed.as_array().unwrap().iter().filter(|p| p["saved"] == true).collect();
        assert_eq!(saved.len(), 1);
        assert_eq!((&saved[0]["name"], &saved[0]["units"]), (&json!("Newsletter"), &json!("millimeters")));
        // By name, case ignored; parameters still override the preset.
        s.execute("file.new", &json!({"preset": "newsletter", "pages": 4})).unwrap();
        let d = &s.doc().unwrap().doc;
        assert_eq!(d.page_count(), 4);
        assert!(!d.settings.facing_pages && d.settings.primary_text_frame);
        assert_eq!(d.settings.horizontal_units, pagenor_geom::Unit::Millimeters);
        assert_eq!(d.settings.bleed, [9.0, 9.0, 0.0, 9.0]);
        assert_eq!(d.page(0).unwrap().margins.outside, 50.0);
        assert!(d.title.starts_with("Untitled-"), "the preset's name isn't the document's");
        // The same name replaces it.
        s.execute("file.savePreset", &json!({"name": "NEWSLETTER", "pages": 2})).unwrap();
        assert_eq!(s.prefs.document_presets.len(), 1);
        s.execute("file.new", &json!({"preset": "Newsletter"})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page_count(), 2);
        // Built-in names, empty and overlong names and impossible sizes are refused.
        for bad in [json!({"name": "a4"}), json!({"name": "  "}), json!({}), json!({"name": "x".repeat(101)}), json!({"name": "Huge", "width": 1e9})]
        {
            assert!(s.execute("file.savePreset", &bad).is_err(), "{bad}");
        }
        s.execute("file.deletePreset", &json!({"name": "newsletter"})).unwrap();
        assert!(s.execute("file.deletePreset", &json!({"name": "newsletter"})).is_err());
        assert!(s.execute("file.new", &json!({"preset": "Newsletter"})).is_err());
    }

    #[test]
    fn persisted_settings_are_bounded_and_presets_survive_restart() {
        let mut nd =
            pagenor_doc::build::NewDocument { pages: usize::MAX, columns: u32::MAX, start_page: u32::MAX, bleed: [1e30; 4], ..Default::default() };
        super::apply_settings(&mut nd, &json!({}));
        assert_eq!((nd.pages, nd.columns, nd.start_page), (9999, 216, 99_999));
        assert_eq!(nd.bleed, [1296.0; 4]);
        super::apply_settings(&mut nd, &json!({"pages": 4_294_967_296_u64}));
        assert_eq!(nd.pages, 9999, "clamp before casting on 32-bit hosts");
        nd.gutter = f64::INFINITY;
        assert!(super::check_size("file.new", &nd).is_err());

        let mut s = Session::new();
        s.execute("file.savePreset", &json!({"name": "Telugu booklet", "pages": 4, "columns": 2})).unwrap();
        let bytes = serde_json::to_vec(&s.prefs).unwrap();
        let mut restarted = Session::new();
        restarted.prefs = serde_json::from_slice(&bytes).unwrap();
        restarted.execute("file.new", &json!({"preset": "Telugu booklet"})).unwrap();
        assert_eq!(restarted.doc().unwrap().doc.page_count(), 4);
        assert_eq!(restarted.doc().unwrap().doc.page(0).unwrap().columns.count, 2);
    }

    #[test]
    fn preset_limit_failure_is_atomic_and_untitled_counter_does_not_overflow() {
        let mut s = Session::new();
        s.prefs.document_presets = (0..501).map(|i| pagenor_doc::build::NewDocument { title: format!("Preset {i}"), ..Default::default() }).collect();
        let before = s.prefs.document_presets.clone();
        assert!(s.execute("file.savePreset", &json!({"name": "Preset 0", "pages": 2})).is_err());
        assert_eq!(s.prefs.document_presets, before, "an over-limit file is unchanged on failure");
        s.prefs.document_presets.truncate(500);
        s.execute("file.savePreset", &json!({"name": "Preset 0", "pages": 2})).unwrap();
        assert_eq!(s.prefs.document_presets.len(), 500);
        assert!(s.execute("file.savePreset", &json!({"name": "Extra"})).is_err());
        s.untitled = u32::MAX;
        s.execute("file.new", &json!({})).unwrap();
        assert_eq!(s.untitled, u32::MAX);
    }

    #[test]
    fn new_document_takes_intent_units_start_page_bleed_and_slug() {
        let mut s = Session::new();
        let p = json!({"preset": "A4", "intent": "web", "units": "pixels", "startPage": 4, "bleed": [9, 0, 3, 6], "slug": 18});
        s.execute("file.new", &p).unwrap();
        let d = &s.doc().unwrap().doc;
        assert_eq!(d.settings.intent, pagenor_doc::Intent::Web);
        assert_eq!(d.settings.horizontal_units, pagenor_geom::Unit::Pixels);
        assert_eq!(d.settings.bleed, [9.0, 0.0, 3.0, 6.0]);
        assert_eq!(d.settings.slug, [18.0; 4]);
        assert_eq!(d.page_name(0), "4");
        // Malformed edges are ignored, hostile ones bounded.
        s.execute("file.new", &json!({"bleed": [1, 2], "slug": [-5, 1e12, 0, 0]})).unwrap();
        let d = &s.doc().unwrap().doc;
        assert_eq!(d.settings.bleed, [0.0; 4]);
        assert_eq!(d.settings.slug, [0.0, 1296.0, 0.0, 0.0]);
    }
}

#[cfg(test)]
mod pdf_crop_tests {
    #[test]
    fn placed_pdf_keeps_its_exact_page_size() {
        // A 100.4 x 50.4 pt page placed 100.4 pt wide is 50.4 pt tall, not stretched to whole points.
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 100.4, "height": 50.4, "pages": 1})).unwrap();
        let b64 = s.execute("file.exportPdf", &json!({})).unwrap()["base64"].as_str().unwrap().to_string();
        let mut t = Session::new();
        t.execute("file.new", &json!({})).unwrap();
        let r = t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "width": 100.4, "x": 0, "y": 0})).unwrap();
        let id = pagenor_doc::ItemId(r["id"].as_u64().unwrap());
        let d = &t.doc().unwrap().doc;
        let it = d.item(id).unwrap();
        let b = it.bounds();
        assert!((b.width() - 100.4).abs() < 1e-3 && (b.height() - 50.4).abs() < 1e-3, "{b:?}");
        let pagenor_doc::Content::Graphic(g) = &it.content else { panic!("not a graphic") };
        assert!((g.size.0 - 100.4).abs() < 1e-3 && (g.size.1 - 50.4).abs() < 1e-3, "{:?}", g.size);
    }
    use serde_json::json;

    use crate::Session;

    #[test]
    fn place_pdf_cropped_to_its_trim_box() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("layout.documentSetup", &json!({"bleed": 9})).unwrap();
        let b64 = s.execute("file.exportPdf", &json!({"bleed": true})).unwrap()["base64"].as_str().unwrap().to_string();
        let mut t = Session::new();
        t.execute("file.new", &json!({})).unwrap();
        let size = |t: &mut Session, crop: &str| {
            t.execute("edit.deselectAll", &json!({})).unwrap();
            let r = t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "pdfCrop": crop, "width": 630, "x": 0, "y": 0})).unwrap();
            let id = pagenor_doc::ItemId(r["id"].as_u64().unwrap());
            t.doc().unwrap().doc.item(id).unwrap().bounds()
        };
        let full = size(&mut t, "crop");
        let trim = size(&mut t, "trim");
        assert!((full.width() - 630.0).abs() < 1e-6, "{full:?}");
        assert!((trim.width() - 612.0).abs() < 1e-3 && (trim.height() - 792.0).abs() < 1e-3, "{trim:?}");
        assert!(t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "pdfCrop": "nope"})).is_err());
    }

    /// Placing with `trim` stores the choice; the graphic spans the trim box and draws only it,
    /// even in a frame larger than the graphic.
    #[test]
    fn a_pdf_placed_with_its_trim_box_stores_and_draws_that_box() {
        // Letter with a 9 pt bleed: black in the bleed's top-left corner and a black square inside.
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("layout.documentSetup", &json!({"bleed": 9})).unwrap();
        for rect in [[-9, -9, 0, 0], [10, 10, 30, 30]] {
            let id = s.execute("frame.create", &json!({"rect": rect})).unwrap()["id"].clone();
            s.execute("object.fill", &json!({"ids": [id], "swatch": "[Black]"})).unwrap();
        }
        let b64 = s.execute("file.exportPdf", &json!({"bleed": true})).unwrap()["base64"].as_str().unwrap().to_string();

        let mut t = Session::new();
        t.execute("file.new", &json!({})).unwrap();
        // `width` is the whole page's (630 pt with its bleed): half size.
        let r = t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "pdfCrop": "trim", "width": 315, "x": 100, "y": 100})).unwrap();
        let id = pagenor_doc::ItemId(r["id"].as_u64().unwrap());
        let d = t.doc().unwrap().doc.clone();
        let it = d.item(id).unwrap();
        let g = it.graphic().unwrap().clone();
        let a = &d.assets[&g.asset];
        assert_eq!(a.pdf_crop, pagenor_doc::PdfCrop::Trim);
        let b = a.shown_box().unwrap();
        let want = [9.0 / 630.0, 9.0 / 810.0, 621.0 / 630.0, 801.0 / 810.0];
        assert!(b.iter().zip(want).all(|(x, y)| (x - y).abs() < 1e-4), "{b:?}");
        assert!((g.size.0 - 612.0).abs() < 1e-3 && (g.size.1 - 792.0).abs() < 1e-3, "{:?}", g.size);
        let shown = (it.xf * g.xf).transform_rect_bbox(pagenor_geom::Rect::new(0.0, 0.0, g.size.0, g.size.1));
        let fb = it.bounds();
        for (x, y) in [(shown.x0, 100.0), (shown.y0, 100.0), (shown.x1, 406.0), (shown.y1, 496.0), (fb.x1, 406.0), (fb.y1, 496.0)] {
            assert!((x - y).abs() < 1e-3, "{shown:?} {fb:?}");
        }

        // A frame larger than the graphic still shows only the trim box.
        t.edit(|d, _| {
            d.item_mut(id).unwrap().path = pagenor_geom::shapes::rectangle(pagenor_geom::Rect::new(90.0, 90.0, 416.0, 506.0));
            Ok(())
        })
        .unwrap();
        let d = t.doc().unwrap().doc.clone();
        let mut rr = pagenor_render::Renderer::new();
        rr.threads = 0;
        let img = rr.render_page(&d, &t.cache, 0, 1.0, false, &Default::default()).unwrap();
        let dark = |x: u32, y: u32| img.pixel(x, y)[0] < 60;
        // The page sits at 95.5: its bleed corner (95.5–100) is cut off, the square (105–115) shows
        // where it is on the page.
        assert!(!dark(97, 97), "the bleed is outside the trim box");
        assert!(!dark(103, 103), "the page is offset by its bleed");
        assert!(dark(110, 110), "the square inside the trim box");
    }
}

#[cfg(test)]
mod layout_place_tests {
    use serde_json::json;

    use crate::Session;

    #[test]
    fn native_pgnr_and_legacy_layouts_open_place_and_keep_existing_save_paths() {
        let dir = std::env::temp_dir().join(format!("pagenor-native-extensions-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut source = Session::new();
        source.execute("file.new", &json!({})).unwrap();
        source.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Layout தமிழ்"})).unwrap();
        for extension in ["pgnr", "PGNR", "pagenor"] {
            let path = dir.join(format!("Layout.{extension}"));
            source.execute("file.saveAs", &json!({"path": path})).unwrap();
            let mut reopened = Session::new();
            reopened.execute("file.open", &json!({"path": path})).unwrap();
            assert_eq!(reopened.doc().unwrap().doc.stories.values().next().unwrap().text, "Layout தமிழ்");
            reopened.execute("file.save", &json!({})).unwrap();
            assert_eq!(reopened.doc().unwrap().path.as_deref(), path.to_str());
            let mut placed = Session::new();
            placed.execute("file.new", &json!({})).unwrap();
            placed.execute("file.place", &json!({"path": path, "x": 72, "y": 72})).unwrap();
            assert!(placed.doc().unwrap().doc.stories.values().any(|s| s.text == "Layout தமிழ்"));
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn place_a_page_of_an_idml_file_as_a_group() {
        let mut a = Session::new();
        a.execute("file.new", &json!({"pages": 2})).unwrap();
        a.execute("frame.create", &json!({"rect": [100, 100, 200, 150], "content": "text", "text": "From page one", "caret": false})).unwrap();
        a.execute("frame.create", &json!({"rect": [300, 400, 350, 450]})).unwrap();
        let idml = pagenor_idml::export_idml(&a.doc().unwrap().doc);
        let b64 = super::base64_encode(&idml);
        let mut b = Session::new();
        b.execute("file.new", &json!({})).unwrap();
        let r = b.execute("file.place", &json!({"base64": b64, "name": "src.idml", "layoutPage": 1, "x": 0, "y": 0})).unwrap();
        assert_eq!(r["items"], 2);
        let d = &b.doc().unwrap().doc;
        let g = d.item(pagenor_doc::ItemId(r["id"].as_u64().unwrap())).unwrap();
        assert_eq!(g.children().len(), 2, "one group");
        let bb = g.bounds();
        assert!((bb.x0 - 100.0).abs() < 1e-6 && (bb.y0 - 100.0).abs() < 1e-6, "page top-left at 0,0: {bb:?}");
        assert!(d.stories.values().any(|st| st.text == "From page one"), "stories come along");
        assert!(b.execute("file.place", &json!({"base64": b64, "name": "src.idml", "layoutPage": 2})).is_err(), "page 2 is empty");
    }
}

#[cfg(test)]
mod document_fonts_tests {
    use std::path::{Path, PathBuf};

    use pagenor_fonts::testing::font_with;
    use pagenor_fonts::{DOCUMENT_FONTS_FOLDER, FaceRef, FontDb, FontSource};
    use serde_json::{Value, json};

    use crate::Session;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dc-open-docfonts-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(DOCUMENT_FONTS_FOLDER)).unwrap();
        dir
    }

    /// A new document in `s` with a text frame holding `text` set in `family`.
    fn new_document(s: &mut Session, family: &str, text: &str) {
        s.execute("file.new", &json!({})).unwrap();
        let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": text})).unwrap();
        s.execute("text.select", &json!({"story": r["story"], "anchor": 0, "focus": text.len()})).unwrap();
        s.execute("type.char", &json!({"fontFamily": family, "fontStyle": "Regular"})).unwrap();
    }

    /// A document whose text ("Hello") is set in `family`, written to `path` (`.idml`: as IDML).
    fn write_document(path: &Path, family: &str) {
        write_document_with(path, family, "Hello");
    }

    fn write_document_with(path: &Path, family: &str, text: &str) {
        let mut s = Session::new();
        new_document(&mut s, family, text);
        if path.extension().is_some_and(|e| e == "idml") {
            std::fs::write(path, pagenor_idml::export_idml(&s.doc().unwrap().doc)).unwrap();
        } else {
            s.execute("file.save", &json!({"path": path.to_string_lossy()})).unwrap();
        }
    }

    fn font_entry(s: &mut Session, family: &str) -> Value {
        let l = s.execute("font.list", &json!({})).unwrap();
        l.as_array().unwrap().iter().find(|f| f["family"] == family).cloned().unwrap_or_else(|| panic!("{family} in {l}"))
    }

    /// Each character of the active document's text with the face it is drawn in.
    fn glyph_faces(s: &Session) -> Vec<(char, FaceRef)> {
        let st = s.doc().unwrap();
        let mut out = Vec::new();
        for (sid, story) in &st.doc.stories {
            let cs = s.cache.get(&st.doc, *sid, None);
            for g in cs.frames.iter().flat_map(|ft| &ft.lines).flat_map(|l| &l.glyphs).filter(|g| g.visible && g.len > 0) {
                out.push((story.text[g.byte..].chars().next().unwrap(), g.face));
            }
        }
        out
    }

    fn open(s: &mut Session, path: &Path) -> Value {
        s.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap()
    }

    #[test]
    fn opening_a_document_loads_its_document_fonts() {
        const FAMILY: &str = "DocFont Open Test";
        let dir = temp_dir("pagenor");
        let fonts = dir.join(DOCUMENT_FONTS_FOLDER);
        std::fs::write(fonts.join("open.ttf"), font_with(FAMILY, &['H', 'e', 'l', 'o']).unwrap()).unwrap();
        std::fs::write(fonts.join("broken.otf"), b"not a font").unwrap();
        let path = dir.join("Brochure.pgnr");
        write_document(&path, FAMILY);
        assert!(!FontDb::global().has_family(FAMILY));

        let mut s = Session::new();
        let r = s.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
        assert_eq!(r["documentFonts"], 1, "{r}");
        assert!(r["warnings"].to_string().contains("broken.otf"), "a damaged font is reported, not fatal: {r}");
        let f = font_entry(&mut s, FAMILY);
        assert_eq!((f["missing"].clone(), f["source"].clone()), (json!(false), json!("document")), "{f}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn opening_an_idml_file_loads_its_document_fonts() {
        const FAMILY: &str = "DocFont IDML Test";
        let dir = temp_dir("idml");
        std::fs::write(dir.join(DOCUMENT_FONTS_FOLDER).join("idml.ttf"), font_with(FAMILY, &['H']).unwrap()).unwrap();
        let path = dir.join("Brochure.idml");
        write_document(&path, FAMILY);

        let mut s = Session::new();
        let r = s.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
        assert_eq!(r["documentFonts"], 1, "{r}");
        assert_eq!(font_entry(&mut s, FAMILY)["missing"], false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_document_fonts_folder_is_found_whatever_its_case() {
        const FAMILY: &str = "DocFont Case Test";
        let dir = std::env::temp_dir().join(format!("pagenor-open-docfonts-case-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let fonts = dir.join("Document fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::write(fonts.join("case.ttf"), font_with(FAMILY, &['H']).unwrap()).unwrap();
        let path = dir.join("Brochure.idml");
        write_document(&path, FAMILY);

        let mut s = Session::new();
        let r = open(&mut s, &path);
        assert_eq!(r["documentFonts"], 1, "{r}");
        assert_eq!(font_entry(&mut s, FAMILY)["missing"], false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_document_without_the_folder_opens_as_before() {
        let dir = std::env::temp_dir().join(format!("dc-open-docfonts-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Plain.pagenor");
        write_document(&path, pagenor_fonts::DEFAULT_FAMILY);
        let r = Session::new().execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
        assert_eq!((r["documentFonts"].clone(), r["warnings"].clone()), (json!(0), json!([])), "{r}");
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn document_fonts_belong_to_their_document() {
        const FAMILY: &str = "DocFont Scoped";
        let (a, b) = (temp_dir("scoped-a"), temp_dir("scoped-b"));
        std::fs::write(a.join(DOCUMENT_FONTS_FOLDER).join("scoped.ttf"), font_with(FAMILY, &['H', 'e', 'l', 'o']).unwrap()).unwrap();
        write_document(&a.join("A.pagenor"), FAMILY);
        write_document(&b.join("B.pagenor"), FAMILY);

        let mut s = Session::new();
        assert_eq!(open(&mut s, &a.join("A.pagenor"))["documentFonts"], 1);
        assert_eq!(open(&mut s, &b.join("B.pagenor"))["documentFonts"], 0);
        // B, open at the same time, doesn't have A's font.
        assert_eq!(font_entry(&mut s, FAMILY)["missing"], true);
        assert!(glyph_faces(&s).iter().all(|(_, f)| f.family != FAMILY), "{:?}", glyph_faces(&s));
        s.execute("file.activate", &json!({"index": 0})).unwrap();
        assert_eq!(font_entry(&mut s, FAMILY)["source"], "document");
        assert!(glyph_faces(&s).iter().all(|(_, f)| f.family == FAMILY), "{:?}", glyph_faces(&s));
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn each_document_sets_its_own_font_of_a_name() {
        const FAMILY: &str = "DocFont Same";
        let (a, b) = (temp_dir("same-a"), temp_dir("same-b"));
        let (a_font, b_font) = (a.join(DOCUMENT_FONTS_FOLDER).join("same.ttf"), b.join(DOCUMENT_FONTS_FOLDER).join("same.ttf"));
        // A's font has both letters, B's only `b`.
        std::fs::write(&a_font, font_with(FAMILY, &['a', 'b']).unwrap()).unwrap();
        std::fs::write(&b_font, font_with(FAMILY, &['b']).unwrap()).unwrap();
        // The same words in both: the word cache must not hand one document's shaping to the other.
        write_document_with(&a.join("A.pagenor"), FAMILY, "ab ab ab");
        write_document_with(&b.join("B.pagenor"), FAMILY, "ab ab ab");

        let mut s = Session::new();
        open(&mut s, &a.join("A.pagenor"));
        open(&mut s, &b.join("B.pagenor"));
        // Fallback fonts for the `a` B's font lacks: never A's font.
        s.execute("document.preferences", &json!({"glyphFallback": true})).unwrap();
        let in_b = glyph_faces(&s);
        s.execute("file.activate", &json!({"index": 0})).unwrap();
        let in_a = glyph_faces(&s);
        assert_eq!((in_a.len(), in_b.len()), (8, 8), "{in_a:?} {in_b:?}");
        for (c, f) in &in_a {
            assert_eq!(f.source, FontSource::Document(a_font.clone()), "`{c}` in A");
        }
        for (c, f) in &in_b {
            match c {
                'a' => assert!(f.family != FAMILY, "B's font lacks `a`, and A's isn't B's: {f:?}"),
                _ => assert_eq!(f.source, FontSource::Document(b_font.clone()), "`{c}` in B"),
            }
        }
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn closing_a_document_forgets_its_fonts_and_reopening_reuses_them() {
        const FAMILY: &str = "DocFont Closed";
        let dir = temp_dir("closed");
        std::fs::write(dir.join(DOCUMENT_FONTS_FOLDER).join("closed.ttf"), font_with(FAMILY, &['H', 'e', 'l', 'o']).unwrap()).unwrap();
        let path = dir.join("Closed.pagenor");
        write_document(&path, FAMILY);

        let mut s = Session::new();
        open(&mut s, &path);
        let first = glyph_faces(&s)[0].1;
        assert_eq!(first.family, FAMILY);
        s.execute("file.close", &json!({})).unwrap();
        // A new document can't use the closed document's font.
        new_document(&mut s, FAMILY, "Hello");
        assert_eq!(font_entry(&mut s, FAMILY)["missing"], true);
        assert!(glyph_faces(&s).iter().all(|(_, f)| f.family != FAMILY));
        // Reopened, the document has its font again: the face loaded the first time.
        assert_eq!(open(&mut s, &path)["documentFonts"], 1);
        assert!(glyph_faces(&s).iter().all(|(_, f)| *f == first), "{:?}", glyph_faces(&s));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_reverted_document_keeps_its_fonts() {
        const FAMILY: &str = "DocFont Reverted";
        let dir = temp_dir("revert");
        std::fs::write(dir.join(DOCUMENT_FONTS_FOLDER).join("revert.ttf"), font_with(FAMILY, &['H', 'e', 'l', 'o']).unwrap()).unwrap();
        let path = dir.join("Revert.pagenor");
        write_document(&path, FAMILY);

        let mut s = Session::new();
        open(&mut s, &path);
        s.execute("frame.create", &json!({"rect": [72, 400, 200, 500], "content": "text", "text": "x"})).unwrap();
        s.execute("file.revert", &json!({})).unwrap();
        assert_eq!(font_entry(&mut s, FAMILY)["source"], "document");
        assert!(glyph_faces(&s).iter().all(|(_, f)| f.family == FAMILY));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod serialization_failure_tests {
    use super::*;
    use pagenor_doc::{Asset, AssetId};

    #[test]
    fn refused_native_save_never_replaces_a_file_with_empty_bytes() {
        let dir = std::env::temp_dir().join(format!("pagenor-refused-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("existing.pgnr");
        let old = b"synthetic existing file";
        std::fs::write(&path, old).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("frame.create", &json!({"rect": [36, 36, 100, 100], "content": "text", "text": "Keep me"})).unwrap();
        let csv = dir.join("source.csv");
        std::fs::write(&csv, b"Name\nSynthetic record\n").unwrap();
        s.execute("data.source.select", &json!({"path": csv.to_string_lossy()})).unwrap();
        let recovery_dir = dir.join("recovery");
        crate::recovery::save(&s, &recovery_dir).unwrap();
        let snapshots = || {
            let mut files: Vec<_> = std::fs::read_dir(&recovery_dir)
                .unwrap()
                .map(|e| {
                    let e = e.unwrap();
                    (e.file_name(), std::fs::read(e.path()).unwrap())
                })
                .collect();
            files.sort();
            files
        };
        let checkpoint = snapshots();
        {
            let st = s.doc_mut().unwrap();
            st.path = Some("original.pgnr".into());
            let d = Arc::make_mut(&mut st.doc);
            for i in 0..4094 {
                let id = AssetId(d.alloc());
                d.assets.insert(
                    id,
                    Arc::new(Asset {
                        id,
                        page: 0,
                        name: format!("asset-{i}"),
                        mime: "application/octet-stream".into(),
                        link: None,
                        data: Arc::new((i as u32).to_le_bytes().to_vec()),
                        pixels: None,
                        ..Default::default()
                    }),
                );
                // Used, distinct assets exceed the archive limit even after save compaction.
                let item_id = pagenor_doc::ItemId(d.alloc());
                let mut item = pagenor_doc::Item::new(
                    item_id,
                    d.default_layer(),
                    pagenor_doc::Shape::Rectangle,
                    pagenor_geom::shapes::rectangle(pagenor_geom::Rect::new(0.0, 0.0, 1.0, 1.0)),
                );
                item.content = pagenor_doc::Content::Graphic(pagenor_doc::Graphic {
                    clip: None,
                    asset: id,
                    size: (1.0, 1.0),
                    xf: pagenor_geom::Affine::IDENTITY,
                    auto_fit: Default::default(),
                    fit_align: 0,
                    crop: [0.0; 4],
                    wrap: Default::default(),
                });
                d.insert_item(pagenor_doc::SpreadRef::Doc(0), item, None).unwrap();
            }
        }
        let saved_revision = s.doc().unwrap().saved_revision;
        let saved_doc = s.doc().unwrap().saved_doc.clone();
        let current_doc = s.doc().unwrap().doc.clone();
        for command in ["file.saveACopy", "file.saveAs", "file.save"] {
            assert!(s.execute(command, &json!({"path": path.to_string_lossy()})).is_err(), "{command} must fail");
            assert_eq!(std::fs::read(&path).unwrap(), old, "{command} must preserve existing bytes");
            let st = s.doc().unwrap();
            assert_eq!(st.path.as_deref(), Some("original.pgnr"));
            assert_eq!(st.saved_revision, saved_revision);
            assert!(Arc::ptr_eq(&st.saved_doc, &saved_doc));
            assert!(Arc::ptr_eq(&st.doc, &current_doc));
            assert!(st.is_dirty());
        }
        assert!(s.execute("file.serialize", &json!({})).is_err());
        assert!(crate::recovery::save(&s, &recovery_dir).is_err());
        assert_eq!(snapshots(), checkpoint, "a refused checkpoint must retain the last valid recovery files");
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn refused_native_open_preserves_the_active_document_and_edits() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("frame.create", &json!({"rect": [36, 36, 100, 100], "content": "text", "text": "Keep me 😀"})).unwrap();
        let original = s.doc().unwrap().doc.clone();
        let uid = s.doc().unwrap().uid;
        assert!(s.execute("file.openBytes", &json!({"name": "broken.pgnr", "base64": base64_encode(b"PK damaged archive")})).is_err());
        assert_eq!(s.documents().len(), 1);
        assert_eq!(s.doc().unwrap().uid, uid);
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &original));
        assert!(s.doc().unwrap().is_dirty());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_save_tests {
    use super::*;

    #[test]
    fn readonly_save_routes_preserve_document_file_and_recovery_then_retry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Saved ఉదయం.pgnr");
        let recovery = dir.path().join("recovery");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.saveAs", &json!({"path": path})).unwrap();
        let old = std::fs::read(&path).unwrap();
        s.execute("frame.create", &json!({"rect": [36, 36, 300, 200], "content": "text", "text": "Keep edits ఉదయం 😀"})).unwrap();
        crate::recovery::save(&s, &recovery).unwrap();
        let checkpoint = || {
            let mut files: Vec<_> = std::fs::read_dir(&recovery)
                .unwrap()
                .map(|e| {
                    let e = e.unwrap();
                    (e.file_name(), std::fs::read(e.path()).unwrap())
                })
                .collect();
            files.sort();
            files
        };
        let before = checkpoint();
        let st = s.doc().unwrap();
        let live = st.doc.clone();
        let saved = st.saved_doc.clone();
        let saved_revision = st.saved_revision;
        let original_permissions = std::fs::metadata(&path).unwrap().permissions();
        let mut readonly = original_permissions.clone();
        readonly.set_readonly(true);
        std::fs::set_permissions(&path, readonly).unwrap();
        for command in ["file.save", "file.saveAs", "file.saveACopy"] {
            assert!(s.execute(command, &json!({"path":path})).is_err(), "{command} must respect read-only files");
            let st = s.doc().unwrap();
            assert_eq!(st.path.as_deref(), path.to_str());
            assert!(Arc::ptr_eq(&live, &st.doc));
            assert!(Arc::ptr_eq(&saved, &st.saved_doc));
            assert_eq!(st.saved_revision, saved_revision);
            assert!(st.is_dirty());
            assert_eq!(std::fs::read(&path).unwrap(), old);
            assert_eq!(checkpoint(), before);
        }
        std::fs::set_permissions(&path, original_permissions).unwrap();
        s.execute("file.save", &json!({})).unwrap();
        assert!(!s.doc().unwrap().is_dirty());
        assert!(crate::recovery::list(&recovery).is_empty());
        let mut reopened = Session::new();
        reopened.execute("file.open", &json!({"path":path})).unwrap();
        assert_eq!(serde_json::to_value(&reopened.doc().unwrap().doc).unwrap(), serde_json::to_value(&live).unwrap());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2, "no staging files remain");
    }

    #[test]
    fn replacement_copy_retains_dirty_state_original_path_and_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Original.pgnr");
        let copy = dir.path().join("Copy తెలుగు.pgnr");
        let recovery = dir.path().join("recovery");
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.saveAs", &json!({"path":path})).unwrap();
        let original = std::fs::read(&path).unwrap();
        s.execute("frame.create", &json!({"rect":[36,36,300,200],"content":"text","text":"Copy ఉదయం"})).unwrap();
        crate::recovery::save(&s, &recovery).unwrap();
        std::fs::write(&copy, b"old copy").unwrap();
        let live = s.doc().unwrap().doc.clone();
        let history = s.doc().unwrap().history.undo.len();
        s.execute("file.saveACopy", &json!({"path":copy})).unwrap();
        let st = s.doc().unwrap();
        assert!(st.is_dirty());
        assert_eq!(st.path.as_deref(), path.to_str());
        assert!(Arc::ptr_eq(&st.doc, &live));
        assert_eq!(st.history.undo.len(), history);
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(crate::recovery::list(&recovery).len(), 1);
        let mut reopened = Session::new();
        reopened.execute("file.open", &json!({"path":copy})).unwrap();
        assert_eq!(serde_json::to_value(&reopened.doc().unwrap().doc).unwrap(), serde_json::to_value(&live).unwrap());
    }
}
