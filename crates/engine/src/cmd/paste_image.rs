//! Bounded raster clipboard insertion, including an inline object at the text caret.
use super::{CommandSpec, Result, Session, bad, cmd, has_doc};
use pagenor_doc::{Asset, AssetId, Content, Graphic, Item, ItemId, Selection, Shape, TextSel};
use pagenor_geom::{Affine, Rect, shapes};
use serde_json::{Value, json};
use std::sync::Arc;

const ID: &str = "edit.pasteImage";
pub const MAX_PASTED_PIXELS: u64 = 64_000_000;
pub const IMAGE_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp"];
const MAX_BYTES: usize = 256 << 20;

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "edit.pasteImage",
        "Paste Image",
        [],
        None,
        "{base64, name?: image filename, spread?, at?: [x,y]} → {id,asset,anchored}; embeds a raster; at a text caret inserts one inline object, otherwise fills a selected empty frame or centers a new frame on the page at its stored resolution (one point per pixel if absent), capped to 60% of page width and height",
        has_doc,
        paste
    )]
}

pub fn check_pasted_size(width: u32, height: u32) -> std::result::Result<(), String> {
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PASTED_PIXELS {
        return Err("The clipboard image must have between 1 and 64 million pixels.".into());
    }
    Ok(())
}

pub fn pasted_png(width: u32, height: u32, rgba: Vec<u8>) -> std::result::Result<Vec<u8>, String> {
    check_pasted_size(width, height)?;
    if rgba.len() as u64 != u64::from(width) * u64::from(height) * 4 {
        return Err("The clipboard image has incomplete pixel data.".into());
    }
    pagenor_images::png_from_rgba(width, height, rgba).ok_or_else(|| "The clipboard image could not be stored.".into())
}

