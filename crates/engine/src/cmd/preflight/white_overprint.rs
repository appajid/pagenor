//! White or [Paper] set to overprint (#2401). A paint that lays down no ink knocks out what is
//! beneath it, which is how white shows on press. Set to overprint, it leaves the inks beneath
//! alone instead, so the object is gone from the printed page while the screen, without
//! Overprint Preview, still shows it.
//!
//! The existing overprint settings are a page item's fill, stroke and stroke gap.
//! Text itself has no overprint attribute, so white text knocks out and
//! prints; the frame's Overprint Fill covers the frame's own fill only.

use std::collections::HashSet;
use std::sync::Arc;

use pagenor_color::{Color, ColorType, SwatchValue};
use pagenor_doc::{AnchoredObject, Content, Document, Item, SpreadRef, StrokeType};
use pagenor_geom::Affine;
use pagenor_pdf::{drawn_children, item_exported, layer_exported, shown_parent_items};

use super::Issue;
use crate::Session;

/// The issue's `kind`.
pub(super) const KIND: &str = "whiteOverprint";

/// Swatch tints that refer to tints, followed before giving up (as `swatch::resolve` does).
const MAX_TINT_DEPTH: usize = 8;

/// Whether a paint lays down no ink: [Paper] (whatever colour it is shown in), or a colour or
/// tint that comes out white (C=0 M=0 Y=0 K=0, a 0% tint, RGB white). A spot colour prints on
/// its own plate whatever colour stands in for it (a white ink, a varnish), so only a 0% tint of
/// it is white. [None] and gradients are not white.
fn prints_no_ink(d: &Document, name: &str, tint: f32) -> bool {
    const INK: f32 = 0.001;
    let (mut base, mut t) = (name, tint);
    for _ in 0..=MAX_TINT_DEPTH {
        match d.swatches.iter().find(|s| s.name == base).map(|s| &s.value) {
            Some(SwatchValue::Paper { .. }) => return true,
            Some(SwatchValue::Tint { base: b, tint: bt }) => (base, t) = (b, t * bt),
            // A NaN tint compares false: not white.
            Some(SwatchValue::Color { color_type: ColorType::Spot, .. }) => return t <= INK,
            Some(SwatchValue::Color { .. } | SwatchValue::Registration) => break,
            Some(SwatchValue::None | SwatchValue::Gradient { .. }) | None => return false,
        }
    }
    // A NaN component compares false: not white.
    match d.resolve_color(name, tint) {
        Some(Color::Cmyk { c, m, y, k }) => [c, m, y, k].iter().all(|v| *v <= INK),
        Some(Color::Gray { k }) => k <= INK,
        Some(Color::Rgb { r, g, b }) => [r, g, b].iter().all(|v| *v >= 1.0 - INK),
        None => false,
    }
}

/// "fill", "fill and stroke", "fill, stroke and stroke gap".
fn list(parts: &[&str]) -> String {
    match parts.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        Some((last, _)) => (*last).to_string(),
        None => String::new(),
    }
}

/// The warning for the overprinting paints of one item; none when no paint is affected.
pub(super) fn issue(parts: &[&str], item: Option<u64>, page: Option<usize>) -> Option<Issue> {
    (!parts.is_empty()).then(|| Issue {
        severity: "warning",
        kind: KIND,
        message: format!("White or [Paper] {} set to overprint may disappear when overprint is honored", list(parts)),
        item,
        page,
    })
}

/// A page item to look at: a spread's or a parent page's (or inside one), or an anchored object.
enum Node {
    Item(Arc<Item>),
    Anchored(Arc<AnchoredObject>),
}

impl Node {
    fn item(&self) -> &Item {
        match self {
            Node::Item(it) => it,
            Node::Anchored(o) => &o.item,
        }
    }
}

/// Where an item is drawn: on spread `si` under its parent's transform (the page is the one it
/// sits on), or on a page given (a parent page's item, an anchored object).
#[derive(Clone, Copy)]
enum On {
    Spread(usize, Affine),
    Page(Option<usize>),
}

