//! The document canvas: pasteboard, spreads (rendered by `pagenor-render`), guides, frame edges,
//! selection, text ports and threads, the text caret, rulers, and pointer/keyboard input.

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use pagenor_compose as compose;
use pagenor_doc::{Content, Document, Item, PageSide, SpreadRef};
use pagenor_engine::doc::Selection;
use pagenor_geom::{Affine, Point, Rect as DRect, Vec2 as DVec2};
use pagenor_tools::{CanvasLayout, Cursor, Mods, Overlay, PointerEvent, PointerKind, ToolKey};
use serde_json::{Value, json};

use crate::{DesignApp, ScreenMode, View, theme::Tokens};

pub const RULER: f32 = 15.0;
/// Width of the view's scrollbars.
pub const SCROLLBAR: f32 = 16.0;
/// Shortest scrollbar thumb.
const MIN_THUMB: f32 = 18.0;

/// Canvas ↔ screen transform.
#[derive(Clone, Copy, Debug)]
pub struct Xf {
    /// Top left of the (unrotated) view rectangle.
    pub min: Pos2,
    pub origin: Point,
    pub zoom: f64,
    /// Quarter turns clockwise about `pivot` (the view centre).
    pub rot: u8,
    pub pivot: Pos2,
}

/// Rotate `p` by `q` quarter turns clockwise about `c` (screen space, y down).
fn quarter(p: Pos2, c: Pos2, q: u8) -> Pos2 {
    let (dx, dy) = (p.x - c.x, p.y - c.y);
    let (x, y) = match q % 4 {
        0 => (dx, dy),
        1 => (-dy, dx),
        2 => (-dx, -dy),
        _ => (dy, -dx),
    };
    pos2(c.x + x, c.y + y)
}

/// The unrotated view rectangle for a screen rectangle and a rotation (width and height swap
/// for quarter turns).
pub fn view_rect(screen: Rect, rot: u8) -> Rect {
    if rot % 2 == 1 { Rect::from_center_size(screen.center(), vec2(screen.height(), screen.width())) } else { screen }
}

impl Xf {
    pub fn new(rect: Rect, v: &View) -> Self {
        Xf { min: rect.min, origin: v.origin, zoom: v.zoom, rot: v.rotation % 4, pivot: rect.center() }
    }
    pub fn to_screen(&self, p: Point) -> Pos2 {
        let s = pos2(self.min.x + ((p.x - self.origin.x) * self.zoom) as f32, self.min.y + ((p.y - self.origin.y) * self.zoom) as f32);
        if self.rot == 0 { s } else { quarter(s, self.pivot, self.rot) }
    }
    /// A screen position in the unrotated view.
    pub fn unrotate(&self, p: Pos2) -> Pos2 {
        if self.rot == 0 { p } else { quarter(p, self.pivot, 4 - self.rot) }
    }
    /// A screen-space movement in the unrotated view.
    pub fn unrotate_delta(&self, d: egui::Vec2) -> egui::Vec2 {
        self.unrotate(self.pivot + d) - self.pivot
    }
    pub fn to_canvas(&self, p: Pos2) -> Point {
        let p = self.unrotate(p);
        Point::new(self.origin.x + (p.x - self.min.x) as f64 / self.zoom, self.origin.y + (p.y - self.min.y) as f64 / self.zoom)
    }
    pub fn rect(&self, r: DRect) -> Rect {
        Rect::from_two_pos(self.to_screen(Point::new(r.x0, r.y0)), self.to_screen(Point::new(r.x1, r.y1)))
    }
    /// Canvas → screen affine (for vello rendering: pixels = points × ppp).
    pub fn affine(&self, ppp: f64, area_min: Pos2) -> Affine {
        let unrotated = Affine::translate((self.min.x as f64, self.min.y as f64))
            * Affine::scale(self.zoom)
            * Affine::translate((-self.origin.x, -self.origin.y));
        let (cx, cy) = (self.pivot.x as f64, self.pivot.y as f64);
        let rot = Affine::translate((cx, cy)) * Affine::rotate(self.rot as f64 * std::f64::consts::FRAC_PI_2) * Affine::translate((-cx, -cy));
        Affine::scale(ppp) * Affine::translate((-(area_min.x as f64), -(area_min.y as f64))) * rot * unrotated
    }
}

fn c32(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Fit the current spread (or page) in the canvas.
pub fn fit(app: &mut DesignApp, rect: Rect, what: &str) {
    let Some(st) = app.session.active() else { return };
    let layout = CanvasLayout::new(&st.doc, st.editing_parents);
    let cur = current_slot(app, &layout).unwrap_or(0);
    let Some(slot) = layout.slots.get(cur) else { return };
    let mut b = slot.bounds;
    if what == "page"
        && let Some(sp) = st.doc.spread(slot.spread)
    {
        let center = app.view().map(|v| v.origin.x + rect.width() as f64 / 2.0 / v.zoom).unwrap_or(0.0);
        let c = slot.to_spread(Point::new(center, slot.bounds.center().y));
        let pi = sp.page_at_x(c.x).unwrap_or(0);
        b = slot.xf.transform_rect_bbox(sp.pages[pi].bounds());
    }
    if what == "all" {
        b = layout.slots.iter().map(|s| s.bounds).reduce(|a, b| a.union(b)).unwrap_or(b);
    }
    if what == "selection" {
        // Fit Selection in Window: the selected objects (on their spread's slot).
        let sel: Vec<pagenor_geom::Rect> = st
            .selection
            .items
            .iter()
            .filter_map(|id| {
                let loc = st.doc.find(*id)?;
                let slot = layout.slots.iter().find(|s| s.spread == loc.spread)?;
                Some(slot.xf.transform_rect_bbox(st.doc.item(*id)?.bounds()))
            })
            .collect();
        let Some(u) = sel.into_iter().reduce(|a, b| a.union(b)) else { return };
        b = u.inflate(u.width().max(u.height()) * 0.05 + 1.0, u.width().max(u.height()) * 0.05 + 1.0);
    }
    let pad = 40.0;
    let zoom = ((rect.width() as f64 - 2.0 * pad) / b.width()).min((rect.height() as f64 - 2.0 * pad) / b.height()).clamp(0.05, 40.0);
    let origin = Point::new(b.center().x - rect.width() as f64 / 2.0 / zoom, b.center().y - rect.height() as f64 / 2.0 / zoom);
    if let Some(v) = app.view_mut() {
        *v = View { zoom, origin, fitted: true, rotation: v.rotation, ruler_origin: v.ruler_origin };
    }
}

/// Index of the slot nearest the view centre.
pub fn current_slot(app: &DesignApp, layout: &CanvasLayout) -> Option<usize> {
    let v = app.view()?;
    let r = app.canvas_rect?;
    let c = Point::new(v.origin.x + r.width() as f64 / 2.0 / v.zoom, v.origin.y + r.height() as f64 / 2.0 / v.zoom);
    layout
        .slots
        .iter()
        .enumerate()
        .min_by(|a, b| {
            let da = (a.1.bounds.center().y - c.y).abs();
            let db = (b.1.bounds.center().y - c.y).abs();
            da.total_cmp(&db)
        })
        .map(|(i, _)| i)
}

/// The spread under the view centre (a parent spread while parents are edited).
pub fn current_spread(app: &DesignApp) -> Option<SpreadRef> {
    let st = app.session.active()?;
    let layout = CanvasLayout::new(&st.doc, st.editing_parents);
    Some(layout.slots.get(current_slot(app, &layout)?)?.spread)
}

/// Absolute page index under the view centre.
pub fn current_page(app: &DesignApp) -> Option<usize> {
    let st = app.session.active()?;
    let layout = CanvasLayout::new(&st.doc, st.editing_parents);
    let i = current_slot(app, &layout)?;
    let slot = layout.slots.get(i)?;
    let SpreadRef::Doc(si) = slot.spread else { return Some(0) };
    let v = app.view()?;
    let r = app.canvas_rect?;
    let c = slot.to_spread(Point::new(v.origin.x + r.width() as f64 / 2.0 / v.zoom, slot.bounds.center().y));
    let pi = st.doc.spreads[si].page_at_x(c.x).unwrap_or(0);
    Some(st.doc.first_page_of_spread(si) + pi)
}

/// Spread and page under the view centre, including the master page being edited.
pub fn current_spread_page(app: &DesignApp) -> Option<(SpreadRef, usize)> {
    let st = app.session.active()?;
    let layout = CanvasLayout::new(&st.doc, st.editing_parents);
    let slot = layout.slots.get(current_slot(app, &layout)?)?;
    let v = app.view()?;
    let r = app.canvas_rect?;
    let c = slot.to_spread(Point::new(v.origin.x + r.width() as f64 / 2.0 / v.zoom, slot.bounds.center().y));
    Some((slot.spread, st.doc.spread(slot.spread)?.page_at_x(c.x).unwrap_or(0)))
}

/// Master spread and side under the view centre.
pub fn current_master_page(app: &DesignApp) -> Option<(usize, usize)> {
    let st = app.session.active()?;
    let SpreadRef::Parent(index) = current_spread(app)? else { return None };
    let layout = CanvasLayout::new(&st.doc, true);
    let slot = layout.slot(SpreadRef::Parent(index))?;
    let v = app.view()?;
    let rect = app.canvas_rect?;
    let c = slot.to_spread(Point::new(v.origin.x + rect.width() as f64 / 2.0 / v.zoom, v.origin.y + rect.height() as f64 / 2.0 / v.zoom));
    Some((index, st.doc.parents.get(index)?.page_at_x(c.x)?))
}

/// Scroll so absolute page `abs` is centred (keeps zoom).
pub fn go_to_page(app: &mut DesignApp, abs: usize) {
    if app.session.active().is_some_and(|st| st.editing_parents) {
        let _ = app.run("layout.parents.edit", json!({"on": false}));
    }
    let Some(st) = app.session.active() else { return };
    let layout = CanvasLayout::new(&st.doc, false);
    let Some(pr) = layout.page_rect(&st.doc, abs) else { return };
    let Some(rect) = app.canvas_rect else { return };
    if let Some(v) = app.view_mut() {
        v.origin = Point::new(pr.center().x - rect.width() as f64 / 2.0 / v.zoom, pr.center().y - rect.height() as f64 / 2.0 / v.zoom);
    }
}

/// Navigate to the selected master side, preserving the document's own view.
pub fn go_to_master(app: &mut DesignApp, index: usize, page: Option<usize>) -> Result<(), String> {
    let st = app.session.active().ok_or("no document open")?;
    let sp = st.doc.parents.get(index).ok_or("no such master")?;
    if page.is_some_and(|i| sp.pages.get(i).is_none()) {
        return Err("no such master page".into());
    }
    app.run("layout.parents.edit", json!({"on": true}))?;
    let st = app.session.active().ok_or("no document open")?;
    let layout = CanvasLayout::new(&st.doc, true);
    let slot = layout.slot(SpreadRef::Parent(index)).ok_or("no such master")?;
    let bounds = match page {
        Some(i) => slot.xf.transform_rect_bbox(st.doc.parents[index].pages[i].bounds()),
        None => slot.bounds,
    };
    let rect = app.canvas_rect.unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)));
    if let Some(v) = app.view_mut() {
        v.zoom =
            ((rect.width() as f64 - 80.0) / bounds.width().max(1.0)).min((rect.height() as f64 - 80.0) / bounds.height().max(1.0)).clamp(0.05, 40.0);
        v.origin = Point::new(bounds.center().x - rect.width() as f64 / 2.0 / v.zoom, bounds.center().y - rect.height() as f64 / 2.0 / v.zoom);
        v.fitted = true;
    }
    Ok(())
}

pub fn zoom_at(app: &mut DesignApp, screen: Pos2, factor: f64) {
    let Some(rect) = app.canvas_rect else { return };
    let Some(v) = app.view_mut() else { return };
    let xf = Xf::new(rect, v);
    let c = xf.to_canvas(screen);
    let screen = xf.unrotate(screen);
    let nz = (v.zoom * factor).clamp(0.05, 40.0);
    v.zoom = nz;
    v.origin = Point::new(c.x - (screen.x - rect.min.x) as f64 / nz, c.y - (screen.y - rect.min.y) as f64 / nz);
}

pub fn set_zoom(app: &mut DesignApp, z: f64) {
    let Some(rect) = app.canvas_rect else { return };
    let cur = app.view().map(|v| v.zoom).unwrap_or(1.0);
    zoom_at(app, rect.center(), z / cur);
}

