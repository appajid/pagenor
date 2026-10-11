//! Pages panel: parents at the top, document spreads with live thumbnails below.

use std::collections::HashMap;
use std::sync::Mutex;

use egui::{Color32, Sense, Stroke, StrokeKind, vec2};
use serde_json::json;

use crate::theme::Tokens;
use crate::{DesignApp, icons};

type ThumbKey = (u64, u64, bool, usize, usize);

static THUMBS: Mutex<Option<HashMap<ThumbKey, egui::TextureHandle>>> = Mutex::new(None);

fn thumb(app: &mut DesignApp, ctx: &egui::Context, abs: usize, h: f32) -> Option<egui::TextureHandle> {
    let st = app.session.active()?;
    let key = (st.uid, st.revision, false, abs, 0);
    let mut g = THUMBS.lock().unwrap_or_else(|e| e.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    if let Some(t) = map.get(&key) {
        return Some(t.clone());
    }
    let page = st.doc.page(abs)?;
    let scale = (h as f64 * ctx.pixels_per_point() as f64) / page.height;
    let mut r = pagenor_render::Renderer::new();
    r.threads = 0;
    let img =
        r.render_page(&st.doc, &app.session.cache, abs, scale, false, &pagenor_render::RenderOptions { printing_only: true, ..Default::default() })?;
    let ci = egui::ColorImage::from_rgba_premultiplied([img.width as usize, img.height as usize], &img.pixels);
    let tex = crate::widgets::load_texture(ctx, format!("thumb{abs}"), ci, egui::TextureOptions::LINEAR);
    // Drop thumbnails of older revisions.
    let (uid, rev) = (st.uid, st.revision);
    map.retain(|k, _| !(k.0 == uid && k.1 != rev));
    map.insert(key, tex.clone());
    Some(tex)
}

/// What is being dragged in the Pages panel.
#[derive(Clone, Debug, PartialEq)]
enum PageDrag {
    /// A document page (absolute index).
    Page(usize),
    /// A parent (prefix; `None` = [None]).
    Parent(Option<String>),
}

/// Selected-page tint (multiplied over the thumbnail).
const SELECTED_TINT: Color32 = Color32::from_rgb(0x88, 0xc7, 0xfb);
/// Page-number badge of the selected page.
const BADGE: Color32 = Color32::from_rgb(0x2f, 0x73, 0xe0);
/// Thumbnail height (InDesign: about 28×37 pt for Letter).
const THUMB_H: f32 = 37.0;
/// Live master thumbnail, cropped to its selected side.
fn master_thumb(app: &mut DesignApp, ctx: &egui::Context, index: usize, pi: usize) -> Option<egui::TextureHandle> {
    let st = app.session.active()?;
    let key = (st.uid, st.revision, true, index, pi);
    let mut g = THUMBS.lock().unwrap_or_else(|e| e.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    if let Some(t) = map.get(&key) {
        return Some(t.clone());
    }
    let page = st.doc.parents.get(index)?.pages.get(pi)?;
    let scale = (THUMB_H as f64 * ctx.pixels_per_point() as f64) / page.height.max(1.0);
    let mut renderer = pagenor_render::Renderer::new();
    renderer.threads = 0;
    let image = renderer.render(
        &st.doc,
        &app.session.cache,
        &[pagenor_render::Placed { spread: pagenor_doc::SpreadRef::Parent(index), xf: pagenor_geom::Affine::IDENTITY }],
        (page.width * scale).ceil() as u32,
        (page.height * scale).ceil() as u32,
        pagenor_geom::Affine::scale(scale) * pagenor_geom::Affine::translate((-page.x, 0.0)),
        &pagenor_render::RenderOptions { printing_only: true, ..Default::default() },
    );
    let ci = egui::ColorImage::from_rgba_premultiplied([image.width as usize, image.height as usize], &image.pixels);
    let tex = crate::widgets::load_texture(ctx, format!("master{index}-{pi}"), ci, egui::TextureOptions::LINEAR);
    map.retain(|k, _| !(k.0 == st.uid && k.1 != st.revision));
    map.insert(key, tex.clone());
    Some(tex)
}

fn master_menu(app: &mut DesignApp, ui: &mut egui::Ui, index: usize, prefix: &str, current_page: usize) {
    if ui.button("Edit master spread").clicked() {
        let _ = app.run("app.editMaster", json!({"index":index}));
        ui.close();
    }
    if ui.button("Master Page Settings…").clicked() {
        let _ = app.run("app.masterSettings", json!({"index":index}));
        ui.close();
    }
    if ui.button("Apply to all pages (default)").clicked() {
        let _ = app.run("layout.pages.applyParent", json!({"parent":prefix,"setDefault":true}));
        ui.close();
    }
    if ui.button("Apply to current page").clicked() {
        let _ = app.run("layout.pages.applyParent", json!({"parent":prefix,"pages":[current_page]}));
        ui.close();
    }
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some(st) = app.session.active() else { return };
    let doc = st.doc.clone();
    let editing_parents = st.editing_parents;

    let mut drop: Option<(PageDrag, usize)> = None;
    ui.spacing_mut().item_spacing.y = 0.0;
    // Masters are separate from document pages, with direct access to both book sides.
    let current_master = match crate::canvas::current_spread(app) {
        Some(pagenor_doc::SpreadRef::Parent(i)) => Some(i),
        _ => None,
    };
    let current_page = crate::canvas::current_page(app).unwrap_or(0);
    let current_master_page = crate::canvas::current_master_page(app);
    ui.horizontal(|ui| {
        ui.strong("Master Pages");
        if ui.small_button("New…").on_hover_text("Create a master page or a left/right master pair").clicked() {
            let _ = app.run("app.newMaster", json!({}));
        }
    });
    if editing_parents {
        if ui.button("Back to document").clicked() {
            let _ = app.run("layout.parents.edit", json!({"on":false}));
        }
        ui.label(egui::RichText::new("Edit common items here; they repeat on document pages.").size(10.5).color(t.text_dim));
    }
    for (index, master) in doc.parents.iter().enumerate() {
        let Some(info) = &master.parent else { continue };
        let active = current_master == Some(index);
        let label = format!("{}{}", info.label(), if doc.default_parent() == Some(master.id) { " · Default" } else { "" });
        ui.horizontal(|ui| {
            let resp = ui.add(egui::Button::selectable(active, &label).sense(Sense::click_and_drag()));
            resp.dnd_set_drag_payload(PageDrag::Parent(Some(info.prefix.clone())));
            if resp.double_clicked() {
                let _ = app.run("app.editMaster", json!({"index":index}));
            }
            resp.context_menu(|ui| master_menu(app, ui, index, &info.prefix, current_page));
            crate::menus::menu_button(ui, "⋯", |ui| master_menu(app, ui, index, &info.prefix, current_page));
        });
        ui.horizontal(|ui| {
            // Visual left/right order stays stable for right-to-left books too.
            let mut pages: Vec<_> = master.pages.iter().enumerate().collect();
            pages.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
            for (pi, page) in pages {
                let name = match page.side {
                    pagenor_doc::PageSide::Left => "Left Master",
                    pagenor_doc::PageSide::Right => "Right Master",
                    _ => "Master",
                };
                ui.vertical(|ui| {
                    let size = vec2(THUMB_H * (page.width / page.height.max(1.0)) as f32, THUMB_H);
                    let resp = if let Some(tex) = master_thumb(app, ui.ctx(), index, pi) {
                        ui.add(egui::Button::image(egui::Image::new((tex.id(), size))).selected(current_master_page == Some((index, pi))))
                    } else {
                        ui.add_sized(size, egui::Button::new("▯"))
                    };
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("{} {name}", info.label())));
                    if resp.on_hover_text(format!("Edit {} {name}", info.label())).clicked() {
                        let _ = app.run("app.editMaster", json!({"index":index,"page":pi}));
                    }
                    if ui.small_button(name).clicked() {
                        let _ = app.run("app.editMaster", json!({"index":index,"page":pi}));
                    }
                });
            }
        });
        if active {
            ui.horizontal_wrapped(|ui| {
                for (label, kind) in [("Header…", "header"), ("Footer…", "footer"), ("Page Numbers", "pageNumber")] {
                    if ui.small_button(label).clicked() {
                        let _ = app.run("app.masterText", json!({"index":index,"kind":kind}));
                    }
                }
                if ui.small_button("Settings…").clicked() {
                    let _ = app.run("app.masterSettings", json!({"index":index}));
                }
            });
        }
        ui.add_space(6.0);
    }
    let none = ui.add(egui::Button::new(egui::RichText::new("[None] · no master").size(10.5).color(t.text_dim)).sense(Sense::click_and_drag()));
    none.dnd_set_drag_payload(PageDrag::Parent(None));
    ui.add_space(4.0);
    let r = ui.available_rect_before_wrap();
    ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(r.min.x - 10.0, r.min.y), vec2(r.width() + 20.0, 1.5)), 0.0, t.section_divider);
    ui.add_space(12.0);
    // ---- Document spreads.
    let cur = crate::canvas::current_page(app).unwrap_or(0);
    let cur_spread = doc.page_loc(cur).map(|l| l.0);
    let center = ui.available_width() / 2.0;
    let dragging = egui::DragAndDrop::has_any_payload(ui.ctx());
    for (si, sp) in doc.spreads.iter().enumerate() {
        let first = doc.first_page_of_spread(si);
        let scale = THUMB_H / doc.settings.page_height.max(1.0) as f32;
        let spine = sp.spine_x() as f32;
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.0 + THUMB_H + 22.0), Sense::hover());
        let top = row.min.y + 8.0;
        // Marker above the current spread.
        if cur_spread == Some(si) && !editing_parents {
            let c = egui::pos2(row.min.x + center, row.min.y + 3.5);
            ui.painter().add(egui::Shape::convex_polygon(vec![c + vec2(-5.0, -3.0), c + vec2(5.0, -3.0), c + vec2(0.0, 3.0)], t.icon, Stroke::NONE));
        }
        for (pi, p) in sp.pages.iter().enumerate() {
            let abs = first + pi;
            let x0 = row.min.x + center + (p.x as f32 - spine) * scale;
            let pr = egui::Rect::from_min_size(egui::pos2(x0, top), vec2(p.width as f32 * scale, p.height as f32 * scale));
            let resp = ui.interact(pr, ui.id().with(("page", abs)), Sense::click_and_drag());
            let selected = abs == cur && !editing_parents;
            let tint = if selected { SELECTED_TINT } else { Color32::WHITE };
            if let Some(tex) = thumb(app, ui.ctx(), abs, THUMB_H) {
                ui.painter().image(tex.id(), pr, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), tint);
            } else {
                ui.painter().rect_filled(pr, 0.0, tint);
            }
            ui.painter().rect_stroke(pr, 0.0, Stroke::new(0.5, Color32::from_gray(0x4a)), StrokeKind::Outside);
            // Parent prefix in the outer top corner (black on white).
            if let Some(pid) = p.parent
                && let Some(pp) = doc.parents.iter().find(|x| x.id == pid)
            {
                let prefix = pp.parent.as_ref().map(|i| i.prefix.clone()).unwrap_or_default();
                let g = ui.painter().layout_no_wrap(prefix, crate::theme::semibold(9.5), Color32::BLACK);
                let w = g.size().x + 2.0;
                let br = if p.side == pagenor_doc::PageSide::Left {
                    egui::Rect::from_min_size(pr.left_top(), vec2(w, 10.0))
                } else {
                    egui::Rect::from_min_size(pr.right_top() - vec2(w, 0.0), vec2(w, 10.0))
                };
                ui.painter().rect_filled(br, 0.0, Color32::WHITE);
                ui.painter().galley(br.min + vec2(1.0, -0.5), g, Color32::BLACK);
            }
            // Drag and drop: pages reorder, parents apply.
            resp.dnd_set_drag_payload(PageDrag::Page(abs));
            if resp.dragged() {
                drag_ghost(ui, &app.session.page_label(abs), &t);
            }
            if dragging && resp.contains_pointer() {
                ui.painter().rect_stroke(pr.expand(1.5), 0.0, Stroke::new(2.0, BADGE), StrokeKind::Outside);
            }
            if let Some(payload) = resp.dnd_release_payload::<PageDrag>() {
                drop = Some(((*payload).clone(), abs));
            }
            if resp.clicked() || resp.double_clicked() {
                if editing_parents {
                    let _ = app.run("layout.parents.edit", json!({"on": false}));
                }
                crate::canvas::go_to_page(app, abs);
            }
            resp.context_menu(|ui| page_menu(app, ui, &doc, abs, si));
            // Page number under the page: white on a blue badge when selected.
            let name = app.session.page_label(abs);
            let color = if selected { Color32::WHITE } else { t.text };
            let g = ui.painter().layout_no_wrap(name, crate::theme::semibold(11.0), color);
            let c = egui::pos2(pr.center().x, pr.max.y + 11.0);
            if selected {
                let br = egui::Rect::from_center_size(c, vec2(g.size().x.max(7.0) + 7.0, 12.5));
                ui.painter().rect_filled(br, 0.0, BADGE);
            }
            ui.painter().galley(c - g.size() / 2.0, g, Color32::WHITE);
        }
    }
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    ui.horizontal(|ui| {
        crate::rtl::label(
            ui,
            egui::RichText::new(
                crate::i18n::tr(&app.ui.language, "{pages} Pages in {spreads} Spreads")
                    .replace("{pages}", &doc.page_count().to_string())
                    .replace("{spreads}", &doc.spreads.len().to_string()),
            )
            .size(11.0)
            .color(t.text_dim),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if icons::button(ui, "trash", 20.0, false, crate::i18n::tr(&app.ui.language, "Delete Page")).clicked() {
                let _ = app.run("layout.pages.delete", json!({"pages": [cur]}));
            }
            if icons::button(ui, "plus", 20.0, false, crate::i18n::tr(&app.ui.language, "Insert Page")).clicked() {
                let _ = app.run("layout.pages.insert", json!({"after": cur, "count": 1}));
            }
        });
    });
    match drop {
        Some((PageDrag::Page(from), to)) if from != to => {
            let _ = app.run("layout.pages.move", json!({"from": from, "to": to}));
        }
        Some((PageDrag::Parent(prefix), to)) => {
            let _ = app.run("layout.pages.applyParent", json!({"pages": [to], "parent": prefix}));
        }
        _ => {}
    }
}

