//! Pages panel: parents at the top, document spreads with live thumbnails below.

use egui::{Color32, Sense, Stroke, StrokeKind, vec2};
use serde_json::json;

use crate::theme::Tokens;
use crate::{DesignApp, icons};

fn thumb(app: &mut DesignApp, ctx: &egui::Context, abs: usize, h: f32) -> Option<egui::TextureHandle> {
    let st = app.session.active()?;
    app.page_thumbnails.get(ctx, st.uid, st.doc.clone(), app.session.cache.clone(), crate::page_thumbnails::Target::Page(abs), h)
}

/// Thumbnail sizes of the Pages panel (Panel Options › Size), InDesign's six.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IconSize {
    ExtraSmall,
    #[default]
    Small,
    Medium,
    Large,
    ExtraLarge,
    Jumbo,
}

impl IconSize {
    pub const ALL: [IconSize; 6] = [IconSize::ExtraSmall, IconSize::Small, IconSize::Medium, IconSize::Large, IconSize::ExtraLarge, IconSize::Jumbo];

    pub fn label(self) -> &'static str {
        match self {
            IconSize::ExtraSmall => "Extra Small",
            IconSize::Small => "Small",
            IconSize::Medium => "Medium",
            IconSize::Large => "Large",
            IconSize::ExtraLarge => "Extra Large",
            IconSize::Jumbo => "Jumbo",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            IconSize::ExtraSmall => "extraSmall",
            IconSize::Small => "small",
            IconSize::Medium => "medium",
            IconSize::Large => "large",
            IconSize::ExtraLarge => "extraLarge",
            IconSize::Jumbo => "jumbo",
        }
    }

    pub fn from_key(k: &str) -> Option<IconSize> {
        IconSize::ALL.into_iter().find(|s| s.key() == k)
    }

    /// Height of a page thumbnail.
    fn page_h(self) -> f32 {
        [14.0, 24.0, THUMB_H, 55.0, 80.0, 120.0][self as usize]
    }

    /// Height of a parent row.
    fn row_h(self) -> f32 {
        [18.0, ROW_H, 40.0, 56.0, 76.0, 110.0][self as usize]
    }
}

/// How the Pages panel's parts share a change of the panel's height (Panel Options › Resize).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PanelResize {
    Proportional,
    PagesFixed,
    #[default]
    ParentsFixed,
}

impl PanelResize {
    pub const ALL: [PanelResize; 3] = [PanelResize::Proportional, PanelResize::PagesFixed, PanelResize::ParentsFixed];

    pub fn label(self) -> &'static str {
        match self {
            PanelResize::Proportional => "Proportional",
            PanelResize::PagesFixed => "Pages Fixed",
            PanelResize::ParentsFixed => "Parents Fixed",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            PanelResize::Proportional => "proportional",
            PanelResize::PagesFixed => "pagesFixed",
            PanelResize::ParentsFixed => "parentsFixed",
        }
    }

    pub fn from_key(k: &str) -> Option<PanelResize> {
        PanelResize::ALL.into_iter().find(|s| s.key() == k)
    }
}

/// The Pages panel's Panel Options, as InDesign's (kept with the UI settings).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PanelOptions {
    pub page_size: IconSize,
    pub page_thumbnails: bool,
    pub parent_size: IconSize,
    pub parents_vertical: bool,
    pub pages_on_top: bool,
    pub resize: PanelResize,
}

impl Default for PanelOptions {
    fn default() -> Self {
        PanelOptions {
            page_size: IconSize::Medium,
            page_thumbnails: true,
            parent_size: IconSize::Small,
            parents_vertical: true,
            pages_on_top: false,
            resize: PanelResize::ParentsFixed,
        }
    }
}

/// What is being dragged in the Pages panel.
#[derive(Clone, Debug, PartialEq)]
enum PageDrag {
    /// A document page (absolute index).
    Page(usize),
    /// A parent (prefix; `None` = [None]).
    Parent(Option<String>),
}

