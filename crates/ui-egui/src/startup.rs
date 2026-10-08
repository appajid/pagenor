//! One brief launch splash, independent of whether startup opens a document.

use egui::{Rect, vec2};

#[derive(Default)]
pub(crate) struct Splash {
    started_at: Option<f64>,
}

impl Splash {
    /// Returns false once the splash is finished. Timing starts at its first UI
    /// frame, so file/recovery loading before window creation cannot skip it.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> bool {
        const SECONDS: f64 = 1.2;
        let now = ui.ctx().input(|i| i.time);
        if !now.is_finite() {
            return false;
        }
        let start = self.started_at.get_or_insert(now);
        if now < *start {
            *start = now;
        }
        let remaining = (SECONDS - (now - *start)).clamp(0.0, SECONDS);
        if remaining == 0.0 {
            return false;
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(remaining));
        let rect = ui.available_rect_before_wrap();
        let tokens = crate::theme::Tokens::get(ui.ctx());
        ui.painter().rect_filled(rect, 0.0, tokens.panel_darker);
        let height = (rect.height() - 48.0).clamp(1.0, 640.0).min((rect.width() - 48.0).max(1.0) * 980.0 / 760.0);
        let art = Rect::from_center_size(rect.center(), vec2(height * 760.0 / 980.0, height));
        crate::about::paint_splash(ui, art);
        true
    }
}