fn page_menu(app: &mut DesignApp, ui: &mut egui::Ui, doc: &pagenor_doc::Document, abs: usize, si: usize) {
    if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Insert Page After"))).clicked() {
        let _ = app.run("layout.pages.insert", json!({"after": abs, "count": 1}));
        ui.close();
    }
    if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Delete Page"))).clicked() {
        let _ = app.run("layout.pages.delete", json!({"pages": [abs]}));
        ui.close();
    }
    if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Duplicate Spread"))).clicked() {
        let _ = app.run("layout.pages.duplicateSpread", json!({"spread": si}));
        ui.close();
    }
    if si > 0
        && ui
            .button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Add to Previous Spread")))
            .on_hover_ui(|ui| {
                crate::rtl::label(ui, crate::i18n::tr(&app.ui.language, "Spreads of up to 10 pages"));
            })
            .clicked()
    {
        let _ = app.run("layout.pages.toSpread", json!({"page": abs + 1, "spread": si - 1}));
        ui.close();
    }
    let mut shuffle = doc.spreads.get(si).is_none_or(|sp| sp.allow_shuffle);
    if ui.checkbox(&mut shuffle, crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Allow Spread to Shuffle"))).clicked() {
        let _ = app.run("layout.spreadShuffle", json!({"spread": si, "allow": shuffle}));
        ui.close();
    }
    ui.separator();
    for (id, label) in [
        ("layout.overrideParentItems", "Override All Master Page Items"),
        ("layout.removeOverrides", "Remove All Local Overrides"),
        ("layout.detachAll", "Detach All Objects from Master"),
    ] {
        if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, label))).clicked() {
            let _ = app.run(id, json!({"page": abs}));
            ui.close();
        }
    }
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Apply Master")), |ui| {
        if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "[None]"))).clicked() {
            let _ = app.run("layout.pages.applyParent", json!({"pages": [abs], "parent": null}));
            ui.close();
        }
        for pp in &doc.parents {
            let l = pp.parent.as_ref().map(|i| i.label()).unwrap_or_default();
            if ui.button(&l).clicked() {
                let _ = app.run("layout.pages.applyParent", json!({"pages": [abs], "parent": pp.parent.as_ref().map(|i| i.prefix.clone())}));
                ui.close();
            }
        }
    });
}

/// A small label following the pointer while dragging.
pub(crate) fn drag_ghost(ui: &egui::Ui, label: &str, t: &Tokens) {
    let Some(p) = ui.ctx().pointer_interact_pos() else { return };
    let painter = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("page_drag")));
    let g = painter.layout_no_wrap(label.to_string(), egui::FontId::proportional(11.0), t.text_strong);
    let r = egui::Rect::from_min_size(p + vec2(10.0, 8.0), g.size() + vec2(10.0, 6.0));
    painter.rect(r, 2.0, t.panel_darker, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
    painter.galley(r.min + vec2(5.0, 3.0), g, t.text_strong);
}
