//! Design tokens (InDesign's four interface brightness levels), fonts and egui style.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Brightness {
    /// Follow desktop appearance. Resolves to Light or Medium Dark; unknown uses Medium Dark.
    System,
    Dark,
    #[default]
    MediumDark,
    MediumLight,
    Light,
    /// Accessibility: black chrome, white text, yellow selection and focus.
    HighContrast,
}

impl Brightness {
    pub const ALL: [Brightness; 6] =
        [Brightness::System, Brightness::Dark, Brightness::MediumDark, Brightness::MediumLight, Brightness::Light, Brightness::HighContrast];
    pub fn label(self) -> &'static str {
        match self {
            Brightness::System => "System",
            Brightness::Dark => "Dark",
            Brightness::MediumDark => "Medium Dark",
            Brightness::MediumLight => "Medium Light",
            Brightness::Light => "Light",
            Brightness::HighContrast => "High Contrast",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Brightness::System => "system",
            Brightness::Dark => "dark",
            Brightness::MediumDark => "mediumDark",
            Brightness::MediumLight => "mediumLight",
            Brightness::Light => "light",
            Brightness::HighContrast => "highContrast",
        }
    }
    /// Resolve without replacing the persisted preference.
    pub fn resolved(self, system: Option<egui::Theme>) -> Self {
        match (self, system) {
            (Self::System, Some(egui::Theme::Light)) => Self::Light,
            (Self::System, _) => Self::MediumDark,
            _ => self,
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.id().eq_ignore_ascii_case(s) || b.label().eq_ignore_ascii_case(s))
    }
}

/// Every colour the UI uses. Widgets never hard-code colours.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub dark: bool,
    pub app_bar: Color32,
    pub panel: Color32,
    pub panel_darker: Color32,
    pub input: Color32,
    pub input_border: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub text_strong: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub tool_active: Color32,
    pub accent: Color32,
    pub accent_strong: Color32,
    pub row_selected: Color32,
    pub pasteboard: Color32,
    pub ruler: Color32,
    pub ruler_tick: Color32,
    pub tab_inactive: Color32,
    pub button: Color32,
    pub radius: u8,
    /// 1 pt chrome edges.
    pub border: Color32,
    /// Outline of fields and buttons.
    pub field_border: Color32,
    /// Active tool / toggle well.
    pub well: Color32,
    pub well_rim: Color32,
    pub ruler_text: Color32,
    pub section_divider: Color32,
    /// Panel tab strips, doc-tab bar, dock headers.
    pub tab_strip: Color32,
    /// Smart guides and spacing marks drawn over the page.
    pub smart_guide: Color32,
    /// Measurement labels on the canvas (smart dimensions, gaps).
    pub measure_bg: Color32,
    pub measure_text: Color32,
    /// Live-corner widget and corner diamonds.
    pub live_corner: Color32,
}

fn hex(s: u32) -> Color32 {
    Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8)
}

