//! PowerPoint export (File › Export › PowerPoint).
//!
//! Every exported page becomes a slide of native, editable objects: frames become shapes (preset
//! rectangles and ellipses, custom geometry otherwise) with their fills, strokes and effects,
//! placed images become pictures cropped like their frames, and text frames become text boxes.
//!
//! Text keeps Pagenor's composition by default ([`LineBreaks::Keep`]): each composed line ends
//! in a line break, justified word spacing and tracking become character spacing, and leading
//! becomes percentage line spacing (PowerPoint rounds exact point spacing to whole points; its
//! 100% line is 1.2 em for every font, so a percentage is exact). A threaded story is split at its
//! frames, and the lines a text wrap pushes aside become boxes of their own, since PowerPoint has
//! neither threads nor wraps. [`LineBreaks::Reflow`] instead writes one box per frame with native
//! columns and lets PowerPoint break the lines.
//!
//! Parent-page items are copied onto each slide; layers are flattened in stacking order. What has
//! no PowerPoint equivalent (blend modes, type on a path, some effects) is approximated or
//! dropped and reported in [`ExportReport::warnings`]; [`needs_raster`] names the objects a caller
//! should rasterise first.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

mod fonts;
mod package;
mod slide;
mod table;
mod text;
mod xml;

#[cfg(test)]
mod tests;

use pagenor_compose::Cache;
use pagenor_doc::Document;

pub use slide::needs_raster;

/// How text frames become text boxes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineBreaks {
    /// Keep Pagenor's line breaks, word spacing and leading (looks the same; edits don't
    /// re-break lines).
    #[default]
    Keep,
    /// One box per frame with native columns; PowerPoint breaks the lines (edits reflow; lines,
    /// hyphenation and wraps differ).
    Reflow,
}

#[derive(Clone, Debug, Default)]
pub struct PptxOptions {
    /// Document pages to export (0-based positions); all when `None`.
    pub pages: Option<Vec<usize>>,
    pub line_breaks: LineBreaks,
    pub title: Option<String>,
    pub author: Option<String>,
    /// Creation time (Unix seconds) for the document properties; none when `None`.
    pub created: Option<i64>,
    /// Embed the TrueType fonts the text uses (when their licences allow it), so the slides
    /// look the same where they aren't installed.
    pub embed_fonts: bool,
}