/// Whether the wheel motion now arriving was made with Alt down. As egui does for Ctrl/Cmd, a
/// motion keeps its key to its end: the wheel is smoothed over a few frames, and a trackpad swipe
/// coasts on after the key is let go.
fn wheel_alt(ui: &egui::Ui) -> bool {
    let id = egui::Id::new("canvas_wheel_alt");
    // Alt, and whether a trackpad swipe is under way.
    let (mut alt, mut swipe): (bool, bool) = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    ui.input(|i| {
        for e in &i.events {
            if let egui::Event::MouseWheel { phase, modifiers, .. } = e {
                (alt, swipe) = match phase {
                    egui::TouchPhase::Start => (modifiers.alt, true),
                    egui::TouchPhase::Move => (modifiers.alt || (swipe && alt), swipe),
                    egui::TouchPhase::End | egui::TouchPhase::Cancel => (false, false),
                };
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(id, (alt, swipe)));
    alt
}

pub fn apply_view_request(app: &mut DesignApp, p: &Value) {
    // Power Zoom (Hand tool, Alt-press): zoom out to the spread, aim a view-sized rectangle, zoom
    // back in where it was released.
    if let Some(phase) = p.get("powerZoom").and_then(Value::as_str) {
        let at = p.get("at").and_then(Value::as_array).map(|a| Point::new(a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0)));
        match phase {
            "start" => {
                if let (Some(v), Some(rect)) = (app.view().copied(), app.canvas_rect) {
                    let centre = Xf::new(rect, &v).to_canvas(rect.center());
                    app.power_zoom = Some((v.zoom, at.unwrap_or(centre)));
                    fit(app, rect, "spread");
                }
            }
            "move" => {
                if let (Some(pz), Some(a)) = (&mut app.power_zoom, at) {
                    pz.1 = a;
                }
            }
            _ => {
                if let (Some((z, c)), Some(rect)) = (app.power_zoom.take(), app.canvas_rect)
                    && let Some(v) = app.view_mut()
                {
                    let c = at.unwrap_or(c);
                    v.zoom = z;
                    v.origin = Point::new(c.x - rect.width() as f64 / 2.0 / z, c.y - rect.height() as f64 / 2.0 / z);
                }
            }
        }
    }
    if let Some(d) = p.get("pan").and_then(Value::as_array) {
        let (dx, dy) = (d.first().and_then(Value::as_f64).unwrap_or(0.0), d.get(1).and_then(Value::as_f64).unwrap_or(0.0));
        if let Some(v) = app.view_mut() {
            v.origin = Point::new(v.origin.x - dx / v.zoom, v.origin.y - dy / v.zoom);
        }
    }
    if let (Some(a), Some(rect)) = (p.get("zoomAt").and_then(Value::as_array), app.canvas_rect)
        && let Some(v) = app.view()
    {
        let c = Point::new(a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0));
        let xf = Xf::new(rect, v);
        let s = xf.to_screen(c);
        zoom_at(app, s, p.get("factor").and_then(Value::as_f64).unwrap_or(2.0));
    }
}

fn mods(i: &egui::InputState, space: bool) -> Mods {
    Mods { shift: i.modifiers.shift, alt: i.modifiers.alt, cmd: i.modifiers.command, ctrl: i.modifiers.ctrl, space }
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let outer = ui.available_rect_before_wrap();
    let rot = app.view().map_or(0, |v| v.rotation % 4);
    let presenting = app.ui.screen_mode == ScreenMode::Presentation;
    // Scrollbars sit outside the view (right and bottom); rulers and the view share the rest.
    let bars = !presenting && app.session.active().is_some();
    let full = if bars { Rect::from_min_max(outer.min, outer.max - vec2(SCROLLBAR, SCROLLBAR)) } else { outer };
    // Rulers measure the unrotated layout, so they hide while the view is rotated.
    let rulers = app.ui.rulers && !presenting && rot == 0;
    let screen = if rulers { Rect::from_min_max(full.min + vec2(RULER, RULER), full.max) } else { full };
    let rect = view_rect(screen, rot);
    app.canvas_rect = Some(rect);
    let resp = ui.allocate_rect(full, Sense::click_and_drag());
    if !app.view().is_some_and(|v| v.fitted) {
        fit(app, rect, "spread");
    }
    handle_input(app, ui, &resp, rect);
    if bars {
        scrollbars(app, ui, outer, screen, rect, &t);
    }
    let inline = app.inline_composition();
    let Some(st) = app.session.active() else { return };
    let Some(&v) = app.view() else { return };
    let xf = Xf::new(rect, &v);
    let doc = inline.map_or_else(|| st.doc.clone(), |p| p.doc);
    let layout = CanvasLayout::new(&doc, st.editing_parents);
    let painter = ui.painter_at(screen);
    let preview = matches!(app.ui.screen_mode, ScreenMode::Preview | ScreenMode::Presentation);
    let bg = if app.ui.screen_mode == ScreenMode::Presentation { Color32::BLACK } else { t.pasteboard };
    painter.rect_filled(screen, 0.0, bg);
    // Page shadow: a hard 1.5 pt black offset on the right and bottom (InDesign 2026).
    for slot in &layout.slots {
        let r = xf.rect(slot.bounds);
        painter.rect_filled(r.translate(vec2(1.5, 1.5)), 0.0, Color32::BLACK);
    }
    // Rendered content.
    render_texture(app, ui.ctx(), rect, &xf, &layout, &doc, preview);
    if let (Some(tex), Some(sh)) = (&app.canvas.texture, app.canvas.shown) {
        // Place the texture where its region is in the current view (shifted while panning,
        // scaled while a zoom re-render is pending).
        // The texture's corners in canvas space, placed through the view (rotation included).
        let (w, h) = (sh.size.0 as f64 / sh.zoom, sh.size.1 as f64 / sh.zoom);
        let o = sh.origin;
        let corners = [
            (o, pos2(0.0, 0.0)),
            (Point::new(o.x + w, o.y), pos2(1.0, 0.0)),
            (Point::new(o.x + w, o.y + h), pos2(1.0, 1.0)),
            (Point::new(o.x, o.y + h), pos2(0.0, 1.0)),
        ];
        let mut mesh = egui::Mesh::with_texture(tex.id());
        for (c, uv) in corners {
            mesh.vertices.push(egui::epaint::Vertex { pos: xf.to_screen(c), uv, color: Color32::WHITE });
        }
        mesh.indices.extend([0, 1, 2, 0, 2, 3]);
        if preview {
            // Preview: only page areas (trim) show content.
            for slot in &layout.slots {
                let r = xf.rect(slot.bounds);
                painter.with_clip_rect(r.intersect(screen)).add(egui::Shape::mesh(mesh.clone()));
            }
        } else {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
    let hair = 1.0 / ui.ctx().pixels_per_point();
    for slot in &layout.slots {
        painter.rect_stroke(xf.rect(slot.bounds), 0.0, Stroke::new(hair, Color32::from_rgb(0x4a, 0x4a, 0x4a)), StrokeKind::Outside);
    }
    if !preview {
        draw_guides(app, &painter, &xf, &doc, &layout, &t);
        if let Some(g) = ui.data(|d| d.get_temp::<GuideDrag>(egui::Id::new(GUIDE_DRAG)))
            && let Some(p) = g.at
        {
            let col = Color32::from_rgb(74, 227, 255);
            let (a, b) = match g.orientation {
                pagenor_doc::Orientation::Horizontal => (pos2(rect.min.x, p.y), pos2(rect.max.x, p.y)),
                pagenor_doc::Orientation::Vertical => (pos2(p.x, rect.min.y), pos2(p.x, rect.max.y)),
            };
            painter.line_segment([a, b], Stroke::new(1.0, col));
        }
        draw_frames(app, &painter, &xf, &doc, &layout);
        if app.ui.hidden_characters {
            draw_hidden_characters(app, &painter, &xf, &doc, &layout);
        }
        if app.ui.flattener_preview {
            draw_flattener_preview(&painter, &xf, &doc, &layout);
        }
        if app.ui.dynamic_spelling && !preview {
            draw_dynamic_spelling(app, ui.ctx(), &painter, &xf, &doc, &layout);
        }
    }
    let sel_rect = draw_selection(app, &painter, &xf, &doc, &layout);
    publish_canvas_ime(app, ui, &resp, &xf, &doc, &layout);
    if !preview {
        // Content grabbers: the selected frame's and the hovered one's (Selection tool).
        let hover = ui.ctx().pointer_hover_pos().filter(|p| screen.contains(*p)).map(|p| xf.to_canvas(p));
        let vi = app.view_info();
        for c in app.session.content_grabbers(hover, vi) {
            let c = xf.to_screen(c);
            painter.circle_stroke(c, 8.0, Stroke::new(1.5, Color32::from_white_alpha(170)));
            painter.circle_stroke(c, 4.0, Stroke::new(1.5, Color32::from_white_alpha(170)));
        }
    }
    draw_tool_overlays(app, &painter, &xf);
    if rulers {
        draw_rulers(app, ui, full, rect, &xf, &doc, &layout, &t);
        ruler_units_menus(app, &resp, full, rect, &doc);
    }
    context_menu(app, ui, &resp);
    if let Some(r) = sel_rect
        && !preview
        && r.intersects(screen)
    {
        crate::taskbar::show(app, &ui.ctx().clone(), r, screen);
    }
    // Cursor.
    if let Some(p) = resp.hover_pos().filter(|p| screen.contains(*p)) {
        let wants_text = app.session.wants_text();
        let space = ui.input(|i| i.key_down(egui::Key::Space)) && !wants_text;
        let middle = ui.input(|i| i.pointer.middle_down());
        let alt_hand: bool = ui.data(|d| d.get_temp(egui::Id::new(("canvas_alt_hand", app.pane)))).unwrap_or(false);
        let tool_drag: bool = ui.data(|d| d.get_temp(egui::Id::new(("canvas_pointer_down", app.pane)))).unwrap_or(false);
        let alt_text = wants_text && !tool_drag && ui.input(|i| i.modifiers.alt);
        let c = if middle || alt_hand {
            Cursor::HandGrab
        } else if space || alt_text {
            Cursor::Hand
        } else {
            app.session.cursor(xf.to_canvas(p), ui.input(|i| mods(i, false)), app.view_info())
        };
        let icon = crate::cursors::show(ui.ctx(), &mut app.cursors, c, p);
        ui.ctx().set_cursor_icon(icon);
        draw_cursor_badge(&painter, p, c, &t);
    }
}

/// The loaded text cursor's badge beside the system cursor: lines of text, a closed chain
/// (a click threads) or a broken one (a click unthreads). Drawn in code.
fn draw_cursor_badge(painter: &egui::Painter, p: Pos2, c: Cursor, t: &Tokens) {
    if !matches!(c, Cursor::LoadedText | Cursor::ThreadLink | Cursor::Unthread) {
        return;
    }
    let r = Rect::from_min_size(p + vec2(12.0, 12.0), vec2(20.0, 16.0));
    painter.rect_filled(r, 3.0, t.measure_bg);
    let ink = Stroke::new(1.5, t.measure_text);
    let c0 = r.center();
    match c {
        Cursor::LoadedText => {
            for (dy, w) in [(-4.0, 12.0), (0.0, 12.0), (4.0, 8.0)] {
                painter.line_segment([pos2(c0.x - 6.0, c0.y + dy), pos2(c0.x - 6.0 + w, c0.y + dy)], ink);
            }
        }
        _ => {
            let gap = if c == Cursor::Unthread { 2.5 } else { -2.0 };
            for side in [-1.0, 1.0] {
                let link = Rect::from_center_size(pos2(c0.x + side * (3.5 + gap / 2.0), c0.y), vec2(9.0, 6.0));
                painter.rect_stroke(link, 3.0, ink, StrokeKind::Middle);
            }
            if c == Cursor::Unthread {
                painter.line_segment([pos2(c0.x + 2.0, c0.y - 6.0), pos2(c0.x - 2.0, c0.y + 6.0)], ink);
            }
        }
    }
}

/// Render options for the canvas texture.
fn screen_options(app: &DesignApp, layout: &CanvasLayout, preview: bool, ppp: f64) -> pagenor_render::RenderOptions {
    pagenor_render::RenderOptions {
        printing_only: preview,
        greek_below_px: 3.0 * ppp,
        highlight_missing_fonts: !preview && app.session.prefs.highlight_substituted_fonts,
        highlight_hj: !preview && app.session.prefs.highlight_hj,
        condition_indicators: !preview,
        note_indicators: !preview && app.session.prefs.show_note_anchors,
        change_markup: !preview && app.session.prefs.show_added_text,
        tag_markers: !preview && app.ui.tag_markers,
        plate: app.ui.separation,
        highlight_keeps: !preview && app.session.prefs.highlight_keeps,
        highlight_custom_tracking: !preview && app.session.prefs.highlight_custom_tracking,
        quality: app.ui.display_quality,
        rich_black: app.ui.rich_black,
        overprint_preview: app.ui.overprint_preview,
        blend_space_view: !app.ui.proof_colors,
        document_grid: if app.ui.document_grid && !preview { grid_areas(layout) } else { vec![] },
        ..Default::default()
    }
}

/// The area each spread's document grid covers, in spread coordinates: the full-width band of
/// the pasteboard nearest that spread, the band in which `CanvasLayout::spread_at` (and so
/// snapping) picks it.
fn grid_areas(layout: &CanvasLayout) -> Vec<(pagenor_doc::SpreadRef, DRect)> {
    let pb = layout.pasteboard;
    let slots = &layout.slots;
    slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let lo = i.checked_sub(1).and_then(|j| slots.get(j)).map_or(pb.y0, |prev| (prev.bounds.y1 + slot.bounds.y0) / 2.0);
            let hi = slots.get(i + 1).map_or(pb.y1, |next| (slot.bounds.y1 + next.bounds.y0) / 2.0);
            (slot.spread, slot.xf.inverse().transform_rect_bbox(DRect::new(pb.x0, lo, pb.x1, hi)))
        })
        .collect()
}

/// Keep the canvas texture current. Panning inside the overscanned region just shifts the
/// texture; zooming shows the old texture scaled until the background render arrives; document
/// edits render synchronously so editing never lags.
fn render_texture(
    app: &mut DesignApp,
    ctx: &egui::Context,
    rect: Rect,
    xf: &Xf,
    layout: &CanvasLayout,
    doc: &std::sync::Arc<Document>,
    preview: bool,
) {
    let Some(st) = app.session.active() else { return };
    let ppp = ctx.pixels_per_point() as f64;
    let doc_key = (st.uid, std::sync::Arc::as_ptr(doc) as u64, st.revision, preview);
    // Overscan: 25% of the viewport on every side, within the GPU's maximum texture size.
    let max_side = ctx.input(|i| i.max_texture_side) as f32 / ppp as f32;
    let (ow, oh) =
        ((rect.width() * 0.25).min((max_side - rect.width()) / 2.0).max(0.0), (rect.height() * 0.25).min((max_side - rect.height()) / 2.0).max(0.0));
    let want = |origin: Point| crate::Shown {
        doc: doc_key,
        origin: Point::new(origin.x - ow as f64 / xf.zoom, origin.y - oh as f64 / xf.zoom),
        zoom: xf.zoom,
        size: (rect.width() + 2.0 * ow, rect.height() + 2.0 * oh),
    };
    let target = want(xf.origin);
    // Does the current texture cover the viewport at this zoom and document state?
    let covers = |s: &crate::Shown| {
        s.doc == doc_key && (s.zoom - xf.zoom).abs() < 1e-9 && {
            let x0 = (xf.origin.x - s.origin.x) * s.zoom;
            let y0 = (xf.origin.y - s.origin.y) * s.zoom;
            x0 >= -0.5 && y0 >= -0.5 && x0 + rect.width() as f64 <= s.size.0 as f64 + 0.5 && y0 + rect.height() as f64 <= s.size.1 as f64 + 0.5
        }
    };
    // Collect finished background renders.
    #[cfg(not(target_arch = "wasm32"))]
    if let (Some(w), Some((tok, shown))) = (app.canvas.worker.as_ref(), app.canvas.pending)
        && let Some(done) = w.poll()
        && done.token == tok
    {
        upload(app, ctx, done.image);
        app.canvas.shown = Some(shown);
        app.canvas.shown_doc = app.canvas.pending_doc.take();
        app.canvas.pending = None;
        app.perf.render_ms = done.ms;
    }
    if app.canvas.shown.is_some_and(|s| covers(&s)) {
        return;
    }
    let doc_changed = app.canvas.shown.is_none_or(|s| s.doc != doc_key);
    let zoom_same = app.canvas.shown.is_some_and(|s| (s.zoom - xf.zoom).abs() < 1e-9);
    let job = |app: &DesignApp| {
        app.session.active()?;
        let placed: Vec<pagenor_render::Placed> = layout.slots.iter().map(|s| pagenor_render::Placed { spread: s.spread, xf: s.xf }).collect();
        let w = (target.size.0 as f64 * ppp).round().max(1.0) as u32;
        let h = (target.size.1 as f64 * ppp).round().max(1.0) as u32;
        let view = Affine::scale(ppp) * Affine::scale(xf.zoom) * Affine::translate((-target.origin.x, -target.origin.y));
        let opts = screen_options(app, layout, preview, ppp);
        Some((doc.clone(), placed, w, h, view, opts))
    };
    // Synchronous path: document edits (keep editing crisp), first frame, or no worker.
    #[cfg(not(target_arch = "wasm32"))]
    if !app.canvas.worker_started {
        app.canvas.worker_started = true;
        app.canvas.worker = render_worker_spawn(ctx);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let has_worker = app.canvas.worker.is_some();
    #[cfg(target_arch = "wasm32")]
    let has_worker = false;
    // An edit with the view unchanged: repaint just the regions it changed.
    if doc_changed && zoom_same && patch_texture(app, layout, ppp, doc_key, doc, preview) {
        return;
    }
    if !has_worker || app.canvas.shown.is_none() || (doc_changed && zoom_same) {
        let t0 = crate::now_ms();
        let Some((doc, placed, w, h, view, opts)) = job(app) else { return };
        let img = app.canvas.renderer.render(&doc, &app.session.cache, &placed, w, h, view, &opts);
        upload(app, ctx, img);
        app.canvas.shown = Some(target);
        app.canvas.shown_doc = Some(doc);
        app.canvas.pending = None;
        app.perf.render_ms = crate::now_ms() - t0;
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Already rendering this view?
        if app.canvas.pending.is_some_and(|(_, p)| {
            p.doc == doc_key && (p.zoom - xf.zoom).abs() < 1e-9 && {
                let x0 = (xf.origin.x - p.origin.x) * p.zoom;
                let y0 = (xf.origin.y - p.origin.y) * p.zoom;
                x0 >= 0.0 && y0 >= 0.0 && x0 + rect.width() as f64 <= p.size.0 as f64 && y0 + rect.height() as f64 <= p.size.1 as f64
            }
        }) {
            return;
        }
        let Some((doc, placed, w, h, view, opts)) = job(app) else { return };
        app.canvas.pending_doc = Some(doc.clone());
        app.canvas.token += 1;
        let token = app.canvas.token;
        if let Some(wk) = app.canvas.worker.as_ref() {
            wk.submit(crate::render_worker::Job { token, doc, cache: app.session.cache.clone(), placed, w, h, view, opts });
        }
        app.canvas.pending = Some((token, target));
    }
}

/// Repaint only the damaged regions of the current texture after an edit (typing, nudging).
/// Returns false when a full render is needed (view moved, too much changed, or damage unknown).
fn patch_texture(
    app: &mut DesignApp,
    layout: &CanvasLayout,
    ppp: f64,
    doc_key: (u64, u64, u64, bool),
    doc: &std::sync::Arc<Document>,
    preview: bool,
) -> bool {
    let (Some(sh), Some(old)) = (app.canvas.shown, app.canvas.shown_doc.clone()) else { return false };
    if sh.doc.0 != doc_key.0 || sh.doc.3 != doc_key.3 || app.canvas.texture.is_none() {
        return false;
    }
    let new = doc.clone();
    let Some(regions) = pagenor_render::damage::damage_with(&old, &new, Some(&app.session.cache)) else { return false };
    let t0 = crate::now_ms();
    let (tw, th) = ((sh.size.0 as f64 * ppp).round() as i64, (sh.size.1 as f64 * ppp).round() as i64);
    // A texture `upload` sampled down to the GPU limit has other pixels than the view: a region
    // computed in view pixels would be written outside it. Render it whole instead.
    if app.canvas.texture.as_ref().is_none_or(|t| t.size() != [tw.max(0) as usize, th.max(0) as usize]) {
        return false;
    }
    let k = sh.zoom * ppp;
    let mut px: Option<(i64, i64, i64, i64)> = None;
    for (r, rect) in &regions {
        let c = layout.xf(*r).transform_rect_bbox(*rect);
        let x0 = ((c.x0 - sh.origin.x) * k).floor() as i64 - 1;
        let y0 = ((c.y0 - sh.origin.y) * k).floor() as i64 - 1;
        let x1 = ((c.x1 - sh.origin.x) * k).ceil() as i64 + 1;
        let y1 = ((c.y1 - sh.origin.y) * k).ceil() as i64 + 1;
        px = Some(px.map_or((x0, y0, x1, y1), |p| (p.0.min(x0), p.1.min(y0), p.2.max(x1), p.3.max(y1))));
    }
    let done = |app: &mut DesignApp| {
        app.canvas.shown = Some(crate::Shown { doc: doc_key, ..sh });
        app.canvas.shown_doc = Some(new.clone());
        // A background render of the old document must not overwrite this.
        app.canvas.pending = None;
        app.canvas.pending_doc = None;
    };
    let Some((x0, y0, x1, y1)) = px.map(|p| (p.0.max(0), p.1.max(0), p.2.min(tw), p.3.min(th))) else {
        done(app);
        return true;
    };
    if x1 <= x0 || y1 <= y0 {
        done(app);
        return true;
    }
    // Big changes: a full render costs about the same.
    if ((x1 - x0) * (y1 - y0)) as f64 > 0.5 * (tw * th) as f64 {
        return false;
    }
    let placed: Vec<pagenor_render::Placed> = layout.slots.iter().map(|s| pagenor_render::Placed { spread: s.spread, xf: s.xf }).collect();
    let view = Affine::translate((-(x0 as f64), -(y0 as f64)))
        * Affine::scale(ppp)
        * Affine::scale(sh.zoom)
        * Affine::translate((-sh.origin.x, -sh.origin.y));
    let opts = screen_options(app, layout, preview, ppp);
    let img = app.canvas.patcher.render(&new, &app.session.cache, &placed, (x1 - x0) as u32, (y1 - y0) as u32, view, &opts);
    let ci = egui::ColorImage::from_rgba_premultiplied([img.width as usize, img.height as usize], &img.pixels);
    if let Some(tex) = app.canvas.texture.as_mut() {
        tex.set_partial([x0 as usize, y0 as usize], ci, egui::TextureOptions::LINEAR);
    }
    done(app);
    app.canvas.patches += 1;
    app.perf.render_ms = crate::now_ms() - t0;
    true
}

#[cfg(not(target_arch = "wasm32"))]
fn render_worker_spawn(ctx: &egui::Context) -> Option<crate::render_worker::Worker> {
    if std::env::var_os("PAGENOR_SYNC_RENDER").is_some() {
        return None;
    }
    crate::render_worker::Worker::spawn(ctx.clone())
}

fn upload(app: &mut DesignApp, ctx: &egui::Context, mut img: pagenor_render::Rendered) {
    if app.ui.proof_colors {
        pagenor_render::proof_view(&mut img, &app.ui.proof_setup);
    }
    // Plates render from source colours; the ink limit is estimated from the composite.
    if app.ui.separation.is_none() && app.ui.ink_limit.is_some() {
        pagenor_render::separation_view(&mut img, None, app.ui.ink_limit);
    }
    let ci = egui::ColorImage::from_rgba_premultiplied([img.width as usize, img.height as usize], &img.pixels);
    let ci = crate::widgets::fit_texture(ci, ctx.input(|i| i.max_texture_side));
    match &mut app.canvas.texture {
        Some(tex) => tex.set(ci, egui::TextureOptions::LINEAR),
        None => app.canvas.texture = Some(ctx.load_texture("canvas", ci, egui::TextureOptions::LINEAR)),
    }
}

fn dashed(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke, dash: f32, gap: f32) {
    painter.extend(egui::Shape::dashed_line(&[a, b], stroke, dash, gap));
}

fn draw_guides(app: &DesignApp, painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout, t: &Tokens) {
    let s = &doc.settings;
    for slot in &layout.slots {
        let Some(sp) = doc.spread(slot.spread) else { continue };
        // Everything below is in spread coordinates, placed through the slot (which may be turned).
        let m = slot.xf;
        let pt = |p: Point| xf.to_screen(m * p);
        let rect = |r: DRect| xf.rect(m.transform_rect_bbox(r));
        // Bleed: one rectangle around the spread (inside bleed only applies at the spread's outer edges).
        let b = s.bleed;
        if app.ui.guides
            && b.iter().any(|v| *v > 0.0)
            && let Some((first, last)) = sp.outer_page_indices().and_then(|(l, r)| Some((sp.pages.get(l)?, sp.pages.get(r)?)))
        {
            let l = if first.side == PageSide::Left { b[3] } else { b[2] };
            let r = if last.side == PageSide::Right || last.side == PageSide::Single { b[3] } else { b[2] };
            let sb = sp.bounds();
            let br = DRect::new(sb.x0 - l, sb.y0 - b[0], sb.x1 + r, sb.y1 + b[1]);
            painter.rect_stroke(rect(br), 0.0, Stroke::new(hair(painter), c32(s.bleed_color)), StrokeKind::Middle);
        }
        let pasteboard = m.inverse().transform_rect_bbox(layout.pasteboard);
        for (pi, p) in sp.pages.iter().enumerate() {
            let pr = p.bounds();
            if !app.ui.guides {
                continue;
            }
            // Baseline grid.
            if app.ui.baseline_grid && xf.zoom >= s.baseline_grid.view_threshold {
                let g = &s.baseline_grid;
                let mut y = g.start;
                while y < p.height {
                    painter.line_segment(
                        [pt(Point::new(pr.x0, pr.y0 + y)), pt(Point::new(pr.x1, pr.y0 + y))],
                        Stroke::new(hair(painter), c32(g.color).gamma_multiply(0.8)),
                    );
                    y += g.increment.max(1.0);
                }
            }
            // Margins (magenta) and columns (violet).
            painter.rect_stroke(rect(p.margin_rect()), 0.0, Stroke::new(hair(painter), c32(s.margin_color)), StrokeKind::Middle);
            let cols = p.column_rects();
            if cols.len() > 1 {
                for (i, c) in cols.iter().enumerate() {
                    let col = Stroke::new(hair(painter), c32(s.column_color));
                    if i > 0 {
                        painter.line_segment([pt(Point::new(c.x0, c.y0)), pt(Point::new(c.x0, c.y1))], col);
                    }
                    if i + 1 < cols.len() {
                        painter.line_segment([pt(Point::new(c.x1, c.y0)), pt(Point::new(c.x1, c.y1))], col);
                    }
                }
            }
            let inherited = doc.inherited_guides(slot.spread, pi);
            for g in p.guides.iter().chain(&inherited).filter(|g| g.visible_in(doc)) {
                // Spread guides cross the pasteboard; page guides their page.
                let span = if g.spread { pasteboard } else { pr };
                let (a, bb) = match g.orientation {
                    pagenor_doc::Orientation::Horizontal => (Point::new(span.x0, g.position), Point::new(span.x1, g.position)),
                    pagenor_doc::Orientation::Vertical => {
                        let span = if g.spread { sp.bounds().inflate(0.0, 36.0) } else { pr };
                        (Point::new(g.position, span.y0), Point::new(g.position, span.y1))
                    }
                };
                let stroke = Stroke::new(hair(painter), Color32::from_rgb(74, 227, 255));
                if g.liquid {
                    // Liquid guides are dashed.
                    painter.extend(egui::Shape::dashed_line(&[pt(a), pt(bb)], stroke, 6.0, 3.0));
                } else {
                    painter.line_segment([pt(a), pt(bb)], stroke);
                }
            }
        }
    }
    let _ = t;
}

/// Spread-space → canvas transform for an item at `loc`.
fn item_canvas_xf(doc: &Document, layout: &CanvasLayout, id: pagenor_doc::ItemId) -> Option<(Affine, &'static str)> {
    let loc = doc.find(id)?;
    Some((layout.xf(loc.spread) * doc.parent_xf(&loc), ""))
}

fn path_screen(it: &Item, xf: &Xf, a: Affine) -> Vec<Vec<Pos2>> {
    let bp = it.path.to_bezpath();
    let full = a * it.xf;
    let mut out = Vec::new();
    let mut cur: Vec<Pos2> = Vec::new();
    pagenor_geom::kurbo::flatten(&bp, 0.25 / xf.zoom.max(0.01), |el| {
        use pagenor_geom::PathEl::*;
        match el {
            MoveTo(p) => {
                if cur.len() > 1 {
                    out.push(std::mem::take(&mut cur));
                }
                cur.clear();
                cur.push(xf.to_screen(full * p));
            }
            LineTo(p) => cur.push(xf.to_screen(full * p)),
            ClosePath => {
                if let Some(f) = cur.first().copied() {
                    cur.push(f);
                }
            }
            _ => {}
        }
    });
    if cur.len() > 1 {
        out.push(cur);
    }
    out
}

fn draw_frames(app: &DesignApp, painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout) {
    if !app.ui.frame_edges {
        return;
    }
    for slot in &layout.slots {
        let Some(sp) = doc.spread(slot.spread) else { continue };
        let a = slot.xf;
        // Parent items on document pages: dotted edges.
        if let SpreadRef::Doc(si) = slot.spread {
            let first = doc.first_page_of_spread(si);
            for (pi, page) in sp.pages.iter().enumerate() {
                let Some((ppi, ppg)) = doc.parent_page_for(first + pi) else { continue };
                let parent = &doc.parents[ppi];
                let dx = page.x - parent.pages[ppg].x;
                for it in &parent.items {
                    if (parent.pages.len() > 1 && parent.page_at_x(it.bounds().center().x) != Some(ppg)) || page.overridden.contains(&it.id) {
                        continue;
                    }
                    let col = doc.layer(it.layer).map(|l| c32(l.color)).unwrap_or(Color32::LIGHT_BLUE);
                    for poly in path_screen(it, xf, a * Affine::translate((dx, 0.0))) {
                        for w in poly.windows(2) {
                            dashed(painter, w[0], w[1], Stroke::new(hair(painter), col.gamma_multiply(0.8)), 2.0, 2.0);
                        }
                    }
                }
            }
        }
        for it in &sp.items {
            draw_item_edges(painter, xf, doc, it, a);
        }
        // Tagged frames in their tag's colour.
        if app.ui.tagged_frames {
            for top in &sp.items {
                top.walk(&mut |it: &Item| {
                    if it.xml_tag.is_empty() {
                        return;
                    }
                    let Some(t) = doc.xml.tags.iter().find(|t| t.name == it.xml_tag) else { return };
                    let col = Color32::from_rgb(t.color[0], t.color[1], t.color[2]);
                    for poly in path_screen(it, xf, a) {
                        for w in poly.windows(2) {
                            painter.line_segment([w[0], w[1]], Stroke::new(2.5, col));
                        }
                    }
                });
            }
        }
    }
}

fn draw_item_edges(painter: &egui::Painter, xf: &Xf, doc: &Document, it: &Item, a: Affine) {
    if it.hidden || doc.layer(it.layer).is_some_and(|l| !l.visible) {
        return;
    }
    let col = doc.layer(it.layer).map(|l| c32(l.color)).unwrap_or(Color32::LIGHT_BLUE);
    if let Content::Group { items } = &it.content {
        for c in items {
            draw_item_edges(painter, xf, doc, c, a * it.xf);
        }
        return;
    }
    // Frames without a stroke show their edge; shapes with strokes are visible already.
    let show = it.stroke.is_none() || matches!(it.content, Content::Text(_) | Content::Graphic(_));
    if show {
        for poly in path_screen(it, xf, a) {
            painter.add(egui::Shape::line(poly, Stroke::new(hair(painter), col.gamma_multiply(0.75))));
        }
    }
    // Empty graphic frame: the X.
    if it.object_style == pagenor_doc::BASIC_GRAPHICS_FRAME && matches!(it.content, Content::Unassigned) {
        let r = it.inner_bounds();
        let m = a * it.xf;
        let p = |x: f64, y: f64| xf.to_screen(m * Point::new(x, y));
        painter.line_segment([p(r.x0, r.y0), p(r.x1, r.y1)], Stroke::new(hair(painter), col.gamma_multiply(0.75)));
        painter.line_segment([p(r.x1, r.y0), p(r.x0, r.y1)], Stroke::new(hair(painter), col.gamma_multiply(0.75)));
    }
}

/// Layer colour of the first selected item: live-corner marks are outlined in it.
fn selection_color(app: &DesignApp) -> Color32 {
    let fallback = Color32::from_rgb(79, 153, 255);
    let Some(st) = app.session.active() else { return fallback };
    st.selection.items.first().and_then(|id| st.doc.item(*id)).map_or(fallback, |it| layer_color(&st.doc, it))
}

fn layer_color(doc: &Document, it: &Item) -> Color32 {
    doc.layer(it.layer).map(|l| c32(l.color)).unwrap_or(Color32::from_rgb(79, 153, 255))
}

/// Draw selection chrome; returns the selection's screen bounds (for the Contextual Task Bar).
fn draw_selection(app: &DesignApp, painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout) -> Option<Rect> {
    let st = app.session.active()?;
    let sel: &Selection = &st.selection;
    let mut text_rect = None;
    // Text selection / caret.
    if let Some(ts) = app.composition_selection().or(sel.text) {
        draw_text_selection(app, painter, xf, doc, layout, ts, sel.cells);
        let fid = ts.frame.or_else(|| {
            let cs = app.session.cache.get(doc, ts.story, None);
            compose::caret(&cs, ts.focus).and_then(|c| cs.frames.get(c.0)).map(|f| f.frame)
        });
        if let Some(fid) = fid
            && let (Some(it), Some((a, _))) = (doc.item(fid), item_canvas_xf(doc, layout, fid))
        {
            text_rect = Some(xf.rect(a.transform_rect_bbox(it.bounds())));
        }
    }
    let mut union: Option<Rect> = None;
    let mut color = Color32::from_rgb(79, 153, 255);
    for id in &sel.items {
        let (Some(it), Some((a, _))) = (doc.item(*id), item_canvas_xf(doc, layout, *id)) else { continue };
        color = layer_color(doc, it);
        for poly in path_screen(it, xf, a) {
            painter.add(egui::Shape::line(poly, Stroke::new(1.0, color)));
        }
        if sel.items.len() == 1 && sel.text.is_none() {
            // InDesign draws a 1 pt bounding box around the selection.
            painter.rect_stroke(xf.rect(a.transform_rect_bbox(it.bounds())), 0.0, Stroke::new(1.0, color), StrokeKind::Middle);
        }
        let b = xf.rect(a.transform_rect_bbox(it.bounds()));
        union = Some(union.map_or(b, |u| u.union(b)));
        // Text frame ports.
        if let Content::Text(tf) = &it.content {
            draw_ports(app, painter, xf, doc, layout, it, tf.story, color);
        }
        if let Content::Graphic(g) = &it.content
            && sel.content
        {
            let gb = xf.rect((a * it.xf * g.xf).transform_rect_bbox(DRect::new(0.0, 0.0, g.size.0, g.size.1)));
            painter.rect_stroke(gb, 0.0, Stroke::new(1.0, Color32::from_rgb(196, 111, 43)), StrokeKind::Middle);
        }
    }
    let direct = matches!(app.session.tool_id(), "directSelection" | "pen" | "addAnchor" | "deleteAnchor" | "convertDirection");
    if direct {
        for id in &sel.items {
            let (Some(it), Some((a, _))) = (doc.item(*id), item_canvas_xf(doc, layout, *id)) else { continue };
            let m = a * it.xf;
            let col = layer_color(doc, it);
            for sp in &it.path.subpaths {
                for an in &sp.anchors {
                    let p = xf.to_screen(m * an.p);
                    for (h, has) in [(an.h_in, an.has_in()), (an.h_out, an.has_out())] {
                        if has {
                            let hp = xf.to_screen(m * h);
                            painter.line_segment([p, hp], Stroke::new(1.0, col));
                            painter.circle_filled(hp, 2.5, col);
                        }
                    }
                    let r = Rect::from_center_size(p, vec2(5.0, 5.0));
                    painter.rect_filled(r, 0.0, Color32::WHITE);
                    painter.rect_stroke(r, 0.0, Stroke::new(1.0, col), StrokeKind::Inside);
                }
            }
        }
    }
    if let Some(u) = union
        && sel.text.is_none()
        && !direct
    {
        if sel.items.len() > 1 {
            painter.rect_stroke(u, 0.0, Stroke::new(1.0, color), StrokeKind::Middle);
        }
        let hs = 6.5;
        // Centre point. The Selection tool draws the live-corner widget as an overlay.
        painter.rect_filled(Rect::from_center_size(u.center(), vec2(3.5, 3.5)), 0.0, color);
        for h in pagenor_tools::select::handles(DRect::new(u.min.x as f64, u.min.y as f64, u.max.x as f64, u.max.y as f64)) {
            let r = Rect::from_center_size(pos2(h.x as f32, h.y as f32), vec2(hs, hs));
            painter.rect_filled(r, 0.0, Color32::WHITE);
            painter.rect_stroke(r, 0.0, Stroke::new(hair(painter), color), StrokeKind::Inside);
        }
    }
    // Threads.
    if app.ui.text_threads || sel.items.iter().any(|i| doc.item(*i).is_some_and(|x| x.is_text_frame())) {
        for id in &sel.items {
            let Some(sid) = doc.item(*id).and_then(|i| i.text_frame()).map(|t| t.story) else { continue };
            let Some(story) = doc.story(sid) else { continue };
            for w in story.frames.windows(2) {
                let (Some(a), Some(b)) = (port_pos(doc, layout, w[0], true), port_pos(doc, layout, w[1], false)) else { continue };
                let col = doc.item(w[0]).map(|i| layer_color(doc, i)).unwrap_or(color);
                painter.line_segment([xf.to_screen(a), xf.to_screen(b)], Stroke::new(1.0, col));
            }
        }
    }
    text_rect.or(union)
}

/// Port location (canvas): out port at bottom-right, in port at top-left.
fn port_pos(doc: &Document, layout: &CanvasLayout, id: pagenor_doc::ItemId, out: bool) -> Option<Point> {
    let it = doc.item(id)?;
    let (a, _) = item_canvas_xf(doc, layout, id)?;
    let r = it.inner_bounds();
    let p = if out { Point::new(r.x1 - 10.0, r.y1) } else { Point::new(r.x0 + 10.0, r.y0) };
    Some(a * it.xf * p)
}

#[allow(clippy::too_many_arguments)]
fn draw_ports(
    app: &DesignApp,
    painter: &egui::Painter,
    xf: &Xf,
    doc: &Document,
    layout: &CanvasLayout,
    it: &Item,
    sid: pagenor_doc::StoryId,
    color: Color32,
) {
    let Some(story) = doc.story(sid) else { return };
    let pos = story.frames.iter().position(|f| *f == it.id).unwrap_or(0);
    let is_last = pos + 1 == story.frames.len();
    let cs = app.session.cache.get(doc, sid, None);
    let overset = is_last && cs.is_overset();
    let s = pagenor_tools::thread::PORT_SIZE_PX as f32;
    // In port on the left edge below the top-left corner; out port on the right edge above the
    // bottom-right corner. The Selection tool hit-tests the same places.
    let port = |out: bool| pagenor_tools::thread::port_center(doc, layout, it.id, out, xf.zoom).map(|c| xf.to_screen(c));
    let (Some(inp), Some(outp)) = (port(false), port(true)) else { return };
    for (c, has_link, is_out) in [(inp, pos > 0, false), (outp, !is_last, true)] {
        let pr = Rect::from_center_size(c, vec2(s, s));
        painter.rect_filled(pr, 0.0, Color32::WHITE);
        painter.rect_stroke(pr, 0.0, Stroke::new(hair(painter), color), StrokeKind::Inside);
        if is_out && overset {
            let red = Color32::from_rgb(230, 20, 20);
            painter.line_segment([c - vec2(2.5, 0.0), c + vec2(2.5, 0.0)], Stroke::new(1.5, red));
            painter.line_segment([c - vec2(0.0, 2.5), c + vec2(0.0, 2.5)], Stroke::new(1.5, red));
        } else if has_link {
            painter.add(egui::Shape::convex_polygon(vec![c + vec2(-2.0, -2.5), c + vec2(2.5, 0.0), c + vec2(-2.0, 2.5)], color, Stroke::NONE));
        }
    }
}

fn draw_text_selection(
    app: &DesignApp,
    painter: &egui::Painter,
    xf: &Xf,
    doc: &Document,
    layout: &CanvasLayout,
    ts: pagenor_doc::TextSel,
    cells: Option<pagenor_doc::TableSel>,
) {
    let cs = app.session.cache.get(doc, ts.story, None);
    let range = ts.range();
    // Selected table cells: tint every selected cell.
    if let Some(sel) = cells.filter(|c| c.story == ts.story) {
        for ft in &cs.frames {
            let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame)) else { continue };
            let m = a * doc.text_xf(it);
            for t in ft.tables.iter().filter(|t| t.table == sel.table) {
                for c in t.cells.iter().filter(|c| sel.range.contains(c.row, c.col)) {
                    let q = [
                        xf.to_screen(m * Point::new(c.rect.x0, c.rect.y0)),
                        xf.to_screen(m * Point::new(c.rect.x1, c.rect.y0)),
                        xf.to_screen(m * Point::new(c.rect.x1, c.rect.y1)),
                        xf.to_screen(m * Point::new(c.rect.x0, c.rect.y1)),
                    ];
                    painter.add(egui::Shape::convex_polygon(q.to_vec(), Color32::from_rgba_unmultiplied(80, 140, 255, 90), Stroke::NONE));
                }
            }
        }
        return;
    }
    if let Some(cell) = ts.cell {
        draw_cell_selection(app, painter, xf, doc, layout, &cs, ts, cell);
        return;
    }
    let mut runs: Vec<(usize, Affine, Vec<[Point; 4]>, Vec<(usize, usize)>)> = Vec::new();
    for (fi, ft) in cs.frames.iter().enumerate() {
        let Some((a, _)) = item_canvas_xf(doc, layout, ft.frame) else { continue };
        let Some(it) = doc.item(ft.frame) else { continue };
        let m = a * doc.text_xf(it);
        // Show the frame edge while typing.
        for poly in path_screen(it, xf, a) {
            painter.add(egui::Shape::line(poly, Stroke::new(1.0, layer_color(doc, it).gamma_multiply(0.6))));
        }
        if range.is_empty() {
            continue;
        }
        let mut quads = Vec::new();
        let mut glyphs = Vec::new();
        for (li, l) in ft.lines.iter().enumerate() {
            let s = range.start.max(l.range.start);
            let e = range.end.min(l.range.end);
            if s > e || (s == e && !(l.range.end < range.end && e == l.range.end)) {
                continue;
            }
            let quad = |x0: f64, x1: f64| {
                [
                    Point::new(x0, l.baseline - l.ascent),
                    Point::new(x1, l.baseline - l.ascent),
                    Point::new(x1, l.baseline + l.descent),
                    Point::new(x0, l.baseline + l.descent),
                ]
            };
            if l.glyphs.iter().any(|g| g.rtl) {
                // Mixed directions: the selected glyphs' boxes, joined where they touch.
                let mut spans: Vec<(f64, f64)> =
                    l.glyphs.iter().filter(|g| g.len > 0 && g.byte >= s && g.byte < e).map(|g| (g.x, g.x + g.adv)).collect();
                spans.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut merged: Vec<(f64, f64)> = Vec::new();
                for (a, b) in spans {
                    match merged.last_mut() {
                        Some(m) if a <= m.1 + 0.5 => m.1 = m.1.max(b),
                        _ => merged.push((a, b)),
                    }
                }
                quads.extend(merged.into_iter().map(|(a, b)| quad(a, b)));
            } else {
                let x0 = compose::caret_x(l, s);
                let x1 = if e == l.range.end && range.end > l.range.end { l.end_x.max(x0 + 3.0) } else { compose::caret_x(l, e) };
                quads.push(quad(x0, x1));
            }
            // A selected drop cap is highlighted down to its baseline.
            if let Some(dc) = l.drop_cap {
                let sel = l.glyphs.iter().filter(|g| g.len > 0 && g.byte < dc.end && g.byte >= s && g.byte < e);
                let (a, b) = sel.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), g| (a.min(g.x), b.max(g.x + g.adv)));
                if a < b {
                    let r = dc.rect;
                    quads.push([Point::new(a, r.y0), Point::new(b, r.y0), Point::new(b, r.y1), Point::new(a, r.y1)]);
                }
            }
            for (gi, g) in l.glyphs.iter().enumerate() {
                if g.visible && g.len > 0 && g.byte >= s && g.byte < e {
                    glyphs.push((li, gi));
                }
            }
        }
        if !quads.is_empty() {
            runs.push((fi, m, quads, glyphs));
        }
    }
    if !runs.is_empty() {
        draw_inverse_highlight(app, painter, xf, doc, &cs, ts.story, &runs);
    }
    if ts.is_caret()
        && let Some((fi, x, bl, asc, desc)) = compose::caret(&cs, ts.focus)
        && let Some(ft) = cs.frames.get(fi)
        && let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame))
    {
        let m = a * doc.text_xf(it);
        let blink = (painter.ctx().input(|i| i.time) * 1.6) as i64 % 2 == 0;
        if blink {
            let p0 = xf.to_screen(m * Point::new(x, bl - asc));
            let p1 = xf.to_screen(m * Point::new(x, bl + desc));
            painter.line_segment([p0, p1], Stroke::new(1.0, Color32::BLACK));
        }
        painter.ctx().request_repaint_after(std::time::Duration::from_millis(330));
    }
}

