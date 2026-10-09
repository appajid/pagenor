//! Pagenor's egui frontend: an InDesign-style UI over `pagenor-engine`.
//!
//! The UI is thin: every action goes through [`DesignApp::run`], which dispatches UI commands
//! (view/window) here and everything else to the engine. Menus, shortcuts, the ⌘K palette and
//! the control channel ([`control`]) share that entry point.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub mod about;
pub mod canvas;
pub mod chrome;
pub mod control;
pub mod credits;
pub mod dialogs;
pub mod dock;
pub mod i18n;
pub mod icons;
pub mod menus;
pub mod panels;
pub mod render_worker;
mod rtl;
mod startup;
pub mod story_editor;
pub mod taskbar;
pub mod theme;
pub mod toolbar;
pub mod widgets;

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use pagenor_engine::{Session, SnapView, UiRequest, ViewInfo};
use pagenor_geom::{Point, Unit};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use control::{ControlRequest, ControlResponse};

pub type ReadFn = Box<dyn Fn(&str) -> Result<Vec<u8>, String>>;
pub type WriteFn = Box<dyn FnMut(&str, &[u8]) -> Result<(), String>>;
pub type PickFn = Box<dyn FnMut(&str) -> Option<String>>;
/// Context captured before any host file-picker or read await.
#[derive(Clone, Debug)]
pub struct ImportRequest {
    pub purpose: String,
    pub target: Option<u64>,
}
pub struct ImportedFile {
    pub request: ImportRequest,
    pub name: String,
    pub bytes: Vec<u8>,
}
pub type OpenAsyncFn = Box<dyn FnMut(ImportRequest)>;
pub type DownloadFn = Box<dyn FnMut(&str, &[u8])>;
pub type ClipboardTextFn = Box<dyn FnMut() -> Option<String>>;
/// Files delivered asynchronously with their original operation and document identity.
pub type Inbox = std::sync::Arc<std::sync::Mutex<Vec<ImportedFile>>>;

/// Platform services injected by the host (desktop or web).
#[derive(Default)]
pub struct Services {
    /// Open-file dialog for a purpose (`open`, `place`) → path.
    pub pick_open: Option<PickFn>,
    /// Save dialog with a suggested name → path.
    pub pick_save: Option<PickFn>,
    pub read: Option<ReadFn>,
    pub write: Option<WriteFn>,
    /// Asynchronous open dialog for a purpose (`open`, `place`); the chosen file arrives later
    /// through [`Services::inbox`]. Used when `pick_open` is unset (web).
    pub open_async: Option<OpenAsyncFn>,
    /// Hand bytes to the user as a named file (browser download). When set, Save uses it
    /// instead of writing to a path.
    pub download: Option<DownloadFn>,
    /// Read plain Unicode text from the host system clipboard.
    pub clipboard_text: Option<ClipboardTextFn>,
    /// Files delivered asynchronously, drained every frame with their captured request context.
    pub inbox: Option<Inbox>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenMode {
    #[default]
    Normal,
    Preview,
    Bleed,
    Slug,
    Presentation,
}

/// A saved workspace: which bars show and where the panels are.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SavedWorkspace {
    pub name: String,
    pub control_bar: bool,
    pub task_bar: bool,
    pub task_bar_pin: Option<[f32; 2]>,
    pub tools_double_column: bool,
    pub dock_tab: String,
    pub dock_expanded: bool,
    pub open_panel: Option<String>,
    pub floating: Vec<(String, [f32; 2])>,
}

/// A missing on-by-default bool stays on. `bool::default` is false.
fn default_true() -> bool {
    true
}

/// A missing snap zone stays at the factory width, in screen pixels.
fn default_zone() -> f64 {
    4.0
}

/// Persisted UI state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiState {
    pub brightness: theme::Brightness,
    pub screen_mode: ScreenMode,
    pub frame_edges: bool,
    pub rulers: bool,
    pub guides: bool,
    pub baseline_grid: bool,
    pub document_grid: bool,
    pub text_threads: bool,
    pub hidden_characters: bool,
    /// View › Structure › Show Tagged Frames.
    pub tagged_frames: bool,
    /// Edit › Menus: hidden menu items (`menu/label`), and Show Full Menus.
    pub hidden_menu_items: Vec<String>,
    pub show_full_menus: bool,
    /// Scripts panel: saved scripts (name, text).
    pub scripts: Vec<(String, String)>,
    /// View › Structure › Show Tag Markers.
    pub tag_markers: bool,
    /// View › Flattener Preview: objects that involve transparency highlighted in red.
    #[serde(skip)]
    pub flattener_preview: bool,
    /// Tab / Shift+Tab: 1 = every panel hidden, 2 = all but the Tools panel (0 = shown).
    pub hidden_panels: u8,
    /// Edit › Color Settings (re-applied at start).
    pub color_settings: Option<pagenor_color::cms::ColorSettings>,
    /// View › Proof Colors and View › Proof Setup.
    pub proof_colors: bool,
    pub proof_setup: pagenor_color::cms::ProofSetup,
    /// Edit › Spelling › Dynamic Spelling: misspelled words underlined on the canvas.
    pub dynamic_spelling: bool,
    /// Preferences › Story Editor Display: text size (points).
    pub story_editor_size: f32,
    /// Edit › Interface Language: supported codes are listed in `i18n::LANGUAGES`.
    pub language: String,
    /// Edit › Transparency Flattener Presets: "" (none), "high", "medium" or "low" for PDF export.
    pub flattener: String,
    /// View › Separations Preview: a process plate (0–3) and/or an ink limit (total, 0–4).
    #[serde(skip)]
    pub separation: Option<u8>,
    #[serde(skip)]
    pub ink_limit: Option<f32>,
    /// View › Display Performance.
    pub display_quality: pagenor_render::DisplayQuality,
    /// Preferences › Interface › UI scaling (1 = 100%).
    pub ui_scale: f32,
    /// Preferences › Appearance of Black: 100% K on screen as rich black.
    pub rich_black: bool,
    /// View › Overprint Preview.
    pub overprint_preview: bool,
    /// Edit › Keyboard Shortcuts: command id → shortcut ("" = none), over the defaults.
    pub shortcuts: std::collections::BTreeMap<String, String>,
    pub control_bar: bool,
    pub tools_double_column: bool,
    /// Expanded right-dock panel group tab.
    pub dock_tab: String,
    /// Panel opened from the collapsed icon column (flyout).
    pub open_panel: Option<String>,
    /// Panels torn off the dock into their own floating windows: (panel id, top-left position).
    pub floating: Vec<(String, [f32; 2])>,
    pub dock_expanded: bool,
    pub units: Unit,
    pub workspace: String,
    /// Window › Workspace › New Workspace: saved panel arrangements.
    pub custom_workspaces: Vec<SavedWorkspace>,
    /// Transform reference point (0..8, row-major; 0 = top-left).
    pub ref_point: u8,
    /// Align To target (`selection`, `keyObject`, `margins`, `page`, `spread`).
    pub align_to: String,
    /// Properties > Text Style tab: 0 = Paragraph Styles, 1 = Character Styles.
    pub text_style_tab: u8,
    pub guides_locked: bool,
    pub smart_guides: bool,
    #[serde(default = "default_true")]
    pub snap_to_guides: bool,
    pub snap_to_document_grid: bool,
    #[serde(default = "default_true")]
    pub align_edges: bool,
    #[serde(default = "default_true")]
    pub align_centers: bool,
    #[serde(default = "default_true")]
    pub smart_dimensions: bool,
    #[serde(default = "default_true")]
    pub smart_spacing: bool,
    #[serde(default = "default_zone")]
    pub snap_zone: f64,
    /// Window > Contextual Task Bar.
    pub task_bar: bool,
    /// Where the Contextual Task Bar is pinned: its top-left, in points from the canvas's top-left
    /// (`None`: it follows the selection).
    pub task_bar_pin: Option<[f32; 2]>,
    /// Where the Contextual Task Bar was last shown, like [`Self::task_bar_pin`].
    #[serde(skip)]
    pub task_bar_at: Option<[f32; 2]>,
    /// Help › About Pagenor is open.
    pub about: bool,
    /// The About window's tab: 0 About, 1 Contributors, 2 Models (`about::ABOUT_TABS`).
    #[serde(skip)]
    pub about_tab: u8,
    /// URLs to open in the browser on the next frame (Help links, About, start screen).
    pub pending_urls: Vec<String>,
    #[serde(skip)]
    pub status: String,
    #[serde(skip)]
    pub dialog: Option<dialogs::Dialog>,
    #[serde(skip)]
    pub palette: Option<String>,
    #[serde(skip)]
    pub flyout: Option<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            brightness: theme::Brightness::MediumDark,
            screen_mode: ScreenMode::Normal,
            frame_edges: true,
            rulers: true,
            guides: true,
            baseline_grid: false,
            document_grid: false,
            text_threads: false,
            hidden_characters: false,
            tagged_frames: false,
            hidden_menu_items: Vec::new(),
            show_full_menus: false,
            story_editor_size: 14.0,
            scripts: vec![
                ("Number the pages".into(), "# A page-number frame at the bottom of the current page.\nframe.create {\"rect\": [288, 740, 324, 760], \"content\": \"text\", \"text\": \"\"}\ntext.select {\"story\": \"$0.story\", \"anchor\": 0, \"focus\": 0}\ntext.insert {\"text\": \"\\ue000\", \"raw\": true}".into()),
                ("Two-column grid".into(), "# Four text frames in a 2 × 2 grid.\nframe.grid {\"rect\": [36, 36, 576, 756], \"cols\": 2, \"rows\": 2, \"gutter\": 12}".into()),
            ],
            tag_markers: false,
            flattener_preview: false,
            hidden_panels: 0,
            color_settings: None,
            proof_colors: false,
            proof_setup: Default::default(),
            dynamic_spelling: false,
            language: String::new(),
            flattener: String::new(),
            separation: None,
            ink_limit: None,
            display_quality: pagenor_render::DisplayQuality::High,
            ui_scale: 1.0,
            rich_black: false,
            overprint_preview: false,
            shortcuts: Default::default(),
            control_bar: false,
            tools_double_column: false,
            dock_tab: "properties".into(),
            open_panel: None,
            floating: Vec::new(),
            dock_expanded: true,
            units: Unit::Picas,
            workspace: "Essentials".into(),
            custom_workspaces: Vec::new(),
            ref_point: 0,
            align_to: "selection".into(),
            text_style_tab: 0,
            guides_locked: false,
            smart_guides: true,
            snap_to_guides: true,
            snap_to_document_grid: false,
            align_edges: true,
            align_centers: true,
            smart_dimensions: true,
            smart_spacing: true,
            snap_zone: 4.0,
            task_bar: true,
            task_bar_pin: None,
            task_bar_at: None,
            about: false,
            about_tab: 0,
            pending_urls: Vec::new(),
            status: String::new(),
            dialog: None,
            palette: None,
            flyout: None,
        }
    }
}

impl UiState {
    /// Copies the live view switches into the snap engine's view.
    pub fn snap_view(&self) -> SnapView {
        SnapView {
            snap_to_guides: self.snap_to_guides,
            snap_to_document_grid: self.snap_to_document_grid,
            show_guides: self.guides,
            smart_guides: self.smart_guides,
            align_edges: self.align_edges,
            align_centers: self.align_centers,
            smart_dimensions: self.smart_dimensions,
            smart_spacing: self.smart_spacing,
            zone_px: self.snap_zone,
        }
    }
}

/// Per-document view: zoom (screen points per document point) and the canvas point at the
/// top-left of the canvas area.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct View {
    pub zoom: f64,
    pub origin: Point,
    pub fitted: bool,
    /// View › Rotate Spread: quarter turns clockwise (0–3).
    pub rotation: u8,
}

