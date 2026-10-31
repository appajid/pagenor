//! Bounded book TOC collection, isolated layout candidates and semantic freshness checks.
use super::super::{bad, toc};
use super::{Book, document_path, load, no_book};
#[cfg(not(target_arch = "wasm32"))]
use super::{save, start_at};
#[cfg(not(target_arch = "wasm32"))]
use crate::EngineError;
use crate::{Result, Session};
use pagenor_doc::{Document, TocEntry, TocScope};
use serde_json::{Value, json};

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn read_bounded(path: &str, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| bad("book", format!("{path}: {e}")))?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| bad("book", e.to_string()))?;
    if bytes.len() as u64 > limit {
        return Err(bad("book", "source exceeds the book input limit"));
    }
    Ok(bytes)
}

struct Chapter {
    path: String,
    doc: Document,
    #[cfg(not(target_arch = "wasm32"))]
    bytes: Vec<u8>,
    _fonts: Option<std::sync::Arc<crate::FontScope>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn verify_manifest(book: &Book) -> Result<()> {
    let bytes = read_bounded(&book.path, 4 << 20)?;
    let disk: Book = serde_json::from_slice(&bytes).map_err(|e| bad("book", e.to_string()))?;
    let folder = std::path::Path::new(&book.path).parent().unwrap_or(std::path::Path::new("."));
    let order = |documents: &[String]| {
        documents
            .iter()
            .map(|path| {
                let path = std::path::Path::new(path);
                let resolved = if path.is_relative() { folder.join(path) } else { path.to_path_buf() };
                document_path(&resolved.to_string_lossy())
            })
            .collect::<Vec<_>>()
    };
    if order(&disk.documents) != order(&book.documents)
        || disk.style_source != book.style_source
        || disk.numbering_needs_update != book.numbering_needs_update
        || disk.toc_state != book.toc_state
    {
        return Err(bad("book", "the book manifest changed on disk; reopen the book before continuing"));
    }
    Ok(())
}

fn snapshot(s: &Session, live: bool) -> Result<(Vec<Chapter>, Option<usize>)> {
    let book = s.book.as_ref().ok_or_else(no_book)?;
    #[cfg(not(target_arch = "wasm32"))]
    verify_manifest(book)?;
    if book.documents.is_empty() || book.documents.len() > 4096 {
        return Err(bad("toc", "a book requires 1–4096 chapters"));
    }
    let mut seen = std::collections::HashSet::new();
    let active = if live { Some(s.doc()?) } else { None };
    let active_path = active.and_then(|d| d.path.as_deref()).map(document_path);
    if live && active_path.is_none() {
        return Err(bad("toc", "save this document and add it to the open book first"));
    }
    let mut member = None;
    let mut total = 0usize;
    let mut chapters = Vec::new();
    for path in &book.documents {
        let key = document_path(path);
        if !seen.insert(key.clone()) {
            return Err(bad("toc", "duplicate book chapter"));
        }
        let is_live = active_path.as_ref() == Some(&key);
        if s.documents()
            .iter()
            .any(|d| d.path.as_deref().is_some_and(|p| document_path(p) == key) && (!is_live || active.is_none_or(|a| d.uid != a.uid)))
        {
            return Err(bad("toc", format!("save and close the other book documents first: {path}")));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let bytes = read_bounded(path, (128usize * 1024 * 1024).saturating_sub(total) as u64)?;
        #[cfg(target_arch = "wasm32")]
        let bytes = Vec::<u8>::new();
        total = total.saturating_add(bytes.len());
        if total > 128 * 1024 * 1024 {
            return Err(bad("toc", "book inputs exceed 128 MiB"));
        }
        let mut doc = if is_live {
            let state = active.ok_or_else(|| bad("toc", "active document missing"))?;
            if state.interaction.is_some() || state.preview_stash.is_some() {
                return Err(bad("toc", "finish the active edit before updating the book TOC"));
            }
            member = Some(chapters.len());
            (*state.doc).clone()
        } else {
            load(path)?
        };
        let fonts = if is_live {
            active.and_then(|d| d.fonts.clone())
        } else {
            let (fonts, _, warnings) = super::super::file::load_document_fonts(&mut doc, path);
            if !warnings.is_empty() {
                return Err(bad("toc", format!("cannot verify layout in {path}: {}", warnings.join("; "))));
            }
            fonts
        };
        chapters.push(Chapter {
            path: path.clone(),
            doc,
            #[cfg(not(target_arch = "wasm32"))]
            bytes,
            _fonts: fonts,
        });
    }
    if live && member.is_none() {
        return Err(bad("toc", "the active document must belong to the open book"));
    }
    Ok((chapters, member))
}

fn collect(chapters: &[Chapter], entries: &[TocEntry]) -> Result<Vec<toc::Found>> {
    let mut found = Vec::new();
    let mut next = 1u32;
    let mut styles = std::collections::HashSet::new();
    for chapter in chapters {
        if chapter.doc.page_number(0) != next {
            return Err(bad("toc", "book numbering is out of date; save and close chapters, then Update Numbering"));
        }
        next = next
            .checked_add(u32::try_from(chapter.doc.page_count()).map_err(|_| bad("toc", "too many pages"))?)
            .ok_or_else(|| bad("toc", "too many pages"))?;
        styles.extend(chapter.doc.styles.paragraph.iter().map(|s| s.name.as_str()));
        found.extend(toc::collect(&chapter.doc, entries, chapter.doc.toc.as_ref().map(|t| t.story)));
        if found.len() > 100_000 {
            return Err(bad("toc", "book TOC exceeds 100000 entries"));
        }
    }
    if entries.iter().any(|e| !styles.contains(e.style.as_str())) {
        return Err(bad("toc", "a selected source style is missing from every book chapter"));
    }
    Ok(found)
}

pub(in crate::cmd) fn book_toc_styles(s: &Session) -> Result<Value> {
    let (chapters, _) = snapshot(s, true)?;
    let styles: std::collections::BTreeSet<_> = chapters
        .iter()
        .flat_map(|c| c.doc.styles.paragraph.iter().filter(|p| p.name != pagenor_doc::NO_PARA_STYLE).map(|p| p.name.clone()))
        .collect();
    Ok(json!(styles))
}

pub(in crate::cmd) fn book_toc_entries(s: &Session, entries: &[TocEntry]) -> Result<Value> {
    let (chapters, _) = snapshot(s, true)?;
    Ok(json!(collect(&chapters, entries)?.into_iter().map(|e| json!({"level":e.level,"text":e.text,"page":e.page})).collect::<Vec<_>>()))
}

pub(in crate::cmd) fn update_active_toc(s: &mut Session, params: Option<&Value>) -> Result<Value> {
    let (mut chapters, member) = snapshot(s, true)?;
    let index = member.ok_or_else(|| bad("toc", "active chapter missing"))?;
    let mut worker = Session::new();
    worker.prefs = s.prefs.clone();
    worker.add_document(crate::DocState::new(chapters.get(index).ok_or_else(|| bad("toc", "active chapter missing"))?.doc.clone(), None));
    if let Some(params) = params {
        toc::configure(&mut worker, params, true)?;
    }
    let definition = worker.doc()?.doc.toc.clone().ok_or_else(|| bad("toc", "no table of contents"))?;
    if chapters.iter().enumerate().any(|(i, c)| i != index && c.doc.toc.as_ref().is_some_and(|t| t.scope == TocScope::EntireBook)) {
        return Err(bad("toc", "one Entire Book TOC is supported per book"));
    }
    let found = collect(&chapters, &definition.entries)?;
    let before = worker.doc()?.doc.clone();
    let count = worker.edit(|d, _| toc::fill_found(d, &definition, None, found))?;
    let uid = worker.doc()?.uid;
    if worker.prefs.smart_text_reflow {
        worker.smart_reflow(Some((uid, &before)));
    }
    let candidate = (*worker.doc()?.doc).clone();
    if pagenor_compose::compose_story(&candidate, definition.story, &Default::default()).overset_at.is_some() {
        return Err(bad("toc", "the book TOC does not fit; allocate more TOC frames/pages before retrying"));
    }
    let old = collect(&chapters, &definition.entries)?;
    chapters.get_mut(index).ok_or_else(|| bad("toc", "active chapter missing"))?.doc = candidate.clone();
    if old != collect(&chapters, &definition.entries)? {
        return Err(bad(
            "toc",
            "the live TOC changed heading positions; reserve fixed TOC space, or save and close chapters and Refresh Book Table of Contents",
        ));
    }
    s.edit(|d, sel| {
        *d = candidate;
        *sel = worker.doc()?.selection.clone();
        Ok(json!({"story":definition.story.0,"entries":count}))
    })
}

fn state(book: &Book, chapters: &[Chapter]) -> Result<Value> {
    let indices: Vec<_> =
        chapters.iter().enumerate().filter(|(_, c)| c.doc.toc.as_ref().is_some_and(|t| t.scope == TocScope::EntireBook)).map(|(i, _)| i).collect();
    if indices.len() != 1 {
        return Err(bad("book.refreshToc", "exactly one Entire Book TOC must be configured and saved"));
    }
    let chapter = chapters.get(*indices.first().ok_or_else(|| bad("toc", "TOC missing"))?).ok_or_else(|| bad("toc", "TOC missing"))?;
    let toc = chapter.doc.toc.as_ref().ok_or_else(|| bad("toc", "TOC missing"))?;
    let headings = collect(chapters, &toc.entries)?;
    let value = json!({"order":book.documents,"definition":toc,"chapters":chapters.iter().map(|c|json!({"path":document_path(&c.path),"pages":c.doc.page_count(),"chapter":c.doc.chapter_label(),"sections":c.doc.sections,"headings":toc::collect(&c.doc,&toc.entries,c.doc.toc.as_ref().map(|t|t.story))})).collect::<Vec<_>>(),"headings":headings,"text":chapter.doc.story(toc.story).map(|s|&s.text)});
    if serde_json::to_vec(&value).map_err(|e| bad("toc", e.to_string()))?.len() > 4 * 1024 * 1024 {
        return Err(bad("toc", "book freshness state exceeds 4 MiB"));
    }
    Ok(value)
}

pub(super) fn verify_export(s: &Session) -> Result<()> {
    let book = s.book.as_ref().ok_or_else(no_book)?;
    let has = book.documents.iter().any(|p| load(p).is_ok_and(|d| d.toc.as_ref().is_some_and(|t| t.scope == TocScope::EntireBook)))
        || s.documents().iter().any(|d| {
            d.path.as_deref().is_some_and(|p| book.documents.iter().any(|b| document_path(b) == document_path(p)))
                && d.doc.toc.as_ref().is_some_and(|t| t.scope == TocScope::EntireBook)
        });
    if !has {
        return Ok(());
    }
    let (chapters, _) = snapshot(s, false)?;
    for chapter in &chapters {
        if let Some(toc) = chapter.doc.toc.as_ref().filter(|t| t.scope == TocScope::EntireBook)
            && pagenor_compose::compose_story(&chapter.doc, toc.story, &Default::default()).overset_at.is_some()
        {
            return Err(bad("book.exportPdf", "the book TOC does not fit its frames; reserve more space and refresh it before exporting"));
        }
    }
    if book.numbering_needs_update || book.toc_state.as_ref() != Some(&state(book, &chapters)?) {
        return Err(bad("book.exportPdf", "the book TOC is out of date; save and close book chapters, then Refresh Book Table of Contents"));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub(super) fn refresh(_: &mut Session) -> Result<Value> {
    Err(bad("book.refreshToc", "books require a native file system"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn refresh(s: &mut Session) -> Result<Value> {
    let mut book = s.book.clone().ok_or_else(no_book)?;
    let manifest_before = read_bounded(&book.path, 4 << 20)?;
    let (mut chapters, _) = snapshot(s, false)?;
    if chapters.iter().any(|c| !pagenor_doc::file_type::is_native_name(&c.path)) {
        return Err(bad("book.refreshToc", "save read-only import chapters as .pgnr first"));
    }
    let mut stable = false;
    for _ in 0..4 {
        let mut next = 1u32;
        let mut number = 0u32;
        for chapter in &mut chapters {
            start_at(&mut chapter.doc, next);
            number = match chapter.doc.settings.chapter_source {
                pagenor_doc::ChapterSource::Automatic => number.saturating_add(1),
                pagenor_doc::ChapterSource::SameAsPrevious => number.max(1),
                pagenor_doc::ChapterSource::UserDefined => chapter.doc.settings.chapter_number.max(1),
            };
            if number > 99_999 {
                return Err(bad("book.refreshToc", "chapter number exceeds 99999"));
            }
            chapter.doc.settings.chapter_number = number;
            super::super::sync_placeholders(&mut chapter.doc);
            next = next
                .checked_add(u32::try_from(chapter.doc.page_count()).map_err(|_| bad("toc", "too many pages"))?)
                .ok_or_else(|| bad("toc", "too many pages"))?;
        }
        let before = state(&book, &chapters)?;
        let index = chapters
            .iter()
            .position(|c| c.doc.toc.as_ref().is_some_and(|t| t.scope == TocScope::EntireBook))
            .ok_or_else(|| bad("toc", "TOC missing"))?;
        let definition = chapters.get(index).and_then(|c| c.doc.toc.clone()).ok_or_else(|| bad("toc", "TOC missing"))?;
        let found = collect(&chapters, &definition.entries)?;
        let original = chapters.get(index).ok_or_else(|| bad("toc", "TOC missing"))?.doc.clone();
        let mut worker = Session::new();
        worker.prefs = s.prefs.clone();
        worker.add_document(crate::DocState::new(original, None));
        let original = worker.doc()?.doc.clone();
        let uid = worker.doc()?.uid;
        worker.edit(|d, _| toc::fill_found(d, &definition, None, found))?;
        if worker.prefs.smart_text_reflow {
            worker.smart_reflow(Some((uid, &original)));
        }
        let candidate = (*worker.doc()?.doc).clone();
        if pagenor_compose::compose_story(&candidate, definition.story, &Default::default()).overset_at.is_some() {
            return Err(bad("book.refreshToc", "the TOC does not fit its frames; configure reflow or allocate more space"));
        }
        chapters.get_mut(index).ok_or_else(|| bad("toc", "TOC missing"))?.doc = candidate;
        if state(&book, &chapters).is_ok_and(|after| after == before) {
            stable = true;
            break;
        }
    }
    if !stable {
        return Err(bad("book.refreshToc", "book layout did not stabilize in four passes; no files were written"));
    }
    book.numbering_needs_update = false;
    book.toc_state = Some(state(&book, &chapters)?);
    let mut prepared = Vec::new();
    let mut total = 0usize;
    for chapter in &chapters {
        let bytes = super::super::file::to_bytes(&chapter.doc)?;
        total = total.saturating_add(bytes.len());
        if total > 256 * 1024 * 1024 {
            return Err(bad("book.refreshToc", "prepared book exceeds 256 MiB"));
        }
        if read_bounded(&chapter.path, 128 << 20)? != chapter.bytes {
            return Err(bad("book.refreshToc", "a source changed while composing; retry"));
        }
        prepared.push((chapter.path.clone(), bytes));
    }
    if read_bounded(&book.path, 4 << 20)? != manifest_before {
        return Err(bad("book.refreshToc", "the book manifest changed while composing; retry"));
    }
    // All serialization and source checks precede writes. Each replacement is atomic;
    // compensate earlier replacements if a later one fails.
    for (i, (path, bytes)) in prepared.iter().enumerate() {
        if let Err(error) = crate::native_save::write(std::path::Path::new(path), bytes) {
            let rollback = rollback(&chapters, i);
            return Err(EngineError::Other(format!("{error}; {rollback}")));
        }
    }
    if let Err(error) = save(&book) {
        let rollback = rollback(&chapters, chapters.len());
        // The atomic book save retains its original on failure.
        return Err(EngineError::Other(format!("{error}; {rollback}")));
    }
    s.book = Some(book);
    Ok(json!({"updated":true,"chapters":chapters.len()}))
}

#[cfg(not(target_arch = "wasm32"))]
fn rollback(chapters: &[Chapter], count: usize) -> String {
    let mut failed = Vec::new();
    for chapter in chapters.iter().take(count) {
        if let Err(error) = crate::native_save::write(std::path::Path::new(&chapter.path), &chapter.bytes) {
            failed.push(format!("{}: {error}", chapter.path));
        }
    }
    if failed.is_empty() { "earlier files restored".into() } else { format!("could not restore: {}", failed.join("; ")) }
}