impl Tokens {
    pub fn for_brightness(b: Brightness) -> Self {
        let dark = Tokens {
            dark: true,
            app_bar: hex(0x262626),
            panel: hex(0x323232),
            panel_darker: hex(0x282828),
            input: hex(0x1f1f1f),
            input_border: hex(0x4a4a4a),
            divider: hex(0x1e1e1e),
            text: hex(0xd8d8d8),
            text_strong: hex(0xf2f2f2),
            text_dim: hex(0xa3a3a3),
            text_disabled: hex(0x6a6a6a),
            icon: hex(0xc8c8c8),
            hover: hex(0x424242),
            tool_active: hex(0x1b1b1b),
            accent: hex(0x378ef0),
            accent_strong: hex(0x1473e6),
            row_selected: hex(0x2d4f7c),
            pasteboard: hex(0x232323),
            ruler: hex(0x2e2e2e),
            ruler_tick: hex(0x9a9a9a),
            tab_inactive: hex(0x262626),
            button: hex(0x404040),
            radius: 1,
            border: hex(0x1e1e1e),
            field_border: hex(0x5a5a5a),
            well: hex(0x1b1b1b),
            well_rim: hex(0x3a3a3a),
            ruler_text: hex(0xd8d8d8),
            section_divider: hex(0x282828),
            tab_strip: hex(0x282828),
            smart_guide: hex(0x00c853),
            measure_bg: Color32::from_rgba_unmultiplied(70, 70, 70, 230),
            measure_text: Color32::WHITE,
            live_corner: hex(0xffe500),
        };
        match b {
            Brightness::Dark => dark,
            // Measured from InDesign 2026 (plan/indesign/11-observed-ui.md §1).
            Brightness::System | Brightness::MediumDark => Tokens {
                app_bar: hex(0x535353),
                panel: hex(0x535353),
                panel_darker: hex(0x424242),
                input: hex(0x454545),
                input_border: hex(0x747474),
                divider: hex(0x4b4b4b),
                text: hex(0xffffff),
                text_strong: hex(0xffffff),
                text_dim: hex(0xb0b0b0),
                text_disabled: hex(0x8a8a8a),
                icon: hex(0xc2c2c2),
                hover: hex(0x606060),
                tool_active: hex(0x303030),
                row_selected: hex(0x4a6a92),
                pasteboard: hex(0x5e5e5e),
                ruler: hex(0x1f1f1f),
                ruler_tick: hex(0x858585),
                tab_inactive: hex(0x424242),
                button: hex(0x535353),
                border: hex(0x383838),
                field_border: hex(0x747474),
                well: hex(0x303030),
                well_rim: hex(0x565656),
                ruler_text: hex(0xffffff),
                section_divider: hex(0x4b4b4b),
                tab_strip: hex(0x424242),
                ..dark
            },
            Brightness::MediumLight => Tokens {
                dark: false,
                app_bar: hex(0xa8a8a8),
                panel: hex(0xb8b8b8),
                panel_darker: hex(0xaaaaaa),
                input: hex(0xd6d6d6),
                input_border: hex(0x8c8c8c),
                divider: hex(0x9c9c9c),
                text: hex(0x1e1e1e),
                text_strong: hex(0x000000),
                text_dim: hex(0x3c3c3c),
                text_disabled: hex(0x7a7a7a),
                icon: hex(0x2a2a2a),
                hover: hex(0xc8c8c8),
                tool_active: hex(0x9a9a9a),
                row_selected: hex(0x8eb2e0),
                pasteboard: hex(0xa0a0a0),
                ruler: hex(0xc0c0c0),
                ruler_tick: hex(0x3a3a3a),
                tab_inactive: hex(0xa6a6a6),
                button: hex(0xb8b8b8),
                border: hex(0x8c8c8c),
                field_border: hex(0x7a7a7a),
                well: hex(0x9a9a9a),
                well_rim: hex(0x8a8a8a),
                ruler_text: hex(0x1e1e1e),
                section_divider: hex(0x9c9c9c),
                tab_strip: hex(0xa6a6a6),
                ..dark
            },
            Brightness::Light => Tokens {
                dark: false,
                app_bar: hex(0xe4e4e4),
                panel: hex(0xf0f0f0),
                panel_darker: hex(0xe2e2e2),
                input: hex(0xffffff),
                input_border: hex(0xb4b4b4),
                divider: hex(0xd0d0d0),
                text: hex(0x1e1e1e),
                text_strong: hex(0x000000),
                text_dim: hex(0x555555),
                text_disabled: hex(0x9a9a9a),
                icon: hex(0x2c2c2c),
                hover: hex(0xdddddd),
                tool_active: hex(0xcfcfcf),
                row_selected: hex(0xc4dbf7),
                pasteboard: hex(0xd4d4d4),
                ruler: hex(0xf2f2f2),
                ruler_tick: hex(0x4a4a4a),
                tab_inactive: hex(0xdcdcdc),
                button: hex(0xf0f0f0),
                border: hex(0xc4c4c4),
                field_border: hex(0xa8a8a8),
                well: hex(0xcfcfcf),
                well_rim: hex(0xbdbdbd),
                ruler_text: hex(0x1e1e1e),
                section_divider: hex(0xd0d0d0),
                tab_strip: hex(0xe2e2e2),
                ..dark
            },
            Brightness::HighContrast => Tokens {
                app_bar: hex(0x000000),
                panel: hex(0x000000),
                panel_darker: hex(0x000000),
                input: hex(0x000000),
                input_border: hex(0xffffff),
                divider: hex(0xffffff),
                text: hex(0xffffff),
                text_strong: hex(0xffffff),
                text_dim: hex(0xe6e6e6),
                text_disabled: hex(0xa0a0a0),
                icon: hex(0xffffff),
                hover: hex(0x3d3d00),
                tool_active: hex(0x5c5c00),
                accent: hex(0xffeb3b),
                accent_strong: hex(0xffd600),
                row_selected: hex(0x5c5c00),
                pasteboard: hex(0x1a1a1a),
                ruler: hex(0x000000),
                ruler_tick: hex(0xffffff),
                tab_inactive: hex(0x000000),
                button: hex(0x000000),
                border: hex(0xffffff),
                field_border: hex(0xffffff),
                well: hex(0x5c5c00),
                well_rim: hex(0xffeb3b),
                ruler_text: hex(0xffffff),
                section_divider: hex(0xffffff),
                tab_strip: hex(0x000000),
                ..dark
            },
        }
    }
}