/// Caret / text highlight inside a table cell (the cell's lines are offset by the cell origin).
#[allow(clippy::too_many_arguments)]
fn draw_cell_selection(
    _app: &DesignApp,
    painter: &egui::Painter,
    xf: &Xf,
    doc: &Document,
    layout: &CanvasLayout,
    cs: &compose::ComposedStory,
    ts: pagenor_doc::TextSel,
    cell: pagenor_doc::CellAddr,
) {
    // Footnote text or a cell: its composed story and where it sits in the frame.
    let found = match cell.footnote_id() {
        Some(id) => compose::find_note(cs, id).map(|(fi, n)| (fi, n.origin, n.text.clone())),
        None => compose::find_cell(cs, cell.table, cell.row, cell.col).map(|(fi, _, pc)| (fi, pc.origin, pc.text.clone())),
    };
    let Some((fi, origin, text)) = found else { return };
    let Some(ft) = cs.frames.get(fi) else { return };
    let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame)) else { return };
    let m = a * doc.text_xf(it) * pagenor_geom::Affine::translate(origin.to_vec2());
    let range = ts.range();
    if !range.is_empty()
        && let Some(cft) = text.frames.first()
    {
        for l in &cft.lines {
            let s = range.start.max(l.range.start);
            let e = range.end.min(l.range.end);
            if s > e || (s == e && !(l.range.end < range.end && e == l.range.end)) {
                continue;
            }
            let x0 = compose::caret_x(l, s);
            let x1 = if e == l.range.end && range.end > l.range.end { l.end_x.max(x0 + 3.0) } else { compose::caret_x(l, e) };
            let q = [
                xf.to_screen(m * Point::new(x0, l.baseline - l.ascent)),
                xf.to_screen(m * Point::new(x1, l.baseline - l.ascent)),
                xf.to_screen(m * Point::new(x1, l.baseline + l.descent)),
                xf.to_screen(m * Point::new(x0, l.baseline + l.descent)),
            ];
            painter.add(egui::Shape::convex_polygon(q.to_vec(), Color32::from_rgba_unmultiplied(80, 140, 255, 110), Stroke::NONE));
        }
    }
    if ts.is_caret()
        && let Some((_, x, bl, asc, desc)) = match cell.footnote_id() {
            Some(id) => compose::note_caret(cs, id, ts.focus),
            None => compose::cell_caret(cs, cell.table, cell.row, cell.col, ts.focus),
        }
    {
        let m = a * doc.text_xf(it);
        let blink = (painter.ctx().input(|i| i.time) * 1.6) as i64 % 2 == 0;
        if blink {
            painter.line_segment(
                [xf.to_screen(m * Point::new(x, bl - asc)), xf.to_screen(m * Point::new(x, bl + desc))],
                Stroke::new(1.0, Color32::BLACK),
            );
        }
        painter.ctx().request_repaint_after(std::time::Duration::from_millis(330));
    }
}

/// InDesign draws selected text inverted: a black highlight with the selected glyphs in the
/// inverse of their colour (white for black text). The highlight and glyph outlines are
/// rasterised with `vello_cpu` into a small screen-space texture over the selection.
fn draw_inverse_highlight(
    app: &DesignApp,
    painter: &egui::Painter,
    xf: &Xf,
    doc: &Document,
    cs: &compose::ComposedStory,
    story: pagenor_doc::StoryId,
    runs: &[(usize, Affine, Vec<[Point; 4]>, Vec<(usize, usize)>)],
) {
    use pagenor_render::vello_cpu::{self, kurbo, peniko};
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point();
    let to_screen = |m: Affine, p: Point| xf.to_screen(m * p);
    let mut bbox = Rect::NOTHING;
    for (_, m, quads, _) in runs {
        for q in quads {
            for p in q {
                bbox.extend_with(to_screen(*m, *p));
            }
        }
    }
    let bbox = bbox.intersect(painter.clip_rect());
    if !bbox.is_positive() {
        return;
    }
    // Snap to device pixels so the texture maps 1:1.
    let bbox = Rect::from_min_max(
        pos2((bbox.min.x * ppp).floor() / ppp, (bbox.min.y * ppp).floor() / ppp),
        pos2((bbox.max.x * ppp).ceil() / ppp, (bbox.max.y * ppp).ceil() / ppp),
    );
    let (w, h) = (((bbox.width() * ppp).round() as u32).clamp(1, 8192), ((bbox.height() * ppp).round() as u32).clamp(1, 8192));
    let Some(st) = app.session.active() else { return };
    let key = egui::Id::new(("inverse_sel", st.uid, st.revision, std::sync::Arc::as_ptr(&st.doc) as usize, story.0))
        .with((runs.iter().map(|r| r.2.len() + r.3.len() * 7919).sum::<usize>(), w, h))
        .with((bbox.min.x.to_bits(), bbox.min.y.to_bits(), xf.zoom.to_bits(), st.selection.text.map(|t| (t.anchor, t.focus))));
    let cache_id = egui::Id::new("inverse_sel_tex");
    let cached: Option<(egui::Id, egui::TextureHandle)> = ctx.data(|d| d.get_temp(cache_id));
    let tex = match cached {
        Some((k, t)) if k == key => t,
        _ => {
            let view = xf.affine(ppp as f64, bbox.min);
            let mut rc = vello_cpu::RenderContext::new_with(w as u16, h as u16, vello_cpu::RenderSettings { num_threads: 0, ..Default::default() });
            rc.set_paint(peniko::Color::from_rgba8(0, 0, 0, 255));
            for (_, m, quads, _) in runs {
                rc.set_transform(view * *m);
                for q in quads {
                    let mut bp = kurbo::BezPath::new();
                    bp.move_to(q[0]);
                    for p in &q[1..] {
                        bp.line_to(*p);
                    }
                    bp.close_path();
                    rc.fill_path(&bp);
                }
            }
            let db = pagenor_fonts::FontDb::global();
            for (fi, m, _, glyphs) in runs {
                let Some(ft) = cs.frames.get(*fi) else { continue };
                for &(li, gi) in glyphs {
                    let Some(l) = ft.lines.get(li) else { continue };
                    let Some(g) = l.glyphs.get(gi) else { continue };
                    let Some(style) = cs.styles.get(g.style as usize) else { continue };
                    let Some(c) = doc.resolve_color(&style.fill, style.fill_tint) else { continue };
                    let [r, gg, b, _] = c.to_rgba8(1.0);
                    rc.set_paint(peniko::Color::from_rgba8(255 - r, 255 - gg, 255 - b, 255));
                    let outline = db.outline(&g.face, g.gid);
                    let skew =
                        if style.skew != 0.0 { Affine::new([1.0, 0.0, -style.skew.to_radians().tan(), 1.0, 0.0, 0.0]) } else { Affine::IDENTITY };
                    let ga = Affine::translate((g.x, l.baseline + g.y)) * skew * Affine::scale_non_uniform(g.sx, g.sy);
                    rc.set_transform(view * *m * ga);
                    rc.fill_path(&outline);
                }
            }
            rc.flush();
            let mut pm = vello_cpu::Pixmap::new(w as u16, h as u16);
            let mut res = vello_cpu::Resources::new();
            rc.render(&mut pm, &mut res);
            let ci = egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], pm.data_as_u8_slice());
            let t = crate::widgets::load_texture(ctx, "inverse_selection", ci, egui::TextureOptions::NEAREST);
            ctx.data_mut(|d| d.insert_temp(cache_id, (key, t.clone())));
            t
        }
    };
    painter.image(tex.id(), bbox, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
}

