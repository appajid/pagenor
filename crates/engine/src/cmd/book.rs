//! Books (File › New / Open Book): an ordered list of documents with continuous page numbers,
//! styles synchronised from a style source, and one PDF for the whole book. Saved as `.pbook`.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{CommandSpec, always, bad, cmd, ok, str_param};
use crate::{EngineError, Result, Session};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Book {
    #[serde(skip)]
    pub path: String,
    pub documents: Vec<String>,
    /// Index of the style source document.
    pub style_source: usize,
}

fn has_book(s: &Session) -> std::result::Result<(), String> {
    if s.book.is_some() { Ok(()) } else { Err("no book open".into()) }
}

fn no_book() -> EngineError {
    bad("book", "no book open")
}

#[cfg(not(target_arch = "wasm32"))]
fn save(b: &Book) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(b).map_err(|e| EngineError::Other(e.to_string()))?;
    crate::write_atomic(std::path::Path::new(&b.path), &bytes).map_err(|e| EngineError::Other(format!("{}: {e}", b.path)))
}

#[cfg(target_arch = "wasm32")]
fn save(_: &Book) -> Result<()> {
    Err(EngineError::Other("books need a file system".into()))
}

fn load(path: &str) -> Result<pagenor_doc::Document> {
    #[cfg(not(target_arch = "wasm32"))]
    let bytes = std::fs::read(path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
    #[cfg(target_arch = "wasm32")]
    let bytes: Vec<u8> = {
        let _ = path;
        vec![]
    };
    if path.to_lowercase().ends_with(".idml") {
        let mut d = pagenor_idml::import_idml(&bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
        super::interchange::resolve_pdf_crops(&mut d);
        return Ok(d);
    }
    super::file::from_bytes(&bytes)
}

fn store(path: &str, d: &pagenor_doc::Document) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    return crate::native_save::write(std::path::Path::new(path), &super::file::to_bytes(d)?).map_err(|e| EngineError::Other(format!("{path}: {e}")));
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, d);
        Err(EngineError::Other("books need a file system".into()))
    }
}

/// Make page numbering of `d` start at `n` (the first section).
fn start_at(d: &mut pagenor_doc::Document, n: u32) {
    if !d.sections.iter().any(|x| x.start == 0) {
        d.sections.insert(
            0,
            pagenor_doc::Section {
                start: 0,
                start_number: None,
                style: Default::default(),
                prefix: String::new(),
                marker: String::new(),
                include_prefix: false,
            },
        );
    }
    if let Some(first) = d.sections.iter_mut().find(|x| x.start == 0) {
        first.start_number = Some(n.max(1));
    }
}

/// Compare aliases as well as literal paths when a book refers to an open document.
pub(super) fn document_path(path: &str) -> std::path::PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(path) = std::fs::canonicalize(path) {
        return path;
    }
    std::path::PathBuf::from(path)
}