impl Default for View {
    fn default() -> Self {
        View { zoom: 0.5, origin: Point::new(-100.0, -100.0), fitted: false, rotation: 0 }
    }
}

/// What the canvas texture currently shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    /// (doc uid, doc pointer, revision, preview).
    pub doc: (u64, u64, u64, bool),
    /// Canvas point at the texture's top-left, zoom, and texture size in screen points.
    pub origin: Point,
    pub zoom: f64,
    pub size: (f32, f32),
}

impl CanvasCache {
    pub fn new() -> Self {
        let mut renderer = pagenor_render::Renderer::new();
        renderer.threads = pagenor_render::default_threads();
        CanvasCache {
            renderer,
            texture: None,
            shown: None,
            pending: None,
            token: 0,
            #[cfg(not(target_arch = "wasm32"))]
            worker: None,
            worker_started: false,
            last_ms: 0.0,
            shown_doc: None,
            pending_doc: None,
            patcher: pagenor_render::Renderer::new(),
            patches: 0,
        }
    }
}

impl Default for CanvasCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CanvasCache {
    pub renderer: pagenor_render::Renderer,
    pub texture: Option<egui::TextureHandle>,
    pub shown: Option<Shown>,
    /// The request in flight on the worker.
    pub pending: Option<(u64, Shown)>,
    pub token: u64,
    #[cfg(not(target_arch = "wasm32"))]
    pub worker: Option<render_worker::Worker>,
    pub worker_started: bool,
    pub last_ms: f64,
    /// The document the texture shows (to repaint only what an edit changed).
    pub shown_doc: Option<std::sync::Arc<pagenor_doc::Document>>,
    /// Document of the request in flight.
    pub pending_doc: Option<std::sync::Arc<pagenor_doc::Document>>,
    /// Renders damaged regions (its own context: region sizes vary).
    pub patcher: pagenor_render::Renderer,
    /// Partial repaints / full renders since start (perf readout).
    pub patches: u64,
}

#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct Perf {
    pub frame_ms: f64,
    pub render_ms: f64,
    pub fps: f64,
}

pub struct DesignApp {
    pending_pdf: Option<ImportedFile>,
    pub session: Session,
    pub ui: UiState,
    pub services: Services,
    pub views: HashMap<u64, View>,
    /// The canvas cache and rect of the pane in `pane` (the other pane's are in `other_pane`).
    pub canvas: CanvasCache,
    pub canvas_rect: Option<egui::Rect>,
    /// Window › Arrange › Split Window: two views of the document side by side.
    pub split: bool,
    /// Window › Arrange › New Window: the second view in its own window (pane 1).
    pub second_window: bool,
    /// The current pane (0 or 1): views, cache and rect refer to it.
    pub pane: u8,
    /// The pane last clicked (menu zoom and scroll go there).
    pub focus_pane: u8,
    pub other_pane: Option<(CanvasCache, Option<egui::Rect>)>,
    /// Power Zoom in progress: the zoom to return to and the canvas point it will centre on.
    pub power_zoom: Option<(f64, pagenor_geom::Point)>,
    /// The persisted colour settings have been applied this session.
    pub color_applied: bool,
    pub perf: Perf,
    pub synthetic: Vec<egui::Event>,
    /// Story open in the Story Editor.
    pub story_editor: Option<pagenor_doc::StoryId>,
    control_rx: Option<Receiver<ControlRequest>>,
    pending_shots: Vec<(u64, Option<String>, Sender<ControlResponse>, f64)>,
    queued_shots: Vec<(u64, f64, u32)>,
    shot_token: u64,
    styled: bool,
    /// The interface language the UI fonts were installed for (it orders the CJK fallbacks).
    fonts_lang: String,
    pub restyle: bool,
    fonts_ready: bool,
    /// Text copied through any command route, awaiting the host's egui output pass.
    pending_clipboard_text: Option<String>,
    /// Transient desktop launch artwork; never persisted with preferences/documents.
    startup_splash: Option<startup::Splash>,
    font_generation: u64,
    pub integrated_titlebar: bool,
    /// The host installed a native menu bar (macOS): don't draw menus in the window.
    pub native_menu: bool,
    /// Shortcuts the native menu handles (skip them in the egui shortcut handler).
    pub native_shortcuts: std::collections::HashSet<String>,
    last_time: f64,
    /// Last automatic recovery attempt (seconds, egui time), including failed attempts.
    pub last_recovery: f64,
    #[cfg(not(target_arch = "wasm32"))]
    recovery_failed: bool,
    #[cfg(not(target_arch = "wasm32"))]
    quit_requested: bool,
}

impl DesignApp {
    pub fn new(session: Session, services: Services) -> Self {
        // The font menus and the first file opened need the installed fonts: catalog them now.
        pagenor_fonts::FontDb::global().scan_in_background();
        DesignApp {
            pending_pdf: None,
            session,
            ui: UiState::default(),
            services,
            views: HashMap::new(),
            canvas: CanvasCache::new(),
            canvas_rect: None,
            split: false,
            second_window: false,
            pane: 0,
            focus_pane: 0,
            other_pane: None,
            power_zoom: None,
            color_applied: false,
            perf: Perf::default(),
            synthetic: vec![],
            story_editor: None,
            control_rx: None,
            pending_shots: vec![],
            queued_shots: vec![],
            shot_token: 0,
            styled: false,
            fonts_lang: String::new(),
            restyle: false,
            fonts_ready: false,
            pending_clipboard_text: None,
            startup_splash: None,
            font_generation: 0,
            integrated_titlebar: false,
            native_menu: false,
            native_shortcuts: Default::default(),
            last_time: 0.0,
            last_recovery: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            recovery_failed: false,
            #[cfg(not(target_arch = "wasm32"))]
            quit_requested: false,
        }
    }

    pub fn with_control(mut self, rx: Receiver<ControlRequest>) -> Self {
        self.control_rx = Some(rx);
        self
    }

    /// Show the supplied artwork briefly on launch, even when opening/restoring
    /// documents. Desktop hosts opt in; the web keeps its existing loader.
    pub fn with_startup_splash(mut self) -> Self {
        self.startup_splash = Some(startup::Splash::default());
        self
    }

    /// The view key of the active document in the current pane.
    fn view_key(&self) -> Option<u64> {
        let uid = self.session.active()?.uid;
        Some(if self.pane == 1 { uid ^ (1 << 63) } else { uid })
    }
    pub fn view(&self) -> Option<&View> {
        self.views.get(&self.view_key()?)
    }
    pub fn view_mut(&mut self) -> Option<&mut View> {
        let k = self.view_key()?;
        Some(self.views.entry(k).or_default())
    }
    /// Make `pane` current: its canvas cache and rect move in.
    pub fn switch_pane(&mut self, pane: u8) {
        if pane == self.pane {
            return;
        }
        let (c, r) = self.other_pane.take().unwrap_or_default();
        let old = (std::mem::replace(&mut self.canvas, c), std::mem::replace(&mut self.canvas_rect, r));
        self.other_pane = Some(old);
        self.pane = pane;
    }
    pub fn view_info(&self) -> ViewInfo {
        ViewInfo { zoom: self.view().map(|v| v.zoom).unwrap_or(1.0), snap: self.ui.snap_view(), unit: self.ui.units }
    }

    pub fn status(&mut self, s: impl Into<String>) {
        self.ui.status = s.into();
    }

    /// THE entry point for every action (menus, shortcuts, palette, panels, control channel).
    pub fn run(&mut self, id: &str, params: Value) -> Result<Value, String> {
        // Pagenor external clipboard bridge. Native menus/accelerators can invoke Paste
        // without an egui Paste event, so fetch Unicode text from the host in that case.
        let mut params = params;
        if matches!(id, "edit.paste" | "edit.pasteWithoutFormatting")
            && params.get("text").is_none()
            // A Copy followed by Paste in the same command batch precedes platform output.
            && let Some(text) = self.pending_clipboard_text.clone().or_else(|| self.services.clipboard_text.as_mut().and_then(|read| read()))
        {
            if let Some(obj) = params.as_object_mut() {
                obj.insert("text".into(), Value::String(text));
            } else {
                params = json!({"text": text});
            }
        }

        if let Some(r) = menus::run_ui(self, id, &params) {
            return r;
        }
        let r = self.session.execute(id, &params).map_err(|e| e.to_string());
        if matches!(id, "edit.copy" | "edit.cut")
            && let Ok(result) = &r
        {
            // Object copies stay internal. Only successful text results reach the OS/browser.
            self.pending_clipboard_text = result.get("text").and_then(Value::as_str).map(str::to_owned);
        }
        self.after_engine();
        if let Err(e) = &r {
            self.status(e.clone());
        } else if id == "font.rescan"
            && let Ok(report) = &r
        {
            self.status(format!("Font scan complete: {} families, {} faces", report["families"], report["faces"]));
        }
        r
    }