fn draw_tool_overlays(app: &mut DesignApp, painter: &egui::Painter, xf: &Xf) {
    if let (Some((z, c)), Some(rect)) = (app.power_zoom, app.canvas_rect) {
        // The view that releasing returns to.
        let (w, h) = (rect.width() as f64 / z, rect.height() as f64 / z);
        let r = xf.rect(pagenor_geom::Rect::new(c.x - w / 2.0, c.y - h / 2.0, c.x + w / 2.0, c.y + h / 2.0));
        painter.rect_stroke(r, 0.0, egui::Stroke::new(1.5, Color32::from_rgb(230, 40, 40)), egui::StrokeKind::Middle);
    }
    let tok = Tokens::get(painter.ctx());
    let ov = app.session.overlays(app.view_info());
    let doc = app.session.active().map(|st| st.doc.clone());
    let layout = app.session.layout();
    let sel_color = selection_color(app);
    for o in ov {
        match o {
            Overlay::Marquee(r) => {
                let r = xf.rect(r);
                painter.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(80, 140, 255, 30));
                for (a, b) in [
                    (r.left_top(), r.right_top()),
                    (r.right_top(), r.right_bottom()),
                    (r.right_bottom(), r.left_bottom()),
                    (r.left_bottom(), r.left_top()),
                ] {
                    dashed(painter, a, b, Stroke::new(1.0, Color32::from_gray(200)), 3.0, 3.0);
                }
            }
            Overlay::Measure { p, text } => {
                let s = xf.to_screen(p) + vec2(14.0, 14.0);
                let g = painter.layout_no_wrap(text, egui::FontId::proportional(11.0), tok.measure_text);
                let r = Rect::from_min_size(s, g.size() + vec2(10.0, 6.0));
                painter.rect_filled(r, 3.0, tok.measure_bg);
                painter.galley(r.min + vec2(5.0, 3.0), g, tok.measure_text);
            }
            Overlay::Line { a, b, color, dashed: d } => {
                let st = Stroke::new(1.0, c32(color));
                if d {
                    dashed(painter, xf.to_screen(a), xf.to_screen(b), st, 3.0, 3.0);
                } else {
                    painter.line_segment([xf.to_screen(a), xf.to_screen(b)], st);
                }
            }
            Overlay::Guide { a, b } => {
                painter.line_segment([xf.to_screen(a), xf.to_screen(b)], Stroke::new(1.0, tok.smart_guide));
            }
            Overlay::Gap { a, b, label } => {
                let (a, b) = (xf.to_screen(a), xf.to_screen(b));
                painter.line_segment([a, b], Stroke::new(1.0, tok.smart_guide));
                let s = a + (b - a) * 0.5;
                let g = painter.layout_no_wrap(label, egui::FontId::proportional(11.0), tok.measure_text);
                let r = Rect::from_min_size(s, g.size() + vec2(10.0, 6.0));
                painter.rect_filled(r, 3.0, tok.measure_bg);
                painter.galley(r.min + vec2(5.0, 3.0), g, tok.measure_text);
            }
            Overlay::LiveCornerWidget(p) => {
                let r = Rect::from_center_size(xf.to_screen(p), vec2(6.0, 6.0));
                painter.rect_filled(r, 0.0, tok.live_corner);
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, sel_color), StrokeKind::Inside);
            }
            Overlay::LiveCornerDiamond(p) => {
                let c = xf.to_screen(p);
                let h = 4.5;
                let pts = vec![c + vec2(0.0, -h), c + vec2(h, 0.0), c + vec2(0.0, h), c + vec2(-h, 0.0)];
                painter.add(egui::Shape::convex_polygon(pts, tok.live_corner, Stroke::new(1.0, sel_color)));
            }
            Overlay::Path { path, color, dashed: d } => draw_path_overlay(painter, xf, &path, c32(color), d),
            Overlay::ContentGhost(id) => {
                if let Some(doc) = &doc {
                    draw_content_ghost(painter, xf, doc, &layout, id);
                }
            }
        }
    }
}

/// A path overlay (canvas coordinates; the Pencil's stroke while drawing, the Smooth and Erase
/// drags) as screen polylines, its curves flattened to about a quarter of a screen pixel.
fn draw_path_overlay(painter: &egui::Painter, xf: &Xf, path: &pagenor_geom::BezPath, color: Color32, dashed_line: bool) {
    use pagenor_geom::PathEl;
    let stroke = Stroke::new(1.0, color);
    let mut runs: Vec<Vec<Pos2>> = Vec::new();
    pagenor_geom::kurbo::flatten(path, 0.25 / xf.zoom.max(1e-9), |el| match el {
        PathEl::MoveTo(p) => runs.push(vec![xf.to_screen(p)]),
        PathEl::LineTo(p) => {
            if let Some(run) = runs.last_mut() {
                run.push(xf.to_screen(p));
            }
        }
        PathEl::ClosePath => {
            if let Some(run) = runs.last_mut()
                && let Some(&first) = run.first()
            {
                run.push(first);
            }
        }
        // Flattening yields only moves, lines and closes.
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
    });
    for run in runs.into_iter().filter(|r| r.len() >= 2) {
        if dashed_line {
            painter.extend(egui::Shape::dashed_line(&run, stroke, 3.0, 3.0));
        } else {
            painter.add(egui::Shape::line(run, stroke));
        }
    }
}

/// A frame's whole content drawn faintly over the canvas, the part outside the frame included,
/// with its bounds outlined (while the content moves inside the frame).
fn draw_content_ghost(painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout, id: pagenor_doc::ItemId) {
    let Some(it) = doc.item(id) else { return };
    let Some((a, _)) = item_canvas_xf(doc, layout, id) else { return };
    let outline = Stroke::new(1.0, Color32::from_rgb(196, 111, 43));
    let Content::Graphic(g) = &it.content else {
        for c in it.children() {
            let r = xf.rect((a * it.xf).transform_rect_bbox(c.bounds()));
            painter.rect_stroke(r, 0.0, outline, StrokeKind::Middle);
        }
        return;
    };
    let m = a * it.xf * g.xf;
    let (w, h) = g.size;
    let corners = [Point::new(0.0, 0.0), Point::new(w, 0.0), Point::new(w, h), Point::new(0.0, h)].map(|p| xf.to_screen(m * p));
    if corners.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return;
    }
    if let Some(tex) = ghost_texture(painter.ctx(), doc, g) {
        let mut mesh = egui::Mesh::with_texture(tex.id());
        let tint = Color32::from_white_alpha(90);
        for (p, uv) in corners.iter().zip([pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)]) {
            mesh.vertices.push(egui::epaint::Vertex { pos: *p, uv, color: tint });
        }
        mesh.indices.extend([0, 1, 2, 0, 2, 3]);
        painter.add(egui::Shape::mesh(mesh));
    }
    painter.add(egui::Shape::closed_line(corners.to_vec(), outline));
}

/// A screen-resolution proxy of a placed graphic for the ghost, cached per asset.
fn ghost_texture(ctx: &egui::Context, doc: &Document, g: &pagenor_doc::Graphic) -> Option<egui::TextureHandle> {
    let asset = doc.assets.get(&g.asset)?;
    let key = egui::Id::new(("content_ghost", std::sync::Arc::as_ptr(&asset.data) as usize, asset.data.len(), asset.page));
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return Some(t);
    }
    // A 72 ppi proxy is enough for a faint preview.
    let pm = pagenor_render::images::mip(&asset.data, asset.page, g.size.0, 1.0)?;
    let (pw, ph) = (pm.width() as usize, pm.height() as usize);
    let max = ctx.input(|i| i.max_texture_side).min(4096);
    if pw == 0 || ph == 0 || pw > max || ph > max {
        return None;
    }
    let ci = egui::ColorImage::from_rgba_premultiplied([pw, ph], pm.data_as_u8_slice());
    let t = ctx.load_texture("content_ghost", ci, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, t.clone()));
    Some(t)
}

#[allow(clippy::too_many_arguments)]
fn draw_rulers(app: &DesignApp, ui: &egui::Ui, full: Rect, rect: Rect, xf: &Xf, doc: &Document, layout: &CanvasLayout, t: &Tokens) {
    let painter = ui.painter_at(full);
    let (top, left) = ruler_rects(full, rect);
    painter.rect_filled(top, 0.0, t.ruler);
    painter.rect_filled(left, 0.0, t.ruler);
    painter.rect_filled(Rect::from_min_max(full.min, rect.min), 0.0, t.ruler);
    painter.line_segment([pos2(rect.min.x, rect.min.y - 0.25), pos2(full.max.x, rect.min.y - 0.25)], Stroke::new(0.5, t.ruler_tick));
    painter.line_segment([pos2(rect.min.x - 0.25, rect.min.y), pos2(rect.min.x - 0.25, full.max.y)], Stroke::new(0.5, t.ruler_tick));
    // Origin: the custom zero point if one was dragged out, else the top-left of the current
    // spread's first page.
    let custom_origin = app.view().and_then(|v| v.ruler_origin);
    let origin = match custom_origin {
        Some(o) => o,
        None => {
            let Some(i) = current_slot(app, layout) else { return };
            let slot = &layout.slots[i];
            Point::new(slot.bounds.x0, slot.bounds.y0)
        }
    };
    // Zero-point crosshair box: highlighted when a custom origin is set.
    let zc = Rect::from_min_max(full.min, rect.min).center();
    let zc_stroke = Stroke::new(1.0, if custom_origin.is_some() { t.accent } else { t.ruler_tick });
    painter.line_segment([zc - vec2(4.0, 0.0), zc + vec2(4.0, 0.0)], zc_stroke);
    painter.line_segment([zc - vec2(0.0, 4.0), zc + vec2(0.0, 4.0)], zc_stroke);
    let font = egui::FontId::proportional(9.0);
    let tick = Stroke::new(1.0, t.ruler_tick);
    // Horizontal.
    let unit = doc.settings.horizontal_units;
    let (major, sub) = unit.ruler_ticks(xf.zoom);
    let c0 = xf.to_canvas(top.min).x - origin.x;
    let c1 = xf.to_canvas(pos2(top.max.x, 0.0)).x - origin.x;
    let mut k = (c0 / major).floor() as i64;
    while (k as f64) * major <= c1 {
        let x0 = k as f64 * major;
        for s in 0..sub {
            let x = x0 + major * s as f64 / sub as f64;
            let sx = xf.to_screen(Point::new(origin.x + x, 0.0)).x;
            let len = if s == 0 {
                RULER
            } else if sub % 2 == 0 && s == sub / 2 {
                7.0
            } else {
                4.0
            };
            painter.line_segment([pos2(sx, top.max.y - len), pos2(sx, top.max.y)], tick);
            if s == 0 {
                let v = unit.from_pt(x);
                painter.text(pos2(sx + 2.0, top.min.y + 1.0), egui::Align2::LEFT_TOP, fmt_tick(v), font.clone(), t.ruler_text);
            }
        }
        k += 1;
    }
    // Vertical.
    let unit = doc.settings.vertical_units;
    let (major, sub) = unit.ruler_ticks(xf.zoom);
    let c0 = xf.to_canvas(pos2(0.0, left.min.y)).y - origin.y;
    let c1 = xf.to_canvas(pos2(0.0, left.max.y)).y - origin.y;
    let mut k = (c0 / major).floor() as i64;
    while (k as f64) * major <= c1 {
        let y0 = k as f64 * major;
        for s in 0..sub {
            let y = y0 + major * s as f64 / sub as f64;
            let sy = xf.to_screen(Point::new(0.0, origin.y + y)).y;
            let len = if s == 0 {
                RULER
            } else if sub % 2 == 0 && s == sub / 2 {
                7.0
            } else {
                4.0
            };
            painter.line_segment([pos2(left.max.x - len, sy), pos2(left.max.x, sy)], tick);
            if s == 0 {
                // Stacked digits like InDesign's vertical ruler.
                let label = fmt_tick(unit.from_pt(y));
                for (j, ch) in label.chars().enumerate() {
                    painter.text(pos2(left.min.x + 3.0, sy + 2.0 + j as f32 * 8.5), egui::Align2::LEFT_TOP, ch, font.clone(), t.ruler_text);
                }
            }
        }
        k += 1;
    }
    // Pointer position markers.
    if let Some(p) = ui.ctx().pointer_hover_pos().filter(|p| rect.contains(*p)) {
        let m = Stroke::new(1.0, t.text_dim);
        dashed(&painter, pos2(p.x, top.min.y), pos2(p.x, top.max.y), m, 1.5, 1.5);
        dashed(&painter, pos2(left.min.x, p.y), pos2(left.max.x, p.y), m, 1.5, 1.5);
    }
}

/// The horizontal (top) and vertical (left) rulers around the view `rect` in `full`.
fn ruler_rects(full: Rect, rect: Rect) -> (Rect, Rect) {
    (
        Rect::from_min_max(pos2(rect.min.x, full.min.y), pos2(full.max.x, rect.min.y)),
        Rect::from_min_max(pos2(full.min.x, rect.min.y), pos2(rect.min.x, full.max.y)),
    )
}

/// Right-clicking a ruler opens a menu of units for that ruler.
fn ruler_units_menus(app: &mut DesignApp, resp: &egui::Response, full: Rect, rect: Rect, doc: &Document) {
    let (top, left) = ruler_rects(full, rect);
    let at = resp.interact_pointer_pos();
    for (ruler, key, cur) in [(top, "horizontalUnits", doc.settings.horizontal_units), (left, "verticalUnits", doc.settings.vertical_units)] {
        let open = if resp.secondary_clicked() {
            Some(egui::SetOpenCommand::Bool(at.is_some_and(|p| ruler.contains(p))))
        } else if resp.clicked() {
            Some(egui::SetOpenCommand::Bool(false))
        } else {
            None
        };
        egui::Popup::context_menu(resp).id(resp.id.with(("ruler_units", key))).open_memory(open).show(|ui| ruler_units_menu(app, ui, key, cur));
    }
}

/// Every unit, the current one checked; picking one sets `key` (`horizontalUnits` or
/// `verticalUnits`) through `document.preferences`.
fn ruler_units_menu(app: &mut DesignApp, ui: &mut egui::Ui, key: &str, cur: pagenor_geom::Unit) {
    for u in pagenor_geom::Unit::ALL {
        let label = crate::i18n::tr(&app.ui.language, u.label());
        let text = if u == cur { format!("✓ {label}") } else { format!("   {label}") };
        if ui.button(crate::rtl::widget(ui, text)).clicked() {
            let _ = app.run("document.preferences", json!({ key: u }));
            ui.close();
        }
    }
}

/// A canvas context menu asked for by a right-click; `open` until the popup is open after a frame.
#[derive(Clone, Copy, Debug)]
struct MenuRequest {
    kind: crate::menus::ContextMenu,
    open: bool,
}

/// The popup id of the canvas context menu in split-view pane `pane`.
fn context_menu_id(pane: u8) -> egui::Id {
    egui::Id::new(("canvas_context_menu", pane))
}

/// Was a canvas context menu open (in either pane of this window) when this frame began?
/// Pane 0 shows its menu, which a click or Escape may close, before pane 1 handles input; that
/// click or key still belongs to the menu. Popups are per window (viewport), and so is input.
pub fn context_menu_open(ctx: &egui::Context) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let key = egui::Id::new("canvas_context_menu_open").with(ctx.viewport_id());
    if let Some((p, open)) = ctx.data(|d| d.get_temp::<(u64, bool)>(key))
        && p == pass
    {
        return open;
    }
    let open = (0..2).any(|pane| egui::Popup::is_id_open(ctx, context_menu_id(pane)));
    ctx.data_mut(|d| d.insert_temp(key, (pass, open)));
    open
}

/// Shows the context menu a right-click asked for. A right-click while the menu is open closes it
/// in that frame (it was a click outside the menu), so the request stays until a frame ends with
/// the menu open: it reopens in the next frame where the new click was.
fn context_menu(app: &mut DesignApp, ui: &egui::Ui, resp: &egui::Response) {
    let id = context_menu_id(app.pane);
    let Some(req) = ui.data(|d| d.get_temp::<MenuRequest>(id)) else { return };
    egui::Popup::menu(resp)
        .id(id)
        .open_memory(req.open.then_some(egui::SetOpenCommand::Bool(true)))
        .at_pointer_fixed()
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| crate::menus::context_menu(app, ui, req.kind));
    if req.open {
        if egui::Popup::is_id_open(ui.ctx(), id) {
            ui.data_mut(|d| d.insert_temp(id, MenuRequest { open: false, ..req }));
        } else {
            ui.ctx().request_repaint();
        }
    }
}

/// Prepares a right-click at canvas point `p`: picks the menu and the selection it acts on.
/// - Typing: a click in a text frame places the caret there as a left click would, except
///   inside the selected text, which stays selected; elsewhere the text is left as it is.
/// - On an object: it is selected first unless it already is, as the Selection tool would.
/// - On nothing: the selection stays.
fn context_target(app: &mut DesignApp, p: Point, zoom: f64) -> crate::menus::ContextMenu {
    use crate::menus::ContextMenu;
    let hit = item_at(app, p, zoom);
    if app.session.wants_text() {
        let text_frame = hit.filter(|(id, _)| app.session.active().and_then(|st| st.doc.item(*id)).is_some_and(Item::is_text_frame));
        if let Some((id, sp)) = text_frame {
            // A press in the selected text keeps it for dragging; end that press without a move.
            if app.run("text.placeCaret", json!({"frame": id.0, "point": [sp.x, sp.y]})).is_ok_and(|r| r["dragging"] == json!(true)) {
                let _ = app.run("text.release", json!({"frame": id.0, "point": [sp.x, sp.y], "moved": true}));
            }
        }
        return ContextMenu::Text;
    }
    let Some((id, _)) = hit else { return ContextMenu::Canvas };
    let direct = app.session.tool_id() == "directSelection";
    let Some((target, selected)) = app.session.active().map(|st| {
        let target = if direct { id } else { st.doc.top_level_of(id).unwrap_or(id) };
        (target, st.selection.contains(target))
    }) else {
        return ContextMenu::Canvas;
    };
    if !selected {
        let _ = app.run("selection.set", json!({"ids": [target.0], "content": direct}));
    }
    ContextMenu::Object
}

/// The frontmost item under canvas point `p` and that point in its spread, hit as the tools do.
fn item_at(app: &DesignApp, p: Point, zoom: f64) -> Option<(pagenor_doc::ItemId, Point)> {
    let st = app.session.active()?;
    let (sr, sp) = CanvasLayout::new(&st.doc, st.editing_parents).spread_at(p)?;
    let tol = 4.0 / zoom.max(1e-6);
    let id = match sr {
        SpreadRef::Doc(si) => st.doc.hit_item(si, sp, tol)?,
        _ => st.doc.spread(sr)?.items.iter().rev().find(|it| !it.locked && pagenor_doc::edit_hit(it, sp, tol))?.id,
    };
    Some((id, sp))
}

pub(crate) fn fmt_tick(v: f64) -> String {
    let r = v.round();
    if (v - r).abs() < 1e-6 { format!("{}", r as i64) } else { format!("{v:.1}") }
}

/// Move the view's content by a screen-space distance `d` (rotated views move in screen axes).
fn pan(app: &mut DesignApp, xf: &Xf, d: egui::Vec2) {
    let d = xf.unrotate_delta(d);
    if let Some(v) = app.view_mut() {
        v.origin = Point::new(v.origin.x - d.x as f64 / v.zoom, v.origin.y - d.y as f64 / v.zoom);
    }
}

/// A scrollbar thumb on a track `track` pixels long, for `visible` (the view) within `content`
/// (the pasteboard), both spans along the bar's axis in screen pixels: the thumb's start and
/// length on the track, and the content pixels one thumb pixel moves (0 when nothing scrolls).
fn thumb_span(track: f32, content: (f32, f32), visible: (f32, f32)) -> (f32, f32, f32) {
    let track = track.max(0.0);
    let (lo, hi) = (content.0.min(visible.0), content.1.max(visible.1));
    let (total, shown) = (hi - lo, visible.1 - visible.0);
    let scrollable = total - shown;
    if !total.is_finite() || !shown.is_finite() || total <= 0.0 || scrollable <= 0.0 {
        return (0.0, track, 0.0);
    }
    let len = (track * shown / total).max(MIN_THUMB.min(track));
    let free = track - len;
    if free <= 0.0 {
        return (0.0, track, 0.0);
    }
    ((visible.0 - lo) / scrollable * free, len, scrollable / free)
}

/// The track and thumb of the vertical (or horizontal) scrollbar of the view `screen` inside
/// `outer`, and the content pixels per thumb pixel.
fn scrollbar_rects(outer: Rect, screen: Rect, content: Rect, vertical: bool) -> (Rect, Rect, f32) {
    if vertical {
        let track = Rect::from_min_max(pos2(outer.max.x - SCROLLBAR, screen.min.y), pos2(outer.max.x, screen.max.y));
        let (start, len, k) = thumb_span(track.height(), (content.min.y, content.max.y), (screen.min.y, screen.max.y));
        (track, Rect::from_min_size(pos2(track.min.x, track.min.y + start), vec2(SCROLLBAR, len)), k)
    } else {
        let track = Rect::from_min_max(pos2(screen.min.x, outer.max.y - SCROLLBAR), pos2(screen.max.x, outer.max.y));
        let (start, len, k) = thumb_span(track.width(), (content.min.x, content.max.x), (screen.min.x, screen.max.x));
        (track, Rect::from_min_size(pos2(track.min.x + start, track.min.y), vec2(len, SCROLLBAR)), k)
    }
}