fn paste(s: &mut Session, p: &Value) -> Result<Value> {
    let encoded = p.get("base64").and_then(Value::as_str).filter(|v| !v.is_empty()).ok_or_else(|| bad(ID, "give a non-empty base64 image"))?;
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(bad(ID, "clipboard image exceeds 256 MiB"));
    }
    let name = match p.get("name") {
        None | Some(Value::Null) => "Pasted image.png",
        Some(Value::String(name))
            if !name.trim().is_empty() && name.len() <= 255 && !name.chars().any(|c| c.is_control() || c == '/' || c == '\\') =>
        {
            name
        }
        _ => return Err(bad(ID, "name must be an image filename of at most 255 bytes")),
    };
    let ext = name.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase()).unwrap_or_default();
    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(bad(ID, "give a raster image filename"));
    }
    let bytes = super::base64_decode(encoded);
    if bytes.len() > MAX_BYTES {
        return Err(bad(ID, "clipboard image exceeds 256 MiB"));
    }
    let (pw, ph) = pagenor_images::pixel_size(&bytes).ok_or_else(|| bad(ID, "the clipboard does not contain a supported raster image"))?;
    check_pasted_size(pw, ph).map_err(|e| bad(ID, e))?;
    if pagenor_images::decode_clipboard_rgba(&bytes).is_none() {
        return Err(bad(ID, "the clipboard image is damaged or exceeds the decoder memory limit"));
    }
    let st = s.doc()?;
    let text = st.selection.text;
    if let Some(t) = text {
        if t.cell.is_some() {
            return Err(bad(ID, "inline image paste into table cells or footnotes is not supported yet"));
        }
        let story = st.doc.story(t.story).ok_or_else(|| bad(ID, "the text insertion point is no longer available"))?;
        if t.range().end > story.len() || !story.text.is_char_boundary(t.range().start) || !story.text.is_char_boundary(t.range().end) {
            return Err(bad(ID, "the text insertion point is invalid"));
        }
    }
    let spread = match p.get("spread") {
        None | Some(Value::Null) => pagenor_doc::SpreadRef::Doc(0),
        Some(Value::Number(n)) => {
            pagenor_doc::SpreadRef::Doc(n.as_u64().and_then(|v| usize::try_from(v).ok()).ok_or_else(|| bad(ID, "invalid spread"))?)
        }
        Some(value) => serde_json::from_value(value.clone()).map_err(|_| bad(ID, "invalid spread"))?,
    };
    let sp = st.doc.spread(spread).ok_or_else(|| bad(ID, "no such spread"))?;
    let center = match p.get("at") {
        None | Some(Value::Null) => sp.pages.first().map(|pg| pg.bounds().center()).unwrap_or_default(),
        Some(Value::Array(values)) if values.len() == 2 => {
            let x = values
                .first()
                .and_then(Value::as_f64)
                .filter(|v| v.is_finite() && v.abs() <= 10_000_000.0)
                .ok_or_else(|| bad(ID, "at must contain two finite numbers"))?;
            let y = values
                .get(1)
                .and_then(Value::as_f64)
                .filter(|v| v.is_finite() && v.abs() <= 10_000_000.0)
                .ok_or_else(|| bad(ID, "at must contain two finite numbers"))?;
            pagenor_geom::Point::new(x, y)
        }
        _ => return Err(bad(ID, "at must contain two finite numbers")),
    };
    let (nw, nh) = pagenor_images::natural_size(&bytes).unwrap_or((f64::from(pw), f64::from(ph)));
    let page = sp.pages.first();
    let max_width = page.map_or(st.doc.settings.page_width, |p| p.width) * 0.6;
    let max_height = page.map_or(st.doc.settings.page_height, |p| p.height) * 0.6;
    let scale = (max_width / nw).min(max_height / nh).min(1.0);
    if !nw.is_finite() || !nh.is_finite() || nw <= 0.0 || nh <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return Err(bad(ID, "the clipboard image has invalid dimensions"));
    }
    let w = nw * scale;
    let h = nh * scale;
    let target = if text.is_none() {
        match st.selection.items.as_slice() {
            [id] if st.doc.item(*id).is_some_and(|it| {
                it.path.subpaths.iter().any(|p| p.closed && p.anchors.len() > 1)
                    && it.inner_bounds().width() > 0.0
                    && it.inner_bounds().height() > 0.0
                    && matches!(it.content, Content::Unassigned)
            }) =>
            {
                Some(*id)
            }
            _ => None,
        }
    } else {
        None
    };
    if target.and_then(|id| st.doc.item(id)).is_some_and(|it| it.locked || st.doc.layer(it.layer).is_some_and(|l| l.locked)) {
        return Err(bad(ID, "the target frame is locked"));
    }
    if target.is_none() && st.doc.layer(st.active_layer).is_some_and(|l| l.locked) {
        return Err(bad(ID, "the active layer is locked"));
    }
    let layer = st.active_layer;
    let mime = pagenor_images::mime(&bytes).to_string();
    s.edit(|d, sel| {
        let fresh = AssetId(d.alloc());
        let aid = d.add_asset(Asset { id: fresh, name: name.into(), mime, data: Arc::new(bytes), pixels: Some((pw, ph)), ..Default::default() });
        let origin = if text.is_some() { pagenor_geom::Point::ZERO } else { pagenor_geom::Point::new(center.x - w / 2.0, center.y - h / 2.0) };
        let rect = Rect::new(origin.x, origin.y, origin.x + w, origin.y + h);
        let mut graphic = Graphic {
            clip: None,
            asset: aid,
            size: (nw, nh),
            xf: Affine::translate((origin.x, origin.y)) * Affine::scale(scale),
            auto_fit: Default::default(),
            fit_align: 4,
            crop: [0.0; 4],
            wrap: Default::default(),
        };
        let id = if let Some(id) = target {
            let item = d.item_mut(id).ok_or_else(|| bad(ID, "the target frame is no longer available"))?;
            let bounds = item.inner_bounds();
            let k = (bounds.width() / nw).max(bounds.height() / nh);
            graphic.xf =
                Affine::translate((bounds.x0 + (bounds.width() - nw * k) / 2.0, bounds.y0 + (bounds.height() - nh * k) / 2.0)) * Affine::scale(k);
            if let Some(fitting) = item.empty_fitting.take() {
                fitting.apply(&mut graphic, bounds);
            }
            super::file::validate_placement(graphic.size, bounds, graphic.xf, ID)?;
            item.content = Content::Graphic(graphic);
            id
        } else {
            let id = ItemId(d.alloc());
            super::file::validate_placement(graphic.size, rect, graphic.xf, ID)?;
            let mut item = Item::new(id, layer, Shape::Rectangle, shapes::rectangle(rect));
            item.object_style = d.styles.default_graphic_frame.clone();
            item.content = Content::Graphic(graphic);
            if let Some(t) = text {
                let end = super::anchored::anchor_items(d, t.story, t.range(), vec![item], &Default::default())?;
                sel.text = Some(TextSel { anchor: end, focus: end, ..t });
                sel.items.clear();
            } else {
                d.insert_item(spread, item, None)?;
            }
            id
        };
        if text.is_none() {
            *sel = Selection::items(vec![id]);
        }
        Ok(json!({"id":id.0,"asset":aid.0,"anchored":text.is_some()}))
    })
}