/// The footer: page count and buttons, always at the panel's bottom.
const FOOTER_H: f32 = 28.0;
/// The draggable rule between the parents and the pages.
const DIVIDER_H: f32 = 9.0;
/// Parent rows shown at first; the divider shows more.
const PARENTS_DEFAULT_ROWS: f32 = 4.5;
/// The footer: page count and buttons, always at the panel's bottom.
/// The draggable rule between the parents and the pages.
/// Parent rows shown at first; the divider shows more.
/// Selected-page tint (multiplied over the thumbnail).
const SELECTED_TINT: Color32 = Color32::from_rgb(0x88, 0xc7, 0xfb);
/// Page-number badge of the selected page.
const BADGE: Color32 = Color32::from_rgb(0x2f, 0x73, 0xe0);
/// Thumbnail height (InDesign: about 28×37 pt for Letter).
const THUMB_H: f32 = 37.0;
const ROW_H: f32 = 29.0;
/// Live master thumbnail, cropped to its selected side.
fn master_thumb(app: &mut DesignApp, ctx: &egui::Context, index: usize, page: usize, h: f32) -> Option<egui::TextureHandle> {
    let st = app.session.active()?;
    app.page_thumbnails.get(ctx, st.uid, st.doc.clone(), app.session.cache.clone(), crate::page_thumbnails::Target::Master(index, page), h)
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
    ui.separator();
    if ui.button("Delete Master").clicked() {
        if let Some(id) = app.session.active().and_then(|st| st.doc.parents.get(index)).map(|p| p.id)
            && app.run("layout.parents.delete", json!({"id":id.0})).is_ok()
            && let Some(v) = app.view_mut()
        {
            v.fitted = false;
        }
        ui.close();
    }
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some(st) = app.session.active() else { return };
    let doc = st.doc.clone();
    let editing_parents = st.editing_parents;
    let aspect = (doc.settings.page_width / doc.settings.page_height.max(1.0)) as f32;
    let mut drop: Option<(PageDrag, usize)> = None;
    ui.spacing_mut().item_spacing.y = 0.0;
    // As in InDesign: parents and pages each scroll on their own (parents on top unless Panel
    // Options says pages), a divider between them resizes them, the footer stays at the bottom.
    let opts = app.ui.pages_panel.clone();
    let total = {
        let h = ui.available_height();
        if h.is_finite() && h < 4000.0 { h } else { 480.0 }
    }
    .max(160.0);
    let row_h = opts.parent_size.row_h();
    let icon_h = row_h - 5.0;
    let card_height = icon_h + 76.0;
    let per_row = (ui.available_width() / (2.0 * icon_h * aspect + 32.0).max(180.0)).floor().max(1.0);
    let natural = if opts.parents_vertical {
        doc.parents.len() as f32 * card_height + 60.0
    } else {
        (doc.parents.len() as f32 / per_row).ceil() * card_height + 60.0
    };
    let space = (total - FOOTER_H - DIVIDER_H).max(2.0 * ROW_H);
    let parents_max = (space - 2.0 * ROW_H).max(ROW_H);
    let parents_min = row_h.min(natural).min(parents_max);
    // The divider's place as last dragged, and how a change of the panel's height is shared.
    let parents_h = match app.ui.pages_parents_height {
        None => natural.min((2.0 * card_height + 60.0).max(PARENTS_DEFAULT_ROWS * ROW_H)),
        Some([h, at]) => match opts.resize {
            PanelResize::ParentsFixed => h,
            PanelResize::PagesFixed => h + (space - at),
            PanelResize::Proportional => h * space / at.max(1.0),
        },
    }
    .clamp(parents_min, parents_max)
    .min(natural);
    let pages_h = (space - parents_h).max(ROW_H);
    let parents_part = |app: &mut DesignApp, ui: &mut egui::Ui| {
        egui::ScrollArea::vertical().id_salt("pages_parents").max_height(parents_h).min_scrolled_height(parents_h).auto_shrink([false, true]).show(
            ui,
            |ui| {
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
                let draw_master = |app: &mut DesignApp, ui: &mut egui::Ui, index: usize, master: &std::sync::Arc<pagenor_doc::Spread>| {
                    let Some(info) = &master.parent else { return };
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
                                let size = vec2(icon_h * (page.width / page.height.max(1.0)) as f32, icon_h);
                                let resp = if let Some(tex) = master_thumb(app, ui.ctx(), index, pi, icon_h) {
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
                };
                if opts.parents_vertical {
                    for (index, master) in doc.parents.iter().enumerate() {
                        draw_master(app, ui, index, master);
                    }
                } else {
                    ui.horizontal_wrapped(|ui| {
                        for (index, master) in doc.parents.iter().enumerate() {
                            ui.allocate_ui_with_layout(
                                vec2((icon_h * aspect * master.pages.len() as f32 + 32.0).max(180.0), icon_h + 72.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    draw_master(app, ui, index, master);
                                },
                            );
                        }
                    });
                }
                let none =
                    ui.add(egui::Button::new(egui::RichText::new("[None] · no master").size(10.5).color(t.text_dim)).sense(Sense::click_and_drag()));
                none.dnd_set_drag_payload(PageDrag::Parent(None));
                ui.add_space(4.0);
            },
        );
    };
    // ---- Document spreads.
    let cur = crate::canvas::current_page(app).unwrap_or(0);
    let cur_spread = doc.page_loc(cur).map(|l| l.0);
    let dragging = egui::DragAndDrop::has_any_payload(ui.ctx());
    let tallest = doc.spreads.iter().flat_map(|sp| sp.pages.iter().map(|p| p.height)).fold(doc.settings.page_height, f64::max).max(1.0);
    let th = opts.page_size.page_h();
    // Extra Small has no room for a thumbnail; Show Thumbnails off draws blank pages.
    let thumbnails = opts.page_thumbnails && opts.page_size != IconSize::ExtraSmall;
    let mut pages_part = |app: &mut DesignApp, ui: &mut egui::Ui| {
        let center = ui.available_width() / 2.0;
        egui::ScrollArea::vertical().id_salt("pages_spreads").max_height(pages_h).min_scrolled_height(pages_h).auto_shrink([false, false]).show(
            ui,
            |ui| {
                ui.add_space(4.0);
                for (si, sp) in doc.spreads.iter().enumerate() {
                    let first = doc.first_page_of_spread(si);
                    // One scale for the whole document (pages of their own size show in proportion): the
                    // document's page size at the panel's size, unless a taller page would pass 1.8 times it.
                    // Each row is as tall as its spread's tallest page.
                    let scale = (th / doc.settings.page_height.max(1.0) as f32).min(1.8 * th / tallest as f32);
                    let row_h = sp.pages.iter().map(|p| p.height as f32 * scale).fold(0.0, f32::max).max(th * 0.25);
                    let spine = sp.spine_x() as f32;
                    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.0 + row_h + 22.0), Sense::hover());
                    let top = row.min.y + 8.0;
                    // Marker above the current spread.
                    if cur_spread == Some(si) && !editing_parents {
                        let c = egui::pos2(row.min.x + center, row.min.y + 3.5);
                        ui.painter().add(egui::Shape::convex_polygon(
                            vec![c + vec2(-5.0, -3.0), c + vec2(5.0, -3.0), c + vec2(0.0, 3.0)],
                            t.icon,
                            Stroke::NONE,
                        ));
                    }
                    // Pages narrower than their numbers ("10" under an Extra Small page) are labelled
                    // once for the spread.
                    let narrow = sp.pages.len() > 1
                        && (first..first + sp.pages.len()).zip(&sp.pages).any(|(abs, p)| {
                            let w = ui.painter().layout_no_wrap(app.session.page_label(abs), crate::theme::semibold(11.0), Color32::WHITE).size().x;
                            (p.width as f32 * scale) < w + 6.0
                        });
                    for (pi, p) in sp.pages.iter().enumerate() {
                        let abs = first + pi;
                        let x0 = row.min.x + center + (p.x as f32 - spine) * scale;
                        let pr = egui::Rect::from_min_size(egui::pos2(x0, top), vec2(p.width as f32 * scale, p.height as f32 * scale));
                        let resp = ui.interact(pr, ui.id().with(("page", abs)), Sense::click_and_drag());
                        let selected = abs == cur && !editing_parents;
                        // Screen readers hear "Page 3".
                        resp.widget_info(|| {
                            egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, format!("Page {}", app.session.page_label(abs)))
                        });
                        let tint = if selected { SELECTED_TINT } else { Color32::WHITE };
                        match thumbnails.then(|| thumb(app, ui.ctx(), abs, th)).flatten() {
                            Some(tex) => {
                                ui.painter().image(tex.id(), pr, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), tint);
                            }
                            None => {
                                ui.painter().rect_filled(pr, 0.0, tint);
                            }
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
                                app.views.clear();
                            }
                            crate::canvas::go_to_page(app, abs);
                        }
                        resp.context_menu(|ui| page_menu(app, ui, &doc, abs, si));
                        // Page number under the page: white on a blue badge when selected. Pages too
                        // narrow for their numbers share one label under the spread (below).
                        if narrow {
                            continue;
                        }
                        let name = app.session.page_label(abs);
                        let g = ui.painter().layout_no_wrap(name, crate::theme::semibold(11.0), if selected { Color32::WHITE } else { t.text });
                        let c = egui::pos2(pr.center().x, pr.max.y + 11.0);
                        if selected {
                            let br = egui::Rect::from_center_size(c, vec2(g.size().x.max(7.0) + 7.0, 12.5));
                            ui.painter().rect_filled(br, 0.0, BADGE);
                        }
                        ui.painter().galley(c - g.size() / 2.0, g, Color32::WHITE);
                    }
                    if narrow {
                        let names: Vec<String> = (first..first + sp.pages.len()).map(|abs| app.session.page_label(abs)).collect();
                        let label = match (names.first(), names.last()) {
                            (Some(a), Some(b)) if names.len() > 1 => format!("{a}\u{2013}{b}"),
                            (Some(a), _) => a.clone(),
                            _ => String::new(),
                        };
                        let g = ui.painter().layout_no_wrap(
                            label,
                            crate::theme::semibold(11.0),
                            if cur_spread == Some(si) && !editing_parents { Color32::WHITE } else { t.text },
                        );
                        let c = egui::pos2(row.min.x + center, top + row_h + 11.0);
                        if cur_spread == Some(si) && !editing_parents {
                            ui.painter().rect_filled(egui::Rect::from_center_size(c, vec2(g.size().x + 7.0, 12.5)), 0.0, BADGE);
                        }
                        ui.painter().galley(c - g.size() / 2.0, g, Color32::WHITE);
                    }
                }
                // Right-click on the empty part of the panel: the short menu.
                let rest = ui.available_rect_before_wrap();
                let (_, empty) = ui.allocate_exact_size(vec2(ui.available_width(), rest.height().max(8.0)), Sense::click());
                empty.context_menu(|ui| empty_menu(app, ui));
            },
        );
    };
    // The divider: drag it to show more or fewer parents.
    let divider = |app: &mut DesignApp, ui: &mut egui::Ui| {
        let (dr, dresp) = ui.allocate_exact_size(vec2(ui.available_width(), DIVIDER_H), Sense::drag());
        let line = egui::Rect::from_center_size(dr.center(), vec2(dr.width() + 20.0, 1.5));
        ui.painter().rect_filled(line, 0.0, if dresp.hovered() || dresp.dragged() { t.text_dim } else { t.section_divider });
        let dresp = dresp.on_hover_cursor(egui::CursorIcon::ResizeVertical);
        if dresp.dragged() {
            let dy = dresp.drag_delta().y * if opts.pages_on_top { -1.0 } else { 1.0 };
            app.ui.pages_parents_height = Some([(parents_h + dy).clamp(parents_min, parents_max), space]);
        }
        dresp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "Parents and pages divider"));
    };
    if opts.pages_on_top {
        pages_part(app, ui);
        divider(app, ui);
        parents_part(app, ui);
    } else {
        parents_part(app, ui);
        divider(app, ui);
        pages_part(app, ui);
    }
    ui.spacing_mut().item_spacing.y = 4.0;
    ui.horizontal(|ui| {
        ui.set_min_height(FOOTER_H - 4.0);
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
            if icons::button(ui, "trash", 20.0, false, crate::i18n::tr(&app.ui.language, "Delete selected pages")).clicked() {
                let _ = crate::dialogs::delete_pages(app, vec![cur]);
            }
            // Create new page; a page dropped on it is duplicated at the end of the document.
            let plus = icons::button(ui, "plus", 20.0, dragging, crate::i18n::tr(&app.ui.language, "Create new page"));
            if let Some(payload) = plus.dnd_release_payload::<PageDrag>() {
                if let PageDrag::Page(from) = *payload
                    && let Some((si, _)) = doc.page_loc(from)
                {
                    let _ = app.run("layout.pages.duplicateSpread", json!({"spread": si, "atEnd": true}));
                }
            } else if plus.clicked() {
                let _ = app.run("layout.pages.insert", json!({"after": cur, "count": 1}));
            }
            // Edit page size: InDesign's presets for the current page, or Custom Page Size….
            let size = icons::button(ui, "panel-pages", 20.0, false, crate::i18n::tr(&app.ui.language, "Edit page size"));
            egui::Popup::menu(&size).show(|ui| {
                let Some((si, pi)) = doc.page_loc(cur) else { return };
                let pg = &doc.spreads[si].pages[pi];
                let landscape = pg.width > pg.height;
                let current = crate::panels::properties::preset_name(pg.width, pg.height);
                for (name, w, h) in crate::panels::properties::PAGE_PRESETS {
                    let (w, h) = if landscape { (h, w) } else { (w, h) };
                    if ui.selectable_label(current == *name, crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, name))).clicked() {
                        let _ = app.run("layout.pageSize", json!({"pages": [cur + 1], "width": w, "height": h}));
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Custom Page Size…"))).clicked() {
                    app.ui.dialog =
                        Some(crate::dialogs::Dialog::new("cmd:layout.pageSize", json!({"pages": [cur + 1], "width": pg.width, "height": pg.height})));
                    ui.close();
                }
            });
        });
    });
    // Alt-drag copies a page that is a spread of its own to where it's dropped, in one undo
    // step; a page from a larger spread moves as usual.
    let alt = ui.input(|i| i.modifiers.alt);
    let copy = |from: usize| {
        let (si, _) = doc.page_loc(from)?;
        (alt && doc.spreads.get(si).is_some_and(|s| s.pages.len() == 1)).then_some(si)
    };
    match drop {
        Some((PageDrag::Page(from), to)) => {
            if let Some(si) = copy(from) {
                let _ = app.run("layout.pages.duplicateSpread", json!({"spread": si, "to": to}));
            } else if from != to {
                let _ = app.run("layout.pages.move", json!({"from": from, "to": to}));
            }
        }
        Some((PageDrag::Parent(prefix), to)) => {
            let _ = app.run("layout.pages.applyParent", json!({"pages": [to], "parent": prefix}));
        }
        _ => {}
    }
}