/// The view's scrollbars, on the right and bottom edges of `outer`: they span the pasteboard;
/// dragging a thumb pans, clicking the track pages by one view.
fn scrollbars(app: &mut DesignApp, ui: &mut egui::Ui, outer: Rect, screen: Rect, rect: Rect, t: &Tokens) {
    let content = |app: &DesignApp| {
        let st = app.session.active()?;
        let v = app.view()?;
        let xf = Xf::new(rect, v);
        Some((xf, xf.rect(CanvasLayout::new(&st.doc, st.editing_parents).pasteboard)))
    };
    let Some((xf, cb)) = content(app) else { return };
    let press = ui.input(|i| i.pointer.press_origin());
    let mut hot = [false; 2];
    for (n, vertical) in [true, false].into_iter().enumerate() {
        let (track, thumb, k) = scrollbar_rects(outer, screen, cb, vertical);
        let id = egui::Id::new(("canvas_scrollbar", app.pane, vertical));
        let resp = ui.interact(track, id, Sense::click_and_drag());
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Other,
                true,
                if vertical { "Vertical canvas scrollbar" } else { "Horizontal canvas scrollbar" },
            )
        });
        let grab_id = id.with("grab");
        if resp.drag_started() {
            let on_thumb = press.is_some_and(|p| thumb.contains(p));
            ui.data_mut(|d| d.insert_temp(grab_id, on_thumb));
        }
        let grabbed = resp.dragged() && ui.data(|d| d.get_temp::<bool>(grab_id)).unwrap_or(false);
        if !resp.dragged() {
            ui.data_mut(|d| d.remove::<bool>(grab_id));
        }
        let along = |p: egui::Vec2| if vertical { p.y } else { p.x };
        let page = along(screen.size());
        let mut moved = 0.0;
        if grabbed {
            moved = -along(resp.drag_delta()) * k;
        } else if resp.clicked()
            && let Some(p) = resp.interact_pointer_pos()
        {
            let (p, a, b) = if vertical { (p.y, thumb.min.y, thumb.max.y) } else { (p.x, thumb.min.x, thumb.max.x) };
            if p < a {
                moved = page;
            } else if p > b {
                moved = -page;
            }
        }
        if moved != 0.0 && moved.is_finite() {
            pan(app, &xf, if vertical { vec2(0.0, moved) } else { vec2(moved, 0.0) });
        }
        if let Some(slot) = hot.get_mut(n) {
            *slot = grabbed || resp.hovered();
        }
    }
    // Painted from the view as it is after this frame's input.
    let Some((_, cb)) = content(app) else { return };
    let painter = ui.painter_at(outer);
    let right = Rect::from_min_max(pos2(outer.max.x - SCROLLBAR, outer.min.y), outer.max);
    let bottom = Rect::from_min_max(pos2(outer.min.x, outer.max.y - SCROLLBAR), outer.max);
    painter.rect_filled(right, 0.0, t.panel_darker);
    painter.rect_filled(bottom, 0.0, t.panel_darker);
    for (n, vertical) in [true, false].into_iter().enumerate() {
        let (_, thumb, _) = scrollbar_rects(outer, screen, cb, vertical);
        let alpha = if hot.get(n).copied().unwrap_or(false) { 0.95 } else { 0.7 };
        painter.rect_filled(thumb.shrink(2.0), SCROLLBAR / 2.0 - 2.0, t.text.gamma_multiply(alpha));
    }
}

// Re-evaluate egui's click region at this event's position, using the same previous-pass
// geometry. A frame's Response may already reflect a later move out of the widget.
fn canvas_pointer_left_widget(ctx: &egui::Context, id: egui::Id, p: Pos2) -> bool {
    let widgets = ctx.viewport(|v| v.prev_pass.widgets.clone());
    let Some(canvas) = widgets.get(id) else { return false };
    if !canvas.enabled || !canvas.sense.senses_click() || !canvas.sense.senses_drag() {
        return false;
    }
    let radius = ctx.global_style().interaction.interact_radius;
    let (order, transforms) = ctx.memory(|m| (m.layer_ids().collect::<Vec<_>>(), m.to_global.clone()));
    let mut layers = widgets.layer_ids().collect::<Vec<_>>();
    layers.sort_by_key(|layer| (layer.order, order.iter().position(|entry| entry == layer)));
    let mut close = Vec::new();
    for layer in layers {
        if !layer.order.allow_interaction() || egui::containers::AreaState::load(ctx, layer.id).is_some_and(|area| !area.interactable) {
            continue;
        }
        let transform = transforms.get(&layer).copied().unwrap_or(egui::emath::TSTransform::IDENTITY);
        let local = transform.inverse() * p;
        for widget in widgets.get_layer(layer) {
            if widget.interact_rect.is_negative() || widget.rect.any_nan() || widget.interact_rect.any_nan() {
                continue;
            }
            if widget.interact_rect.distance_sq_to_pos(local) <= radius * radius {
                let widget = widget.transform(transform);
                if !widget.rect.any_nan() && !widget.interact_rect.any_nan() {
                    close.push(widget);
                }
            }
        }
    }
    // Preserve egui's layer occlusion and nearby-hit margin, including drag-only neighbors.
    let mut included = Vec::new();
    for widget in close.iter().rev() {
        included.push(widget.layer_id);
        if widget.interact_rect.shrink(radius).contains(p) {
            break;
        }
    }
    close.retain(|widget| included.contains(&widget.layer_id));
    let hidden = close
        .iter()
        .enumerate()
        .filter_map(|(i, widget)| {
            close
                .iter()
                .skip(i + 1)
                .any(|above| above.layer_id != widget.layer_id && above.interact_rect.contains_rect(widget.interact_rect))
                .then_some(widget.id)
        })
        .collect::<Vec<_>>();
    close.retain(|widget| !hidden.contains(&widget.id));
    if close.iter().any(|widget| widget.id == id && widget.interact_rect.contains(p)) {
        return false;
    }
    let closest = |click: bool, drag: bool, limit: f32| {
        let mut best = None;
        let mut distance = limit;
        for widget in &close {
            if widget.enabled && ((click && widget.sense.senses_click()) || (drag && widget.sense.senses_drag())) {
                let d = widget.interact_rect.distance_sq_to_pos(p);
                if d <= distance {
                    distance = d;
                    best = Some(*widget);
                }
            }
        }
        best
    };
    if closest(true, false, 0.0).is_some() {
        return true; // A direct click hit outside the canvas cannot be this canvas.
    }
    let click = closest(true, false, f32::INFINITY);
    if let Some(drag) = closest(false, true, 0.0) {
        return !click.is_some_and(|widget| widget.id == id && drag.interact_rect.contains_rect(widget.interact_rect));
    }
    !closest(true, true, f32::INFINITY).is_some_and(|widget| widget.id == id)
}

#[cfg(test)]
#[path = "canvas/tests_control_modifiers.rs"]
mod tests_control_modifiers;
#[cfg(test)]
#[path = "canvas/tests_text_selection.rs"]
mod tests_text_selection;

fn handle_input(app: &mut DesignApp, ui: &mut egui::Ui, resp: &egui::Response, rect: Rect) {
    // While a context menu is open (in either pane), the click that closes it and the keys go to the menu.
    let menu_open = context_menu_open(ui.ctx());
    let Some(v) = app.view().copied() else { return };
    let xf = Xf::new(rect, &v);
    let wants_text = app.session.wants_text();
    let space = ui.input(|i| i.key_down(egui::Key::Space)) && !wants_text && !ui.ctx().text_edit_focused();
    // Scroll / zoom.
    let alt = wheel_alt(ui);
    if resp.hovered() {
        let (scroll, zoom_delta, m, hover) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta(), i.modifiers, i.pointer.hover_pos()));
        if let Some(hp) = hover {
            if zoom_delta != 1.0 {
                zoom_at(app, hp, zoom_delta as f64);
            } else if m.command && scroll.y != 0.0 {
                zoom_at(app, hp, (1.0 + scroll.y as f64 * 0.003).clamp(0.5, 2.0));
            } else if alt && scroll != egui::Vec2::ZERO {
                // Alt + wheel zooms by as much as Ctrl/Cmd + wheel, which egui turns into `zoom_delta`.
                let speed = ui.ctx().options(|o| o.input_options.scroll_zoom_speed);
                zoom_at(app, hp, (speed * (scroll.x + scroll.y)).exp() as f64);
            } else if scroll != egui::Vec2::ZERO
                && let Some(v) = app.view_mut()
            {
                let scroll = xf.unrotate_delta(scroll);
                v.origin = Point::new(v.origin.x - scroll.x as f64 / v.zoom, v.origin.y - scroll.y as f64 / v.zoom);
            }
        }
    }
    // Space-drag = hand (unless a tool drag is under way: then Space is a modifier, e.g. Live
    // Distribute while resizing). Middle-drag pans too, with any tool; tools only
    // see the primary button.
    let tool_drag: bool = ui.data(|d| d.get_temp(egui::Id::new(("canvas_pointer_down", app.pane)))).unwrap_or(false);
    if !tool_drag && ((space && resp.dragged()) || resp.dragged_by(egui::PointerButton::Middle)) {
        let d = xf.unrotate_delta(resp.drag_delta());
        if let Some(v) = app.view_mut() {
            v.origin = Point::new(v.origin.x - d.x as f64 / v.zoom, v.origin.y - d.y as f64 / v.zoom);
        }
        return;
    }
    // Ruler zero-point and guides need the rulers (hidden while the view is rotated).
    if !space && !menu_open && xf.rot == 0 && ruler_origin_drag(app, ui, resp, rect, &xf) {
        return;
    }
    if !space && !menu_open && xf.rot == 0 && guide_drag(app, ui, resp, rect, &xf) {
        return;
    }
    // Dispatch physical pointer, text and key events in their original order. The
    // frame-wide pointer summary can already include a move after the click ended.
    let modifiers_id = egui::Id::new(("canvas_modifiers", app.pane));
    let frame_modifiers = ui.input(|i| mods(i, space));
    let mut m: Mods = ui.data(|d| d.get_temp(modifiers_id)).unwrap_or(frame_modifiers);
    m.space = space;
    let send = |app: &mut DesignApp, kind, p, mods, clicks| {
        let Some(v) = app.view().copied() else { return };
        let event = PointerEvent { kind, pos: Xf::new(rect, &v).to_canvas(p), mods, clicks };
        if let Err(err) = app.session.pointer(&event, app.view_info()) {
            app.status(err.to_string());
        }
        app.after_engine();
    };
    let down_id = egui::Id::new(("canvas_pointer_down", app.pane));
    let gesture_id = egui::Id::new(("canvas_pointer_gesture", app.pane));
    let clicks_id = egui::Id::new(("canvas_clicks", app.pane));
    let hold_id = egui::Id::new(("canvas_hold", app.pane));
    let alt_hand_id = egui::Id::new(("canvas_alt_hand", app.pane));
    let mut alt_hand: bool = ui.data(|d| d.get_temp(alt_hand_id)).unwrap_or(false);
    let mut down: bool = ui.data(|d| d.get_temp(down_id)).unwrap_or(false);
    let mut gesture: Option<(Pos2, Pos2, f64, bool)> = ui.data(|d| d.get_temp(gesture_id));
    let mut clicks: Option<(f64, Pos2, u8)> = ui.data(|d| d.get_temp(clicks_id));
    let (dist, duration, delay) =
        ui.ctx().options(|o| (o.input_options.max_click_dist, o.input_options.max_click_duration, o.input_options.max_double_click_delay));
    let (now, events, mut focused) = ui.input(|i| (i.time, i.events.clone(), i.focused));
    let in_view = |p| if xf.rot == 0 { rect.contains(p) } else { resp.rect.contains(p) };
    // A quiet repaint must keep the caret's native text-service focus too.
    if app.session.wants_text()
        && !menu_open
        && app.ui.dialog.is_none()
        && app.ui.palette.is_none()
        && !egui::Popup::is_any_open(ui.ctx())
        && app.pane == app.focus_pane
        && focused
        && (!ui.ctx().text_edit_focused() || resp.has_focus())
    {
        if !resp.has_focus() {
            resp.request_focus();
        }
        ui.ctx().memory_mut(|m| {
            m.set_focus_lock_filter(resp.id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true })
        });
    }
    for event in events {
        match event {
            egui::Event::ModifiersChanged(modifiers) => {
                m = Mods { shift: modifiers.shift, alt: modifiers.alt, cmd: modifiers.command, ctrl: modifiers.ctrl, space };
                continue;
            }
            egui::Event::WindowFocused(false) => {
                focused = false;
                m = Mods::default();
                continue;
            }
            egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers } => {
                m = Mods { shift: modifiers.shift, alt: modifiers.alt, cmd: modifiers.command, ctrl: modifiers.ctrl, space };
                if pressed {
                    if !space && !menu_open && app.ui.dialog.is_none() && in_view(p) && !canvas_pointer_left_widget(ui.ctx(), resp.id, p) {
                        app.focus_pane = app.pane;
                        let n =
                            clicks.filter(|(at, last, _)| now - at <= delay && p.distance(*last) <= dist).map_or(1, |(_, _, n)| n.saturating_add(1));
                        alt_hand = m.alt
                            && app.session.wants_text()
                            && !down
                            && app.session.active().is_none_or(|s| s.selection.text.is_none_or(|t| t.range().is_empty()));
                        clicks = Some((f64::NEG_INFINITY, p, n));
                        gesture = Some((p, p, now, false));
                        down = true;
                        if !alt_hand {
                            send(app, PointerKind::Down, p, m, n);
                        }
                        ui.data_mut(|d| d.remove::<(f64, bool)>(hold_id));
                        if !resp.has_focus() {
                            resp.request_focus();
                        }
                    } else {
                        clicks = None;
                    }
                } else if down {
                    if let Some((origin, last, at, dragging)) = gesture {
                        let moved = p.distance(origin) > dist;
                        if alt_hand {
                            if p != last {
                                pan(app, &xf, p - last);
                            }
                        } else {
                            if p != last && (dragging || moved) {
                                send(app, PointerKind::Drag, p, m, 1);
                            }
                            send(app, PointerKind::Up, p, m, 1);
                        }
                        if !alt_hand && !dragging && !moved && now - at <= duration {
                            if let Some((_, press, n)) = clicks {
                                clicks = Some((now, press, n));
                            }
                        } else {
                            clicks = None;
                        }
                    }
                    down = false;
                    alt_hand = false;
                    gesture = None;
                    ui.data_mut(|d| d.remove::<(f64, bool)>(hold_id));
                }
                continue;
            }
            egui::Event::PointerMoved(p) => {
                if down {
                    if let Some((origin, last, at, dragging)) = gesture.as_mut() {
                        *dragging |= p.distance(*origin) > dist || now - *at > duration || canvas_pointer_left_widget(ui.ctx(), resp.id, p);
                        if alt_hand {
                            if p != *last {
                                pan(app, &xf, p - *last);
                            }
                        } else if *dragging && p != *last {
                            send(app, PointerKind::Drag, p, m, 1);
                        }
                        *last = p;
                    }
                } else if !menu_open && in_view(p) && !canvas_pointer_left_widget(ui.ctx(), resp.id, p) {
                    send(app, PointerKind::Move, p, m, 1);
                }
                continue;
            }
            _ => {}
        }
        // Keys belong to the focused pane and the current caret, which a preceding
        // click in this same event batch may have just created or removed.
        if menu_open
            || (ui.ctx().text_edit_focused() && !resp.has_focus())
            || app.ui.dialog.is_some()
            || app.ui.palette.is_some()
            || egui::Popup::is_any_open(ui.ctx())
            || app.pane != app.focus_pane
            || !focused
        {
            continue;
        }
        let wants_text = app.session.wants_text();
        if wants_text {
            if !resp.has_focus() {
                resp.request_focus();
            }
            ui.ctx().memory_mut(|m| {
                m.set_focus_lock_filter(resp.id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true })
            });
        } else {
            app.cancel_canvas_composition(ui.ctx());
        }
        app.validate_canvas_composition(ui.ctx());
        let composing = app.canvas_composing(ui.ctx());
        match event {
            egui::Event::Ime(event) if wants_text => {
                app.canvas_ime_event(ui.ctx(), &event);
            }
            egui::Event::Text(t) if wants_text && !app.canvas_ime.active() => {
                let _ = app.run("text.insert", json!({"text": t}));
            }
            egui::Event::Paste(t) if wants_text => {
                // Formatted when the system clipboard still holds what was copied here;
                // Shift (⇧⌘V) pastes without formatting.
                let plain = m.shift;
                let id = if plain { "edit.pasteWithoutFormatting" } else { "edit.paste" };
                let _ = app.run(id, json!({"text": t}));
            }
            egui::Event::Copy | egui::Event::Cut if wants_text => {
                // `run` puts the copied text on the system clipboard.
                let id = if matches!(event, egui::Event::Cut) { "edit.cut" } else { "edit.copy" };
                let _ = app.run(id, json!({}));
            }
            // Without a native menu bar the copy and paste keys arrive only as these events.
            egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_) if !app.native_menu => {
                let id = match event {
                    egui::Event::Copy => "edit.copy",
                    egui::Event::Cut => "edit.cut",
                    _ => "edit.paste",
                };
                if crate::menus::enabled(app, id) {
                    crate::menus::activate(app, id, &serde_json::Value::Null);
                }
            }
            egui::Event::Key { key, pressed: true, modifiers, .. } if !composing => {
                let k = match key {
                    egui::Key::ArrowLeft => Some(ToolKey::Left),
                    egui::Key::ArrowRight => Some(ToolKey::Right),
                    egui::Key::ArrowUp => Some(ToolKey::Up),
                    egui::Key::ArrowDown => Some(ToolKey::Down),
                    egui::Key::Enter => Some(ToolKey::Enter),
                    egui::Key::Escape => Some(ToolKey::Escape),
                    egui::Key::Backspace => Some(ToolKey::Backspace),
                    egui::Key::Delete => Some(ToolKey::Delete),
                    egui::Key::Tab => Some(ToolKey::Tab),
                    egui::Key::Home => Some(ToolKey::Home),
                    egui::Key::End => Some(ToolKey::End),
                    _ => None,
                };
                if let Some(k) = k {
                    let m = Mods { shift: modifiers.shift, alt: modifiers.alt, cmd: modifiers.command, ctrl: modifiers.ctrl, space: false };
                    let handled = app.session.tool_key(k, m, app.view_info()).unwrap_or(false);
                    if !handled && k == ToolKey::Escape {
                        let _ = app.run("edit.deselectAll", json!({}));
                    }
                    app.after_engine();
                }
            }
            _ => {}
        }
    }
    ui.data_mut(|d| {
        d.insert_temp(down_id, down);
        d.insert_temp(alt_hand_id, alt_hand);
        d.insert_temp(modifiers_id, m);
        if let Some(g) = gesture {
            d.insert_temp(gesture_id, g);
        } else {
            d.remove::<(Pos2, Pos2, f64, bool)>(gesture_id);
        }
        if let Some(c) = clicks {
            d.insert_temp(clicks_id, c);
        } else {
            d.remove::<(f64, Pos2, u8)>(clicks_id);
        }
    });
    // Power Zoom keeps its half-second hold gesture with the Hand tool.
    if down && !alt_hand && app.session.tool_id() == "hand" && !m.alt {
        let (start, fired): (f64, bool) = ui.data(|d| d.get_temp(hold_id)).unwrap_or((now, false));
        let still = gesture.is_some_and(|(origin, last, _, _)| origin.distance(last) < 3.0);
        if !fired && still && now - start >= 0.5 {
            let p = gesture.map_or(rect.center(), |(_, last, _, _)| last);
            send(app, PointerKind::Up, p, m, 1);
            send(app, PointerKind::Down, p, Mods { alt: true, ..m }, 1);
            ui.data_mut(|d| d.insert_temp(hold_id, (start, true)));
        } else {
            ui.data_mut(|d| d.insert_temp(hold_id, (start, fired)));
            if !fired {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
    } else {
        ui.data_mut(|d| d.remove::<(f64, bool)>(hold_id));
    }
    if !down && let Some(p) = resp.hover_pos() {
        send(app, PointerKind::Move, p, m, 1);
    }
    // Secondary clicks open the context menu without starting a tool gesture.
    if resp.secondary_clicked()
        && !down
        && !space
        && app.ui.screen_mode != ScreenMode::Presentation
        && let Some(p) = resp.interact_pointer_pos().filter(|p| in_view(*p))
    {
        let kind = context_target(app, Xf::new(rect, &app.view().copied().unwrap_or(v)).to_canvas(p), xf.zoom);
        ui.data_mut(|d| d.insert_temp(context_menu_id(app.pane), MenuRequest { kind, open: true }));
        ui.ctx().request_repaint();
    }
    let _ = DVec2::ZERO;
}

/// Enable the host's normal text service at the actual caret, including cells and notes.
fn publish_canvas_ime(app: &DesignApp, ui: &egui::Ui, resp: &egui::Response, xf: &Xf, doc: &Document, layout: &CanvasLayout) {
    if !resp.has_focus() || app.pane != app.focus_pane || app.ui.dialog.is_some() || app.ui.palette.is_some() || egui::Popup::is_any_open(ui.ctx()) {
        return;
    }
    let Some(st) = app.session.active() else { return };
    let Some(ts) = app.composition_selection().or(st.selection.text).filter(|_| st.selection.cells.is_none()) else { return };
    let cs = app.session.cache.get(doc, ts.story, None);
    let caret = match ts.cell {
        Some(cell) => match cell.footnote_id() {
            Some(id) => compose::note_caret(&cs, id, ts.focus),
            None => compose::cell_caret(&cs, cell.table, cell.row, cell.col, ts.focus),
        },
        None => compose::caret(&cs, ts.focus),
    };
    let cursor = if let Some((fi, x, bl, asc, desc)) = caret
        && let Some(ft) = cs.frames.get(fi)
        && let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame))
    {
        let m = a * doc.text_xf(it);
        Rect::from_two_pos(xf.to_screen(m * Point::new(x, bl - asc)), xf.to_screen(m * Point::new(x, bl + desc))).expand(0.5)
    } else {
        // Overset/empty stories still need the text service to accept input.
        let p = ui.clip_rect().left_top() + vec2(4.0, 4.0);
        Rect::from_min_size(p, vec2(1.0, 16.0))
    };
    let to_global = ui.ctx().layer_transform_to_global(ui.layer_id()).unwrap_or_default();
    ui.output_mut(|o| {
        o.ime = Some(egui::output::IMEOutput {
            purpose: egui::IMEPurpose::Normal,
            rect: to_global * cursor,
            cursor_rect: to_global * cursor,
            should_interrupt_composition: false,
        });
    });
}

