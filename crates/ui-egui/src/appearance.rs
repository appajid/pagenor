//! Appearance shortcuts and favorite themes, preserving Pagenor's System preference.
use crate::{
    DesignApp,
    dialogs::Dialog,
    theme::{Brightness, Tokens},
};
use egui::{Sense, Stroke, pos2, vec2};
use serde_json::{Value, json};

pub fn header_button(app: &mut DesignApp, ui: &mut egui::Ui) {
    let current = app.ui.brightness;
    let tip = format!("Interface appearance: {}", current.label());
    let response = crate::icons::button(ui, "monitor", 22.0, false, &tip);
    if response.clicked() {
        let _ = app.run("window.nextAppearanceMode", json!({}));
    }
    response.context_menu(|ui| {
        if ui.button("Appearance Settings…").clicked() {
            let _ = app.run("app.preferences", json!({}));
            if let Some(d) = &mut app.ui.dialog {
                d.fields.insert("section".into(), json!("interface"));
            }
            ui.close();
        }
        for light in [true, false] {
            let favorites: Vec<_> = app.ui.theme_favorites.themes(light).collect();
            if favorites.is_empty() {
                continue;
            }
            ui.separator();
            ui.label(if light { "Light themes" } else { "Dark themes" });
            for b in favorites {
                if ui.selectable_label(current == b, b.label()).clicked() {
                    let _ = app.run("window.brightness", json!({"brightness": b.id()}));
                    ui.close();
                }
            }
        }
    });
}

pub fn preferences_rows(lang: &str, ui: &mut egui::Ui, d: &mut Dialog) {
    crate::rtl::label(ui, egui::RichText::new("Appearance").font(crate::theme::semibold(12.0)));
    let current = Brightness::parse(&d.s("brightness")).unwrap_or_default();
    let mut favorites = crate::theme_favorites::Favorites::from_value(d.fields.get("themeFavorites").unwrap_or(&Value::Null));
    for b in Brightness::ALL {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(40.0, 22.0), Sense::hover());
            let tokens = Tokens::for_brightness(b);
            ui.painter().rect_filled(r, 2.0, tokens.panel);
            ui.painter().rect_filled(egui::Rect::from_min_max(r.min, pos2(r.max.x, r.min.y + 6.0)), 2.0, tokens.app_bar);
            ui.painter().line_segment([r.left_center() + vec2(5.0, 2.0), r.right_center() - vec2(5.0, -2.0)], Stroke::new(1.0, tokens.text));
            if ui.radio(b == current, crate::rtl::widget(ui, crate::i18n::tr(lang, b.label()))).clicked() {
                d.fields.insert("brightness".into(), json!(b.id()));
            }
            if b != Brightness::System {
                let hovered = ui.rect_contains_pointer(ui.max_rect());
                if crate::theme_favorites::star(ui, favorites.contains(b), hovered, "Add to favorites", "Remove from favorites", b.label()).clicked()
                {
                    favorites.toggle(b);
                    d.fields.insert("themeFavorites".into(), json!(favorites));
                }
            }
        });
    }
    ui.add_space(8.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appearance_cycle_preserves_saved_system_preference_and_document_history() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), Default::default());
        app.run("file.new", json!({})).unwrap();
        app.run("window.brightness", json!({"brightness": "system"})).unwrap();
        let doc = app.session.active().unwrap().doc.clone();
        for b in ["light", "mediumDark", "system"] {
            app.run("window.nextAppearanceMode", json!({})).unwrap();
            assert_eq!(app.ui.brightness.id(), b);
            assert!(std::sync::Arc::ptr_eq(&doc, &app.session.active().unwrap().doc));
        }
        app.ui.theme_favorites.toggle(Brightness::HighContrast);
        let saved = serde_json::to_vec(&app.ui).unwrap();
        let restored: crate::UiState = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored.brightness, Brightness::System);
        assert!(restored.theme_favorites.contains(Brightness::HighContrast));
    }
}