/// The page items PDF export draws, as it draws them: on visible, printable layers, without the
/// hidden and nonprinting ones and everything inside them, the parent pages' items on the pages
/// that show them, and anchored objects in the frames that place them. Returns the white
/// overprint issues of those items.
pub(super) fn printed(s: &Session, d: &Document) -> Vec<Issue> {
    let mut issues = Vec::new();
    // A parent page's item shows on every page based on it: it is reported once, on the first.
    // This also stops an anchored object that would contain its own frame.
    let mut seen = HashSet::new();
    for (si, sp) in d.spreads.iter().enumerate() {
        let first = d.first_page_of_spread(si);
        // With the page the PDF composes the item's text for (a parent page's, per page).
        let mut stack: Vec<(Node, On, Option<usize>)> = Vec::new();
        for pi in (0..sp.pages.len()).rev() {
            let page = Some(first + pi);
            stack.extend(shown_parent_items(d, si, pi).into_iter().rev().map(|(_, it)| (Node::Item(it.clone()), On::Page(page), page)));
        }
        let exported = sp.items.iter().rev().filter(|it| d.layer(it.layer).is_some_and(layer_exported));
        stack.extend(exported.map(|it| (Node::Item(it.clone()), On::Spread(si, Affine::IDENTITY), None)));
        while let Some((node, on, composed)) = stack.pop() {
            let it = node.item();
            if !item_exported(it) || !seen.insert(it.id) {
                continue;
            }
            let (page, inside) = match on {
                On::Spread(si, parent) => (placed_page(d, SpreadRef::Doc(si), it, parent), On::Spread(si, parent * it.xf)),
                On::Page(page) => (page, on),
            };
            issues.extend(item_issue(d, it, page));
            let children: Vec<_> = drawn_children(it).map(|c| (Node::Item(c.clone()), inside, composed)).collect();
            if let Content::Text(tf) = &it.content {
                let cs = s.cache.get(d, tf.story, composed);
                // The frame's anchored objects, and those in its footnotes and table cells.
                let mut pending: Vec<_> = cs.frame(it.id).into_iter().collect();
                while let Some(frame) = pending.pop() {
                    stack.extend(frame.objects.iter().map(|o| (Node::Anchored(o.object.clone()), On::Page(page), composed)));
                    pending.extend(frame.notes.iter().filter_map(|n| n.text.frames.first()));
                    pending.extend(frame.tables.iter().flat_map(|t| &t.cells).filter_map(|c| c.text.frames.first()));
                }
            }
            stack.extend(children.into_iter().rev());
        }
    }
    issues
}

/// A page item's fill, stroke or stroke gap that is white and set to overprint.
fn item_issue(d: &Document, it: &Item, page: Option<usize>) -> Option<Issue> {
    if it.nonprinting {
        return None;
    }
    let mut parts = Vec::new();
    if it.fill.overprint && !it.fill.is_none() && prints_no_ink(d, &it.fill.swatch, it.fill.tint) {
        parts.push("fill");
    }
    let st = &it.stroke;
    if st.overprint && !st.is_none() && prints_no_ink(d, &st.swatch, st.tint) {
        parts.push("stroke");
    }
    // The gap colour shows between dashes, dots and stripes only.
    if st.gap_overprint && !st.is_none() && !matches!(d.stroke_kind(&st.kind), StrokeType::Solid) && prints_no_ink(d, &st.gap_swatch, st.gap_tint) {
        parts.push("stroke gap");
    }
    issue(&parts, Some(it.id.0), page)
}

fn placed_page(d: &Document, sr: SpreadRef, it: &Item, parent: Affine) -> Option<usize> {
    let SpreadRef::Doc(si) = sr else { return None };
    let sp = d.spreads.get(si)?;
    Some(d.first_page_of_spread(si) + sp.page_at_x(parent.transform_rect_bbox(it.bounds()).center().x).unwrap_or(0))
}
#[cfg(test)]
mod tests;