/// View → Show Hidden Characters: ¶ paragraph ends, » tabs, · spaces, ¬ forced line breaks, # end of story.
fn draw_hidden_characters(app: &DesignApp, painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout) {
    for story in doc.stories.values() {
        let cs = app.session.cache.get(doc, story.id, None);
        for ft in &cs.frames {
            let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame)) else { continue };
            let m = a * doc.text_xf(it);
            let col = layer_color(doc, it);
            for l in &ft.lines {
                let size = (l.ascent * 0.75 * xf.zoom).clamp(6.0, 40.0) as f32;
                let font = egui::FontId::proportional(size);
                for g in &l.glyphs {
                    if g.len == 0 {
                        continue;
                    }
                    let ch = story.text[g.byte.min(story.text.len())..].chars().next().unwrap_or(' ');
                    let mark = match ch {
                        ' ' => "·",
                        '\t' => "»",
                        '\u{2028}' => "¬",
                        '\u{A0}' => "°",
                        _ => continue,
                    };
                    let p = xf.to_screen(m * Point::new(g.x + if ch == ' ' { g.adv / 2.0 } else { 0.0 }, l.baseline));
                    let align = if ch == ' ' { egui::Align2::CENTER_BOTTOM } else { egui::Align2::LEFT_BOTTOM };
                    painter.text(p, align, mark, font.clone(), col);
                }
                if l.last_in_para {
                    let end = l.range.end;
                    let mark = if end >= story.text.len() { "#" } else { "¶" };
                    let p = xf.to_screen(m * Point::new(l.end_x + 1.0, l.baseline));
                    painter.text(p, egui::Align2::LEFT_BOTTOM, mark, font.clone(), col);
                }
            }
        }
    }
}

/// View › Flattener Preview: red over every object that involves transparency (opacity, blend
/// modes, effects, knockout) — what flattening or export rasterising affects.
fn draw_flattener_preview(painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout) {
    fn transparent(it: &Item) -> bool {
        it.opacity < 1.0 || it.blend != Default::default() || it.effects.any() || it.knockout || it.children().iter().any(|c| transparent(c))
    }
    let red = Color32::from_rgba_unmultiplied(230, 30, 30, 110);
    for slot in &layout.slots {
        let Some(sp) = doc.spread(slot.spread) else { continue };
        for it in sp.items.iter().filter(|it| !it.hidden && transparent(it)) {
            let r = it.bounds().inflate(pagenor_render::effect_outset(it), pagenor_render::effect_outset(it));
            painter.rect_filled(xf.rect(slot.xf.transform_rect_bbox(r)), 0.0, red);
        }
    }
}

/// Edit › Spelling › Dynamic Spelling: red squiggles under misspelled words (checked once per
/// story version).
fn draw_dynamic_spelling(app: &DesignApp, ctx: &egui::Context, painter: &egui::Painter, xf: &Xf, doc: &Document, layout: &CanvasLayout) {
    for story in doc.stories.values() {
        let key = egui::Id::new(("dyn_spell", std::sync::Arc::as_ptr(story) as usize, doc.user_words.len()));
        let bad: std::sync::Arc<Vec<std::ops::Range<usize>>> = match ctx.data(|d| d.get_temp(key)) {
            Some(v) => v,
            None => {
                let v = std::sync::Arc::new(pagenor_engine::cmd::spelling::misspellings(doc, story, &doc.user_words));
                ctx.data_mut(|d| d.insert_temp(key, v.clone()));
                v
            }
        };
        if bad.is_empty() {
            continue;
        }
        let cs = app.session.cache.get(doc, story.id, None);
        let red = Stroke::new(1.0, Color32::from_rgb(230, 30, 30));
        for ft in &cs.frames {
            let (Some((a, _)), Some(it)) = (item_canvas_xf(doc, layout, ft.frame), doc.item(ft.frame)) else { continue };
            let m = a * doc.text_xf(it);
            for l in &ft.lines {
                for r in bad.iter().filter(|r| r.start < l.range.end && r.end > l.range.start) {
                    let gs: Vec<_> = l.glyphs.iter().filter(|g| g.len > 0 && g.byte >= r.start && g.byte < r.end).collect();
                    let (Some(x0), Some(x1)) = (gs.iter().map(|g| g.x).reduce(f64::min), gs.iter().map(|g| g.x + g.adv).reduce(f64::max)) else {
                        continue;
                    };
                    let y = l.baseline + l.descent * 0.5;
                    let (p0, p1) = (xf.to_screen(m * Point::new(x0, y)), xf.to_screen(m * Point::new(x1, y)));
                    // A zigzag 2 px high, 4 px per wave.
                    let n = ((p1.x - p0.x) / 2.0).max(1.0) as usize;
                    let pts: Vec<Pos2> =
                        (0..=n).map(|k| pos2(p0.x + (p1.x - p0.x) * k as f32 / n as f32, p0.y + if k % 2 == 0 { 0.0 } else { 2.0 })).collect();
                    painter.add(egui::Shape::line(pts, red));
                }
            }
        }
    }
}

/// Plain text of the current text selection (story markers resolved to readable characters).
pub fn selected_text(app: &DesignApp) -> Option<String> {
    let st = app.session.active()?;
    let t = st.selection.text?;
    let story = st.doc.text_story(t.story, t.cell)?;
    let s = story.slice(t.range());
    if s.is_empty() {
        return None;
    }
    Some(s.replace(pagenor_doc::story::FORCED_LINE_BREAK, "\n"))
}

/// One device pixel, the width InDesign uses for guides and frame edges.
fn hair(painter: &egui::Painter) -> f32 {
    1.0 / painter.ctx().pixels_per_point()
}

const RULER_ORIGIN_DRAG: &str = "canvas_ruler_origin_drag";

/// Dragging the ruler zero-point crosshair (the corner box where the two rulers meet): sets a
/// custom ruler origin live as the pointer moves. Double-clicking it resets to the default
/// origin (the current spread's first page). Returns true while it owns the pointer.
fn ruler_origin_drag(app: &mut DesignApp, ui: &mut egui::Ui, resp: &egui::Response, rect: Rect, xf: &Xf) -> bool {
    let id = egui::Id::new((RULER_ORIGIN_DRAG, app.pane));
    let corner = Rect::from_min_max(resp.rect.min, rect.min);
    if corner.width() <= 0.0 || corner.height() <= 0.0 {
        return false;
    }
    let (pressed, released, press_origin, latest, dbl) = ui.input(|i| {
        let origin = i.pointer.press_origin().or_else(|| {
            i.events.iter().find_map(|e| match e {
                egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, .. } => Some(*pos),
                _ => None,
            })
        });
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            origin,
            i.pointer.latest_pos(),
            i.pointer.button_double_clicked(egui::PointerButton::Primary),
        )
    });
    let active: Option<(Pos2, bool)> = ui.data(|d| d.get_temp(id));
    let (start, mut moved) = match active {
        Some(state) => state,
        None if pressed => {
            let Some(start) = press_origin.filter(|o| corner.contains(*o)) else { return false };
            (start, false)
        }
        None => return false,
    };
    // Double-click completes (fires on release): reset to the default origin instead of
    // leaving it at the clicked point.
    if released && dbl {
        if let Some(v) = app.view_mut() {
            v.ruler_origin = None;
        }
        ui.data_mut(|d| d.remove::<(Pos2, bool)>(id));
        return true;
    }
    if let Some(p) = latest {
        moved |= (p - start).length() >= 3.0;
        if moved && let Some(v) = app.view_mut() {
            v.ruler_origin = Some(xf.to_canvas(p));
        }
    }
    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    if released {
        ui.data_mut(|d| d.remove::<(Pos2, bool)>(id));
    } else {
        ui.data_mut(|d| d.insert_temp(id, (start, moved)));
    }
    true
}

const GUIDE_DRAG: &str = "canvas_guide_drag";

/// A ruler guide being dragged: out of a ruler (`from` None) or an existing one.
#[derive(Clone, Debug)]
struct GuideDrag {
    orientation: pagenor_doc::Orientation,
    from: Option<(pagenor_doc::SpreadRef, usize, usize)>,
    /// Pointer position (screen).
    at: Option<Pos2>,
}

/// The guide under a screen point (within 3 px), as (spread, page, index, orientation).
fn guide_at(app: &DesignApp, xf: &Xf, p: Pos2) -> Option<(pagenor_doc::SpreadRef, usize, usize, pagenor_doc::Orientation)> {
    let st = app.session.active()?;
    let layout = CanvasLayout::new(&st.doc, st.editing_parents);
    let cp = xf.to_canvas(p);
    let tol = 3.0 / xf.zoom;
    for slot in &layout.slots {
        let sp = st.doc.spread(slot.spread)?;
        let local = slot.to_spread(cp);
        for (pi, pg) in sp.pages.iter().enumerate() {
            let pr = if pg.guides.iter().any(|g| g.spread) { slot.xf.inverse().transform_rect_bbox(layout.pasteboard) } else { pg.bounds() };
            for (gi, g) in pg.guides.iter().enumerate() {
                if !g.editable_in(&st.doc) {
                    continue;
                }
                let r = if g.spread { pr } else { pg.bounds() };
                let hit = match g.orientation {
                    pagenor_doc::Orientation::Horizontal => (local.y - g.position).abs() <= tol && local.x >= r.x0 && local.x <= r.x1,
                    pagenor_doc::Orientation::Vertical => (local.x - g.position).abs() <= tol && local.y >= r.y0 - tol && local.y <= r.y1 + tol,
                };
                if hit && !g.locked {
                    return Some((slot.spread, pi, gi, g.orientation));
                }
            }
        }
    }
    None
}

/// Ruler guides: drag one out of a ruler, drag an existing one (Selection tools, not over an
/// object) to move it, drop it back on a ruler to delete it. Returns true while it owns the pointer.
fn guide_drag(app: &mut DesignApp, ui: &mut egui::Ui, resp: &egui::Response, rect: Rect, xf: &Xf) -> bool {
    let id = egui::Id::new(GUIDE_DRAG);
    let (pressed, released, origin, latest) =
        ui.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_released(), i.pointer.press_origin(), i.pointer.latest_pos()));
    let mut drag: Option<GuideDrag> = ui.data(|d| d.get_temp(id));
    if drag.is_none() && pressed && resp.hovered() && app.ui.guides && !app.ui.guides_locked {
        let Some(o) = origin else { return false };
        let full = resp.rect;
        if full.contains(o) && !rect.contains(o) {
            // In a ruler (not the corner).
            let orientation = if o.y < rect.min.y && o.x >= rect.min.x {
                Some(pagenor_doc::Orientation::Horizontal)
            } else if o.x < rect.min.x && o.y >= rect.min.y {
                Some(pagenor_doc::Orientation::Vertical)
            } else {
                None
            };
            if let Some(orientation) = orientation {
                drag = Some(GuideDrag { orientation, from: None, at: Some(o) });
            }
        } else if rect.contains(o)
            && matches!(app.session.tool_id(), "selection" | "directSelection")
            && let Some((r, pi, gi, orientation)) = guide_at(app, xf, o)
        {
            // Objects take precedence over the guides behind them.
            let over_item = app.session.active().is_some_and(|st| {
                let layout = CanvasLayout::new(&st.doc, st.editing_parents);
                layout.spread_at(xf.to_canvas(o)).is_some_and(|(sr, p)| match sr {
                    pagenor_doc::SpreadRef::Doc(si) => st.doc.hit_item(si, p, 3.0 / xf.zoom).is_some(),
                    _ => false,
                })
            });
            if !over_item {
                drag = Some(GuideDrag { orientation, from: Some((r, pi, gi)), at: Some(o) });
            }
        }
    }
    let Some(mut g) = drag else { return false };
    if let Some(p) = latest {
        g.at = Some(p);
    }
    if !released {
        ui.ctx().set_cursor_icon(match g.orientation {
            pagenor_doc::Orientation::Horizontal => egui::CursorIcon::ResizeVertical,
            pagenor_doc::Orientation::Vertical => egui::CursorIcon::ResizeHorizontal,
        });
        ui.data_mut(|d| d.insert_temp(id, g));
        return true;
    }
    ui.data_mut(|d| d.remove::<GuideDrag>(id));
    let Some(p) = g.at else { return true };
    let on_canvas = rect.contains(p);
    let target = app.session.active().and_then(|st| {
        let layout = CanvasLayout::new(&st.doc, st.editing_parents);
        let (r, sp) = layout.spread_at(xf.to_canvas(p))?;
        let spread = st.doc.spread(r)?;
        let on_page = spread.pages.iter().position(|pg| pg.bounds().contains(sp));
        Some((r, sp, on_page))
    });
    let pos_of = |sp: Point| match g.orientation {
        pagenor_doc::Orientation::Horizontal => sp.y,
        pagenor_doc::Orientation::Vertical => sp.x,
    };
    let orient = match g.orientation {
        pagenor_doc::Orientation::Horizontal => "horizontal",
        pagenor_doc::Orientation::Vertical => "vertical",
    };
    match (g.from, on_canvas, target) {
        // Dropped back on a ruler: delete.
        (Some((r, pi, gi)), false, _) => {
            let _ = app.run("guide.delete", json!({"spread": r, "page": pi, "index": gi}));
        }
        (Some((r, pi, gi)), true, Some((tr, sp, _))) if tr == r => {
            let _ = app.run("guide.move", json!({"spread": r, "page": pi, "index": gi, "position": pos_of(sp)}));
        }
        (None, true, Some((r, sp, on_page))) => {
            let mut params = json!({"spread": r, "orientation": orient, "position": pos_of(sp), "at": sp.x, "spreadGuide": on_page.is_none()});
            if let Some(pi) = on_page {
                params["page"] = json!(pi);
            }
            let _ = app.run("guide.add", params);
        }
        _ => {}
    }
    true
}

#[cfg(test)]
mod tests {
    use egui_kittest::{Harness, kittest::Queryable};
    use pagenor_geom::Unit;

    use super::*;