/// Install built-in UI faces immediately, then installed fallbacks once their worker is ready.
/// The returned generation changes when activated fonts are refreshed.
pub fn install_fonts(ctx: &egui::Context, lang: &str) -> Option<u64> {
    let mut fonts = font_definitions(pagenor_fonts::CRAFT_FONTS, lang);
    let snapshot = system_ui_fonts(lang);
    if let Some((_, system)) = &snapshot {
        add_system_fallbacks(&mut fonts, system);
    }
    ctx.set_fonts(fonts);
    snapshot.map(|(generation, _)| generation)
}

#[derive(Clone)]
struct SystemFont {
    name: String,
    data: Arc<FontData>,
}

impl SystemFont {
    #[cfg(any(test, not(target_arch = "wasm32")))]
    fn from_face(face: &pagenor_fonts::FontFace) -> Self {
        let data = FontData { index: face.index(), tweak: baseline_tweak(face.data(), face.index()), ..FontData::from_owned(face.data().to_vec()) };
        Self { name: format!("system-ui-{}", face.id()), data: Arc::new(data) }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct UiFontCache {
    ready: Option<(u64, String, Vec<SystemFont>)>,
    running: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn ui_font_cache() -> &'static std::sync::Mutex<UiFontCache> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<UiFontCache>> = std::sync::OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Start only metadata-ready font work, off the UI thread. Retain one generation of selected
/// faces, rather than every installed file. Source bytes are shared across language reinstalls.
fn system_ui_fonts(lang: &str) -> Option<(u64, Vec<SystemFont>)> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = lang;
        Some((0, Vec::new()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let db = pagenor_fonts::FontDb::global();
        if !db.system_fonts_cataloged() {
            db.scan_in_background();
            return None;
        }
        let generation = db.scan_generation();
        let mut cache = ui_font_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some((ready_generation, ready_lang, faces)) = &cache.ready
            && *ready_generation == generation
            && ready_lang == lang
        {
            return Some((generation, faces.clone()));
        }
        if !cache.running {
            cache.running = true;
            let lang = lang.to_owned();
            if let Err(e) = std::thread::Builder::new().name("ui-fonts".into()).spawn(move || {
                let faces = installed_ui_fonts(db, pagenor_fonts::CRAFT_FONTS, &lang);
                let mut cache = ui_font_cache().lock().unwrap_or_else(|e| e.into_inner());
                // A later catalogue update will cause another pass; never label stale faces
                // with a newer generation.
                cache.ready = Some((generation, lang, faces));
                cache.running = false;
            }) {
                cache.running = false;
                log::warn!("Could not start interface font worker: {e}");
            }
        }
        None
    }
}

/// Current ready UI generation, also scheduling a refresh when the font catalogue changed.
pub fn system_font_generation(lang: &str) -> Option<u64> {
    system_ui_fonts(lang).map(|(generation, _)| generation)
}

// Installed families for scripts absent from the optional bundled fonts. Check a letter and
// dependent vowel together for Indic scripts; an ASCII/legacy font of the same name is skipped.
#[cfg(any(test, not(target_arch = "wasm32")))]
const SYSTEM_UI_FONTS: &[(&str, &str, &[&str])] = &[
    ("Taml", "தி", &["Tamil Sangam MN", "Tamil MN", "Nirmala UI", "Latha", "Noto Sans Tamil", "Noto Sans Tamil UI", "Lohit Tamil"]),
    ("Deva", "कि", &["Kohinoor Devanagari", "Devanagari Sangam MN", "Nirmala UI", "Mangal", "Noto Sans Devanagari", "Lohit Devanagari"]),
    ("Beng", "কি", &["Kohinoor Bangla", "Bangla Sangam MN", "Nirmala UI", "Vrinda", "Noto Sans Bengali", "Lohit Bengali"]),
    ("Gujr", "કિ", &["Kohinoor Gujarati", "Gujarati Sangam MN", "Nirmala UI", "Shruti", "Noto Sans Gujarati", "Lohit Gujarati"]),
    ("Guru", "ਕਿ", &["Gurmukhi Sangam MN", "Gurmukhi MN", "Nirmala UI", "Raavi", "Noto Sans Gurmukhi", "Lohit Gurmukhi"]),
    ("Knda", "ಕಿ", &["Kannada Sangam MN", "Nirmala UI", "Tunga", "Noto Sans Kannada", "Lohit Kannada"]),
    ("Mlym", "കി", &["Malayalam Sangam MN", "Nirmala UI", "Kartika", "Noto Sans Malayalam", "Lohit Malayalam"]),
    ("Telu", "కి", &["Kohinoor Telugu", "Telugu Sangam MN", "Nirmala UI", "Gautami", "Noto Sans Telugu", "Lohit Telugu"]),
    ("Orya", "କି", &["Oriya Sangam MN", "Nirmala UI", "Kalinga", "Noto Sans Oriya", "Lohit Odia"]),
    ("Sinh", "කි", &["Sinhala Sangam MN", "Nirmala UI", "Iskoola Pota", "Noto Sans Sinhala", "LKLUG"]),
    ("Thai", "ก", &["Thonburi", "Leelawadee UI", "Tahoma", "Noto Sans Thai", "Loma"]),
    ("Khmr", "ក", &["Khmer Sangam MN", "Leelawadee UI", "Khmer UI", "Noto Sans Khmer", "Khmer OS"]),
    ("Laoo", "ກ", &["Lao Sangam MN", "Leelawadee UI", "Lao UI", "Noto Sans Lao", "Phetsarath OT"]),
    ("Mymr", "က", &["Myanmar Sangam MN", "Myanmar Text", "Noto Sans Myanmar", "Padauk"]),
    ("Hebr", "א", &["Arial Hebrew", "Segoe UI", "Noto Sans Hebrew", "DejaVu Sans"]),
    ("Armn", "Ա", &["Mshtakan", "Segoe UI", "Sylfaen", "Noto Sans Armenian", "DejaVu Sans"]),
    ("Geor", "ა", &["Segoe UI", "Sylfaen", "Noto Sans Georgian", "DejaVu Sans"]),
    ("Ethi", "አ", &["Kefa", "Ebrima", "Nyala", "Noto Sans Ethiopic", "Abyssinica SIL"]),
];

#[cfg(any(test, not(target_arch = "wasm32")))]
fn installed_ui_fonts(db: &pagenor_fonts::FontDb, craft: &[pagenor_fonts::CraftFont], lang: &str) -> Vec<SystemFont> {
    let mut faces = Vec::new();
    let mut seen = std::collections::HashSet::new();
    // Preserve the existing local-build CJK fallback policy.
    if craft.is_empty() {
        for c in ['中', 'あ', '한'] {
            if let Some(face) = db.fallback_for(c, u32::MAX, Some(lang))
                && seen.insert(face.id())
            {
                faces.push(SystemFont::from_face(&face));
            }
        }
    }
    for (script, sample, families) in SYSTEM_UI_FONTS {
        if craft.iter().any(|f| f.scripts.contains(script)) {
            continue;
        }
        let found = families.iter().find_map(|family| {
            if !db.has_family(family) {
                return None;
            }
            let face = db.face(family, "Regular");
            (face.family.eq_ignore_ascii_case(family) && !face.data().is_empty() && face.covers_cluster(sample)).then_some(face)
        });
        if let Some(face) = found
            && seen.insert(face.id())
        {
            faces.push(SystemFont::from_face(&face));
        }
    }
    faces
}

fn add_system_fallbacks(fonts: &mut FontDefinitions, system: &[SystemFont]) {
    for face in system {
        fonts.font_data.insert(face.name.clone(), face.data.clone());
        for stack in fonts.families.values_mut() {
            if !stack.contains(&face.name) {
                stack.push(face.name.clone());
            }
        }
    }
}

/// Shift fallback line metrics onto the primary UI face's baseline (egui otherwise centers
/// the fallback line box). Unreadable or extreme metrics get a bounded offset.
#[cfg(any(test, not(target_arch = "wasm32")))]
fn baseline_tweak(data: &[u8], index: u32) -> egui::FontTweak {
    let metrics =
        |bytes: &[u8], i| pagenor_fonts::line_metrics(bytes, i).map(|(em, ascent, descent, gap)| (ascent / em, (ascent - descent + gap) / em));
    let shift = match (metrics(include_bytes!("../../../assets/fonts/SourceSans3-Regular.ttf"), 0), metrics(data, index)) {
        (Some((ap, hp)), Some((af, hf))) => ap - (af + 0.5 * (hp - hf)),
        _ => 0.0,
    };
    egui::FontTweak { y_offset_factor: if shift.is_finite() { shift.clamp(-0.5, 0.5) } else { 0.0 }, ..Default::default() }
}

/// The UI fonts: the app's own, then the craft-fonts faces from `craft` (empty without
/// `CRAFT_FONTS_DIR`) as fallbacks at the end of every family: Japanese (BIZ UDPGothic first) and
/// Simplified Chinese, in the interface language's order, then Arabic. The `arabic` families
/// put the Arabic face first for right-to-left runs. egui has no system-font discovery, so
/// without craft-fonts CJK and Arabic UI text has no glyphs.
pub(crate) fn font_definitions(craft: &'static [pagenor_fonts::CraftFont], lang: &str) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let add = |fonts: &mut FontDefinitions, name: &str, data: &'static [u8]| {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(data)));
    };
    add(&mut fonts, "ui", include_bytes!("../../../assets/fonts/SourceSans3-Regular.ttf"));
    add(&mut fonts, "ui-semibold", include_bytes!("../../../assets/fonts/SourceSans3-Semibold.ttf"));
    add(&mut fonts, "mono", include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf"));
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ui".into());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "mono".into());
    fonts.families.insert(FontFamily::Name("semibold".into()), vec!["ui-semibold".into(), "ui".into()]);
    let pick = |script: &str| -> Vec<&pagenor_fonts::CraftFont> { craft.iter().filter(|f| f.scripts.contains(&script)).collect() };
    let mut japanese = pick("Jpan");
    // BIZ UDPGothic (the UI face) first, Regular before Bold; the semibold family prefers Bold.
    japanese.sort_by_key(|f| (f.family != "BIZ UDPGothic", f.style != "Regular"));
    let chinese = pick("Hans");
    let arabic = pick("Arab");
    let name = |f: &pagenor_fonts::CraftFont| format!("craft-{}-{}", f.family, f.style);
    for f in japanese.iter().chain(&chinese).chain(&arabic) {
        add(&mut fonts, &name(f), f.bytes);
    }
    // Han characters take the forms of the interface language: Chinese faces first for zh.
    let zh = lang.starts_with("zh");
    for (family, stack) in fonts.families.iter_mut() {
        let bold = *family == FontFamily::Name("semibold".into());
        let mut ja = japanese.clone();
        if bold {
            ja.sort_by_key(|f| (f.family != "BIZ UDPGothic", f.style != "Bold"));
        }
        let (first, second) = if zh { (&chinese, &ja) } else { (&ja, &chinese) };
        stack.extend(first.iter().chain(second).chain(&arabic).map(|f| name(f)));
    }
    // Arabic runs (rtl.rs): the Arabic face first, so letters and spaces shape as one run.
    for family in ["arabic", "arabic-semibold"] {
        let ui = if family == "arabic" { "ui" } else { "ui-semibold" };
        let stack = arabic.iter().map(|f| name(f)).chain([ui.to_string(), "ui".to_string()]).chain(japanese.iter().chain(&chinese).map(|f| name(f)));
        let mut seen = std::collections::HashSet::new();
        fonts.families.insert(FontFamily::Name(family.into()), stack.filter(|n| seen.insert(n.clone())).collect());
    }
    fonts
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn apply(ctx: &egui::Context, t: &Tokens) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, *t));
    let text_styles: std::collections::BTreeMap<TextStyle, FontId> = [
        (TextStyle::Small, FontId::proportional(10.0)),
        (TextStyle::Body, FontId::proportional(11.0)),
        (TextStyle::Button, FontId::proportional(11.0)),
        (TextStyle::Heading, FontId::new(15.0, FontFamily::Name("semibold".into()))),
        (TextStyle::Monospace, FontId::monospace(11.5)),
    ]
    .into();
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    let r = CornerRadius::same(t.radius);
    v.panel_fill = t.panel;
    v.window_fill = t.panel;
    v.extreme_bg_color = t.input;
    v.faint_bg_color = t.panel_darker;
    v.window_stroke = Stroke::new(1.0, t.divider);
    v.window_corner_radius = CornerRadius::same(4);
    v.menu_corner_radius = CornerRadius::same(4);
    v.selection.bg_fill = t.accent_strong;
    v.selection.stroke = Stroke::new(1.0, t.text_strong);
    v.hyperlink_color = t.accent;
    v.override_text_color = Some(t.text);
    craft_ui::theme::WidgetPalette { noninteractive: t.panel, inactive: t.button, hovered: t.hover, active: t.tool_active, open: t.hover }.apply_to(
        &mut v.widgets,
        r,
        Stroke::new(1.0, t.text),
    );
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.divider);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.field_border);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.field_border);
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.field_border);
    // InDesign buttons are outline-only.
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_fill = t.input;
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha(90) };
    // Keep native appearance inherited so macOS ThemeChanged continues to reach egui.
    // Both branches use the resolved palette, including fixed manual and High Contrast choices.
    ctx.set_theme(egui::ThemePreference::System);
    ctx.set_visuals_of(egui::Theme::Light, v.clone());
    ctx.set_visuals_of(egui::Theme::Dark, v);
    ctx.all_styles_mut(|s| {
        s.text_styles = text_styles.clone();
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 2.0);
        s.spacing.interact_size = egui::vec2(20.0, 21.0);
        s.spacing.menu_margin = egui::Margin::same(4);
        s.animation_time = 0.06;
    });
}

