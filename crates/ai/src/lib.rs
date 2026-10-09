//! AI protocol and review utilities, independent of the engine and UI.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
use pagenor_doc::ai::AiDocument;
mod text_edits;
pub use text_edits::{same_layout, text_edits};

/// All model offsets count Unicode scalar values, never UTF-16 units or UTF-8 bytes.
pub fn byte_offset(text: &str, character: usize) -> Option<usize> {
    text.char_indices().map(|(i, _)| i).chain(std::iter::once(text.len())).nth(character)
}
/// Resolve an exact quote inside a captured selection. Never use fuzzy or nearest matches.
/// Explicit offsets remain available to deterministic engine clients.
pub fn quote_offset(text: &str, original: &str, offset: Option<usize>) -> Result<usize, String> {
    if let Some(start) = offset.and_then(|n| byte_offset(text, n))
        && text[start..].starts_with(original)
    {
        return Ok(start);
    }
    if original.is_empty() {
        return Err("Insertion suggestions need an exact position or identical neighboring text in original and replacement.".into());
    }
    let mut matches = text.char_indices().filter_map(|(i, _)| text[i..].starts_with(original).then_some(i));
    let first = matches.next().ok_or("Exact original text was not found. Copy the snapshot text including punctuation, spaces and line breaks.")?;
    if matches.next().is_some() {
        return Err("Original occurs more than once. Include identical surrounding text in original and replacement to make the quote unique; do not guess positions.".into());
    }
    Ok(first)
}
pub fn protected(text: &str) -> bool {
    text.chars().any(|c| ('\u{e000}'..='\u{f8ff}').contains(&c) || c == '\u{fffc}')
}
/// Paragraph-first batching with bounded long paragraphs. Ranges are byte offsets.
pub fn batches(text: &str, limit: usize) -> Vec<std::ops::Range<usize>> {
    let mut out = vec![];
    let mut start = 0;
    while start < text.len() {
        let rest = &text[start..];
        let hard = byte_offset(rest, limit.max(1)).unwrap_or(rest.len());
        let end = if hard < rest.len() { rest[..hard].rfind('\n').map_or(hard, |n| n + 1) } else { hard };
        out.push(start..start + end);
        start += end;
    }
    out
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}
pub fn report(title: &str, data: &AiDocument) -> Result<String, String> {
    data.check().map_err(|e| e.to_string())?;
    if title.len() > 4096 {
        return Err("Report title exceeds the size limit".into());
    }
    let mut html = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>{}</title><style>body{{font:16px system-ui;max-width:1100px;margin:40px auto;padding:20px}}article{{border-top:1px solid #aaa;padding:20px 0}}pre{{white-space:pre-wrap;overflow-wrap:anywhere}}del{{background:#ffe8e8}}ins{{background:#e3f6e5}}small{{color:#555}}</style><h1>{} · Proofreading Report</h1>",
        escape(title),
        escape(title)
    );
    for s in &data.suggestions {
        let piece = format!(
            "<article><h2>#{0} · {1}</h2><small>Story {2} · Page {3} · Original UTF-8 positions {4}–{5}</small><h3>Original Context</h3><pre>{6}</pre><h3>Change</h3><pre><del>{7}</del> → <ins>{8}</ins></pre><p>{9}</p></article>",
            s.id,
            s.status.label(),
            s.story.0,
            escape(s.page_label.as_deref().unwrap_or("Unknown / overset")),
            s.source_start,
            s.source_end,
            escape(&s.context),
            escape(&s.original),
            escape(&s.replacement),
            escape(&s.reason)
        );
        if html.len().saturating_add(piece.len()) > 16 << 20 {
            return Err("HTML report exceeds the 16 MiB limit".into());
        }
        html.push_str(&piece);
    }
    html.push_str("</html>");
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_and_long_paragraph_batches() {
        let text = "😀甲乙\n丙丁戊己庚";
        let parts = batches(text, 3);
        assert_eq!(parts.iter().map(|r| &text[r.clone()]).collect::<String>(), text);
        assert!(parts.iter().all(|r| text[r.clone()].chars().count() <= 3));
        assert_eq!(byte_offset("😀中e\u{301}", 2), Some(7));
        assert_eq!(byte_offset("😀", 2), None);
    }
    #[test]
    fn report_escapes_document_text() {
        let html = report("<script>evil</script>", &AiDocument::default()).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