/// The exported presentation and what could not be represented exactly.
#[derive(Clone, Debug, Default)]
pub struct ExportReport {
    pub bytes: Vec<u8>,
    pub slides: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PptxError {
    #[error("the document has no pages to export")]
    NoPages,
    #[error("page {0} does not exist")]
    BadPage(usize),
    #[error("could not write the presentation: {0}")]
    Write(String),
    #[error("PowerPoint export exceeds its limits: {0}")]
    Limit(&'static str),
}

pub type Result<T> = std::result::Result<T, PptxError>;

/// Export `doc` as a PowerPoint presentation.
pub fn export_pptx(doc: &Document, cache: &Cache, opts: &PptxOptions) -> Result<ExportReport> {
    validate_document(doc)?;
    let count = doc.page_count();
    if opts.pages.as_ref().map_or(count, Vec::len) > MAX_PAGES {
        return Err(PptxError::Limit("slide count"));
    }
    let pages: Vec<usize> = match &opts.pages {
        Some(v) => v.clone(),
        None => (0..count).collect(),
    };
    if pages.is_empty() {
        return Err(PptxError::NoPages);
    }
    if let Some(bad) = pages.iter().find(|p| **p >= count) {
        return Err(PptxError::BadPage(bad.saturating_add(1)));
    }
    let mut ex = slide::Exporter::new(doc, cache, opts);
    let mut slides = Vec::with_capacity(pages.len());
    let mut output = 0usize;
    for abs in &pages {
        let slide = ex.slide(*abs)?;
        output = output.checked_add(slide.shapes.len()).ok_or(PptxError::Limit("slide XML"))?;
        let media = ex.media.iter().try_fold(0usize, |n, m| n.checked_add(m.bytes.len())).ok_or(PptxError::Limit("media"))?;
        if output > MAX_OUTPUT || media > MAX_OUTPUT.saturating_sub(output) {
            return Err(PptxError::Limit("slide XML and media"));
        }
        slides.push(slide);
    }
    let size = slides.first().map(|s| s.size).ok_or(PptxError::NoPages)?;
    if slides.iter().any(|s| (s.size.0 - size.0).abs() > 0.5 || (s.size.1 - size.1).abs() > 0.5) {
        ex.warn("pages of different sizes: PowerPoint has one slide size, so every slide uses the first page's");
    }
    let fonts = if opts.embed_fonts {
        let (fonts, skipped) = ex.fonts.embedded();
        if !skipped.is_empty() {
            ex.warn(format!("fonts not embedded: {}", skipped.join(", ")));
        }
        fonts
    } else {
        let mut names: Vec<String> = ex.fonts.used.iter().map(|(f, _)| f.family.clone()).collect();
        names.dedup();
        if !names.is_empty() {
            ex.warn(format!("fonts are not embedded; the slides need {} installed to look the same", names.join(", ")));
        }
        Vec::new()
    };
    let title = opts.title.clone().unwrap_or_else(|| doc.title.clone());
    let bytes = package::write(&package::Package {
        size,
        slides: &slides,
        media: &ex.media,
        fonts: &fonts,
        title: &title,
        author: opts.author.as_deref().unwrap_or(""),
        created: opts.created,
    })?;
    Ok(ExportReport { bytes, slides: slides.len(), warnings: ex.warnings })
}

const MAX_PAGES: usize = 4096;
pub(crate) const MAX_OUTPUT: usize = 512 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;

/// Check input structure before recursive composition/rasterization or output allocation.
/// Asset bytes are already shared; JSON is streamed into a counting sink, without a copy.
pub fn validate_document(doc: &Document) -> Result<()> {
    enum Node<'a> {
        Item(&'a pagenor_doc::Item),
        Story(&'a pagenor_doc::Story),
    }
    let mut queue = Vec::new();
    let mut nodes = 0usize;
    let mut push = |queue: &mut Vec<_>, node, depth| -> Result<()> {
        nodes = nodes.checked_add(1).ok_or(PptxError::Limit("document nodes"))?;
        if nodes > MAX_NODES || depth > MAX_DEPTH {
            return Err(PptxError::Limit("document nodes or nesting"));
        }
        queue.push((node, depth));
        Ok(())
    };
    if doc.page_count() > MAX_PAGES {
        return Err(PptxError::Limit("document pages"));
    }
    for sp in doc.spreads.iter().chain(&doc.parents) {
        for it in &sp.items {
            push(&mut queue, Node::Item(it), 0)?;
        }
        for pg in &sp.pages {
            if ![pg.width, pg.height, pg.x].iter().all(|n| n.is_finite()) || pg.width <= 0.0 || pg.height <= 0.0 {
                return Err(PptxError::Limit("invalid page geometry"));
            }
        }
    }
    for story in doc.stories.values() {
        push(&mut queue, Node::Story(story), 0)?;
    }
    while let Some((node, depth)) = queue.pop() {
        match node {
            Node::Item(it) => {
                for child in it.children() {
                    push(&mut queue, Node::Item(child), depth + 1)?;
                }
            }
            Node::Story(st) => {
                for n in st.notes.iter().chain(&st.endnotes) {
                    push(&mut queue, Node::Story(&n.text), depth + 1)?;
                }
                for table in st.tables.values() {
                    for cell in &table.cells {
                        push(&mut queue, Node::Story(&cell.text), depth + 1)?;
                    }
                }
                for obj in &st.objects {
                    push(&mut queue, Node::Item(&obj.item), depth + 1)?;
                }
            }
        }
    }
    let bytes = doc.assets.values().try_fold(0usize, |n, a| n.checked_add(a.data.len())).ok_or(PptxError::Limit("source assets"))?;
    if bytes > MAX_OUTPUT {
        return Err(PptxError::Limit("source assets"));
    }
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.checked_add(bytes.len()).ok_or_else(|| std::io::Error::other("source document overflow"))?;
            if self.0 > 64 * 1024 * 1024 {
                return Err(std::io::Error::other("source document limit"));
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0), doc).map_err(|_| PptxError::Limit("source document JSON"))?;
    Ok(())
}