impl Tokens {
    /// Tokens stored by [`apply`].
    pub fn get(ctx: &egui::Context) -> Tokens {
        ctx.data(|d| d.get_temp::<Tokens>(egui::Id::NULL)).unwrap_or_else(|| Tokens::for_brightness(Brightness::Dark))
    }
}

#[cfg(test)]
mod japanese_font_tests {
    fn families() -> [egui::FontFamily; 3] {
        [egui::FontFamily::Proportional, egui::FontFamily::Monospace, egui::FontFamily::Name("semibold".into())]
    }

    fn ctx_with(craft: &'static [pagenor_fonts::CraftFont]) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_fonts(super::font_definitions(craft, "ja"));
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        ctx
    }

    #[test]
    fn japanese_ui_glyphs_are_available_in_every_family() {
        if pagenor_fonts::CRAFT_FONTS.is_empty() {
            eprintln!("skipped: built without craft-fonts (set CRAFT_FONTS_DIR to a checkout)");
            return;
        }
        let ctx = ctx_with(pagenor_fonts::CRAFT_FONTS);
        ctx.fonts_mut(|fonts| {
            for family in families() {
                let font = egui::FontId::new(13.0, family);
                // Real glyphs (no tofu) for every character.
                for ch in "日本語の文字縦書き横書き組み方向ルビ圏点".chars() {
                    assert!(fonts.has_glyph(&font, ch), "missing {ch} in {font:?}");
                }
            }
        });
        // The UI face is BIZ UDPGothic, ahead of the Mincho faces.
        let defs = super::font_definitions(pagenor_fonts::CRAFT_FONTS, "ja");
        let stack = &defs.families[&egui::FontFamily::Proportional];
        assert_eq!(stack.iter().find(|n| n.starts_with("craft-")).map(String::as_str), Some("craft-BIZ UDPGothic-Regular"));
    }