    fn harness(app: DesignApp) -> Harness<'static, DesignApp> {
        let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h
    }

    /// Like `harness`, but with a tiny per-step time delta so a rapid click sequence stays
    /// within egui's double-click window.
    fn fast_harness(app: DesignApp) -> Harness<'static, DesignApp> {
        let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).with_step_dt(0.001).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h
    }

    #[test]
    fn master_side_navigation_and_return_preserve_document_view() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages":6})).unwrap();
        app.run("layout.parents.new", json!({"prefix":"B","name":"Book"})).unwrap();
        app.run("app.masterPages", json!({})).unwrap();
        let mut h = harness(app);
        go_to_page(h.state_mut(), 3);
        let document_view = *h.state().view().unwrap();
        h.get_by_label("B-Book Right Master").click();
        h.run_steps(3);
        assert_eq!(current_spread(h.state()), Some(SpreadRef::Parent(1)));
        let v = h.state().view().unwrap();
        let rect = h.state().canvas_rect.unwrap();
        let c = Point::new(v.origin.x + rect.width() as f64 / 2.0 / v.zoom, v.origin.y + rect.height() as f64 / 2.0 / v.zoom);
        let st = h.state().session.active().unwrap();
        let layout = CanvasLayout::new(&st.doc, true);
        let sp = &st.doc.parents[1];
        let pi = sp.page_at_x(layout.slot(SpreadRef::Parent(1)).unwrap().to_spread(c).x).unwrap();
        assert_eq!(sp.pages[pi].side, PageSide::Right);
        h.state_mut().run("layout.parents.edit", json!({"on":false})).unwrap();
        h.run_steps(2);
        assert_eq!(h.state().view().unwrap().origin, document_view.origin);
        assert_eq!(current_page(h.state()), Some(3));
    }

    #[test]
    fn master_layout_dialogs_and_guide_commands_target_the_master_in_view() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages":4})).unwrap();
        app.run("layout.parents.new", json!({"prefix":"B","name":"Book","applyAll":true})).unwrap();
        let mut h = harness(app);
        h.state_mut().run("app.editMaster", json!({"index":1})).unwrap();
        h.run_steps(2);
        h.state_mut().run("layout.createGuides", json!({"rows":2})).unwrap();
        h.state_mut().run("layout.marginsAndColumns", json!({"margins":52,"side":"all"})).unwrap();
        let doc = &h.state().session.active().unwrap().doc;
        assert!(doc.parents[0].pages.iter().all(|p| p.guides.is_empty()));
        assert!(doc.parents[1].pages.iter().all(|p| p.guides.len() == 2 && p.margins.top == 52.0));
        assert!(doc.spreads.iter().flat_map(|sp| &sp.pages).all(|p| p.guides.is_empty() && p.margins.top == 52.0));
        h.state_mut().run("app.deleteAllGuides", json!({})).unwrap();
        assert!(h.state().session.active().unwrap().doc.parents[1].pages.iter().all(|p| p.guides.is_empty()));
        h.state_mut().run("app.masterSettings", json!({"index":1,"side":"left"})).unwrap();
        h.run_steps(2);
        h.get_by_label("Master Page Settings");
        assert_eq!(h.state().ui.dialog.as_ref().unwrap().fields["side"], "left");
    }

    fn right_click(h: &mut Harness<'_, DesignApp>, p: Pos2) {
        h.hover_at(p);
        h.step();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: p, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() });
        }
        h.run_steps(3);
    }

    fn new_document() -> Harness<'static, DesignApp> {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        harness(app)
    }

    fn held(modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::ModifiersChanged(modifiers)
    }

    fn wheel(phase: egui::TouchPhase, up: f32, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: vec2(0.0, up), phase, modifiers }
    }

    // Plays `events` with the pointer at `at`, a frame each, and lets the motion settle.
    fn play(h: &mut Harness<'_, DesignApp>, at: Pos2, events: &[egui::Event]) -> View {
        h.hover_at(at);
        for event in events {
            h.event(event.clone());
            h.step();
        }
        h.run_steps(30);
        *h.state().view().unwrap()
    }

    /// Alt + wheel scrolled the view like the plain wheel; it zooms at the pointer (#470).
    #[test]
    fn alt_wheel_zooms_at_the_pointer() {
        use egui::{Modifiers, TouchPhase::Move};
        let mut h = new_document();
        let rect = h.state().canvas_rect.unwrap();
        let at = rect.center() + vec2(150.0, 90.0);
        let before = *h.state().view().unwrap();
        let under = Xf::new(rect, &before).to_canvas(at);
        let zoomed = play(&mut h, at, &[held(Modifiers::ALT), wheel(Move, 100.0, Modifiers::ALT)]);
        assert!(zoomed.zoom > before.zoom * 1.2, "wheel up zooms in: {} -> {}", before.zoom, zoomed.zoom);
        let still = Xf::new(rect, &zoomed).to_canvas(at);
        assert!((still.x - under.x).abs() < 0.01 && (still.y - under.y).abs() < 0.01, "the point under the pointer stays: {under:?} -> {still:?}");
        let back = play(&mut h, at, &[held(Modifiers::ALT), wheel(Move, -100.0, Modifiers::ALT)]);
        assert!((back.zoom - before.zoom).abs() < 1e-6, "wheel down zooms out: {} -> {}", zoomed.zoom, back.zoom);
        // The plain wheel still scrolls.
        let scrolled = play(&mut h, at, &[held(Modifiers::NONE), wheel(Move, 100.0, Modifiers::NONE)]);
        assert!(scrolled.zoom == back.zoom && scrolled.origin.y < back.origin.y, "{back:?} -> {scrolled:?}");
    }

    /// Alt zooms exactly as Ctrl/Cmd does, also when the key is let go while the wheel's motion is
    /// still arriving: the rest of it zooms too and does not scroll the page.
    #[test]
    fn alt_wheel_zooms_like_ctrl_wheel_to_the_end_of_the_motion() {
        use egui::{
            Modifiers,
            TouchPhase::{End, Move, Start},
        };
        const NONE: Modifiers = Modifiers::NONE;
        // Each motion in steps; the views are compared after every step.
        let motions: [fn(Modifiers) -> Vec<Vec<egui::Event>>; 4] = [
            |key| vec![vec![held(key), wheel(Move, 100.0, key)]],
            // A wheel turned fast, the key let go a frame later.
            |key| vec![vec![held(key), wheel(Move, 400.0, key), held(NONE)]],
            // A trackpad swipe that coasts on after the key is let go; once it has ended, the wheel scrolls.
            |key| {
                vec![
                    vec![held(key), wheel(Start, 0.0, key), wheel(Move, 50.0, key), held(NONE), wheel(Move, 200.0, NONE)],
                    vec![wheel(End, 0.0, NONE), wheel(Move, 100.0, NONE)],
                ]
            },
            |key| vec![vec![held(key | Modifiers::SHIFT), wheel(Move, 100.0, key | Modifiers::SHIFT)]],
        ];
        for (i, motion) in motions.iter().enumerate() {
            let (mut with_alt, mut with_command) = (new_document(), new_document());
            let at = with_alt.state().canvas_rect.unwrap().center() + vec2(150.0, 90.0);
            let start = *with_alt.state().view().unwrap();
            for (alt, command) in motion(Modifiers::ALT).iter().zip(motion(Modifiers::COMMAND)) {
                let (alt, command) = (play(&mut with_alt, at, alt), play(&mut with_command, at, &command));
                assert!(command.zoom > start.zoom, "motion {i} zooms");
                assert_eq!(alt, command, "motion {i}");
            }
        }
    }

    fn units(h: &Harness<'_, DesignApp>) -> (Unit, Unit) {
        let s = &h.state().session.doc().unwrap().doc.settings;
        (s.horizontal_units, s.vertical_units)
    }

    /// A viewport wider than the GPU's largest texture: the canvas texture is sampled down, and an
    /// edit repaints it whole. A region repainted in view pixels would land in the wrong place,
    /// or outside the texture (a wgpu validation error). A view that fits is still patched.
    #[test]
    fn an_edit_repaints_a_canvas_texture_sampled_down_to_the_gpu_limit_whole() {
        for (width, sampled_down) in [(1440.0, false), (2600.0, true)] {
            let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
            app.run("file.new", json!({})).unwrap();
            let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Text"})).unwrap();
            let mut h = Harness::builder().with_size(vec2(width, 900.0)).build_ui_state(
                |ui, app: &mut DesignApp| {
                    let ctx = ui.ctx().clone();
                    app.logic(&ctx);
                    app.ui(ui);
                },
                app,
            );
            h.run_steps(6);
            let limit = h.ctx.input(|i| i.max_texture_side);
            let size = h.state().canvas.texture.as_ref().map(|t| t.size()).unwrap();
            assert!(size[0] <= limit && size[1] <= limit, "{size:?}");
            let shown = h.state().canvas.shown.unwrap().size;
            assert_eq!(shown.0 > limit as f32, sampled_down, "view {shown:?}, limit {limit}");
            let patches = h.state().canvas.patches;
            // A small edit: one more character in the frame.
            h.state_mut().run("text.insert", json!({"story": frame["story"], "at": 4, "text": "s"})).unwrap();
            h.run_steps(6);
            assert_eq!(h.state().canvas.patches == patches, sampled_down, "width {width}");
            assert_eq!(h.state().canvas.texture.as_ref().map(|t| t.size()), Some(size));
        }
    }

    #[test]
    fn right_clicking_a_ruler_sets_that_rulers_units() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let mut h = harness(app);
        let rect = h.state().canvas_rect.unwrap();
        assert_eq!(units(&h), (Unit::Picas, Unit::Picas));
        // Vertical ruler: the current unit is checked; picking one sets only the vertical units.
        right_click(&mut h, pos2(rect.min.x - RULER / 2.0, rect.center().y));
        h.get_by_label("✓ Picas");
        h.get_by_label("   Millimeters").click();
        h.run_steps(3);
        assert_eq!(units(&h), (Unit::Picas, Unit::Millimeters));
        // Horizontal ruler.
        right_click(&mut h, pos2(rect.center().x, rect.min.y - RULER / 2.0));
        h.get_by_label("✓ Picas");
        h.get_by_label("   Inches").click();
        h.run_steps(3);
        assert_eq!(units(&h), (Unit::Inches, Unit::Millimeters));
        // The page and the ruler corner open no units menu.
        for p in [rect.center(), rect.min - vec2(RULER / 2.0, RULER / 2.0)] {
            right_click(&mut h, p);
            assert!(h.query_by_label("   Points").is_none());
        }
    }

    /// Screen position of spread 0's point (x, y).
    fn screen_of(h: &Harness<'_, DesignApp>, x: f64, y: f64) -> Pos2 {
        let app = h.state();
        let st = app.session.active().unwrap();
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        xf.to_screen(CanvasLayout::new(&st.doc, st.editing_parents).xf(SpreadRef::Doc(0)) * Point::new(x, y))
    }

    /// A menu row, excluding the named toggle buttons in the Properties panel.
    fn menu_row<'h>(h: &'h Harness<'_, DesignApp>, label: &'h str) -> Option<egui_kittest::Node<'h>> {
        h.query_by(move |n| {
            n.role() == egui::accesskit::Role::Button
                && n.toggled().is_none()
                && n.label().is_some_and(|l| l == label || l.strip_prefix(label).and_then(|r| r.strip_prefix(' ')).is_some_and(|r| !r.contains(' ')))
        })
    }

    fn enabled(h: &Harness<'_, DesignApp>, label: &str) -> bool {
        use egui_kittest::kittest::NodeT;
        !menu_row(h, label).unwrap_or_else(|| panic!("no menu row {label}")).accesskit_node().is_disabled()
    }

    fn spread_items(h: &Harness<'_, DesignApp>) -> Vec<u64> {
        h.state().session.doc().unwrap().doc.spreads[0].items.iter().map(|i| i.id.0).collect()
    }

    fn selected(h: &Harness<'_, DesignApp>) -> Vec<u64> {
        h.state().session.doc().unwrap().selection.items.iter().map(|i| i.0).collect()
    }

    #[test]
    fn right_clicking_an_object_selects_it_and_its_menu_arranges_it() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let a = app.run("frame.create", json!({"rect": [36, 36, 200, 200]})).unwrap()["id"].as_u64().unwrap();
        let b = app.run("frame.create", json!({"rect": [150, 150, 300, 300]})).unwrap()["id"].as_u64().unwrap();
        let mut h = harness(app);
        assert_eq!(selected(&h), [b]);
        let on_a = screen_of(&h, 60.0, 60.0);
        right_click(&mut h, on_a);
        assert_eq!(selected(&h), [a], "the object under the pointer is selected first");
        // An empty frame's menu: Content, but no Fitting (nothing to fit) and no text frame entries.
        assert!(menu_row(&h, "Content").is_some());
        assert!(menu_row(&h, "Fitting").is_none() && menu_row(&h, "Text Frame Options…").is_none());
        assert!(enabled(&h, "Copy") && !enabled(&h, "Paste"), "nothing copied yet: Paste is greyed");
        menu_row(&h, "Arrange").unwrap().click();
        h.run_steps(3);
        menu_row(&h, "Bring to Front").unwrap().click();
        h.run_steps(3);
        assert_eq!(spread_items(&h), [b, a]);
        assert!(menu_row(&h, "Arrange").is_none(), "choosing an entry closes the menu");
    }

    #[test]
    fn right_clicking_empty_canvas_offers_paste_after_a_copy() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let a = app.run("frame.create", json!({"rect": [36, 36, 200, 200]})).unwrap()["id"].as_u64().unwrap();
        app.run("edit.copy", json!({})).unwrap();
        app.select_tool("rectangleFrame");
        let mut h = harness(app);
        let empty = screen_of(&h, 400.0, 500.0);
        right_click(&mut h, empty);
        assert_eq!(selected(&h), [a], "a right-click on nothing keeps the selection");
        assert!(menu_row(&h, "Arrange").is_none(), "the page menu, not the object menu");
        assert!(enabled(&h, "Paste in Place"));
        assert!(menu_row(&h, "✓ Show/Hide Rulers").is_some(), "toggles show their state");
        // Escape closes the menu and does nothing else: the tool would deselect, the shortcuts
        // would switch to the Selection tool.
        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        assert!(menu_row(&h, "Paste in Place").is_none());
        assert_eq!((h.state().session.tool_id(), selected(&h)), ("rectangleFrame", vec![a]));
        right_click(&mut h, empty);
        menu_row(&h, "Paste").unwrap().click();
        h.run_steps(3);
        assert_eq!(spread_items(&h).len(), 2);
    }

    #[test]
    fn right_clicking_another_object_while_the_menu_is_open_reopens_it_there() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let a = app.run("frame.create", json!({"rect": [36, 36, 200, 200]})).unwrap()["id"].as_u64().unwrap();
        let b = app.run("frame.create", json!({"rect": [36, 600, 200, 760]})).unwrap()["id"].as_u64().unwrap();
        let mut h = harness(app);
        // The menu opens down from A, with the pointer on its first row (no submenu open); B is below it.
        let (on_a, on_b) = (screen_of(&h, 60.0, 60.0), screen_of(&h, 50.0, 700.0));
        right_click(&mut h, on_a);
        assert_eq!(selected(&h), [a]);
        assert!(menu_row(&h, "Arrange").is_some());
        right_click(&mut h, on_b);
        assert_eq!(selected(&h), [b]);
        assert!(menu_row(&h, "Arrange").is_some(), "the menu reopens at the new click");
    }

    #[test]
    fn a_context_menu_in_one_pane_keeps_the_keys_from_the_other() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let a = app.run("frame.create", json!({"rect": [36, 36, 100, 100]})).unwrap()["id"].as_u64().unwrap();
        app.select_tool("rectangleFrame");
        app.split = true;
        let mut h = harness(app);
        // The focused pane is the left one; this point is near its right edge, away from the frame.
        let left = h.state().canvas_rect.unwrap();
        right_click(&mut h, pos2(left.max.x - 40.0, left.center().y));
        assert!(menu_row(&h, "Paste in Place").is_some());
        // Escape closes the left pane's menu before the right pane handles the keys.
        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        assert!(menu_row(&h, "Paste in Place").is_none());
        assert_eq!((h.state().session.tool_id(), selected(&h)), ("rectangleFrame", vec![a]));
    }

    #[test]
    fn right_clicking_text_keeps_the_selected_text_or_places_the_caret() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [36, 36, 400, 200], "content": "text"})).unwrap();
        app.select_tool("type");
        app.run("text.insert", json!({"text": "Hello world"})).unwrap();
        let sid = *app.session.doc().unwrap().doc.stories.keys().next().unwrap();
        app.run("text.select", json!({"story": sid.0, "anchor": 0, "focus": 5})).unwrap();
        let mut h = harness(app);
        let text_sel = |h: &Harness<'_, DesignApp>| h.state().session.doc().unwrap().selection.text.as_ref().map(|t| (t.anchor, t.focus));
        // Inside "Hello": the selection stays.
        let inside = screen_of(&h, 45.0, 44.0);
        right_click(&mut h, inside);
        assert_eq!(text_sel(&h), Some((0, 5)));
        assert!(menu_row(&h, "Paste without Formatting").is_some(), "the text menu");
        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        // Past the end of the line: the caret goes there.
        let past_end = screen_of(&h, 300.0, 44.0);
        right_click(&mut h, past_end);
        assert_eq!(text_sel(&h), Some((11, 11)));
        assert!(menu_row(&h, "Paste without Formatting").is_some());
    }

    #[test]
    fn rows_hidden_in_the_menu_bar_are_hidden_in_context_menus() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [36, 36, 200, 200]})).unwrap();
        app.ui.hidden_menu_items.push(crate::menus::menu_key("Object/Arrange", "Bring to Front"));
        let mut h = harness(app);
        let on_frame = screen_of(&h, 60.0, 60.0);
        right_click(&mut h, on_frame);
        menu_row(&h, "Arrange").unwrap().click();
        h.run_steps(3);
        assert!(menu_row(&h, "Bring to Front").is_none());
        assert!(menu_row(&h, "Send to Back").is_some());
        assert!(h.query_by_label("Show All Menu Items").is_some());
    }

    #[test]
    fn a_secondary_drag_neither_moves_marquees_nor_draws() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let a = app.run("frame.create", json!({"rect": [36, 36, 200, 200]})).unwrap()["id"].as_u64().unwrap();
        let mut h = harness(app);
        // A secondary-button drag between spread points.
        let drag = |h: &mut Harness<'_, DesignApp>, from: (f64, f64), to: (f64, f64)| {
            let (from, to) = (screen_of(h, from.0, from.1), screen_of(h, to.0, to.1));
            h.hover_at(from);
            h.step();
            h.event(egui::Event::PointerButton { pos: from, button: egui::PointerButton::Secondary, pressed: true, modifiers: Default::default() });
            h.step();
            for t in [0.25, 0.5, 0.75, 1.0] {
                h.event(egui::Event::PointerMoved(from + (to - from) * t));
                h.step();
            }
            h.event(egui::Event::PointerButton { pos: to, button: egui::PointerButton::Secondary, pressed: false, modifiers: Default::default() });
            h.run_steps(3);
        };
        let xf = |h: &Harness<'_, DesignApp>| h.state().session.doc().unwrap().doc.item(pagenor_doc::ItemId(a)).unwrap().xf;
        let before = xf(&h);
        // Selection tool: on the object (no move), and from empty page across it (no marquee).
        drag(&mut h, (100.0, 100.0), (300.0, 300.0));
        assert_eq!(xf(&h), before);
        h.state_mut().run("edit.deselectAll", json!({})).unwrap();
        drag(&mut h, (400.0, 400.0), (20.0, 20.0));
        assert!(selected(&h).is_empty());
        // Rectangle Frame tool: no frame.
        h.state_mut().select_tool("rectangleFrame");
        drag(&mut h, (300.0, 300.0), (500.0, 500.0));
        assert_eq!(spread_items(&h), [a]);
    }

    /// The live-corner widget draws on a selected rectangle frame; clicking it draws the diamonds.
    #[test]
    fn live_corner_widget_and_diamonds_draw() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [100, 100, 300, 250], "content": "graphic"})).unwrap();
        let id = app.session.doc().unwrap().doc.spreads[0].items[0].id;
        app.run("selection.set", json!({"ids": [id.0]})).unwrap();
        let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).with_pixels_per_point(1.0).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let screen = |h: &Harness<'_, DesignApp>, x: f64, y: f64| {
            let app = h.state();
            let layout = CanvasLayout::new(&app.session.doc().unwrap().doc, false);
            let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
            xf.to_screen(layout.to_canvas(SpreadRef::Doc(0), Point::new(x, y)))
        };
        let widget = screen(&h, 300.0, 100.0) + vec2(0.0, 11.5);
        let diamond = screen(&h, 100.0, 100.0) + vec2(12.0, 0.0);
        // Inspect the actual paint output, including the pointer-driven transition, without
        // requiring a particular GPU. Pixel screenshots are covered by native visual checks.
        let yellow = |h: &Harness<'_, DesignApp>, p: Pos2| -> bool {
            fn contains(shape: &egui::Shape, p: Pos2) -> bool {
                let yellow = egui::Color32::from_rgb(255, 229, 0);
                match shape {
                    egui::Shape::Rect(r) => r.fill == yellow && r.rect.contains(p),
                    egui::Shape::Path(path) => path.fill == yellow && egui::Rect::from_points(&path.points).contains(p),
                    egui::Shape::Vec(shapes) => shapes.iter().any(|s| contains(s, p)),
                    _ => false,
                }
            }
            h.output().shapes.iter().any(|s| contains(&s.shape, p))
        };
        let w = yellow(&h, widget);
        assert!(w, "no widget at {widget:?}");
        assert!(!yellow(&h, diamond));
        h.hover_at(widget);
        h.step();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: widget, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
            h.step();
        }
        h.run_steps(3);
        assert!(yellow(&h, diamond), "no diamond at {diamond:?}");
        assert!(!yellow(&h, widget));
    }

    #[test]
    fn upright_content_inside_a_rotated_frame_has_a_tight_selection_outline() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let f = app.run("frame.create", json!({"rect":[100,100,300,300],"content":"graphic"})).unwrap()["id"].clone();
        let png = pagenor_render::Rendered { width: 200, height: 100, pixels: vec![255; 200 * 100 * 4] }.to_png();
        app.run("file.place", json!({"base64":pagenor_engine::cmd::base64_encode(&png),"frame":f,"name":"synthetic.png"})).unwrap();
        app.run("transform.rotate", json!({"ids":[f],"angle":-45})).unwrap();
        app.run("selection.set", json!({"ids":[f],"content":true})).unwrap();
        app.run("transform.set", json!({"rotation":0})).unwrap();
        let mut h = harness(app);
        h.run_steps(3);
        fn outline(s: &egui::Shape) -> Option<egui::Rect> {
            match s {
                egui::Shape::Rect(r) if r.stroke.color == Color32::from_rgb(196, 111, 43) => Some(r.rect),
                egui::Shape::Vec(v) => v.iter().find_map(outline),
                _ => None,
            }
        }
        let r = h.output().shapes.iter().find_map(|s| outline(&s.shape)).unwrap();
        let zoom = h.state().view().unwrap().zoom as f32;
        assert!((r.width() - 400.0 * zoom).abs() < 0.01 && (r.height() - 200.0 * zoom).abs() < 0.01, "{r:?} at zoom {zoom}");
    }

    #[test]
    fn document_grid_is_drawn_where_points_snap_to_it() {
        use pagenor_tools::{SnapView, ToolContext};
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let doc = app.session.active().unwrap().doc.clone();
        let layout = CanvasLayout::new(&doc, false);
        let sr = SpreadRef::Doc(0);
        // Where the grid snaps a point in the first page (72 pt / 8 subdivisions = 9 pt).
        let page = doc.spreads[0].pages[0].bounds();
        let sel = Selection::default();
        let cx = ToolContext {
            doc: &doc,
            selection: &sel,
            cache: &app.session.cache,
            layout: &layout,
            zoom: 1.0,
            layer: doc.layers[0].id,
            snap: SnapView { snap_to_document_grid: true, zone_px: 20.0, ..SnapView::OFF },
            scale_strokes: app.session.prefs.scale_strokes,
            unit: Unit::Points,
        };
        let (snapped, _) = pagenor_tools::snap::snap_point(&cx, sr, Point::new(page.x0 + 31.3, page.y0 + 47.9));
        assert_eq!((snapped.x - page.x0, snapped.y - page.y0), (27.0, 45.0));
        // Render 40 × 40 pt around it at 4 px/pt.
        let scale = 4.0;
        let corner = layout.xf(sr) * Point::new(snapped.x - 20.0, snapped.y - 20.0);
        let view = Affine::scale(scale) * Affine::translate((-corner.x, -corner.y));
        let placed: Vec<_> = layout.slots.iter().map(|s| pagenor_render::Placed { spread: s.spread, xf: s.xf }).collect();
        let render = |app: &DesignApp| {
            let opts = screen_options(app, &layout, false, 1.0);
            pagenor_render::Renderer::new().render(&doc, &app.session.cache, &placed, 160, 160, view, &opts)
        };
        // The snapped point is at pixel (80, 80); halfway to the next line is 18 px further.
        let darkest = |img: &pagenor_render::Rendered, xs: [u32; 2], ys: [u32; 2]| {
            (xs[0]..xs[1]).flat_map(|x| (ys[0]..ys[1]).map(move |y| (x, y))).map(|(x, y)| img.pixel(x, y)[0]).min().unwrap()
        };
        let off = render(&app);
        assert_eq!(darkest(&off, [78, 82], [60, 64]), 255, "no grid while it is hidden");
        app.run("view.documentGrid", json!({})).unwrap();
        let on = render(&app);
        assert!(darkest(&on, [79, 81], [60, 64]) < 250, "a vertical line at the snapped x");
        assert!(darkest(&on, [60, 64], [79, 81]) < 250, "a horizontal line at the snapped y");
        assert_eq!(darkest(&on, [96, 100], [96, 100]), 255, "nothing between lines");
        // Preview hides it.
        app.ui.screen_mode = ScreenMode::Preview;
        assert!(screen_options(&app, &layout, true, 1.0).document_grid.is_empty());
    }

    #[test]
    fn rulers_open_no_units_menu_without_a_document() {
        let mut h = harness(DesignApp::new(pagenor_engine::Session::new(), crate::Services::default()));
        let Some(rect) = h.state().canvas_rect else { return };
        right_click(&mut h, pos2(rect.center().x, rect.min.y - RULER / 2.0));
        assert!(h.query_by_label("   Points").is_none());
    }

    fn drag(h: &mut Harness<'_, DesignApp>, from: Pos2, to: Pos2) {
        h.event(egui::Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
        h.run_steps(2);
        h.event(egui::Event::PointerMoved(to));
        h.run_steps(2);
        h.event(egui::Event::PointerButton { pos: to, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
        h.run_steps(2);
    }

    fn double_click(h: &mut Harness<'_, DesignApp>, p: Pos2) {
        for _ in 0..2 {
            h.event(egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
            h.run_steps(1);
            h.event(egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
            h.run_steps(1);
        }
        h.run_steps(2);
    }

    #[test]
    fn a_single_ruler_corner_click_preserves_the_origin_and_document() {
        for origin in [None, Some(Point::new(100.0, 200.0))] {
            let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
            app.run("file.new", json!({})).unwrap();
            let mut h = fast_harness(app);
            h.state_mut().view_mut().unwrap().ruler_origin = origin;
            let before = h.state().session.doc().unwrap().doc.clone();
            let corner = h.state().canvas_rect.unwrap().min - vec2(RULER / 2.0, RULER / 2.0);
            for pressed in [true, false] {
                h.event(egui::Event::PointerButton { pos: corner, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
                h.run_steps(1);
            }
            assert_eq!(h.state().view().unwrap().ruler_origin, origin);
            assert!(std::sync::Arc::ptr_eq(&before, &h.state().session.doc().unwrap().doc));
        }
    }

    #[test]
    fn dragging_the_ruler_corner_sets_a_custom_zero_point() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let mut h = fast_harness(app);
        let rect = h.state().canvas_rect.unwrap();
        assert!(h.state().view().unwrap().ruler_origin.is_none());
        let corner = rect.min - vec2(RULER / 2.0, RULER / 2.0);
        let dest = rect.min + vec2(60.0, 40.0);
        drag(&mut h, corner, dest);
        let origin = h.state().view().unwrap().ruler_origin.expect("drag should set a custom ruler origin");
        let xf = Xf::new(rect, h.state().view().unwrap());
        let expected = xf.to_canvas(dest);
        assert!((origin.x - expected.x).abs() < 1.0, "origin.x = {}, expected = {}", origin.x, expected.x);
        assert!((origin.y - expected.y).abs() < 1.0, "origin.y = {}, expected = {}", origin.y, expected.y);
        // Double-clicking the corner resets it to the default origin.
        double_click(&mut h, corner);
        assert!(h.state().view().unwrap().ruler_origin.is_none());
    }
}

/// Context actions share the same command path as menus, including the host clipboard bridge.
#[cfg(test)]
mod context_menu_tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    fn harness(app: DesignApp) -> Harness<'static, DesignApp> {
        let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h
    }
    fn right_click(h: &mut Harness<'_, DesignApp>) {
        let app = h.state();
        let doc = &app.session.active().unwrap().doc;
        let layout = CanvasLayout::new(doc, false);
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        let pos = xf.to_screen(layout.xf(SpreadRef::Doc(0)) * doc.spreads[0].items[0].bounds().center());
        h.hover_at(pos);
        h.step();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() });
        }
        h.run_steps(3);
    }
    #[test]
    fn right_click_duplicate_preserves_selection_and_undo() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let id = app.run("frame.create", json!({"rect":[72,72,200,200]})).unwrap()["id"].clone();
        app.run("selection.set", json!({"ids":[id]})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let selection = app.session.active().unwrap().selection.clone();
        let mut h = harness(app);
        right_click(&mut h);
        assert_eq!(h.state().session.active().unwrap().selection, selection);
        h.get_by(|n| n.role() == egui::accesskit::Role::Button && n.label().is_some_and(|l| l == "Duplicate" || l.starts_with("Duplicate "))).click();
        h.run_steps(3);
        assert_ne!(h.state().session.active().unwrap().doc, before);
        h.state_mut().run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().session.active().unwrap().doc, before);
    }
    #[test]
    fn right_click_text_paste_uses_host_unicode_clipboard_and_undo() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.services.clipboard_text = Some(Box::new(|| Some("தமிழ் क्षेत्र".into())));
        app.run("file.new", json!({})).unwrap();
        let frame = app.run("frame.create", json!({"rect":[72,72,400,200],"content":"text","text":"","caret":true})).unwrap();
        app.run("tool.select", json!({"tool":"type"})).unwrap();
        app.run("text.select", json!({"story":frame["story"],"anchor":0,"focus":0})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        let mut h = harness(app);
        right_click(&mut h);
        h.get_by(|n| {
            n.role() == egui::accesskit::Role::Button
                && n.label().is_some_and(|l| l == "Paste" || l.strip_prefix("Paste ").is_some_and(|r| !r.contains(' ')))
        })
        .click();
        h.run_steps(3);
        assert!(h.state().session.active().unwrap().doc.stories.values().any(|story| story.text == "தமிழ் क्षेत्र"));
        h.state_mut().run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().session.active().unwrap().doc, before);
    }
}