    fn flush_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(text) = self.pending_clipboard_text.take() {
            ctx.copy_text(text);
        }
    }

    /// Handle requests produced by tools/commands (dialogs, view changes, file pickers).
    pub fn after_engine(&mut self) {
        let reqs = std::mem::take(&mut self.session.ui_requests);
        for r in reqs {
            match r {
                UiRequest::Dialog { id, params } => self.ui.dialog = Some(dialogs::Dialog::new(&id, params)),
                UiRequest::View { params } => canvas::apply_view_request(self, &params),
                UiRequest::Pick { purpose, params } => {
                    let _ = params;
                    let _ = self.pick_and_open(&purpose);
                }
            }
        }
        // New documents get a fitted view.
        if let Some(st) = self.session.active() {
            let uid = st.uid;
            self.views.entry(uid).or_default();
        }
    }

    /// Ask the host for a file to open (`open`) or place (`place`). Synchronous pickers run the
    /// command right away; asynchronous ones (web) deliver the file through the inbox.
    pub fn pick_and_open(&mut self, purpose: &str) -> Result<Value, String> {
        let target = self.session.active().map(|d| d.uid);
        let cmd = if purpose == "place" { "file.place" } else { "file.open" };
        if let Some(pick) = self.services.pick_open.as_mut() {
            return match pick(purpose) {
                // Image Import Options: a multi-page PDF asks which page.
                Some(path) if purpose == "place" && path.to_ascii_lowercase().ends_with(".pdf") => {
                    let pages = self.services.read.as_mut().and_then(|r| r(&path).ok()).and_then(|b| pagenor_render::pdf_page_count(&b)).unwrap_or(1);
                    if pages > 1 {
                        self.ui.dialog =
                            Some(dialogs::Dialog::new("pdfImport", json!({"path": path, "page": "1", "pages": pages, "target": target})));
                        Ok(Value::Null)
                    } else {
                        self.open_file(cmd, json!({"path": path}))
                    }
                }
                Some(path) => self.open_file(cmd, json!({"path": path})),
                None => Ok(Value::Null),
            };
        }
        let request = self.import_request(purpose);
        if let Some(open) = self.services.open_async.as_mut() {
            open(request);
        }
        Ok(Value::Null)
    }

    pub fn import_request(&self, purpose: &str) -> ImportRequest {
        ImportRequest { purpose: purpose.into(), target: self.session.active().map(|d| d.uid) }
    }

    pub(crate) fn activate_import_target(&mut self, request: &ImportRequest) -> Result<(), String> {
        let Some(index) = request.target.and_then(|uid| self.session.documents().iter().position(|d| d.uid == uid)) else {
            let message = "Import cancelled: the originating document has closed";
            self.status(message);
            return Err(message.into());
        };
        self.run("file.activate", json!({"index": index}))?;
        Ok(())
    }

    pub(crate) fn cancel_pdf_import(&mut self) {
        self.pending_pdf = None;
    }

    pub(crate) fn confirm_pdf_import(&mut self, page: u64, crop: String) -> Result<Value, String> {
        let file = self.pending_pdf.take().ok_or("PDF import cancelled: no pending file")?;
        self.activate_import_target(&file.request)?;
        self.run(
            "file.place",
            json!({"name": file.name, "base64": pagenor_engine::cmd::base64_encode(&file.bytes), "pdfPage": page, "pdfCrop": crop}),
        )
    }

    /// A single unresolved PDF owns its bytes. Later deliveries cancel explicitly rather than
    /// replacing its options. Cancellation, replacement and target closure release its payload.
    fn drain_inbox(&mut self) {
        if let Some(file) = &self.pending_pdf {
            let target_exists = self.session.documents().iter().any(|d| Some(d.uid) == file.request.target);
            let owns_dialog =
                self.ui.dialog.as_ref().is_some_and(|d| d.id == "pdfImport" && d.fields.get("async").and_then(Value::as_bool) == Some(true));
            if !target_exists || !owns_dialog {
                self.pending_pdf = None;
                if !target_exists {
                    if owns_dialog {
                        self.ui.dialog = None;
                    }
                    self.status("PDF import cancelled: the originating document has closed");
                }
            }
        }
        let Some(inbox) = self.services.inbox.clone() else { return };
        let files = std::mem::take(&mut *inbox.lock().unwrap_or_else(|e| e.into_inner()));
        for file in files {
            if self.ui.dialog.is_some() {
                self.status(format!("{}: import cancelled while a dialog is open; choose the file again", file.name));
                continue;
            }
            let name = file.name.clone();
            if let Err(e) = self.import_file(file) {
                self.status(format!("{name}: {e}"));
            }
        }
    }

    fn import_file(&mut self, file: ImportedFile) -> Result<Value, String> {
        let name = file.name.clone();
        let place = file.request.purpose == "place" || (file.request.purpose == "drop" && !opens_as_document(&name));
        let result = self.import_file_inner(file);
        if let Err(error) = &result {
            let title = if place { "Can't Place the File" } else { "Can't Open the File" };
            self.ui.dialog = Some(dialogs::Dialog::new("alert", json!({"title": title, "file": name, "message": error})));
        }
        result
    }

    fn import_file_inner(&mut self, file: ImportedFile) -> Result<Value, String> {
        let lower = file.name.to_ascii_lowercase();
        let layout = opens_as_document(&file.name);
        let place =
            file.request.purpose == "place" || (matches!(file.request.purpose.as_str(), "drop" | "open") && !layout && !lower.ends_with(".ase"));
        if place || lower.ends_with(".ase") || file.request.purpose == "swatches" {
            self.activate_import_target(&file.request)?;
        }
        if place && lower.ends_with(".pdf") {
            let pages = pagenor_render::pdf_page_count(&file.bytes).unwrap_or(1);
            if pages > 1 {
                self.ui.dialog =
                    Some(dialogs::Dialog::new("pdfImport", json!({"async": true, "name": file.name.clone(), "page": "1", "pages": pages})));
                self.pending_pdf = Some(file);
                return Ok(Value::Null);
            }
        }
        let b64 = pagenor_engine::cmd::base64_encode(&file.bytes);
        if lower.ends_with(".ase") || file.request.purpose == "swatches" {
            self.run("swatch.load", json!({"base64": b64}))
        } else if place {
            self.run("file.place", json!({"name": file.name, "base64": b64}))
        } else {
            let title = file.name.rsplit_once('.').map_or(file.name.as_str(), |(stem, _)| stem);
            self.run("file.openBytes", json!({"name": title, "base64": b64}))
        }
    }

    pub fn select_tool(&mut self, id: &str) {
        let _ = self.run("tool.select", json!({"tool": id}));
        self.ui.flyout = None;
    }

    /// Per-frame logic before layout.
    pub fn logic(&mut self, ctx: &egui::Context) {
        if !self.color_applied {
            self.color_applied = true;
            if let Some(cs) = self.ui.color_settings.clone() {
                let _ = pagenor_color::cms::set_active(&cs);
            }
        }
        let scale = self.ui.ui_scale.clamp(0.5, 3.0);
        if (ctx.zoom_factor() - scale).abs() > 1e-3 {
            ctx.set_zoom_factor(scale);
        }
        if !self.styled {
            theme::install_fonts(ctx, &self.ui.language);
            self.fonts_lang = self.ui.language.clone();
            self.styled = true;
            self.restyle = true;
        } else {
            if self.fonts_lang != self.ui.language {
                theme::install_fonts(ctx, &self.ui.language);
                self.fonts_lang = self.ui.language.clone();
            }
            self.fonts_ready = true;
        }
        if self.restyle {
            theme::apply(ctx, &theme::Tokens::for_brightness(self.ui.brightness));
            self.restyle = false;
        }
        let now = ctx.input(|i| i.time);
        #[cfg(not(target_arch = "wasm32"))]
        self.recovery_tick(ctx, now);
        let dt = now - self.last_time;
        if dt > 0.0 {
            self.perf.fps = self.perf.fps * 0.9 + (1.0 / dt).min(240.0) * 0.1;
        }
        self.last_time = now;
        self.drain_control(ctx);
        if self.fonts_ready {
            self.drain_inbox();
        }
        if !self.synthetic.is_empty() {
            ctx.request_repaint();
        }
        self.collect_screenshots(ctx);
        self.issue_screenshots(ctx);
        if self.fonts_ready {
            menus::shortcuts(self, ctx);
        }
        #[cfg(not(target_arch = "wasm32"))]
        for f in ctx.input(|i| i.raw.dropped_files.clone()) {
            let p = f.path().to_string_lossy().to_string();
            if !p.is_empty() {
                let _ = self.open_dropped(&p);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.handle_close_request(ctx);
        self.flush_clipboard(ctx);
    }

    /// Flush current edits before desktop shutdown, without marking documents saved.
    /// Hosts also call this as a last-chance fallback when termination cannot be cancelled.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn prepare_shutdown(&mut self) -> Result<(), String> {
        if !pagenor_engine::recovery::needs_save(&self.session) {
            return Ok(());
        }
        let result = if self.session.recovery_dir.is_some() {
            self.session.execute("file.recovery.save", &json!({})).map(|_| ()).map_err(|e| e.to_string())
        } else {
            Err("No recovery folder is configured for unsaved documents.".into())
        };
        match result {
            Ok(()) => {
                if self.recovery_failed {
                    self.status("Recovery is up to date.");
                }
                self.recovery_failed = false;
                Ok(())
            }
            Err(e) => {
                self.recovery_failed = true;
                let message = format!("Couldn't write recovery data: {e}");
                self.status(&message);
                Err(message)
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn request_quit(&mut self) -> Result<Value, String> {
        // Control callers receive write failures immediately. Close generates a cancellable
        // viewport event on the next pass, where any intervening edits are checkpointed again.
        self.quit_requested = false;
        if let Err(e) = self.prepare_shutdown() {
            self.status(format!("{e} Pagenor will stay open."));
            return Err(e);
        }
        self.quit_requested = true;
        Ok(Value::Null)
    }

    /// Run in logic (including occluded windows) and again after host UI processing, so
    /// commands later in a closing frame cannot leave newer edits outside the checkpoint.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.viewport_id() != egui::ViewportId::ROOT {
            return;
        }
        if ctx.input(|i| i.viewport().close_requested())
            && let Err(e) = self.prepare_shutdown()
        {
            self.status(format!("{e} Pagenor will stay open."));
            self.quit_requested = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if std::mem::take(&mut self.quit_requested) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Save dirty documents and remove obsolete owned checkpoints on the same schedule.
    /// Schedule our own wakeup: recovery must not depend on caret animation or font refresh.
    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_tick(&mut self, ctx: &egui::Context, now: f64) {
        use std::time::Duration;
        if self.session.recovery_dir.is_none() || !pagenor_engine::recovery::needs_save(&self.session) {
            return;
        }
        if !now.is_finite() {
            ctx.request_repaint_after(Duration::from_secs(5));
            return;
        }
        // Match the preferences dialog's one-hour maximum; clamp before multiplication so
        // large finite preferences cannot overflow to infinity and silently disable recovery.
        let minutes = self.session.prefs.recovery_minutes;
        let interval = if minutes.is_finite() { minutes.clamp(5.0 / 60.0, 60.0) * 60.0 } else { 30.0 };
        if !self.last_recovery.is_finite() || now < self.last_recovery {
            self.last_recovery = now;
        }
        if now - self.last_recovery >= interval {
            // Throttle failures too: a full or unavailable disk must not retry every frame.
            self.last_recovery = now;
            match self.session.execute("file.recovery.save", &json!({})) {
                Ok(_) => {
                    if self.recovery_failed {
                        self.status("Recovery is up to date.");
                    }
                    self.recovery_failed = false;
                }
                Err(e) => {
                    self.recovery_failed = true;
                    self.status(format!("Couldn't write recovery data: {e}"));
                }
            }
        }
        if pagenor_engine::recovery::needs_save(&self.session) {
            let remaining = (interval - (now - self.last_recovery)).clamp(0.001, interval);
            ctx.request_repaint_after(Duration::try_from_secs_f64(remaining).unwrap_or(Duration::from_secs(5)));
        }
    }

    /// A file dropped on the window: documents open, anything else is placed.
    pub fn open_dropped(&mut self, path: &str) -> Result<Value, String> {
        let cmd = if opens_as_document(path) { "file.open" } else { "file.place" };
        self.open_file(cmd, json!({"path": path}))
    }

    /// Open (`file.open`, `file.openBytes`) or place (`file.place`) a file the user chose. A file
    /// that can't be opened or placed gets an alert with the reason, as well as the status line.
    pub fn open_file(&mut self, cmd: &str, params: Value) -> Result<Value, String> {
        let file = match params.get("path").and_then(Value::as_str) {
            Some(path) => std::path::Path::new(path).file_name().map_or_else(|| path.to_string(), |n| n.to_string_lossy().to_string()),
            None => params.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
        };
        let r = self.run(cmd, params);
        if let Err(e) = &r {
            let title = if cmd == "file.place" { "Can't Place the File" } else { "Can't Open the File" };
            self.ui.dialog = Some(dialogs::Dialog::new("alert", json!({"title": title, "file": file, "message": e})));
        }
        r
    }

    /// Inject synthetic events (one pointer event per frame).
    pub fn raw_input_hook(&mut self, raw: &mut egui::RawInput) {
        if self.synthetic.is_empty() {
            return;
        }
        let n = match self.synthetic[0] {
            egui::Event::PointerMoved(_) | egui::Event::PointerButton { .. } => 1,
            _ => self.synthetic.iter().position(|e| matches!(e, egui::Event::Key { pressed: false, .. })).map_or(self.synthetic.len(), |i| i + 1),
        };
        if let Some(egui::Event::PointerMoved(p) | egui::Event::PointerButton { pos: p, .. }) = self.synthetic.first() {
            raw.events.push(egui::Event::PointerMoved(*p));
        }
        raw.events.extend(self.synthetic.drain(..n));
    }

    /// Lay out the whole window.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.flush_clipboard(&ctx);
        let db = pagenor_fonts::FontDb::global();
        db.refresh_in_background();
        #[cfg(not(target_arch = "wasm32"))]
        ctx.request_repaint_after(std::time::Duration::from_secs(3));
        let generation = db.scan_generation();
        if self.font_generation != generation {
            self.font_generation = generation;
            self.session.cache.clear();
            self.canvas.shown = None;
            self.canvas.pending = None;
            self.canvas.shown_doc = None;
            self.canvas.pending_doc = None;
            if let Some((canvas, _)) = &mut self.other_pane {
                canvas.shown = None;
                canvas.pending = None;
                canvas.shown_doc = None;
                canvas.pending_doc = None;
            }
            if self.ui.dialog.as_ref().is_some_and(|d| d.id == "fontDiscovery")
                && let Ok(Value::Object(report)) = self.run("font.diagnostics", json!({}))
                && let Some(dialog) = &mut self.ui.dialog
            {
                dialog.fields = report;
            }
        }
        if let Some(splash) = &mut self.startup_splash {
            if splash.show(ui) {
                return;
            }
            self.startup_splash = None;
        }
        if !self.fonts_ready {
            ctx.request_repaint();
            return;
        }
        let t0 = now_ms();
        let t = theme::Tokens::get(&ctx);
        let presenting = self.ui.screen_mode == ScreenMode::Presentation;
        let dbg = std::env::var_os("PAGENOR_DEBUG_LAYOUT").is_some();
        if dbg {
            eprintln!("root start {:?}", ui.available_rect_before_wrap());
        }
        if !presenting {
            chrome::app_bar(self, ui);
            if dbg {
                eprintln!("after app bar {:?}", ui.available_rect_before_wrap());
            }
            if self.ui.control_bar && self.session.active().is_some() && self.ui.hidden_panels == 0 {
                chrome::control_bar(self, ui);
            }
            chrome::status_bar(self, ui);
            if dbg {
                eprintln!("after status {:?}", ui.available_rect_before_wrap());
            }
            if self.ui.hidden_panels != 1 {
                toolbar::show(self, ui);
            }
            if self.ui.hidden_panels == 0 {
                dock::show(self, ui);
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.pasteboard)).show(ui, |ui| {
            if self.session.active().is_none() {
                chrome::start_screen(self, ui);
                return;
            }
            if !presenting {
                chrome::doc_tabs(self, ui);
            }
            if self.split && !presenting {
                // Two panes with a divider; each has its own view (zoom, scroll) and render cache.
                let r = ui.available_rect_before_wrap();
                let mid = r.center().x;
                let halves = [
                    egui::Rect::from_min_max(r.min, egui::pos2(mid - 1.0, r.max.y)),
                    egui::Rect::from_min_max(egui::pos2(mid + 1.0, r.min.y), r.max),
                ];
                ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(mid - 1.0, r.min.y), egui::pos2(mid + 1.0, r.max.y)), 0.0, t.border);
                for (k, half) in halves.into_iter().enumerate() {
                    self.switch_pane(k as u8);
                    // A new pane starts at the other pane's view.
                    if k == 1 && self.view().is_none() {
                        self.switch_pane(0);
                        let v = self.view().copied();
                        self.switch_pane(1);
                        if let (Some(mut v), Some(m)) = (v, self.view_mut()) {
                            v.fitted = false;
                            *m = v;
                        }
                    }
                    ui.scope_builder(egui::UiBuilder::new().max_rect(half), |ui| canvas::show(self, ui));
                    if ui.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| half.contains(p))) {
                        self.focus_pane = k as u8;
                    }
                }
                self.switch_pane(self.focus_pane);
            } else {
                self.switch_pane(0);
                // With a second window, the main one takes the focus back when clicked.
                if !self.second_window || (ui.input(|i| i.pointer.any_pressed()) && ui.ui_contains_pointer()) {
                    self.focus_pane = 0;
                }
                canvas::show(self, ui);
            }
        });
        // New Window: pane 1 in a window of its own.
        if self.second_window
            && !presenting
            && let Some(title) = self.session.active().map(|d| format!("{} — 2", d.doc.title))
        {
            let fill = t.pasteboard;
            let ctx = ui.ctx().clone();
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("pagenor_second_window"),
                egui::ViewportBuilder::default().with_title(title).with_inner_size([960.0, 720.0]),
                |ui, _class| {
                    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(fill)).show(ui, |ui| {
                        self.switch_pane(1);
                        if self.view().is_none() {
                            self.switch_pane(0);
                            let v = self.view().copied();
                            self.switch_pane(1);
                            if let (Some(mut v), Some(m)) = (v, self.view_mut()) {
                                v.fitted = false;
                                *m = v;
                            }
                        }
                        if ui.input(|i| i.pointer.any_pressed()) {
                            self.focus_pane = 1;
                        }
                        canvas::show(self, ui);
                    });
                    if ui.input(|i| i.viewport().close_requested()) {
                        self.second_window = false;
                        self.focus_pane = 0;
                    }
                },
            );
            self.switch_pane(self.focus_pane);
        }
        dock::flyout(self, &ctx);
        dock::floating(self, &ctx);
        story_editor::show(self, &ctx);
        dialogs::show(self, &ctx);
        about::show(self, &ctx);
        menus::palette(self, &ctx);
        for url in std::mem::take(&mut self.ui.pending_urls) {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        self.flush_clipboard(&ctx);
        self.perf.frame_ms = now_ms() - t0;
    }

    fn drain_control(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.control_rx.take() else { return };
        while let Ok(req) = rx.try_recv() {
            let reply = req.reply.clone();
            match control::handle(self, ctx, &req) {
                control::Outcome::Done(v) => {
                    let _ = reply.send(v);
                }
                control::Outcome::Screenshot { path } => {
                    self.shot_token += 1;
                    let token = self.shot_token;
                    self.queued_shots.push((token, now_ms() + 120.0, 0));
                    self.pending_shots.push((token, path, reply, now_ms() + 8000.0));
                }
            }
        }
        self.control_rx = Some(rx);
    }

    fn issue_screenshots(&mut self, ctx: &egui::Context) {
        let now = now_ms();
        self.queued_shots.retain_mut(|(token, at, frames)| {
            *frames += 1;
            if now >= *at && *frames >= 3 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(*token)));
                false
            } else {
                true
            }
        });
        if !self.queued_shots.is_empty() || !self.pending_shots.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }

    fn collect_screenshots(&mut self, ctx: &egui::Context) {
        if self.pending_shots.is_empty() {
            return;
        }
        let events: Vec<_> = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { user_data, image, .. } => {
                        let token = user_data.data.as_ref().and_then(|d| d.downcast_ref::<u64>()).copied()?;
                        Some((token, image.clone()))
                    }
                    _ => None,
                })
                .collect()
        });
        for (token, image) in events {
            if let Some(i) = self.pending_shots.iter().position(|(t, ..)| *t == token) {
                let (_, path, reply, _) = self.pending_shots.remove(i);
                let _ = reply.send(control::save_screenshot(self, &image, path.as_deref()));
            }
        }
        let now = now_ms();
        self.pending_shots.retain(|(_, _, reply, deadline)| {
            if now < *deadline {
                return true;
            }
            let _ = reply.send(json!({"ok": false, "error": "no frame was presented (screen locked or window hidden); use ui.render"}));
            false
        });
    }
}