    #[test]
    fn ui_works_without_craft_fonts() {
        let ctx = ctx_with(&[]);
        ctx.fonts_mut(|fonts| {
            // (egui's has_glyph reports false for characters of the face that also supplies the
            // replacement glyph, as Source Sans Semibold does in the semibold stack.)
            let body = egui::FontId::new(13.0, egui::FontFamily::Proportional);
            assert!(fonts.has_glyphs(&body, "Pagenor"));
            for family in families() {
                let font = egui::FontId::new(13.0, family);
                let galley = fonts.layout_no_wrap("日本語 Pagenor".into(), font, egui::Color32::WHITE);
                assert!(galley.size().x > 0.0);
            }
        });
        // And the real installer works with whatever this build has.
        super::install_fonts(&egui::Context::default(), "ja");
    }
}
#[cfg(test)]
mod russian_font_tests {
    #[test]
    fn every_catalog_cyrillic_glyph_exists_in_the_bundled_ui_fonts() {
        // Same check as for Ukrainian: the installed face data only, without craft-fonts or system fonts.
        let definitions = super::font_definitions(&[], "ru");
        let db = pagenor_fonts::FontDb::with_font_dirs(Vec::new());
        db.set_system_fallback(false);
        let characters: std::collections::HashSet<_> = crate::i18n::russian_catalog()
            .iter()
            .flat_map(|text| text.chars())
            .chain("АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя".chars())
            .filter(|c| matches!(*c, '\u{0400}'..='\u{052f}'))
            .collect();
        for (name, family, style) in
            [("ui", "Source Sans 3", "Regular"), ("ui-semibold", "Source Sans 3", "Semibold"), ("mono", "JetBrains Mono", "Regular")]
        {
            let data = &definitions.font_data[name].font;
            let face = db.face(family, style);
            assert_eq!(face.data(), data.as_ref(), "configured UI face {name}");
            for character in &characters {
                assert_ne!(face.glyph_for(*character), 0, "missing {character} in {name}");
            }
        }
    }
}