#[cfg(test)]
mod manual_flow_tests {
    use super::*;
    use egui_kittest::Harness;

    fn type_drag() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages": 2, "facingPages": false})).unwrap();
        app.session.set_tool("type");
        let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let rect = h.state().canvas_rect.unwrap();
        let xf = Xf::new(rect, h.state().view().unwrap());
        let a = xf.to_screen(Point::new(72.0, 72.0));
        let b = xf.to_screen(Point::new(450.0, 240.0));
        h.hover_at(a);
        h.step();
        h.event(egui::Event::PointerButton { pos: a, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
        h.step();
        h.hover_at(b);
        h.step();
        h.event(egui::Event::PointerButton { pos: b, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
        h.run_steps(3);
        let st = h.state().session.active().unwrap();
        let text = st.selection.text.expect("drag should enter text editing");
        let frame = st.doc.item(text.frame.unwrap()).unwrap();
        assert!(frame.inner_bounds().width() > 300.0);
        assert!(frame.inner_bounds().height() > 100.0);
    }

    #[test]
    fn type_drag_creates_a_frame_through_the_window() {
        type_drag();
    }

    fn raw_frame(ctx: &egui::Context, app: &mut DesignApp, events: Vec<egui::Event>) {
        let mut output = ctx.run_ui(
            egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1440.0, 900.0))), events, ..Default::default() },
            |ui| {
                app.logic(ctx);
                app.ui(ui);
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn type_drag_handles_a_whole_gesture_in_one_frame() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages": 2, "facingPages": false})).unwrap();
        app.session.set_tool("type");
        for _ in 0..4 {
            raw_frame(&ctx, &mut app, vec![]);
        }
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        let a = xf.to_screen(Point::new(72.0, 72.0));
        let b = xf.to_screen(Point::new(450.0, 240.0));
        raw_frame(&ctx, &mut app, vec![egui::Event::PointerMoved(a)]);
        raw_frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerButton { pos: a, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() },
                egui::Event::PointerMoved(b),
                egui::Event::PointerButton { pos: b, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() },
            ],
        );
        let st = app.session.active().unwrap();
        let text = st.selection.text.expect("batched drag should enter text editing");
        let frame = st.doc.item(text.frame.unwrap()).unwrap();
        assert!(frame.inner_bounds().width() > 300.0);
        assert!(frame.inner_bounds().height() > 100.0);
    }

    fn quick_click(ctx: &egui::Context, app: &mut DesignApp, p: Pos2) {
        raw_frame(ctx, app, vec![egui::Event::PointerMoved(p)]);
        raw_frame(
            ctx,
            app,
            vec![
                egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() },
                egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() },
            ],
        );
    }

    #[test]
    fn rapid_canvas_clicks_select_word_line_paragraph_and_story() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let text = "Hello brave world\nSecond line here";
        let frame = app.run("frame.create", json!({"content":"text", "rect":[72,72,500,200], "text":text})).unwrap();
        app.session.set_tool("type");
        for _ in 0..4 {
            raw_frame(&ctx, &mut app, vec![]);
        }
        let before = app.session.doc().unwrap().doc.clone();
        let history = app.session.doc().unwrap().history.undo.len();
        let cs = app.session.cache.get(&before, pagenor_doc::StoryId(frame["story"].as_u64().unwrap()), None);
        let (line, glyph) = cs.frames[0].lines.iter().find_map(|l| l.glyphs.iter().find(|g| g.byte == 7).map(|g| (l, g))).unwrap();
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        let point = xf.to_screen(Point::new(glyph.x + glyph.adv * 0.25, line.baseline - 3.0));
        quick_click(&ctx, &mut app, point);
        for want in ["brave", "Hello brave world", "Hello brave world", text] {
            quick_click(&ctx, &mut app, point);
            let sel = app.session.doc().unwrap().selection.text.unwrap();
            assert_eq!(&text[sel.anchor.min(sel.focus)..sel.anchor.max(sel.focus)], want);
        }
        assert!(std::sync::Arc::ptr_eq(&before, &app.session.doc().unwrap().doc));
        assert_eq!(app.session.doc().unwrap().history.undo.len(), history);
    }

    #[test]
    fn complete_double_and_triple_clicks_in_one_paint_select_their_units() {
        for (clicks, want) in [(2, "brave"), (3, "Hello brave world")] {
            let ctx = egui::Context::default();
            let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
            app.run("file.new", json!({})).unwrap();
            let text = "Hello brave world\nSecond line here";
            let f = app.run("frame.create", json!({"content":"text", "rect":[72,72,500,200], "text":text})).unwrap();
            app.session.set_tool("type");
            for _ in 0..4 {
                raw_frame(&ctx, &mut app, vec![]);
            }
            let before = app.session.doc().unwrap().doc.clone();
            let cs = app.session.cache.get(&before, pagenor_doc::StoryId(f["story"].as_u64().unwrap()), None);
            let (line, glyph) = cs.frames[0].lines.iter().find_map(|l| l.glyphs.iter().find(|g| g.byte == 7).map(|g| (l, g))).unwrap();
            let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
            let p = xf.to_screen(Point::new(glyph.x + glyph.adv * 0.25, line.baseline - 3.0));
            raw_frame(&ctx, &mut app, vec![egui::Event::PointerMoved(p)]);
            let mut events = Vec::new();
            for _ in 0..clicks {
                for pressed in [true, false] {
                    events.push(egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
                }
            }
            raw_frame(&ctx, &mut app, events);
            let sel = app.session.doc().unwrap().selection.text.unwrap();
            assert_eq!(&text[sel.anchor.min(sel.focus)..sel.anchor.max(sel.focus)], want);
            assert!(std::sync::Arc::ptr_eq(&before, &app.session.doc().unwrap().doc));
        }
    }

    #[test]
    fn batched_ruler_drag_and_double_click_reset_leave_the_document_unchanged() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        for _ in 0..4 {
            raw_frame(&ctx, &mut app, vec![]);
        }
        let before = app.session.doc().unwrap().doc.clone();
        let rect = app.canvas_rect.unwrap();
        let corner = rect.min - vec2(RULER / 2.0, RULER / 2.0);
        let dest = rect.min + vec2(80.0, 50.0);
        let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        raw_frame(&ctx, &mut app, vec![egui::Event::PointerMoved(corner)]);
        raw_frame(&ctx, &mut app, vec![button(corner, true), egui::Event::PointerMoved(dest), button(dest, false)]);
        assert!(app.view().unwrap().ruler_origin.is_some());
        quick_click(&ctx, &mut app, corner);
        quick_click(&ctx, &mut app, corner);
        assert!(app.view().unwrap().ruler_origin.is_none());
        assert!(std::sync::Arc::ptr_eq(&before, &app.session.doc().unwrap().doc));
    }

    #[test]
    fn quick_canvas_click_places_a_caret_and_accepts_paste() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let frame = app.run("frame.create", json!({"content":"text", "rect":[72,72,450,150], "text":"Original"})).unwrap();
        app.session.set_tool("type");
        for _ in 0..4 {
            raw_frame(&ctx, &mut app, vec![]);
        }
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        quick_click(&ctx, &mut app, xf.to_screen(Point::new(90.0, 90.0)));
        assert!(app.session.wants_text(), "quick click should place a caret");
        raw_frame(&ctx, &mut app, vec![egui::Event::Paste("Pasted ".into())]);
        let sid = pagenor_doc::StoryId(frame["story"].as_u64().unwrap());
        assert!(app.session.doc().unwrap().doc.story(sid).unwrap().text.contains("Pasted "));
    }

    #[test]
    fn quick_port_click_threads_text_to_an_existing_frame_on_page_two_and_undoes() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages":2, "facingPages":false})).unwrap();
        let text = "Text should continue to page two. ".repeat(24);
        let a = app.run("frame.create", json!({"content":"text", "rect":[72,72,450,112], "text":text})).unwrap();
        let b = app.run("frame.create", json!({"content":"text", "spread":SpreadRef::Doc(1), "rect":[72,72,450,720]})).unwrap();
        let aid = pagenor_doc::ItemId(a["id"].as_u64().unwrap());
        let bid = pagenor_doc::ItemId(b["id"].as_u64().unwrap());
        let sid = pagenor_doc::StoryId(a["story"].as_u64().unwrap());
        app.run("selection.set", json!({"ids":[aid]})).unwrap();
        app.session.set_tool("selection");
        let before = app.session.doc().unwrap().doc.clone();
        for _ in 0..4 {
            raw_frame(&ctx, &mut app, vec![]);
        }
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        let layout = app.session.layout();
        let port = pagenor_tools::thread::port_center(&before, &layout, aid, true, xf.zoom).unwrap();
        quick_click(&ctx, &mut app, xf.to_screen(port));
        go_to_page(&mut app, 1);
        raw_frame(&ctx, &mut app, vec![]);
        let xf = Xf::new(app.canvas_rect.unwrap(), app.view().unwrap());
        let dest = layout.to_canvas(SpreadRef::Doc(1), Point::new(90.0, 90.0));
        quick_click(&ctx, &mut app, xf.to_screen(dest));
        let st = app.session.doc().unwrap();
        assert_eq!(st.doc.story(sid).unwrap().frames, vec![aid, bid]);
        assert_eq!(st.doc.story(sid).unwrap().text, text);
        let composed = pagenor_compose::compose_story(&st.doc, sid, &Default::default());
        assert!(!composed.frame(bid).unwrap().range.is_empty(), "page two should show the continuation");
        assert!(!composed.is_overset());
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, before);
    }
}

#[cfg(test)]
mod pencil_preview_tests {
    use egui::{Pos2, pos2, vec2};

    /// Lines painted in `color` (egui paths) and the points they pass through.
    fn lines_in(h: &egui_kittest::Harness<'static, crate::test_window::Window>, color: egui::Color32) -> Vec<Vec<Pos2>> {
        fn walk(s: &egui::Shape, color: egui::Color32, out: &mut Vec<Vec<Pos2>>) {
            match s {
                egui::Shape::Path(p) if p.stroke.color == egui::epaint::ColorMode::Solid(color) && p.points.len() >= 2 => out.push(p.points.clone()),
                egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, color, out)),
                _ => {}
            }
        }
        let mut out = vec![];
        h.output().shapes.iter().for_each(|s| walk(&s.shape, color, &mut out));
        out
    }

    #[test]
    fn the_pencil_stroke_shows_while_drawing() {
        let mut app = crate::DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", serde_json::json!({})).unwrap();
        app.run("tool.select", serde_json::json!({"tool": "pencil"})).unwrap();
        let mut h = crate::test_window::open(app, vec2(1400.0, 900.0));
        let doc = &h.state().app.session.doc().unwrap().doc;
        let layer = doc.layers[0].color;
        let color = egui::Color32::from_rgb(layer[0], layer[1], layer[2]);
        let items = |h: &egui_kittest::Harness<'static, crate::test_window::Window>| h.state().app.session.doc().unwrap().doc.spreads[0].items.len();
        let before = items(&h);
        // Press on the page and drag, without letting go.
        let start = pos2(520.0, 380.0);
        h.hover_at(start);
        h.drag_at(start);
        h.run_steps(2);
        let path: Vec<Pos2> = (1..=12).map(|i| start + vec2(i as f32 * 12.0, (i as f32 * 0.6).sin() * 30.0)).collect();
        for p in &path {
            h.hover_at(*p);
            h.run_steps(1);
        }
        let drawn = lines_in(&h, color);
        let stroke = drawn.iter().max_by_key(|l| l.len()).expect("the stroke is painted while the button is down");
        let near = |a: Pos2, b: Pos2| (a - b).length() < 2.0;
        assert!(near(stroke[0], start), "it starts where the press was: {:?}", stroke[0]);
        assert!(path.iter().all(|p| stroke.iter().any(|q| near(*p, *q))), "it follows the pointer: {stroke:?}");
        assert_eq!(items(&h), before, "nothing is made before the release");
        // Letting go makes the path and ends the preview.
        h.drop_at(*path.last().unwrap());
        h.run_steps(3);
        assert_eq!(items(&h), before + 1);
        let app = &mut h.state_mut().app;
        let view = app.view_info();
        assert!(app.session.overlays(view).is_empty(), "the preview is gone");
    }
}

#[cfg(test)]
mod scrollbar_tests {
    use egui::{Event, Modifiers, PointerButton};
    use egui_kittest::Harness;

    use super::*;
    use crate::test_window::{Window, open, wheel};

    fn window() -> Harness<'static, Window> {
        let mut h = open(DesignApp::new(pagenor_engine::Session::new(), crate::Services::default()), vec2(1440.0, 900.0));
        // Zoomed in so both axes scroll.
        set_zoom(&mut h.state_mut().app, 2.0);
        h.run_steps(3);
        h
    }

    fn origin(h: &Harness<'static, Window>) -> Point {
        h.state().app.view().map(|v| v.origin).unwrap_or_default()
    }

    /// The scrollbar's track and thumb as the canvas lays them out for the current view, and
    /// the content pixels per thumb pixel.
    fn bar(h: &Harness<'static, Window>, vertical: bool) -> (Rect, Rect, f32) {
        let app = &h.state().app;
        let rect = app.canvas_rect.unwrap();
        let st = app.session.active().unwrap();
        let xf = Xf::new(rect, app.view().unwrap());
        let cb = xf.rect(CanvasLayout::new(&st.doc, st.editing_parents).pasteboard);
        let outer = Rect::from_min_max(rect.min, rect.max + vec2(SCROLLBAR, SCROLLBAR));
        scrollbar_rects(outer, rect, cb, vertical)
    }

    /// A primary-button drag from `a` to `b` in a few moves, with `mods` held throughout.
    fn drag(h: &mut Harness<'static, Window>, a: Pos2, b: Pos2, mods: Modifiers) {
        h.event(Event::ModifiersChanged(mods));
        h.hover_at(a);
        h.step();
        h.event(Event::PointerButton { pos: a, button: PointerButton::Primary, pressed: true, modifiers: mods });
        h.step();
        for n in 1..=4 {
            h.hover_at(a + (b - a) * n as f32 / 4.0);
            h.step();
        }
        h.event(Event::PointerButton { pos: b, button: PointerButton::Primary, pressed: false, modifiers: mods });
        h.event(Event::ModifiersChanged(Modifiers::NONE));
        h.run_steps(3);
    }

    #[test]
    fn document_views_have_scrollbars_beside_the_view() {
        let h = window();
        let rect = h.state().app.canvas_rect.unwrap();
        for vertical in [true, false] {
            let r = h.ctx.read_response(egui::Id::new(("canvas_scrollbar", 0u8, vertical))).expect("scrollbar").rect;
            let (track, thumb, k) = bar(&h, vertical);
            assert_eq!(r, track);
            assert!(k > 0.0, "the zoomed-in view scrolls");
            if vertical {
                assert_eq!((r.min.x, r.width(), r.min.y, r.max.y), (rect.max.x, SCROLLBAR, rect.min.y, rect.max.y));
                assert!(thumb.height() < track.height());
            } else {
                assert_eq!((r.min.y, r.height(), r.min.x, r.max.x), (rect.max.y, SCROLLBAR, rect.min.x, rect.max.x));
                assert!(thumb.width() < track.width());
            }
        }
    }

    #[test]
    fn dragging_a_thumb_pans_and_clicking_the_track_pages() {
        let mut h = window();
        let zoom = h.state().app.view().unwrap().zoom;
        // Vertical thumb: down 40 px moves the view down by 40 thumb pixels' worth of content.
        let (_, thumb, k) = bar(&h, true);
        let before = origin(&h);
        drag(&mut h, thumb.center(), thumb.center() + vec2(0.0, 40.0), Modifiers::NONE);
        let after = origin(&h);
        let want = 40.0 * k as f64 / zoom;
        assert!((after.y - before.y - want).abs() < want * 0.2, "{before:?} → {after:?}, want +{want}");
        assert_eq!(after.x, before.x);
        // Horizontal thumb: left 30 px.
        let (_, thumb, k) = bar(&h, false);
        drag(&mut h, thumb.center(), thumb.center() - vec2(30.0, 0.0), Modifiers::NONE);
        let next = origin(&h);
        let want = 30.0 * k as f64 / zoom;
        assert!((after.x - next.x - want).abs() < want * 0.2, "{after:?} → {next:?}, want -{want}");
        assert_eq!(next.y, after.y);
        // A click on the vertical track above the thumb pages up by one view.
        let (track, thumb, _) = bar(&h, true);
        assert!(thumb.min.y - track.min.y > 4.0);
        crate::test_window::click_at(&mut h, pos2(track.center().x, track.min.y + 2.0));
        let paged = origin(&h);
        let view = h.state().app.canvas_rect.unwrap().height() as f64 / zoom;
        assert!((next.y - paged.y - view).abs() < 0.01, "{next:?} → {paged:?}, want -{view}");
    }

    #[test]
    fn scrolling_the_view_moves_the_thumbs() {
        let mut h = window();
        let (_, v0, _) = bar(&h, true);
        let (_, h0, _) = bar(&h, false);
        let c = h.state().app.canvas_rect.unwrap().center();
        wheel(&mut h, c, vec2(-50.0, -80.0));
        let (_, v1, _) = bar(&h, true);
        let (_, h1, _) = bar(&h, false);
        assert!(v1.min.y > v0.min.y && h1.min.x > h0.min.x, "{v0:?} → {v1:?}, {h0:?} → {h1:?}");
        let r = h.ctx.read_response(egui::Id::new(("canvas_scrollbar", 0u8, true))).expect("scrollbar");
        assert_eq!(r.rect, bar(&h, true).0);
    }

    #[test]
    fn alt_drag_while_editing_text_pans_and_keeps_the_caret() {
        let mut h = window();
        let app = &mut h.state_mut().app;
        let rect = app.canvas_rect.unwrap();
        let xf = Xf::new(rect, app.view().unwrap());
        // A text frame filling the visible part of the page, with the caret in it.
        let (a, b) = (xf.to_canvas(rect.min + vec2(20.0, 20.0)), xf.to_canvas(rect.max - vec2(20.0, 20.0)));
        let r = app.run("frame.create", json!({"rect": [a.x, a.y, b.x, b.y], "content": "text", "text": "Hello brave world"})).unwrap();
        let sid = r["story"].as_u64().unwrap();
        app.run("tool.select", json!({"tool": "type"})).unwrap();
        app.run("text.select", json!({"story": sid, "anchor": 3, "focus": 3})).unwrap();
        assert!(app.session.wants_text());
        h.run_steps(2);
        let caret = h.state().app.session.doc().unwrap().selection.text;
        let before = origin(&h);
        let zoom = h.state().app.view().unwrap().zoom;
        let from = rect.center();
        drag(&mut h, from, from + vec2(60.0, 40.0), Modifiers::ALT);
        let after = origin(&h);
        assert!(after.x < before.x && after.y < before.y, "{before:?} → {after:?}");
        assert!((before.x - after.x - 60.0 / zoom).abs() < 15.0 / zoom, "{before:?} → {after:?}");
        assert_eq!(h.state().app.session.doc().unwrap().selection.text, caret);
        assert!(h.state_mut().app.session.wants_text(), "still editing text");
        // Without Alt the same drag selects text and leaves the view alone.
        drag(&mut h, from, from + vec2(60.0, 40.0), Modifiers::NONE);
        assert_eq!(origin(&h), after);
    }
}

#[cfg(test)]
#[path = "clipboard_menu_paste_tests.rs"]
mod clipboard_menu_paste_tests;