/// Files that open as documents (when dropped or picked) rather than being placed: Pagenor
/// and IDML.
pub fn opens_as_document(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    pagenor_doc::file_type::is_native_name(name) || n.ends_with(".idml")
}

pub fn now_ms() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0.0
    }
}

/// Is `text` the name of a key ("AltGraph", "Dead", "AudioVolumeUp") instead of typed text?
/// Filter explicit browser key names rather than capitalized words: text/IME events can
/// contain whole words, including English names and Unicode strings.
pub fn is_key_name(text: &str) -> bool {
    matches!(
        text,
        "AltGraph"
            | "Dead"
            | "Unidentified"
            | "AudioVolumeUp"
            | "AudioVolumeDown"
            | "AudioVolumeMute"
            | "MediaTrackNext"
            | "MediaTrackPrevious"
            | "MediaPlayPause"
            | "MediaStop"
            | "MediaFastForward"
            | "MediaRewind"
            | "CapsLock"
            | "NumLock"
            | "ScrollLock"
            | "Fn"
            | "FnLock"
            | "SymbolLock"
            | "Hyper"
            | "Super"
            | "Compose"
    )
}

/// Drop text events that are key names. eframe's web backend sends the name of every key it
/// doesn't know as text, so AltGr typed "AltGraph" into the story. `text_field_focused`: an
/// egui text field has the keyboard; its text comes from the browser's input events, which can
/// hold whole words, and is left alone.
pub fn drop_key_name_text(raw: &mut egui::RawInput, text_field_focused: bool) {
    if !text_field_focused {
        raw.events.retain(|e| !matches!(e, egui::Event::Text(t) if is_key_name(t)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An app with one document that has an unsaved edit.
    fn app_with_unsaved_document() -> DesignApp {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), Services::default());
        app.run("file.new", json!({})).unwrap();
        assert!(!app.session.documents()[0].is_dirty(), "a new document has nothing to save");
        app.run("frame.create", json!({"rect": [36, 36, 136, 136]})).unwrap();
        assert!(app.session.documents()[0].is_dirty());
        app
    }

    /// File ▸ Close (and its shortcut) closed a document with unsaved changes without asking, and
    /// closing discards the document's recovery data too.
    #[test]
    fn closing_an_unsaved_document_asks_first() {
        let mut app = app_with_unsaved_document();
        menus::activate(&mut app, "file.close", &Value::Null);
        assert_eq!(app.session.documents().len(), 1, "the document stays open until the user answers");
        assert_eq!(app.ui.dialog.as_ref().map(|d| d.id.as_str()), Some("closeDocument"));
        // Cancel: nothing happens.
        app.ui.dialog = None;
        assert!(app.session.documents()[0].is_dirty());
        // The tab's × asks too.
        menus::close_document(&mut app, Some(0));
        assert_eq!(app.ui.dialog.as_ref().map(|d| d.id.as_str()), Some("closeDocument"));
        assert_eq!(app.session.documents().len(), 1);
    }

    #[test]
    fn closing_without_saving_discards_the_document() {
        let mut app = app_with_unsaved_document();
        menus::close_document(&mut app, None);
        app.ui.dialog.as_mut().unwrap().fields.insert("discard".into(), json!(true));
        dialogs::confirm(&mut app).unwrap();
        assert!(app.session.documents().is_empty());
        assert!(app.ui.dialog.is_none());
    }

    #[test]
    fn closing_with_save_writes_the_file_then_closes() {
        let mut app = app_with_unsaved_document();
        let path = std::env::temp_dir().join(format!("pagenor-close-test-{}.pgnr", std::process::id()));
        app.run("file.saveAs", json!({"path": path})).unwrap();
        let saved = std::fs::metadata(&path).unwrap().len();
        app.run("frame.create", json!({"rect": [200, 200, 300, 300]})).unwrap();
        menus::close_document(&mut app, None);
        dialogs::confirm(&mut app).unwrap();
        assert!(app.session.documents().is_empty());
        let written = std::fs::metadata(&path).unwrap().len();
        std::fs::remove_file(&path).unwrap();
        assert!(written > saved, "the second frame was saved ({saved} → {written} bytes)");
    }

    /// Save on a document that has no file opens the file picker; cancelling it must not close the
    /// document. (No picker service here: the same as cancelling.)
    #[test]
    fn a_cancelled_save_keeps_the_document_open() {
        let mut app = app_with_unsaved_document();
        menus::close_document(&mut app, None);
        dialogs::confirm(&mut app).unwrap();
        assert_eq!(app.session.documents().len(), 1);
        assert!(app.session.documents()[0].is_dirty());
    }

    #[test]
    fn closing_a_saved_document_does_not_ask() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), Services::default());
        app.run("file.new", json!({})).unwrap();
        menus::activate(&mut app, "file.close", &Value::Null);
        assert!(app.session.documents().is_empty());
        assert!(app.ui.dialog.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_save_and_copy_dialogs_use_pgnr_and_keep_copy_state() {
        let dir = std::env::temp_dir().join(format!("pagenor-ui-pgnr-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let names = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let seen = names.clone();
        let folder = dir.clone();
        let services = Services {
            pick_save: Some(Box::new(move |name| {
                seen.borrow_mut().push(name.to_string());
                Some(folder.join(name).to_string_lossy().to_string())
            })),
            ..Default::default()
        };
        let mut app = DesignApp::new(Session::new(), services);
        app.run("file.new", json!({"title": "தமிழ் Brochure"})).unwrap();
        app.run("app.save", json!({})).unwrap();
        assert_eq!(names.borrow().as_slice(), ["தமிழ் Brochure.pgnr"]);
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        let before = (app.session.doc().unwrap().path.clone(), app.session.doc().unwrap().is_dirty(), app.session.doc().unwrap().history.undo.len());
        app.run("app.saveCopyDialog", json!({})).unwrap();
        assert_eq!(names.borrow().as_slice(), ["தமிழ் Brochure.pgnr", "தமிழ் Brochure copy.pgnr"]);
        let after = (app.session.doc().unwrap().path.clone(), app.session.doc().unwrap().is_dirty(), app.session.doc().unwrap().history.undo.len());
        assert_eq!(before, after);
        let copy = pagenor_engine::cmd::from_bytes(&std::fs::read(dir.join("தமிழ் Brochure copy.pgnr")).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(copy).unwrap(), serde_json::to_value(&app.session.doc().unwrap().doc).unwrap());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn inbox_opens_new_uppercase_and_legacy_native_documents() {
        let mut source = Session::new();
        source.execute("file.new", &json!({})).unwrap();
        source.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Imported தமிழ்"})).unwrap();
        let bytes = pagenor_engine::cmd::to_bytes(&source.doc().unwrap().doc).unwrap();
        let inbox = Inbox::default();
        for name in ["New.pgnr", "Upper.PGNR", "Legacy.pagenor"] {
            inbox.lock().unwrap().push(ImportedFile {
                request: ImportRequest { purpose: "drop".into(), target: None },
                name: name.into(),
                bytes: bytes.clone(),
            });
        }
        let mut app = DesignApp::new(Session::new(), Services { inbox: Some(inbox), ..Default::default() });
        let ctx = egui::Context::default();
        for _ in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
            output.textures_delta.clear();
        }
        assert_eq!(app.session.documents().len(), 3);
        for doc in app.session.documents() {
            assert_eq!(doc.doc.stories.values().next().unwrap().text, "Imported தமிழ்");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn desktop_drop_opens_pgnr_and_legacy_documents() {
        #[derive(Debug)]
        struct Drop(std::path::PathBuf);
        impl egui::DroppedFile for Drop {
            fn path(&self) -> &std::path::Path {
                &self.0
            }
            fn bytes(&self) -> Result<Vec<u8>, String> {
                std::fs::read(&self.0).map_err(|e| e.to_string())
            }
        }
        let dir = std::env::temp_dir().join(format!("pagenor-drop-extensions-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut source = Session::new();
        source.execute("file.new", &json!({"title": "Dropped"})).unwrap();
        let bytes = pagenor_engine::cmd::to_bytes(&source.doc().unwrap().doc).unwrap();
        let mut app = DesignApp::new(Session::new(), Services::default());
        let ctx = egui::Context::default();
        for ext in ["pgnr", "PGNR", "pagenor"] {
            let path = dir.join(format!("Doc.{ext}"));
            std::fs::write(&path, &bytes).unwrap();
            let raw = egui::RawInput { dropped_files: vec![std::sync::Arc::new(Drop(path.clone()))], ..Default::default() };
            let mut output = ctx.run_ui(raw, |ui| app.logic(ui.ctx()));
            output.textures_delta.clear();
            assert_eq!(app.session.doc().unwrap().path.as_deref(), path.to_str());
        }
        assert_eq!(app.session.documents().len(), 3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn launch_splash_appears_with_or_without_documents_then_finishes_once() {
        for has_document in [false, true] {
            let ctx = egui::Context::default();
            let mut app = DesignApp::new(Session::new(), Services::default()).with_startup_splash();
            if has_document {
                app.run("file.new", json!({})).unwrap();
                app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": "தமிழ் café"})).unwrap();
                app.session.doc_mut().unwrap().path = Some("/local/தமிழ்.pagenor".into());
            }
            let before: Vec<_> = app
                .session
                .documents()
                .iter()
                .map(|d| (serde_json::to_value(&d.doc).unwrap(), d.path.clone(), d.is_dirty(), d.history.undo.len()))
                .collect();
            // The first visible frame can arrive long after process start.
            for (time, visible) in [(20.0, true), (20.8, true), (21.21, false), (22.0, false)] {
                let raw = egui::RawInput {
                    time: Some(time),
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 560.0))),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(raw, |ui| {
                    app.logic(ui.ctx());
                    app.ui(ui);
                });
                assert_eq!(app.startup_splash.is_some(), visible);
                assert!(!output.shapes.is_empty());
                output.textures_delta.clear();
            }
            let after: Vec<_> = app
                .session
                .documents()
                .iter()
                .map(|d| (serde_json::to_value(&d.doc).unwrap(), d.path.clone(), d.is_dirty(), d.history.undo.len()))
                .collect();
            assert_eq!(before, after);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn launch_splash_preserves_recovered_document_and_shutdown_checkpoint() {
        let (mut old, _, dir) = recovery_app("startup-splash");
        old.run("file.new", json!({})).unwrap();
        old.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Recovered தமிழ் café"})).unwrap();
        old.session.doc_mut().unwrap().path = Some("/local/recovered.pagenor".into());
        old.prepare_shutdown().unwrap();
        let mut session = Session::new();
        session.recovery_dir = Some(dir.clone());
        session.execute("file.recovery.open", &json!({})).unwrap();
        let mut app = DesignApp::new(session, Services::default()).with_startup_splash();
        let before = serde_json::to_value(&app.session.doc().unwrap().doc).unwrap();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        output.textures_delta.clear();
        assert!(app.startup_splash.is_some());
        app.prepare_shutdown().unwrap();
        let mut recovered = Session::new();
        recovered.recovery_dir = Some(dir.clone());
        recovered.execute("file.recovery.open", &json!({})).unwrap();
        assert_eq!(serde_json::to_value(&recovered.doc().unwrap().doc).unwrap(), before);
        assert_eq!(recovered.doc().unwrap().path.as_deref(), Some("/local/recovered.pagenor"));
        assert!(recovered.doc().unwrap().is_dirty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_app(name: &str) -> (DesignApp, egui::Context, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("pagenor-ui-recovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut session = Session::new();
        session.recovery_dir = Some(dir.clone());
        (DesignApp::new(session, Services::default()), egui::Context::default(), dir)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn logic_at(app: &mut DesignApp, ctx: &egui::Context, time: f64) {
        let mut output = ctx.run_ui(egui::RawInput { time: Some(time), ..Default::default() }, |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn close_frame(app: &mut DesignApp, ctx: &egui::Context) -> egui::FullOutput {
        let mut raw = egui::RawInput { time: Some(1.0), ..Default::default() };
        raw.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().events.push(egui::ViewportEvent::Close);
        let mut output = ctx.run_ui(raw, |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
        output
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn window_close_checkpoints_edits_before_timer_deadline() {
        let (mut app, ctx, dir) = recovery_app("close-before-timer");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        let output = close_frame(&mut app, &ctx);
        assert!(!output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        let mut restarted = Session::new();
        assert_eq!(pagenor_engine::recovery::open(&mut restarted, &dir).unwrap().len(), 1);
        assert_eq!(restarted.doc().unwrap().doc.as_ref(), app.session.doc().unwrap().doc.as_ref());
        assert!(app.session.doc().unwrap().is_dirty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn window_close_is_cancelled_on_recovery_failure() {
        let (mut app, ctx, dir) = recovery_app("close-failure");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        let blocked = dir.join("blocked");
        std::fs::write(&blocked, b"fixture").unwrap();
        app.session.recovery_dir = Some(blocked);
        let output = close_frame(&mut app, &ctx);
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        assert_eq!(app.session.documents().len(), 1);
        assert!(app.session.doc().unwrap().is_dirty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shutdown_preserves_old_checkpoint_on_failure_then_retries_all_documents() {
        let (mut app, ctx, dir) = recovery_app("shutdown-retry");
        let path = dir.join("Original café.pagenor");
        app.run("file.new", json!({})).unwrap();
        app.run("file.saveAs", json!({"path": path})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        app.prepare_shutdown().unwrap();
        let uid = pagenor_engine::recovery::list(&dir)[0].0;
        let doc = dir.join(format!("{uid}.pagenor"));
        let meta = dir.join(format!("{uid}.json"));
        let before = std::fs::read(&doc).unwrap();
        app.run("frame.create", json!({"rect": [20, 20, 80, 80]})).unwrap();
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [30, 30, 70, 70]})).unwrap();
        std::fs::remove_file(&meta).unwrap();
        std::fs::create_dir(&meta).unwrap();
        let output = close_frame(&mut app, &ctx);
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(app.ui.status.ends_with("Pagenor will stay open."));
        assert_eq!(std::fs::read(&doc).unwrap(), before);
        assert_eq!(app.session.documents().len(), 2);
        std::fs::remove_dir(&meta).unwrap();
        let output = close_frame(&mut app, &ctx);
        assert!(!output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        assert_eq!(app.ui.status, "Recovery is up to date.");
        let mut restarted = Session::new();
        assert_eq!(pagenor_engine::recovery::open(&mut restarted, &dir).unwrap().len(), 2);
        for (actual, expected) in restarted.documents().iter().zip(app.session.documents()) {
            assert_eq!(actual.doc, expected.doc);
            assert_eq!(actual.path, expected.path);
            assert!(expected.is_dirty());
        }
        assert_eq!(app.session.documents()[0].path, Some(path.to_string_lossy().into_owned()));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shutdown_rejects_failed_clean_checkpoint_cleanup_and_retries() {
        let (mut app, ctx, dir) = recovery_app("shutdown-cleanup");
        app.run("file.new", json!({})).unwrap();
        app.run("file.saveAs", json!({"path": dir.join("saved.pagenor")})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        app.prepare_shutdown().unwrap();
        let uid = pagenor_engine::recovery::list(&dir)[0].0;
        let doc = dir.join(format!("{uid}.pagenor"));
        app.run("edit.undo", json!({})).unwrap();
        assert!(!app.session.doc().unwrap().is_dirty());
        std::fs::remove_file(&doc).unwrap();
        std::fs::create_dir(&doc).unwrap();
        let output = close_frame(&mut app, &ctx);
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(pagenor_engine::recovery::needs_save(&app.session));
        std::fs::remove_dir(&doc).unwrap();
        let output = close_frame(&mut app, &ctx);
        assert!(!output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(!pagenor_engine::recovery::needs_save(&app.session));
        assert!(pagenor_engine::recovery::list(&dir).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn quit_command_and_control_routes_report_failures_before_requesting_close() {
        let (mut app, ctx, dir) = recovery_app("quit-command");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        app.session.recovery_dir = None;
        assert!(control::all_commands(&app).as_array().unwrap().iter().any(|v| v["id"] == "app.quit"));
        for (method, params) in [
            ("app.quit", json!({})),
            ("ui.menu.invoke", json!({"command": "app.quit"})),
            ("engine.execute", json!({"command": "app.quit", "params": {}})),
        ] {
            let (req, _) = ControlRequest::new(method, params);
            let control::Outcome::Done(reply) = control::handle(&mut app, &ctx, &req) else { panic!("expected synchronous quit response") };
            assert_eq!(reply["ok"], false);
            assert!(!app.quit_requested);
        }
        app.session.recovery_dir = Some(dir.clone());
        let (req, _) = ControlRequest::new("app.quit", json!({}));
        let control::Outcome::Done(reply) = control::handle(&mut app, &ctx, &req) else { panic!("expected synchronous quit response") };
        assert_eq!(reply["ok"], true);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::Close));
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn close_guard_catches_edits_and_write_failures_after_logic() {
        let (mut app, ctx, dir) = recovery_app("late-close-edit");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        app.run("app.quit", json!({})).unwrap();
        // An edit can arrive after the quit command and before its viewport close event.
        app.run("frame.create", json!({"rect": [20, 20, 80, 80]})).unwrap();
        let mut raw = egui::RawInput { time: Some(1.0), ..Default::default() };
        raw.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().events.push(egui::ViewportEvent::Close);
        let mut output = ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            let uid = pagenor_engine::recovery::list(&dir)[0].0;
            let meta = dir.join(format!("{uid}.json"));
            std::fs::remove_file(&meta).unwrap();
            std::fs::create_dir(&meta).unwrap();
            app.run("frame.create", json!({"rect": [30, 30, 70, 70]})).unwrap();
            app.handle_close_request(ui.ctx());
        });
        output.textures_delta.clear();
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::CancelClose));
        let uid = pagenor_engine::recovery::list(&dir)[0].0;
        std::fs::remove_dir(dir.join(format!("{uid}.json"))).unwrap();
        close_frame(&mut app, &ctx);
        let mut restarted = Session::new();
        pagenor_engine::recovery::open(&mut restarted, &dir).unwrap();
        assert_eq!(restarted.doc().unwrap().doc, app.session.doc().unwrap().doc);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn secondary_window_close_does_not_checkpoint_or_cancel() {
        let (mut app, ctx, dir) = recovery_app("secondary-close");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        let secondary = egui::ViewportId::from_hash_of("secondary");
        let mut raw = egui::RawInput { viewport_id: secondary, time: Some(1.0), ..Default::default() };
        raw.viewports.insert(
            secondary,
            egui::ViewportInfo { parent: Some(egui::ViewportId::ROOT), events: vec![egui::ViewportEvent::Close], ..Default::default() },
        );
        let mut output = ctx.run_ui(raw, |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
        assert!(output.viewport_output.values().all(|v| !v.commands.contains(&egui::ViewportCommand::CancelClose)));
        assert!(pagenor_engine::recovery::list(&dir).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_removes_checkpoint_after_undo_to_saved_state() {
        let (mut app, ctx, dir) = recovery_app("undo-clean");
        app.run("file.new", json!({})).unwrap();
        app.run("file.saveAs", json!({"path": dir.join("saved.pagenor")})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        logic_at(&mut app, &ctx, 31.0);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1);
        app.run("edit.undo", json!({})).unwrap();
        assert!(!app.session.doc().unwrap().is_dirty());
        logic_at(&mut app, &ctx, 62.0);
        assert!(pagenor_engine::recovery::list(&dir).is_empty(), "clean undo must not resurrect an old edit after restart");
        let mut restarted = Session::new();
        assert!(pagenor_engine::recovery::open(&mut restarted, &dir).unwrap().is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_bounds_oversized_interval_from_preferences() {
        let (mut app, ctx, dir) = recovery_app("interval-overflow");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        app.run("prefs.set", json!({"recoveryMinutes": 1e308})).unwrap();
        logic_at(&mut app, &ctx, 3601.0);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1, "finite preferences must not overflow into a disabled timer");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_obeys_default_and_minimum_deadlines() {
        for (name, minutes, deadline) in [("default-deadline", 0.5, 30.0), ("minimum-deadline", 0.0, 5.0)] {
            let (mut app, ctx, dir) = recovery_app(name);
            app.session.prefs.recovery_minutes = minutes;
            app.run("file.new", json!({})).unwrap();
            logic_at(&mut app, &ctx, deadline + 1.0);
            assert!(pagenor_engine::recovery::list(&dir).is_empty(), "clean documents need no snapshot");
            app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
            // Use a fresh clock origin after the clean frame above.
            app.last_recovery = deadline + 1.0;
            logic_at(&mut app, &ctx, 2.0 * deadline + 0.9);
            assert!(pagenor_engine::recovery::list(&dir).is_empty(), "not before the deadline");
            logic_at(&mut app, &ctx, 2.0 * deadline + 1.0);
            assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1, "save exactly at the deadline");
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_reports_failure_preserves_checkpoint_and_retries_on_schedule() {
        let (mut app, ctx, dir) = recovery_app("retry");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        logic_at(&mut app, &ctx, 30.0);
        let uid = pagenor_engine::recovery::list(&dir)[0].0;
        let doc = dir.join(format!("{uid}.pagenor"));
        let meta = dir.join(format!("{uid}.json"));
        let previous = std::fs::read(&doc).unwrap();
        let metadata = std::fs::read(&meta).unwrap();
        app.run("frame.create", json!({"rect": [0, 0, 20, 20]})).unwrap();
        std::fs::remove_file(&meta).unwrap();
        std::fs::create_dir(&meta).unwrap();
        logic_at(&mut app, &ctx, 60.0);
        assert!(app.ui.status.starts_with("Couldn't write recovery data:"));
        assert_eq!(std::fs::read(&doc).unwrap(), previous);
        assert!(app.session.doc().unwrap().is_dirty());
        std::fs::remove_dir(&meta).unwrap();
        std::fs::write(&meta, metadata).unwrap();
        logic_at(&mut app, &ctx, 60.1);
        assert_eq!(std::fs::read(&doc).unwrap(), previous, "no busy-loop retry immediately after a failure");
        assert_eq!(app.last_recovery, 60.0);
        logic_at(&mut app, &ctx, 90.0);
        assert_eq!(app.ui.status, "Recovery is up to date.");
        assert_eq!(pagenor_engine::cmd::from_bytes(&std::fs::read(doc).unwrap()).unwrap(), *app.session.doc().unwrap().doc);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1, "retry retains the owned key");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_handles_nonfinite_preferences_and_reset_clocks() {
        for (name, minutes) in [("nan-interval", f64::NAN), ("infinite-interval", f64::INFINITY), ("negative-infinite-interval", f64::NEG_INFINITY)] {
            let (mut app, ctx, dir) = recovery_app(name);
            app.session.prefs.recovery_minutes = minutes;
            app.run("file.new", json!({})).unwrap();
            app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
            app.last_recovery = f64::NAN;
            app.recovery_tick(&ctx, 2.0);
            assert_eq!(app.last_recovery, 2.0);
            app.last_recovery = 100.0;
            app.recovery_tick(&ctx, 3.0);
            assert_eq!(app.last_recovery, 3.0, "reset clocks cannot indefinitely postpone recovery");
            app.recovery_tick(&ctx, f64::INFINITY);
            assert_eq!(app.last_recovery, 3.0);
            app.recovery_tick(&ctx, 32.9);
            assert!(pagenor_engine::recovery::list(&dir).is_empty());
            app.recovery_tick(&ctx, 33.0);
            assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1, "nonfinite intervals fall back to 30 seconds");
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_handles_multiple_documents_and_preserves_unclaimed_files() {
        let (mut app, ctx, dir) = recovery_app("multiple");
        for _ in 0..2 {
            app.run("file.new", json!({})).unwrap();
            app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        }
        let unknown = dir.join(format!("{}.pagenor", u64::MAX));
        std::fs::write(&unknown, b"pending unreadable entry").unwrap();
        logic_at(&mut app, &ctx, 30.0);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 3);
        app.session.set_active(0);
        app.run("edit.undo", json!({})).unwrap();
        app.run("file.new", json!({})).unwrap(); // Active document is clean; inactive dirty document still needs recovery.
        logic_at(&mut app, &ctx, 60.0);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 2);
        assert_eq!(std::fs::read(&unknown).unwrap(), b"pending unreadable entry");
        app.session.set_active(1);
        app.run("edit.undo", json!({})).unwrap();
        logic_at(&mut app, &ctx, 90.0);
        assert_eq!(pagenor_engine::recovery::list(&dir).len(), 1);
        assert!(!pagenor_engine::recovery::needs_save(&app.session), "unclaimed files do not schedule clean-session recovery");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn recovery_timer_schedules_idle_wakeup_and_leaves_disabled_recovery_alone() {
        let (mut app, ctx, dir) = recovery_app("wakeup");
        app.run("file.new", json!({})).unwrap();
        app.run("frame.create", json!({"rect": [10, 10, 100, 100]})).unwrap();
        // Settle egui's initial repaints; do not run the font or caret animation logic.
        for _ in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
        }
        let mut output = ctx.run_ui(egui::RawInput { time: Some(1.0), ..Default::default() }, |ui| app.recovery_tick(ui.ctx(), 1.0));
        let delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        output.textures_delta.clear();
        assert!(delay.as_secs_f64() > 28.0 && delay.as_secs_f64() <= 29.0, "recovery itself schedules the next frame");
        app.session.recovery_dir = None;
        app.recovery_tick(&ctx, 100.0);
        assert_eq!(app.last_recovery, 0.0);
        assert!(pagenor_engine::recovery::list(&dir).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn host_clipboard_paste_is_one_reversible_edit_and_explicit_text_bypasses_the_host() {
        for command in ["edit.paste", "edit.pasteWithoutFormatting"] {
            let host = "தமிழ் 👩‍💻\r\nاردو";
            let services = Services { clipboard_text: Some(Box::new(move || Some(host.into()))), ..Default::default() };
            let mut app = DesignApp::new(Session::new(), services);
            app.run("file.new", json!({})).unwrap();
            let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": "A界B"})).unwrap();
            let story = pagenor_doc::StoryId(frame["story"].as_u64().unwrap());
            app.run("text.select", json!({"story": story.0, "anchor": 1, "focus": 4})).unwrap();
            let selection = app.session.doc().unwrap().selection.clone();
            let undo = app.session.doc().unwrap().history.undo.len();
            app.run(command, json!({})).unwrap();
            let text = format!("A{}B", host.replace("\r\n", "\n"));
            assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, text);
            assert_eq!(app.session.doc().unwrap().history.undo.len(), undo + 1);
            app.run("edit.undo", json!({})).unwrap();
            assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, "A界B");
            assert_eq!(app.session.doc().unwrap().selection, selection);
            app.run("edit.redo", json!({})).unwrap();
            assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, text);
            app.run("edit.undo", json!({})).unwrap();
            app.services.clipboard_text = Some(Box::new(|| panic!("explicit paste text must not read the host")));
            app.run(command, json!({"text": "Explicit हिन्दी"})).unwrap();
            assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, "AExplicit हिन्दीB");
        }
    }

    #[test]
    fn menu_and_control_copy_cut_emit_unicode_to_the_platform_clipboard() {
        for command in ["edit.copy", "edit.cut"] {
            for control in [false, true] {
                let mut app = DesignApp::new(Session::new(), Services::default());
                let ctx = egui::Context::default();
                app.run("file.new", json!({})).unwrap();
                let text = "café e\u{301} தமிழ் తెలుగు 👩🏽‍💻\nSecond line";
                let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": text})).unwrap();
                let story = pagenor_doc::StoryId(frame["story"].as_u64().unwrap());
                app.run("text.select", json!({"story": story.0, "anchor": 0, "focus": text.len()})).unwrap();
                let before = app.session.doc().unwrap().doc.clone();
                let selection = app.session.doc().unwrap().selection.clone();
                let undo = app.session.doc().unwrap().history.undo.len();
                let journal = app.session.journal.len();
                if control {
                    let (req, _) = ControlRequest::new("engine.execute", json!({"command": command, "params": {}}));
                    let control::Outcome::Done(reply) = control::handle(&mut app, &ctx, &req) else { panic!("expected command result") };
                    assert_eq!(reply["ok"], true);
                } else {
                    menus::activate(&mut app, command, &Value::Null);
                }
                let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
                output.textures_delta.clear();
                let copied: Vec<_> = output
                    .platform_output
                    .commands
                    .iter()
                    .filter_map(|c| match c {
                        egui::OutputCommand::CopyText(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(copied, [text], "{command}, control={control}: all command routes must reach the host clipboard once");
                if command == "edit.copy" {
                    assert_eq!(app.session.doc().unwrap().doc, before);
                    assert_eq!(app.session.doc().unwrap().history.undo.len(), undo);
                } else {
                    assert_eq!(app.session.doc().unwrap().history.undo.len(), undo + 1);
                    assert_eq!(app.session.journal.len(), journal + 1);
                    assert!(app.session.doc().unwrap().doc.story(story).unwrap().text.is_empty());
                    app.run("edit.undo", json!({})).unwrap();
                    assert_eq!(app.session.doc().unwrap().doc, before);
                    assert_eq!(app.session.doc().unwrap().selection, selection);
                }
                let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
                output.textures_delta.clear();
                assert!(
                    !output.platform_output.commands.iter().any(|c| matches!(c, egui::OutputCommand::CopyText(_))),
                    "do not rewrite the clipboard every frame"
                );
            }
        }
    }

    #[test]
    fn canvas_copy_cut_emit_once_through_the_shared_clipboard_bridge() {
        for event in [egui::Event::Copy, egui::Event::Cut] {
            let ctx = egui::Context::default();
            let mut app = DesignApp::new(Session::new(), Services::default());
            app.run("file.new", json!({})).unwrap();
            let text = "Canvas café தமிழ்";
            let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": text})).unwrap();
            app.run("text.select", json!({"story": frame["story"], "anchor": 0, "focus": text.len()})).unwrap();
            app.session.set_tool("type");
            let undo = app.session.doc().unwrap().history.undo.len();
            for events in [vec![], vec![], vec![event.clone()]] {
                let raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 750.0))),
                    events,
                    ..Default::default()
                };
                let has_event = !raw.events.is_empty();
                let mut output = ctx.run_ui(raw, |ui| {
                    app.logic(ui.ctx());
                    app.ui(ui);
                });
                output.textures_delta.clear();
                let copied: Vec<_> = output
                    .platform_output
                    .commands
                    .iter()
                    .filter_map(|c| match c {
                        egui::OutputCommand::CopyText(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();
                if has_event {
                    assert_eq!(copied, [text]);
                } else {
                    assert!(copied.is_empty());
                }
            }
            assert_eq!(app.session.doc().unwrap().history.undo.len(), undo + usize::from(matches!(event, egui::Event::Cut)));
        }
    }

    #[test]
    fn queued_copy_is_used_for_same_batch_paste_then_external_clipboard_is_read() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(Session::new(), Services::default());
        app.run("file.new", json!({})).unwrap();
        let text = "தமிழ்";
        let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": text})).unwrap();
        let story = pagenor_doc::StoryId(frame["story"].as_u64().unwrap());
        app.run("text.select", json!({"story": story.0, "anchor": 0, "focus": text.len()})).unwrap();
        app.run("edit.copy", json!({})).unwrap();
        app.run("text.select", json!({"story": story.0, "anchor": text.len(), "focus": text.len()})).unwrap();
        app.services.clipboard_text = Some(Box::new(|| panic!("same-batch copy must precede stale OS clipboard text")));
        app.run("edit.paste", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, format!("{text}{text}"));
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
        assert_eq!(output.platform_output.commands.iter().filter(|c| matches!(c, egui::OutputCommand::CopyText(_))).count(), 1);
        app.services.clipboard_text = Some(Box::new(|| Some(" External café".into())));
        app.run("edit.pasteWithoutFormatting", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc.story(story).unwrap().text, format!("{text}{text} External café"));
    }

    #[test]
    fn object_and_failed_copy_cut_do_not_replace_the_platform_clipboard() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(Session::new(), Services::default());
        app.run("file.new", json!({})).unwrap();
        assert!(app.run("edit.copy", json!({})).is_err());
        assert!(app.run("edit.cut", json!({})).is_err());
        let frame = app.run("frame.create", json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Text"})).unwrap();
        app.run("text.select", json!({"story": frame["story"], "anchor": 0, "focus": 4})).unwrap();
        app.run("edit.copy", json!({})).unwrap();
        // A newer object copy supersedes the queued text copy but stays internal.
        app.run("frame.create", json!({"rect": [20, 20, 60, 60]})).unwrap();
        app.run("edit.copy", json!({})).unwrap();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx()));
        output.textures_delta.clear();
        assert!(!output.platform_output.commands.iter().any(|c| matches!(c, egui::OutputCommand::CopyText(_))));
        app.run("edit.paste", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc.spreads.iter().map(|s| s.items.len()).sum::<usize>(), 3);
    }

    /// The Control panel's two-row groups were centred as if they were one row tall, so the second
    /// row hung below the panel and was cut off.
    #[test]
    fn control_bar_shows_its_second_row() {
        use egui_kittest::kittest::Queryable as _;
        let mut app = DesignApp::new(pagenor_engine::Session::new(), Services::default());
        app.run("file.newSample", json!({})).unwrap();
        app.ui.control_bar = true;
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut DesignApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let bar = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("control_bar")).unwrap().outer_rect;
        assert!(bar.height() > 40.0, "{bar:?}");
        let rows: Vec<egui::Rect> = h.query_all_by_label("Y:").map(|n| n.rect()).filter(|r| bar.contains(r.left_top())).collect();
        assert_eq!(rows.len(), 1, "the Y: caption of the Control panel");
        assert!(rows[0].bottom() <= bar.bottom(), "Y: caption {:?} hangs below the Control panel {bar:?}", rows[0]);
    }

    #[test]
    fn key_names_sent_as_text_are_dropped() {
        let text = |t: &str| egui::Event::Text(t.to_string());
        let events = || {
            vec![
                text("AltGraph"),
                text("é"),
                text("Dead"),
                text("A"),
                text("Unidentified"),
                text("ß"),
                text("AudioVolumeUp"),
                text("ab"),
                text("Æ"),
                text("Hello"),
                text("Copy"),
                text("Print"),
                text("தமிழ்"),
            ]
        };
        let mut raw = egui::RawInput { events: events(), ..Default::default() };
        drop_key_name_text(&mut raw, false);
        assert_eq!(
            raw.events,
            vec![text("é"), text("A"), text("ß"), text("ab"), text("Æ"), text("Hello"), text("Copy"), text("Print"), text("தமிழ்")]
        );
        // A focused text field gets its text from input events, which can be whole words.
        let mut raw = egui::RawInput { events: events(), ..Default::default() };
        drop_key_name_text(&mut raw, true);
        assert_eq!(raw.events, events());
    }

    #[test]
    fn snap_view_uses_the_saved_switches() {
        let mut ui = UiState::default();
        assert!(ui.snap_view().snap_to_guides);
        assert!(!ui.snap_view().snap_to_document_grid);
        assert_eq!(ui.snap_view().zone_px, 4.0);
        ui.snap_to_guides = false;
        ui.snap_zone = 0.0;
        assert!(!ui.snap_view().snap_to_guides);
        assert_eq!(ui.snap_view().zone_px, 0.0);
    }

    /// A temporary file named `name` whose bytes aren't a document or a graphic.
    fn unreadable_file(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("pagenor-ui-{}-{name}", std::process::id()));
        std::fs::write(&path, b"neither a document nor a graphic").unwrap();
        path
    }

    /// The open alert dialog: (title, message).
    fn alert(app: &DesignApp) -> (String, String) {
        let d = app.ui.dialog.as_ref().expect("an alert is open");
        assert_eq!(d.id, "alert");
        (d.fields["title"].as_str().unwrap().to_string(), d.fields["message"].as_str().unwrap().to_string())
    }

    #[test]
    fn dropped_documents_open_and_the_rest_is_placed() {
        for name in ["a.pagenor", "B.IDML"] {
            assert!(opens_as_document(name), "{name}");
        }
        for name in ["a.png", "b.pdf", "idml.txt", "c.idml.zip"] {
            assert!(!opens_as_document(name), "{name}");
        }
    }

    #[test]
    fn dropping_an_unreadable_file_alerts() {
        // A document opens, so it says it can't be opened.
        let path = unreadable_file("broken.pagenor");
        let mut app = DesignApp::new(Session::new(), Services::default());
        let r = app.open_dropped(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let e = r.unwrap_err();
        let (title, message) = alert(&app);
        assert_eq!(title, "Can't Open the File", "a document opens, it isn't placed");
        assert_eq!(message, e);
        assert!(app.ui.status.contains(&e), "the status line says it too");
        assert!(app.session.documents().is_empty());
        dialogs::confirm(&mut app).unwrap();
        assert!(app.ui.dialog.is_none(), "OK closes the alert");
        // Anything else is placed, so it says it can't be placed.
        app.run("file.new", json!({})).unwrap();
        let path = unreadable_file("notes.xyz");
        let r = app.open_dropped(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let e = r.unwrap_err();
        assert_eq!(alert(&app), ("Can't Place the File".to_string(), e));
    }

    #[test]
    fn file_open_failures_show_an_alert() {
        let path = unreadable_file("open.pagenor");
        let picked = path.to_string_lossy().to_string();
        let services = Services { pick_open: Some(Box::new(move |_| Some(picked.clone()))), ..Default::default() };
        let mut app = DesignApp::new(Session::new(), services);
        let e = app.run("app.openDialog", json!({})).unwrap_err();
        assert_eq!(alert(&app), ("Can't Open the File".to_string(), e));
        // Placing it says so too.
        app.ui.dialog = None;
        app.run("file.new", json!({})).unwrap();
        let e = app.run("app.placeDialog", json!({})).unwrap_err();
        let _ = std::fs::remove_file(&path);
        assert_eq!(alert(&app), ("Can't Place the File".to_string(), e));
    }

    #[test]
    fn other_command_errors_stay_in_the_status_line() {
        let mut app = DesignApp::new(Session::new(), Services::default());
        assert!(app.run("file.open", json!({"path": "/nonexistent/x.pagenor"})).is_err());
        assert!(app.ui.dialog.is_none(), "a scripted file.open doesn't raise an alert");
        assert!(!app.ui.status.is_empty());
    }
}

/// The whole window in a headless test harness, at a chosen window size.
#[cfg(test)]
pub(crate) mod test_window {
    use egui_kittest::Harness;

    pub struct Window {
        pub app: crate::DesignApp,
        ready: bool,
    }

    /// The window at `size` with a new document open.
    pub fn open(mut app: crate::DesignApp, size: egui::Vec2) -> Harness<'static, Window> {
        if app.session.active().is_none() {
            app.run("file.new", serde_json::json!({})).unwrap();
        }
        let mut h = Harness::builder().with_size(size).with_max_steps(1000).build_ui_state(
            |ui, w: &mut Window| {
                // The builder runs a frame before the texture size can be raised: skip it.
                if !w.ready {
                    return;
                }
                let ctx = ui.ctx().clone();
                w.app.logic(&ctx);
                w.app.ui(ui);
            },
            Window { app, ready: false },
        );
        h.input_mut().max_texture_side = Some(8192);
        h.state_mut().ready = true;
        h.run_steps(6);
        h
    }

    /// A primary click at `pos`, then a few frames.
    pub fn click_at(h: &mut Harness<'static, Window>, pos: egui::Pos2) {
        h.hover_at(pos);
        h.drag_at(pos);
        h.drop_at(pos);
        h.run_steps(4);
    }
    /// The mouse wheel turned over `pos`: positive `delta` moves the content right and down.
    pub fn wheel(h: &mut Harness<'static, Window>, pos: egui::Pos2, delta: egui::Vec2) {
        h.hover_at(pos);
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta,
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(10);
    }

    /// The outer rect of a side or top panel, from egui's memory.
    pub fn panel_rect(h: &Harness<'static, Window>, id: &str) -> egui::Rect {
        egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new(id)).map(|s| s.outer_rect).unwrap()
    }
}

#[cfg(test)]
mod browser_import_tests {
    use super::*;

    fn app() -> DesignApp {
        let mut app = DesignApp::new(Session::new(), Services { inbox: Some(Inbox::default()), ..Default::default() });
        app.run("file.new", json!({"title": "A"})).unwrap();
        app
    }
    fn deliver(app: &mut DesignApp, request: ImportRequest, name: &str, bytes: Vec<u8>) {
        app.services.inbox.as_ref().unwrap().lock().unwrap().push(ImportedFile { request, name: name.into(), bytes });
        app.drain_inbox();
    }
    fn image() -> Vec<u8> {
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#ff0000"/></svg>"##.to_vec()
    }
    fn pdf_pages(count: usize) -> Vec<u8> {
        let kids = (0..count).map(|i| format!("{} 0 R", 3 + i * 2)).collect::<Vec<_>>().join(" ");
        let mut objects = vec!["<< /Type /Catalog /Pages 2 0 R >>".to_string(), format!("<< /Type /Pages /Kids [{kids}] /Count {count} >>")];
        for (i, color) in ["1 0 0", "0 1 0", "0 0 1"].iter().take(count).enumerate() {
            let content = format!("{color} rg 0 0 200 200 re f\n");
            objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << >> /Contents {} 0 R >>", 4 + i * 2));
            objects.push(format!("<< /Length {} >>\nstream\n{content}endstream", content.len()));
        }
        let mut result = "%PDF-1.4\n".to_string();
        let mut offsets = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            offsets.push(result.len());
            result.push_str(&format!("{} 0 obj\n{object}\nendobj\n", i + 1));
        }
        let xref = result.len();
        result.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1));
        for offset in offsets {
            result.push_str(&format!("{offset:010} 00000 n \n"));
        }
        result.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1));
        result.into_bytes()
    }
    fn pdf() -> Vec<u8> {
        pdf_pages(3)
    }

    #[test]
    fn picker_receives_context_before_returning_and_cancel_does_not_edit() {
        let captured = std::sync::Arc::new(std::sync::Mutex::new(None));
        let copy = captured.clone();
        let mut app = app();
        app.services.open_async = Some(Box::new(move |request| *copy.lock().unwrap() = Some(request)));
        let uid = app.session.doc().unwrap().uid;
        app.pick_and_open("place").unwrap();
        app.run("file.new", json!({"title": "B"})).unwrap();
        let request = captured.lock().unwrap().take().unwrap();
        assert_eq!(request.purpose, "place");
        assert_eq!(request.target, Some(uid));
        assert!(app.session.documents().iter().all(|d| d.doc.assets.is_empty()));
        deliver(&mut app, request, "broken.pdf", b"not a PDF".to_vec());
        assert!(app.session.documents().iter().all(|d| d.doc.assets.is_empty()));
        assert!(app.pending_pdf.is_none());
    }

    #[test]
    fn delayed_place_activates_original_document_and_undo_belongs_to_it() {
        let mut app = app();
        let request = app.import_request("place");
        app.run("file.new", json!({"title": "B"})).unwrap();
        deliver(&mut app, request, "red.svg", image());
        assert_eq!(app.session.active_index(), Some(0));
        assert_eq!(app.session.documents()[0].doc.assets.len(), 1);
        assert!(app.session.documents()[1].doc.assets.is_empty());
        app.run("edit.undo", json!({})).unwrap();
        assert!(app.session.doc().unwrap().doc.assets.is_empty());
        app.run("edit.redo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc.assets.len(), 1);
    }

    #[test]
    fn closed_target_is_not_replaced_by_a_reused_tab_index() {
        let mut app = app();
        let request = app.import_request("place");
        app.run("file.close", json!({})).unwrap();
        app.run("file.new", json!({"title": "replacement"})).unwrap();
        deliver(&mut app, request, "red.svg", image());
        assert!(app.session.doc().unwrap().doc.assets.is_empty());
    }

    #[test]
    fn closing_earlier_tab_does_not_shift_import_target() {
        let mut app = app();
        app.run("file.new", json!({"title": "target"})).unwrap();
        let request = app.import_request("place");
        app.run("file.new", json!({"title": "other"})).unwrap();
        app.run("file.close", json!({"index": 0})).unwrap();
        deliver(&mut app, request, "red.svg", image());
        assert_eq!(app.session.doc().unwrap().doc.title, "target");
        assert_eq!(app.session.documents()[0].doc.assets.len(), 1);
        assert!(app.session.documents()[1].doc.assets.is_empty());
    }

    #[test]
    fn layout_place_and_open_preserve_explicit_intent() {
        let mut source = Session::new();
        source.execute("file.new", &json!({"title": "source"})).unwrap();
        source.execute("frame.create", &json!({"rect": [10, 10, 50, 50], "content": "text", "text": "placed"})).unwrap();
        let native = pagenor_engine::cmd::to_bytes(&source.doc().unwrap().doc).unwrap();
        let idml = pagenor_idml::export_idml(&source.doc().unwrap().doc);
        for (name, bytes) in [("source.pagenor", native), ("source.idml", idml)] {
            let mut app = app();
            let request = app.import_request("place");
            deliver(&mut app, request, name, bytes.clone());
            assert_eq!(app.session.documents().len(), 1);
            assert_eq!(app.session.doc().unwrap().doc.title, "A");
            assert!(app.session.doc().unwrap().doc.stories.values().any(|s| s.text == "placed"));
            app.run("edit.undo", json!({})).unwrap();
            assert!(app.session.doc().unwrap().doc.stories.is_empty());
            app.run("edit.redo", json!({})).unwrap();
            assert!(!app.session.doc().unwrap().doc.stories.is_empty());
            let request = app.import_request("open");
            deliver(&mut app, request, name, bytes);
            assert_eq!(app.session.documents().len(), 2);
        }
    }

    #[test]
    fn pdf_options_keep_bytes_target_and_selected_page() {
        let mut app = app();
        let bytes = pdf();
        let request = app.import_request("place");
        deliver(&mut app, request, "three.pdf", bytes.clone());
        assert!(app.session.doc().unwrap().doc.assets.is_empty());
        assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfImport");
        app.run("file.new", json!({"title": "B"})).unwrap();
        app.ui.dialog.as_mut().unwrap().fields.insert("page".into(), json!("2"));
        dialogs::confirm(&mut app).unwrap();
        assert_eq!(app.session.active_index(), Some(0));
        let asset = app.session.doc().unwrap().doc.assets.values().next().unwrap();
        assert_eq!(asset.page, 1);
        assert_eq!(*asset.data, bytes);
        assert!(app.pending_pdf.is_none());
        assert!(app.session.documents()[1].doc.assets.is_empty());
        let exported = app.run("file.exportPdf", json!({})).unwrap();
        let exported = pagenor_engine::cmd::base64_decode(exported["base64"].as_str().unwrap());
        let pixels = pagenor_render::render_pdf_page(&exported, 0, 400).unwrap();
        assert!(pixels.data().iter().filter(|p| p.g > 240 && p.r < 10 && p.b < 10).count() > 1000);
        assert_eq!(pixels.data().iter().filter(|p| p.r > 240 && p.g < 10 && p.b < 10).count(), 0);
    }

    #[test]
    fn pdf_options_reject_oversized_page_without_editing() {
        let mut app = app();
        for page in [0_u64, 4, 4_294_967_296, 4_294_967_297, u64::MAX] {
            let request = app.import_request("place");
            deliver(&mut app, request, "three.pdf", pdf());
            app.ui.dialog.as_mut().unwrap().fields.insert("page".into(), json!(page.to_string()));
            // The native dialog clamps zero to one; positive invalid pages must fail.
            if page == 0 {
                assert!(app.confirm_pdf_import(page, "crop".into()).is_err());
            } else {
                assert!(dialogs::confirm(&mut app).is_err());
            }
            app.ui.dialog = None;
            assert!(app.session.doc().unwrap().doc.assets.is_empty());
            assert!(app.pending_pdf.is_none());
        }
        let request = app.import_request("place");
        deliver(&mut app, request, "three.pdf", pdf());
        app.ui.dialog.as_mut().unwrap().fields.insert("page".into(), json!("2"));
        dialogs::confirm(&mut app).unwrap();
        assert_eq!(app.session.doc().unwrap().doc.assets.values().next().unwrap().page, 1);
    }

    #[test]
    fn drop_routing_and_nonlayout_open_remain_compatible() {
        let mut app = app();
        let native = pagenor_engine::cmd::to_bytes(&app.session.doc().unwrap().doc).unwrap();
        let request = app.import_request("drop");
        deliver(&mut app, request, "document.pagenor", native);
        assert_eq!(app.session.documents().len(), 2);
        let request = app.import_request("drop");
        app.run("file.activate", json!({"index": 0})).unwrap();
        deliver(&mut app, request, "red.svg", image());
        assert_eq!(app.session.active_index(), Some(1));
        assert_eq!(app.session.doc().unwrap().doc.assets.len(), 1);
        let request = app.import_request("open");
        deliver(&mut app, request, "single.pdf", pdf_pages(1));
        assert_eq!(app.session.documents().len(), 2);
        assert!(app.pending_pdf.is_none());
        assert!(app.ui.dialog.is_none());
    }

    #[test]
    fn ase_swatches_keep_purpose_and_target() {
        let mut source = Session::new();
        source.execute("file.new", &json!({})).unwrap();
        source.execute("swatch.create", &json!({"name": "Import Teal", "color": "#108080"})).unwrap();
        let data = source.execute("swatch.save", &json!({"names": ["Import Teal"]})).unwrap();
        let bytes = pagenor_engine::cmd::base64_decode(data["base64"].as_str().unwrap());
        for (purpose, name) in [("swatches", "picker-file"), ("drop", "palette.ase")] {
            let mut app = app();
            let request = app.import_request(purpose);
            app.run("file.new", json!({"title": "B"})).unwrap();
            deliver(&mut app, request, name, bytes.clone());
            assert_eq!(app.session.active_index(), Some(0));
            assert!(app.session.documents()[0].doc.swatches.iter().any(|s| s.name == "Import Teal"));
            assert!(!app.session.documents()[1].doc.swatches.iter().any(|s| s.name == "Import Teal"));
        }
    }

    #[test]
    fn native_pdf_dialog_keeps_path_and_uid_at_confirmation() {
        let mut app = app();
        let path = std::env::temp_dir().join(format!("pagenor-native-import-{}-{}.pdf", std::process::id(), app.session.doc().unwrap().uid));
        std::fs::write(&path, pdf()).unwrap();
        let pick_path = path.to_string_lossy().to_string();
        app.services.pick_open = Some(Box::new(move |_| Some(pick_path.clone())));
        app.services.read = Some(Box::new(|path| std::fs::read(path).map_err(|e| e.to_string())));
        app.pick_and_open("place").unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfImport");
        app.run("file.new", json!({"title": "B"})).unwrap();
        app.ui.dialog.as_mut().unwrap().fields.insert("page".into(), json!("2"));
        dialogs::confirm(&mut app).unwrap();
        assert_eq!(app.session.active_index(), Some(0));
        assert_eq!(app.session.doc().unwrap().doc.assets.values().next().unwrap().page, 1);
        assert!(app.session.documents()[1].doc.assets.is_empty());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pending_pdf_cancel_close_supersession_and_second_delivery_release_bytes() {
        let mut app = app();
        let request = app.import_request("place");
        deliver(&mut app, request.clone(), "first.pdf", pdf());
        deliver(&mut app, request.clone(), "second.svg", image());
        assert_eq!(app.pending_pdf.as_ref().unwrap().name, "first.pdf");
        assert!(app.session.doc().unwrap().doc.assets.is_empty());
        app.ui.dialog = None;
        app.drain_inbox();
        assert!(app.pending_pdf.is_none());
        deliver(&mut app, request.clone(), "first.pdf", pdf());
        app.ui.dialog = Some(dialogs::Dialog::new("newDocument", json!({})));
        app.drain_inbox();
        assert!(app.pending_pdf.is_none());
        app.ui.dialog = None;
        deliver(&mut app, request, "first.pdf", pdf());
        app.run("file.close", json!({})).unwrap();
        app.run("file.new", json!({})).unwrap();
        assert!(dialogs::confirm(&mut app).is_err());
        assert!(app.pending_pdf.is_none());
        assert!(app.session.doc().unwrap().doc.assets.is_empty());
    }
}