#[cfg(test)]
mod installed_ui_font_tests {
    use super::*;
    // Avoid the already-existing CJK resolver in these non-CJK coverage tests.
    const CRAFT: &[pagenor_fonts::CraftFont] =
        &[pagenor_fonts::CraftFont { family: "Test bundled CJK", style: "Regular", scripts: &["Jpan"], bytes: &[] }];

    #[test]
    fn indic_fallbacks_require_marks_and_deduplicate_shared_faces() {
        let db = pagenor_fonts::FontDb::with_font_dirs(Vec::new());
        db.add_font(pagenor_fonts::testing::font_with("Tamil Sangam MN", &['த']).unwrap());
        db.add_font(pagenor_fonts::testing::font_with("Nirmala UI", &['த', 'ி', 'క', 'ి', 'क', 'ि']).unwrap());
        let faces = installed_ui_fonts(&db, CRAFT, "en");
        assert_eq!(faces.len(), 1, "one shared face serves Tamil, Telugu and Devanagari");
        let mut defs = font_definitions(&[], "en");
        add_system_fallbacks(&mut defs, &faces);
        add_system_fallbacks(&mut defs, &faces);
        for stack in defs.families.values() {
            assert_eq!(stack.iter().filter(|name| *name == &faces[0].name).count(), 1);
        }
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            for family in [FontFamily::Proportional, FontFamily::Monospace, FontFamily::Name("semibold".into())] {
                for ch in ['த', 'ி', 'క', 'ి', 'क', 'ि'] {
                    assert!(fonts.has_glyph(&FontId::new(13.0, family.clone()), ch), "missing {ch}");
                }
            }
        });
    }

    #[test]
    fn newly_activated_interface_font_changes_generation_and_coverage() {
        let db = pagenor_fonts::FontDb::with_font_dirs(Vec::new());
        assert!(installed_ui_fonts(&db, CRAFT, "en").is_empty());
        let generation = db.scan_generation();
        db.add_font(pagenor_fonts::testing::font_with("Noto Sans Telugu", &['క', 'ి']).unwrap());
        assert_ne!(db.scan_generation(), generation);
        let faces = installed_ui_fonts(&db, CRAFT, "en");
        assert_eq!(faces.len(), 1);
        let mut defs = font_definitions(&[], "en");
        add_system_fallbacks(&mut defs, &faces);
        assert!(Arc::ptr_eq(&defs.font_data[&faces[0].name], &faces[0].data));
    }

    #[test]
    fn fallback_baseline_matches_primary_line_metrics() {
        let ui = include_bytes!("../../../assets/fonts/SourceSans3-Regular.ttf");
        let base = pagenor_fonts::testing::font_with("Pagenor tall interface", &['క']).unwrap();
        let tall = pagenor_fonts::testing::with_table_u16(base, b"hhea", 4, 1200).unwrap();
        let tall = pagenor_fonts::testing::with_table_u16(tall, b"hhea", 8, 500).unwrap();
        let em = |m: (f32, f32, f32, f32)| (m.1 / m.0, (m.1 - m.2 + m.3) / m.0);
        let (ap, hp) = em(pagenor_fonts::line_metrics(ui, 0).unwrap());
        let (af, hf) = em(pagenor_fonts::line_metrics(&tall, 0).unwrap());
        let tweak = baseline_tweak(&tall, 0);
        assert!((af + 0.5 * (hp - hf) + tweak.y_offset_factor - ap).abs() < 1e-4);
        assert_eq!(baseline_tweak(ui, 0).y_offset_factor, 0.0);
        assert_eq!(baseline_tweak(b"broken", 0).y_offset_factor, 0.0);
    }
}