/// Page actions from the panel's menu, using the current absolute page.
pub fn panel_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let Some(st) = app.session.active() else { return };
    let doc = st.doc.clone();
    let cur = crate::canvas::current_page(app).unwrap_or(0);
    let Some((si, _)) = doc.page_loc(cur) else { return };
    page_menu(app, ui, &doc, cur, si);
}

/// Right-click on the panel's empty part, as InDesign's: the commands that need no page.
fn empty_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let cur = crate::canvas::current_page(app).unwrap_or(0);
    let item = |ui: &mut egui::Ui, label: &str| ui.button(crate::rtl::widget(ui, crate::i18n::tr(&lang, label))).clicked();
    if item(ui, "Insert Pages…") {
        crate::dialogs::open_insert_pages(app, cur);
        ui.close();
    }
    if item(ui, "Move Pages…") {
        app.ui.dialog = Some(crate::dialogs::Dialog::new("cmd:layout.pages.move", json!({"from": cur, "to": cur})));
        ui.close();
    }
    ui.separator();
    if item(ui, "Panel Options…") {
        open_panel_options(app);
        ui.close();
    }
}

/// Panel Options…, with the panel's current options.
pub fn open_panel_options(app: &mut DesignApp) {
    let o = &app.ui.pages_panel;
    app.ui.dialog = Some(crate::dialogs::Dialog::new(
        "pagesPanelOptions",
        json!({
            "pageSize": o.page_size.key(), "pageThumbnails": o.page_thumbnails,
            "parentSize": o.parent_size.key(), "parentsVertical": o.parents_vertical,
            "layout": if o.pages_on_top { "pagesOnTop" } else { "parentsOnTop" }, "resize": o.resize.key(),
        }),
    ));
}