fn paginate(s: &mut Session) -> Result<Value> {
    let b = s.book.clone().ok_or_else(no_book)?;
    if b.documents.len() > 9_999 {
        return Err(bad("book.paginate", "a book supports at most 9999 documents"));
    }
    let mut next = 1u32;
    let mut chapter = 0u32;
    let mut plans = Vec::new();
    let mut seen = std::collections::HashSet::new();
    // Resolve and validate every input before the first write. Keep only numbering
    // metadata in the plan, rather than retaining every document in memory.
    for path in &b.documents {
        let key = document_path(path);
        if !seen.insert(key.clone()) {
            return Err(bad("book.paginate", format!("duplicate book document: {path}")));
        }
        if path.to_lowercase().ends_with(".idml") {
            return Err(bad("book.paginate", "save IDML book documents as .pgnr before updating numbering"));
        }
        if s.documents().iter().any(|st| {
            st.path.as_deref().is_some_and(|p| document_path(p) == key) && (st.is_dirty() || st.interaction.is_some() || st.preview_stash.is_some())
        }) {
            return Err(bad("book.paginate", format!("save or finish editing the open document first: {path}")));
        }
        let d = load(path)?;
        if d.settings.chapter_style == pagenor_doc::NumberStyle::Symbols {
            return Err(bad("book.paginate", format!("symbols do not number chapters: {path}")));
        }
        chapter = match d.settings.chapter_source {
            pagenor_doc::ChapterSource::Automatic => chapter.saturating_add(1),
            pagenor_doc::ChapterSource::UserDefined => d.settings.chapter_number.max(1),
            pagenor_doc::ChapterSource::SameAsPrevious => chapter.max(1),
        };
        if chapter > 99_999 {
            return Err(bad("book.paginate", format!("chapter number exceeds 99999: {path}")));
        }
        let pages = u32::try_from(d.page_count()).map_err(|_| bad("book.paginate", "too many pages"))?;
        plans.push((path.clone(), key, next, chapter));
        next = next.checked_add(pages).ok_or_else(|| bad("book.paginate", "too many pages"))?;
    }
    let mut chapters = Vec::new();
    for (path, key, first, chapter) in plans {
        let mut d = load(&path)?;
        start_at(&mut d, first);
        d.settings.chapter_number = chapter;
        super::sync_placeholders(&mut d);
        store(&path, &d)?;
        chapters.push(d.chapter_label());
        // Reflect the successful disk update in clean open documents. Keep their
        // fonts, selections and history, so a later Save cannot restore stale numbers.
        for st in &mut s.docs {
            if st.path.as_deref().is_some_and(|p| document_path(p) == key) {
                let mut open = (*st.doc).clone();
                open.sections = d.sections.clone();
                open.settings.chapter_number = chapter;
                super::sync_placeholders(&mut open);
                st.doc = std::sync::Arc::new(open);
                st.revision = st.revision.saturating_add(1);
                st.saved_doc = st.doc.clone();
                st.saved_revision = st.revision;
            }
        }
    }
    Ok(json!({"pages": next.saturating_sub(1), "chapters": chapters}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(noundo "book.new", "New Book", ["File", "New"], None, "{path} — an empty book file (.pbook)", always, |s, p| {
            let path = str_param(p, "path").ok_or_else(|| bad("book.new", "`path` required"))?.to_string();
            let b = Book { path, ..Default::default() };
            save(&b)?;
            s.book = Some(b);
            ok()
        }),
        cmd!(noundo "book.open", "Open Book", [], None, "{path} → {documents}", always, |s, p| {
            let path = str_param(p, "path").ok_or_else(|| bad("book.open", "`path` required"))?.to_string();
            #[cfg(not(target_arch = "wasm32"))]
            let text = std::fs::read_to_string(&path).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
            #[cfg(target_arch = "wasm32")]
            let text = String::new();
            let mut b: Book = serde_json::from_str(&text).map_err(|e| bad("book.open", format!("not a book: {e}")))?;
            b.path = path;
            let n = b.documents.len();
            s.book = Some(b);
            Ok(json!({"documents": n}))
        }),
        cmd!(noundo "book.close", "Close Book", [], None, "{}", has_book, |s, _| {
            s.book = None;
            ok()
        }),
        cmd!(noundo "book.add", "Add Document", [], None, "{path, at?: index}", has_book, |s, p| {
            let path = str_param(p, "path").ok_or_else(|| bad("book.add", "`path` required"))?.to_string();
            load(&path)?;
            let b = s.book.as_mut().ok_or_else(no_book)?;
            let at = p.get("at").and_then(Value::as_u64).map_or(b.documents.len(), |i| (i as usize).min(b.documents.len()));
            b.documents.insert(at, path);
            save(b)?;
            ok()
        }),
        cmd!(noundo "book.remove", "Remove Document", [], None, "{index}", has_book, |s, p| {
            let i = p.get("index").and_then(Value::as_u64).ok_or_else(|| bad("book.remove", "`index` required"))? as usize;
            let b = s.book.as_mut().ok_or_else(no_book)?;
            if i >= b.documents.len() {
                return Err(bad("book.remove", format!("no document {i}")));
            }
            b.documents.remove(i);
            if b.style_source >= b.documents.len() {
                b.style_source = 0;
            }
            save(b)?;
            ok()
        }),
        cmd!(noundo "book.styleSource", "Style Source", [], None, "{index}", has_book, |s, p| {
            let i = p.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let b = s.book.as_mut().ok_or_else(no_book)?;
            if i >= b.documents.len() {
                return Err(bad("book.styleSource", format!("no document {i}")));
            }
            b.style_source = i;
            save(b)?;
            ok()
        }),
        cmd!(query "book.list", "Book", [], None, "{} → {path, styleSource, documents: [{path, pages, firstPage}]}", has_book, |s, _| {
            let b = s.book.as_ref().ok_or_else(no_book)?;
            let mut first = 1usize;
            let mut docs = Vec::new();
            for p in &b.documents {
                let pages = load(p).map(|d| d.page_count()).unwrap_or(0);
                docs.push(json!({"path": p, "pages": pages, "firstPage": first}));
                first = first.saturating_add(pages);
            }
            Ok(json!({"path": b.path, "styleSource": b.style_source, "documents": docs}))
        }),
        cmd!(noundo "book.paginate", "Update Numbering", [], None, "{} — each document starts numbering where the previous one ended, and takes its chapter number (automatic: one more than the previous document's; same as previous; or its own start) (files are saved; save open modified documents first) → {pages, chapters}", has_book, |s, _| paginate(s)),
        cmd!(noundo "book.syncStyles", "Synchronize Book", [], None, "{} — paragraph and character styles and swatches of the style source go into every document (by name)", has_book, |s, _| {
            let b = s.book.clone().ok_or_else(no_book)?;
            let src_path = b.documents.get(b.style_source).ok_or_else(|| bad("book.syncStyles", "the book is empty"))?;
            let src = load(src_path)?;
            let mut n = 0;
            for (i, p) in b.documents.iter().enumerate() {
                if i == b.style_source {
                    continue;
                }
                let mut d = load(p)?;
                {
                    let st = d.styles_mut();
                    for ps in &src.styles.paragraph {
                        st.paragraph.retain(|x| x.name != ps.name);
                        st.paragraph.push(ps.clone());
                    }
                    for cs in &src.styles.character {
                        st.character.retain(|x| x.name != cs.name);
                        st.character.push(cs.clone());
                    }
                }
                for sw in &src.swatches {
                    match d.swatches.iter_mut().find(|w| w.name == sw.name) {
                        Some(w) if !w.locked => *w = sw.clone(),
                        Some(_) => {}
                        None => d.swatches.push(sw.clone()),
                    }
                }
                store(p, &d)?;
                n += 1;
            }
            Ok(json!({"synced": n}))
        }),
        cmd!(noundo "book.exportPdf", "Export Book to PDF…", [], None, "{path?} — every document in order as one PDF → {path, bytes, pages} (no path: {base64, …})", has_book, |s, p| {
            let b = s.book.clone().ok_or_else(no_book)?;
            let mut parts = Vec::new();
            for path in &b.documents {
                let mut d = load(path)?;
                // The chapter's own fonts, for this export (skipped font files are logged).
                let _fonts = super::file::load_document_fonts(&mut d, path).0;
                let cache = pagenor_compose::Cache::new();
                let r = pagenor_pdf::export_pdf_with_report(&super::interchange::with_pdf_boxes(&d), &cache, &pagenor_pdf::PdfOptions::default()).map_err(|e| EngineError::Other(e.to_string()))?;
                let n = pagenor_render::pdf_page_count(&r.bytes).unwrap_or(0);
                let sizes: Vec<(f32, f32)> = (0..n).filter_map(|i| pagenor_render::pdf_page_size(&r.bytes, i)).map(|(w, h)| (w as f32, h as f32)).collect();
                parts.push((r.bytes, sizes));
            }
            let pages: usize = parts.iter().map(|p| p.1.len()).sum();
            let bytes = pagenor_pdf::merge_pdfs(&parts, None).map_err(|e| EngineError::Other(e.to_string()))?;
            match str_param(p, "path") {
                Some(path) => {
                    #[cfg(not(target_arch = "wasm32"))]
                    crate::write_atomic(std::path::Path::new(path), &bytes).map_err(|e| EngineError::Other(format!("{path}: {e}")))?;
                    Ok(json!({"path": path, "bytes": bytes.len(), "pages": pages}))
                }
                None => Ok(json!({"base64": super::file::base64_encode(&bytes), "bytes": bytes.len(), "pages": pages})),
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Session;

    #[test]
    fn book_numbering_preflights_and_updates_open_documents_safely() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chapter.pgnr").to_string_lossy().to_string();
        let book = dir.path().join("guide.pbook").to_string_lossy().to_string();
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.saveAs", &json!({"path": path})).unwrap();
        s.execute("book.new", &json!({"path": book})).unwrap();
        s.execute("book.add", &json!({"path": path})).unwrap();
        let before = std::fs::read(&path).unwrap();
        s.execute("layout.chapterNumbering", &json!({"start": 7})).unwrap();
        assert!(s.execute("book.paginate", &json!({})).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(s.doc().unwrap().is_dirty());
        s.execute("file.save", &json!({})).unwrap();
        let before = std::fs::read(&path).unwrap();
        s.book.as_mut().unwrap().documents.push(dir.path().join("missing.pgnr").to_string_lossy().to_string());
        assert!(s.execute("book.paginate", &json!({})).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before, "preflight precedes every write");
        s.book.as_mut().unwrap().documents.pop();
        s.execute("book.paginate", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc.chapter_label(), "7");
        assert!(!s.doc().unwrap().is_dirty());
        s.execute("file.save", &json!({})).unwrap();
        assert_eq!(super::load(&path).unwrap().chapter_label(), "7");
        #[cfg(unix)]
        {
            let alias = dir.path().join("alias.pgnr").to_string_lossy().to_string();
            std::os::unix::fs::symlink(&path, &alias).unwrap();
            s.book.as_mut().unwrap().documents.push(alias);
            let before = std::fs::read(&path).unwrap();
            assert!(s.execute("book.paginate", &json!({})).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), before);
        }
    }

    #[test]
    fn book_numbering_resolves_chapter_numbers_in_order() {
        let dir = std::env::temp_dir().join(format!("pagenor-book-chapters-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = |n: &str| dir.join(n).to_string_lossy().to_string();
        let mut s = Session::new();
        s.execute("book.new", &json!({"path": p("Field Guide.pbook")})).unwrap();
        let chapters = [
            ("a.pgnr", json!({"style": "upperRoman"})),
            ("b.pgnr", json!({"source": "sameAsPrevious"})),
            ("c.pgnr", json!({})),
            ("d.pgnr", json!({"start": 7, "style": "lowerLetters"})),
            ("e.pgnr", json!({})),
        ];
        for (name, numbering) in &chapters {
            s.execute("file.new", &json!({})).unwrap();
            if numbering.as_object().is_some_and(|o| !o.is_empty()) {
                s.execute("layout.chapterNumbering", numbering).unwrap();
            }
            s.execute("file.saveAs", &json!({"path": p(name)})).unwrap();
            s.execute("book.add", &json!({"path": p(name)})).unwrap();
        }
        let r = s.execute("book.paginate", &json!({})).unwrap();
        // Same as previous takes the number, written in the document's own style.
        assert_eq!(r["chapters"], json!(["I", "1", "2", "g", "8"]));
        assert_eq!(super::load(&p("e.pgnr")).unwrap().settings.chapter_number, 8);
        // The open document (e) names its book.
        assert_eq!(s.execute("layout.chapterNumbering", &json!({})).unwrap()["book"], "Field Guide");
        assert_eq!(s.execute("layout.chapterNumbering", &json!({})).unwrap()["label"], "8", "open document reflects the book update");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn book_paginates_syncs_and_exports() {
        let dir = std::env::temp_dir().join(format!("dc-book-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = |n: &str| dir.join(n).to_string_lossy().to_string();
        let mut s = Session::new();
        for (name, pages, style) in [("a.pagenor", 3, "Chapter Head"), ("b.pagenor", 2, "Other")] {
            s.execute("file.new", &json!({"pages": pages})).unwrap();
            s.execute("style.paragraph.create", &json!({"name": style, "chars": {"size": 20}})).unwrap();
            s.execute("file.saveAs", &json!({"path": p(name)})).unwrap();
        }
        s.execute("book.new", &json!({"path": p("book.pbook")})).unwrap();
        s.execute("book.add", &json!({"path": p("a.pagenor")})).unwrap();
        s.execute("book.add", &json!({"path": p("b.pagenor")})).unwrap();
        let l = s.execute("book.list", &json!({})).unwrap();
        assert_eq!(l["documents"][1]["firstPage"], 4);
        assert_eq!(s.execute("book.paginate", &json!({})).unwrap()["pages"], 5);
        let b = super::load(&p("b.pagenor")).unwrap();
        assert_eq!(b.page_name(0), "4", "continues after the first document");
        assert_eq!(s.execute("book.syncStyles", &json!({})).unwrap()["synced"], 1);
        assert!(super::load(&p("b.pagenor")).unwrap().styles.para("Chapter Head").is_some());
        let r = s.execute("book.exportPdf", &json!({})).unwrap();
        assert_eq!(r["pages"], 5);
        let bytes = super::super::file::base64_decode(r["base64"].as_str().unwrap());
        assert_eq!(pagenor_render::pdf_page_count(&bytes), Some(5));
        // A fresh session reopens the book.
        let mut t = Session::new();
        assert_eq!(t.execute("book.open", &json!({"path": p("book.pbook")})).unwrap()["documents"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}
