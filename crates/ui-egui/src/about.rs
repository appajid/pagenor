//! Help › About Pagenor: tabs About · Contributors · Models. About is the splash with the
//! supplied Pagenor artwork and the running version; Contributors and Models are the compiled-in credits (`crate::credits`).

use egui::{Color32, Rect, Sense, pos2, vec2};

use crate::DesignApp;
use crate::theme::Tokens;

/// The About window's tabs, in `UiState::about_tab` order (`help.about {tab}` names them in lowercase).
pub const ABOUT_TABS: [&str; 3] = ["About", "Contributors", "Models"];

/// Use the primary or inverse Pagenor wordmark for the current theme.
pub fn paint_logo(ui: &egui::Ui, rect: Rect) {
    let (key, bytes): (&str, &[u8]) = if ui.visuals().dark_mode {
        ("pagenor_logo_inverse", include_bytes!("../../../assets/brand/runtime-logo-inverse.svg"))
    } else {
        ("pagenor_logo_primary", include_bytes!("../../../assets/brand/runtime-logo-primary.svg"))
    };
    paint_brand(ui, rect, key, bytes);
}

fn paint_brand(ui: &egui::Ui, rect: Rect, name: &str, bytes: &[u8]) {
    let ctx = ui.ctx();
    let px = ((rect.width().max(rect.height()) * ctx.pixels_per_point()).round() as u32).clamp(8, 2048);
    let key = egui::Id::new((name, px));
    let cached: Option<egui::TextureHandle> = ctx.data(|d| d.get_temp(key));
    let texture = cached.or_else(|| {
        let (pixels, width, height) = pagenor_images::render_svg(bytes, px)?;
        let image = egui::ColorImage::from_rgba_premultiplied([width as usize, height as usize], &pixels);
        let texture = crate::widgets::load_texture(ctx, name, image, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(key, texture.clone()));
        Some(texture)
    });
    if let Some(texture) = texture {
        ui.painter().image(texture.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
}

/// The supplied welcome artwork; its sample loading bar is removed in the runtime derivative.
pub fn paint_splash(ui: &egui::Ui, rect: Rect) {
    paint_brand(ui, rect, "pagenor_splash", include_bytes!("../../../assets/brand/runtime-splash-screen.svg"));
}

fn paint_about(ui: &egui::Ui, rect: Rect) {
    paint_brand(ui, rect, "pagenor_about", include_bytes!("../../../assets/brand/runtime-about-page.svg"));
    // The editable reference's placeholder version is replaced by the running build's version.
    let scale = rect.width() / 760.0;
    ui.painter().text(
        pos2(rect.center().x, rect.top() + (289.0 - 56.0) * scale),
        egui::Align2::CENTER_BOTTOM,
        format!("Version {}", env!("CARGO_PKG_VERSION")),
        egui::FontId::proportional(19.0 * scale),
        Color32::from_rgb(0x50, 0x64, 0x8c),
    );
}

pub fn show(app: &mut DesignApp, ctx: &egui::Context) {
    if !app.ui.about {
        return;
    }
    let t = Tokens::get(ctx);
    let resp = egui::Modal::new(egui::Id::new("about")).show(ctx, |ui| {
        ui.set_width(if app.ui.about_tab == 0 { 460.0 } else { 640.0 });
        ui.horizontal(|ui| {
            for (i, l) in ABOUT_TABS.iter().enumerate() {
                let i = i as u8;
                if ui.selectable_label(app.ui.about_tab == i, crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, l))).clicked() {
                    app.ui.about_tab = i;
                }
            }
        });
        ui.separator();
        if app.ui.about_tab != 0 {
            let h = 400.0_f32.min(ctx.content_rect().height() * 0.7).max(160.0);
            ui.allocate_ui_with_layout(vec2(ui.available_width(), h), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_min_height(h);
                if app.ui.about_tab == 1 {
                    crate::credits::contributors_ui(ui);
                } else {
                    crate::credits::models_ui(ui);
                }
            });
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "  Close  "))).clicked() {
                    app.ui.about = false;
                }
            });
            return;
        }
        ui.vertical_centered(|ui| {
            let height = (ctx.content_rect().height() - 170.0).clamp(160.0, 640.0);
            let width = (height * 760.0 / 924.0).min(ui.available_width());
            let (rect, _) = ui.allocate_exact_size(vec2(width, width * 924.0 / 760.0), Sense::hover());
            paint_about(ui, rect);
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Software: MIT OR Apache-2.0. Original copyright and attribution notices retained.").size(10.0).color(t.text_dim),
            );
            if ui.button(crate::rtl::widget(ui, crate::i18n::tr(&app.ui.language, "  Close  "))).clicked() {
                app.ui.about = false;
            }
        });
    });
    if resp.should_close() {
        app.ui.about = false;
    }
}