fn page_menu(app: &mut DesignApp, ui: &mut egui::Ui, doc: &pagenor_doc::Document, abs: usize, si: usize) {
    if ui.button("Insert Pages…").clicked() {
        crate::dialogs::open_insert_pages(app, abs);
        ui.close();
    }
    if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Insert Page After"))).clicked() {
        let _ = app.run("layout.pages.insert", json!({"after": abs, "count": 1}));
        ui.close();
    }
    if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Delete Page"))).clicked() {
        let _ = crate::dialogs::delete_pages(app, vec![abs]);
        ui.close();
    }
    if doc.spreads.get(si).is_some_and(|sp| sp.pages.len() > 1)
        && ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "Delete Spread"))).clicked()
    {
        let first = doc.first_page_of_spread(si);
        let count = doc.spreads.get(si).map_or(0, |sp| sp.pages.len());
        let _ = crate::dialogs::delete_pages(app, (first..first.saturating_add(count)).collect());
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

#[cfg(test)]
mod panel_options_regressions {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn the_footer_stays_put_and_the_divider_resizes_the_parents() {
        use egui::{Event, Modifiers, PointerButton};
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages": 60, "facingPages": true})).unwrap();
        for prefix in ["B", "C", "D", "E", "F", "G"] {
            app.run("layout.parents.new", json!({"prefix": prefix})).unwrap();
        }
        app.ui.dock_tab = "pages".into();
        let mut h = crate::test_window::open(app, vec2(1400.0, 900.0));
        let window = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1400.0, 900.0));
        for label in ["60 Pages in 31 Spreads", "Create new page", "Delete selected pages"] {
            assert!(window.contains_rect(h.get_by_label(label).rect()), "{label} is on screen");
        }
        // Seven parent rows: four and a half show at first; the rest scroll.
        let divider = h.get_by_label("Parents and pages divider").rect();
        let before = divider.center().y;
        assert!(h.query_by_label("G-Parent").is_none_or(|n| n.rect().top() > divider.top()), "the last parent is scrolled away");
        let (a, b) = (divider.center(), divider.center() + vec2(0.0, 80.0));
        h.input_mut().events.push(Event::PointerMoved(a));
        h.input_mut().events.push(Event::PointerButton { pos: a, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        h.step();
        for k in 1..=4 {
            h.input_mut().events.push(Event::PointerMoved(a + (b - a) * (k as f32 / 4.0)));
            h.step();
        }
        h.input_mut().events.push(Event::PointerButton { pos: b, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        h.run_steps(3);
        let after = h.get_by_label("Parents and pages divider").rect().center().y;
        assert!(after > before + 40.0, "the divider moved down: {before} → {after}");
        assert!(h.state().app.ui.pages_parents_height.is_some());
        assert!(window.contains_rect(h.get_by_label("Create new page").rect()), "the footer is still there");
    }

    #[test]
    fn panel_options_size_and_arrange_the_panel() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages": 6, "facingPages": true})).unwrap();
        app.run("layout.parents.new", json!({"prefix": "B"})).unwrap();
        app.ui.dock_tab = "pages".into();
        let mut h = crate::test_window::open(app, vec2(1400.0, 900.0));
        let page = |h: &Harness<'_, crate::test_window::Window>| h.get_by_label("Page 1").rect();
        let medium = page(&h).height();
        // The right-click menu on the panel's empty part leads to Panel Options.
        let empty = h.get_by_label("Create new page").rect().center() - vec2(0.0, 60.0);
        h.input_mut().events.push(egui::Event::PointerMoved(empty));
        for pressed in [true, false] {
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: empty,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        h.run_steps(3);
        h.get_by_label("Panel Options…").click();
        h.run_steps(3);
        let d = h.state().app.ui.dialog.clone().unwrap();
        assert_eq!(
            (d.id.as_str(), d.fields["pageSize"].clone(), d.fields["parentSize"].clone(), d.fields["layout"].clone()),
            ("pagesPanelOptions", json!("medium"), json!("small"), json!("parentsOnTop"))
        );
        // Jumbo pages, parents side by side, pages on top.
        let set = |h: &mut Harness<'_, crate::test_window::Window>, k: &str, v: serde_json::Value| {
            h.state_mut().app.ui.dialog.as_mut().unwrap().fields.insert(k.into(), v);
        };
        set(&mut h, "pageSize", json!("jumbo"));
        set(&mut h, "parentsVertical", json!(false));
        set(&mut h, "layout", json!("pagesOnTop"));
        crate::dialogs::confirm(&mut h.state_mut().app).unwrap();
        h.run_steps(4);
        assert!(page(&h).height() > 2.5 * medium, "Jumbo: {} vs {medium}", page(&h).height());
        let o = &h.state().app.ui.pages_panel;
        assert_eq!((o.page_size, o.parents_vertical, o.pages_on_top), (IconSize::Jumbo, false, true));
        // Pages on top: the parents come after the pages; side by side: [None] and A sit in one row.
        let masters = h.get_by_label("Master Pages").rect();
        assert!(masters.top() > page(&h).top(), "masters below the pages");
        assert!(!h.state().app.ui.pages_panel.parents_vertical, "master-card tiling selected");
        // Extra Small: no thumbnails, just pages.
        crate::panels::pages::open_panel_options(&mut h.state_mut().app);
        set(&mut h, "pageSize", json!("extraSmall"));
        crate::dialogs::confirm(&mut h.state_mut().app).unwrap();
        h.run_steps(4);
        assert!(page(&h).height() < 0.5 * medium);
    }
}

#[cfg(test)]
mod delete_warning_regressions {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    fn menu_harness(app: DesignApp, abs: usize, si: usize) -> Harness<'static, DesignApp> {
        let mut h = Harness::builder().with_size(vec2(320.0, 600.0)).build_ui_state(
            move |ui, app: &mut DesignApp| {
                let key = egui::Id::new("test_fonts");
                if ui.data(|d| d.get_temp::<bool>(key)).is_none() {
                    crate::theme::install_fonts(ui.ctx(), "");
                    ui.data_mut(|d| d.insert_temp(key, true));
                    return;
                }
                let doc = app.session.doc().unwrap().doc.clone();
                page_menu(app, ui, &doc, abs, si);
            },
            app,
        );
        h.run_steps(3);
        h
    }
    /// As in InDesign: deleting a page with objects asks first; an empty page goes at once, and
    /// "Don't show again" stops the question.
    #[test]
    fn deleting_a_page_with_objects_asks_first() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages": 5, "facingPages": true})).unwrap();
        let doc = app.session.doc().unwrap().doc.clone();
        let (si, pi) = doc.page_loc(2).unwrap();
        let x = doc.spreads[si].pages[pi].x;
        app.run("frame.create", json!({"spread": si, "rect": [x + 36.0, 36.0, x + 200.0, 200.0]})).unwrap();
        let count = |h: &Harness<'_, DesignApp>| h.state().session.doc().unwrap().doc.page_count();
        let mut h = menu_harness(app, 2, si);
        h.get_by_label("Delete Page").click();
        h.run_steps(3);
        assert_eq!(count(&h), 5, "nothing deleted yet");
        let d = h.state().ui.dialog.clone().unwrap();
        assert_eq!((d.id.as_str(), d.fields["pages"].clone()), ("deletePages", json!([2])));
        // Cancel keeps the page.
        crate::dialogs::cancel(h.state_mut());
        assert_eq!(count(&h), 5);
        // The whole spread: one question for both pages.
        h.get_by_label("Delete Spread").click();
        h.run_steps(3);
        assert_eq!(h.state().ui.dialog.as_ref().unwrap().fields["pages"], json!([1, 2]));
        crate::dialogs::cancel(h.state_mut());
        // OK deletes it.
        h.get_by_label("Delete Page").click();
        h.run_steps(3);
        crate::dialogs::confirm(h.state_mut()).unwrap();
        assert_eq!(count(&h), 4);
        assert!(!h.state().ui.skip_delete_pages_warning);
        // An empty page goes without a question.
        let app = h.state_mut();
        crate::dialogs::delete_pages(app, vec![0]).unwrap();
        assert!(app.ui.dialog.is_none());
        assert_eq!(app.session.doc().unwrap().doc.page_count(), 3);
        // "Don't show again".
        let doc = app.session.doc().unwrap().doc.clone();
        let (si, pi) = doc.page_loc(1).unwrap();
        let x = doc.spreads[si].pages[pi].x;
        app.run("frame.create", json!({"spread": si, "rect": [x + 36.0, 36.0, x + 200.0, 200.0]})).unwrap();
        app.run("frame.create", json!({"spread": si, "rect": [x + 36.0, 300.0, x + 200.0, 400.0]})).unwrap();
        crate::dialogs::delete_pages(app, vec![1]).unwrap();
        app.ui.dialog.as_mut().unwrap().fields.insert("dontShowAgain".into(), json!(true));
        crate::dialogs::confirm(app).unwrap();
        assert!(app.ui.skip_delete_pages_warning);
        assert_eq!(app.session.doc().unwrap().doc.page_count(), 2);
        let doc = app.session.doc().unwrap().doc.clone();
        let (si, pi) = doc.page_loc(0).unwrap();
        let x = doc.spreads[si].pages[pi].x;
        app.run("frame.create", json!({"spread": si, "rect": [x + 36.0, 36.0, x + 200.0, 200.0]})).unwrap();
        crate::dialogs::delete_pages(app, vec![0]).unwrap();
        assert!(app.ui.dialog.is_none(), "no more questions");
        assert_eq!(app.session.doc().unwrap().doc.page_count(), 1);
    }
}

#[cfg(test)]
mod short_panel_regressions {
    use super::*;
    #[test]
    fn jumbo_master_previews_in_a_short_panel_do_not_panic() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        app.ui.pages_panel.parent_size = IconSize::Jumbo;
        let mut h = egui_kittest::Harness::builder().with_size(vec2(260.0, 150.0)).build_ui_state(
            |ui, app: &mut DesignApp| {
                let id = egui::Id::new("short_panel_fonts");
                if ui.data(|d| d.get_temp::<bool>(id)).is_none() {
                    crate::theme::install_fonts(ui.ctx(), "");
                    ui.data_mut(|d| d.insert_temp(id, true));
                    return;
                }
                show(app, ui);
            },
            app,
        );
        h.run_steps(4);
        assert_eq!(h.state().session.doc().unwrap().doc.page_count(), 1);
    }
}
