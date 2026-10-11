//! Pagenor desktop app. See PAGENOR_UPSTREAM.md for fork provenance.
//!
//! Usage: `pagenor [--control <port>] [--sample] [files…]`
//!
//! `--control <port>` explicitly starts an authenticated localhost JSON-lines control server:
//! `{"id":1,"token":"…","method":"ui.inspect","params":{}}` → `{"id":1,"ok":true,"result":…}`.
//! See `pagenor_ui_egui::control` for the methods.
//!
//! The graphics backend (DirectX 12, Vulkan, Metal or OpenGL) and the adapter that draws the
//! window are chosen in `gpu`, with a fallback for a driver or adapter that fails at start-up;
//! `WGPU_BACKEND`, `WGPU_POWER_PREF` and `WGPU_ADAPTER_NAME` override them. Why the app stopped
//! is logged (`logging`), also when the window never opened.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod appearance;
mod control_server;
mod gpu;
#[cfg(any(target_os = "windows", test))]
mod graphics;
mod logging;
#[cfg(target_os = "macos")]
mod native_input;
#[cfg(target_os = "macos")]
mod native_menu;
mod profile;

use egui_winit::winit::raw_window_handle::HasDisplayHandle;
use pagenor_engine::Session;
use pagenor_ui_egui::{DesignApp, PasteRequest, Services};

struct NativeClipboard {
    adapter: std::sync::Mutex<egui_winit::clipboard::Clipboard>,
    // The output plugin shares this owner; the adapter must drop before the window.
    _window: Option<std::sync::Arc<egui_winit::winit::window::Window>>,
}

// Copy at the backend handoff, after selectable labels emit their late output.
// Mirroring in a later frame could overwrite a newer external clipboard value.
struct NativeClipboardOutput<F>(F);

impl<F: FnMut(&str) + Send + Sync + 'static> egui::Plugin for NativeClipboardOutput<F> {
    fn debug_name(&self) -> &'static str {
        "Native clipboard output"
    }

    fn output_hook(&mut self, _ctx: &egui::Context, output: &mut egui::FullOutput) {
        if let Some(text) = output.platform_output.commands.iter().rev().find_map(|command| match command {
            egui::OutputCommand::CopyText(text) => Some(text),
            _ => None,
        }) {
            (self.0)(text);
        }
    }
}

struct App {
    app: DesignApp,
    clipboard: std::sync::Arc<NativeClipboard>,
    paste_requests: std::sync::mpsc::Receiver<PasteRequest>,
    paste_completions: Vec<(PasteRequest, String)>,
    #[cfg(target_os = "macos")]
    menu: Option<native_menu::NativeMenu>,
    /// The window's start: the adapter and backend chosen for it; see `gpu`.
    startup: std::sync::Arc<gpu::Startup>,
    /// Frames begun so far.
    frames: u32,
    #[cfg(target_os = "macos")]
    shutdown_windows: native_input::ShutdownWindows,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.startup.enter();
        if let Some(note) = self.startup.take_notice() {
            self.app.status(note);
        }
        // The third call: two frames went through `ui`, so the first was presented and the driver
        // survived it (a fault at the first present is what the fallback in `gpu` is for).
        if self.frames == 2 {
            self.startup.presented();
        }
        self.frames = self.frames.saturating_add(1);
        #[cfg(target_os = "macos")]
        {
            if self.menu.is_none() && std::env::var_os("PAGENOR_NO_NATIVE_MENU").is_none() {
                self.menu = Some(native_menu::NativeMenu::install(&mut self.app));
            }
            if let Some(m) = &mut self.menu {
                m.poll(&mut self.app, ctx);
            }
        }
        self.app.logic(ctx);
        self.startup.leave();
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        // Read once before the UI passes, after the previous frame's complete
        // clipboard output. Keyboard Paste remains ordinary input.
        while let Ok(request) = self.paste_requests.try_recv() {
            let text = self.clipboard.adapter.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get();
            if let Some(text) = text.filter(|text| !text.is_empty()) {
                self.paste_completions.push((request, text.replace("\r\n", "\n")));
            }
        }
        self.app.raw_input_hook(raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.startup.enter();
        self.app.ui(ui);
        // Process current keyboard/canvas input before checking the captured target.
        for (request, text) in self.paste_completions.drain(..) {
            self.app.receive_clipboard_paste(ui.ctx(), request, text);
            ui.ctx().request_repaint();
        }
        self.app.handle_close_request(ui.ctx());
        self.startup.leave();
    }
    fn on_exit(&mut self) {
        // A normal exit before the third frame is not a crash.
        self.startup.presented();
        #[cfg(target_os = "macos")]
        self.shutdown_windows.prepare();
        if let Err(e) = self.app.prepare_shutdown() {
            log::error!("Final recovery failed during termination: {e}");
        }
        save_prefs(&self.app);
    }
}

/// `PAGENOR_NO_PREFS`: read and write nothing in the settings directory (tests, demos).
fn no_prefs() -> bool {
    std::env::var_os("PAGENOR_NO_PREFS").is_some()
}

fn prefs_path() -> Option<std::path::PathBuf> {
    profile::config_dir().map(|b| b.join("ui.json"))
}

fn load_prefs(app: &mut DesignApp) {
    if no_prefs() {
        return;
    }
    if let Some(p) = prefs_path()
        && let Ok(bytes) = std::fs::read(&p)
        && let Ok(ui) = pagenor_ui_egui::UiState::from_preferences_json(&bytes)
    {
        app.ui = ui;
    }
    // Engine preferences (Preferences dialog, favourites…) live beside the UI state.
    if let Some(p) = prefs_path().map(|p| p.with_file_name("prefs.json"))
        && let Ok(bytes) = std::fs::read(&p)
        && let Ok(prefs) = serde_json::from_slice::<pagenor_engine::Prefs>(&bytes)
    {
        app.session.prefs = prefs;
    }
}

fn save_prefs(app: &DesignApp) {
    if no_prefs() {
        return;
    }
    if let Some(p) = prefs_path() {
        let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
        if let Ok(bytes) = serde_json::to_vec_pretty(&app.ui) {
            let _ = std::fs::write(&p, bytes);
        }
        if let Ok(bytes) = serde_json::to_vec_pretty(&app.session.prefs) {
            let _ = std::fs::write(p.with_file_name("prefs.json"), bytes);
        }
    }
}

/// One file-type row in an open dialog. `open_filters` is what `pick_open` applies.
struct OpenFilter {
    name: &'static str,
    extensions: &'static [&'static str],
}

fn open_filters(purpose: &str) -> &'static [OpenFilter] {
    match purpose {
        "swatches" => &[OpenFilter { name: "Swatch Exchange (ASE)", extensions: &["ase"] }],
        "script" => &[OpenFilter { name: "Script", extensions: &["pscript", "dcscript", "txt", "json"] }],
        "icc" => &[OpenFilter { name: "ICC profile", extensions: &["icc", "icm"] }],
        "book" => &[OpenFilter { name: "Book", extensions: &["pbook", "dcbook"] }],
        "xml" => &[OpenFilter { name: "XML", extensions: &["xml"] }],
        "library" => &[OpenFilter { name: "Object Library", extensions: &["plib", "dclib"] }],
        "dataMerge" => {
            &[OpenFilter { name: "Data source (CSV, TSV, text, Excel, JSON)", extensions: &["csv", "tsv", "tab", "txt", "xlsx", "xls", "json"] }]
        }
        "relink" => &[OpenFilter {
            name: "Graphics",
            extensions: &["png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp", "psd", "svg", "pdf", "ai", "eps"],
        }],
        "place" => &[
            OpenFilter {
                name: "Graphics and text",
                extensions: &[
                    "png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp", "psd", "svg", "pdf", "ai", "eps", "txt", "docx", "rtf", "md",
                    "markdown", "xlsx", "idml", "indd", "pgnr", "pagenor", "mp4", "m4v", "mov", "webm", "mp3", "m4a", "wav", "ogg",
                ],
            },
            OpenFilter {
                name: "Graphics",
                extensions: &["png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp", "psd", "svg", "pdf", "ai", "eps"],
            },
            OpenFilter { name: "Text (Word, RTF, Markdown, plain, Excel)", extensions: &["docx", "rtf", "txt", "md", "markdown", "xlsx"] },
            OpenFilter { name: "Video and sound", extensions: &["mp4", "m4v", "mov", "webm", "mp3", "m4a", "wav", "ogg"] },
        ],
        _ => &[
            OpenFilter {
                name: "Pagenor and publishing documents",
                extensions: &["pgnr", "pagenor", "idml", "indd", "p65", "t65", "pmd", "pmt", "pm7"],
            },
            OpenFilter { name: "Pagenor", extensions: pagenor_engine::doc::file_type::EXTENSIONS },
            OpenFilter { name: "InDesign Markup (IDML)", extensions: &["idml"] },
            OpenFilter { name: "InDesign Document (INDD)", extensions: &["indd"] },
            OpenFilter { name: "PageMaker 6.5/7 (experimental)", extensions: &["p65", "t65", "pmd", "pmt", "pm7"] },
        ],
    }
}

fn services() -> Services {
    Services {
        pick_open: Some(Box::new(|purpose: &str| {
            let mut dialog = rfd::FileDialog::new();
            for filter in open_filters(purpose) {
                dialog = dialog.add_filter(filter.name, filter.extensions);
            }
            dialog.pick_file().map(|p| p.to_string_lossy().to_string())
        })),
        pick_save: Some(Box::new(|name: &str| {
            let mut dialog = rfd::FileDialog::new().set_file_name(name);
            if pagenor_engine::doc::file_type::is_native_name(name) {
                dialog = dialog.add_filter("Pagenor", &[pagenor_engine::doc::file_type::EXTENSION]);
            }
            dialog.save_file().map(|p| p.to_string_lossy().to_string())
        })),
        read: Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string()))),
        write: Some(Box::new(|p: &str, b: &[u8]| std::fs::write(p, b).map_err(|e| e.to_string()))),
        clipboard_text: Some(Box::new(|| arboard::Clipboard::new().ok()?.get_text().ok())),
        ..Default::default()
    }
}

/// Desktop app ID: the Linux `.desktop` file name and hicolor icon name (Wayland matches windows by it).
const APP_ID: &str = "org.pagenor.app";

/// The window, Dock, taskbar and Alt-Tab icon. macOS gets Apple's icon grid (transparent margin);
/// elsewhere the full tile. Regenerate with `packaging/icons.sh`.
fn app_icon() -> Option<egui::IconData> {
    #[cfg(target_os = "macos")]
    let bytes = include_bytes!("../../../assets/app-icon/pagenor-macos-512.png").as_slice();
    #[cfg(not(target_os = "macos"))]
    let bytes = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/org.pagenor.app.png").as_slice();
    eframe::icon_data::from_png_bytes(bytes).ok()
}

/// Malformed values keep automation disabled. Do not echo a supplied value into diagnostics.
fn control_port_from(value: Option<String>) -> Option<u16> {
    match value.and_then(|value| value.trim().parse().ok()) {
        Some(port) => Some(port),
        None => {
            log::warn!("--control needs a valid port number; the control channel is off");
            None
        }
    }
}

fn main() -> std::process::ExitCode {
    // First, so every start-up record (and the engine's panic hook, installed with the first
    // Session) is captured; see `logging`.
    let logger = logging::install();
    let mut control_port: Option<u16> = None; // Pagenor: local automation OFF by default
    let mut files = Vec::new();
    let mut sample = false;
    let mut skip = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--control" => control_port = control_port_from(args.next()),
            "--sample" => sample = true,
            "--version" => {
                println!("pagenor {}", env!("CARGO_PKG_VERSION"));
                return std::process::ExitCode::SUCCESS;
            }
            _ => match gpu::skip_arg(&a) {
                Some(list) => skip = list,
                None => files.push(a),
            },
        }
    }
    // `--version` returns above without rotating or creating diagnostic files.
    if let Some(logger) = logger {
        match profile::config_dir() {
            Some(dir) => match logger.attach_dir(&dir.join("logs")) {
                Ok(path) => log::info!("Pagenor {}, log file {}", env!("CARGO_PKG_VERSION"), path.display()),
                Err(error) => log::warn!("no log file: {error}"),
            },
            None => {
                logger.disable_file();
                log::warn!("no settings directory; diagnostics use standard error only");
            }
        }
    }
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pagenor")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 560.0])
            .with_decorations(!cfg!(target_os = "windows"))
            .with_drag_and_drop(true)
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false),
        ..Default::default()
    };
    #[cfg(target_os = "windows")]
    graphics::configure(&mut options, eframe::wgpu::Backends::from_env());
    options.viewport = options.viewport.with_app_id(APP_ID);
    if let Some(icon) = app_icon() {
        options.viewport = options.viewport.with_icon(icon);
    }
    // The graphics backend, decided before the window exists: a driver that faults takes the
    // process down before any Rust code can catch it (see `gpu::backend`).
    let fallback = gpu::backend::Fallback::begin(
        if no_prefs() { None } else { profile::config_dir().map(|d| d.join(gpu::backend::FILE)) },
        env!("CARGO_PKG_VERSION"),
        &gpu::backend::skipped_backends(&skip),
    );
    let instance_backends = fallback.as_ref().map(gpu::backend::Fallback::backends);
    let startup = std::sync::Arc::new(gpu::Startup::new(fallback, skip.clone()));
    // The adapter: the GPU that drives the (primary) display unless `WGPU_POWER_PREF` chooses a
    // kind; a GPU without a monitor reset its driver and took every monitor down. Logged first:
    // the first question in every black-window report.
    let power_env = eframe::wgpu::PowerPreference::from_env();
    let power = gpu::power_preference(power_env);
    let displays = gpu::preferred_displays(power_env);
    let chosen_gpu = !gpu::automatic(power_env);
    if chosen_gpu {
        log::info!("graphics processor chosen by WGPU_POWER_PREF ({power:?}): which GPU drives the display isn't considered");
    } else {
        let listed: Vec<String> = displays.iter().map(ToString::to_string).collect();
        log::info!("display GPUs (PCI vendor:device): {}", if listed.is_empty() { "unknown".to_string() } else { listed.join(", ") });
    }
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup {
        if let Some(backends) = instance_backends {
            create.instance_descriptor.backends = backends;
        }
        create.instance_descriptor.flags = gpu::instance_flags();
        // Only adapters that can show the window, in the order `gpu` gives.
        create.native_adapter_selector = Some(gpu::selector(power, displays, startup.clone()));
    }
    gpu::watch_panics();
    gpu::watch_first_frame(startup.clone());
    // winit has no file drag-and-drop on Wayland (only on X11), so dropping images from the file
    // manager showed a "no" cursor. Run through XWayland when it's there; PAGENOR_WAYLAND=1
    // keeps the native Wayland backend.
    #[cfg(all(unix, not(target_os = "macos")))]
    if std::env::var_os("DISPLAY").is_some() && std::env::var_os("PAGENOR_WAYLAND").is_none() {
        options.event_loop_builder = Some(Box::new(|b| {
            use winit::platform::x11::EventLoopBuilderExtX11;
            b.with_x11();
        }));
    }
    #[cfg(target_os = "macos")]
    let shutdown_windows = native_input::ShutdownWindows::default();
    #[cfg(target_os = "macos")]
    let app_shutdown_windows = shutdown_windows.clone();
    let created = startup.clone();
    // A panic that ends the window (egui-wgpu's own) is caught here: never a crash.
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "Pagenor",
            options,
            Box::new(move |cc| {
                let mut restarted = None;
                if let Some(rs) = &cc.wgpu_render_state {
                    // What a bug report needs to know about the graphics driver.
                    let info = rs.adapter.get_info();
                    log::info!(
                        "graphics: {} on {:?} ({:?}; driver {} {}; power preference {power:?})",
                        info.name,
                        info.backend,
                        info.device_type,
                        info.driver,
                        info.driver_info
                    );
                    created.started_with(info.backend);
                    created.created(&cc.egui_ctx);
                    if !skip.is_empty() {
                        // A processor chosen by `WGPU_POWER_PREF` can be the one that can't show the window.
                        let hint = if chosen_gpu { " (without WGPU_POWER_PREF it uses the one that drives your display)" } else { "" };
                        restarted = Some(format!(
                            "The graphics processor tried first couldn't show the window, so Pagenor started again on {} ({:?}, {:?}){hint}.",
                            info.name.trim(),
                            info.backend,
                            info.device_type
                        ));
                    }
                }
                let mut session = Session::new();
                // Crash recovery: reopen what a previous run left unsaved, then keep it current.
                let recovery = profile::recovery_dir();
                let recovery_error = recovery.as_ref().err().cloned();
                session.recovery_dir = recovery.ok().flatten();
                let recovered = session.execute("file.recovery.open", &serde_json::json!({})).ok();
                let clipboard = std::sync::Arc::new(NativeClipboard {
                    adapter: std::sync::Mutex::new(egui_winit::clipboard::Clipboard::new(cc.display_handle().ok().map(|handle| handle.as_raw()))),
                    _window: cc.winit_window().cloned(),
                });
                let copy_clipboard = std::sync::Arc::clone(&clipboard);
                cc.egui_ctx.add_plugin(NativeClipboardOutput(move |text: &str| {
                    copy_clipboard.adapter.lock().unwrap_or_else(std::sync::PoisonError::into_inner).set_text(text.to_owned());
                }));
                let (paste_tx, paste_requests) = std::sync::mpsc::channel::<PasteRequest>();
                let paste_ctx = cc.egui_ctx.clone();
                let mut platform_services = services();
                platform_services.request_paste = Some(Box::new(move |request| {
                    let _ = paste_tx.send(request);
                    paste_ctx.request_repaint();
                }));
                let mut app = DesignApp::new(session, platform_services).with_startup_splash();
                if let Some(line) = restarted.or_else(|| created.status_line()) {
                    app.status(line);
                }
                // Recovered documents matter more than the graphics note; both are in the log.
                if let Some(n) = recovered.as_ref().and_then(|r| r["opened"].as_array()).map(Vec::len).filter(|n| *n > 0) {
                    app.status(format!("Recovered {n} unsaved document{} from the last session.", if n == 1 { "" } else { "s" }));
                }
                load_prefs(&mut app);
                if let Some(error) = recovery_error {
                    log::warn!("{error}");
                    app.status(error);
                }
                app.sync_recent_documents();
                app.services.system_theme = appearance::service(&cc.egui_ctx);
                app.integrated_titlebar = cfg!(target_os = "macos");
                app.custom_titlebar = cfg!(target_os = "windows");
                if let Some(port) = control_port {
                    match control_server::start(port, cc.egui_ctx.clone()) {
                        Ok(rx) => app = app.with_control(rx),
                        Err(error) => {
                            log::warn!("control server unavailable: {error}");
                            app.status(format!("Control server unavailable: {error}"));
                        }
                    }
                }
                if sample {
                    let _ = app.run("file.newSample", serde_json::json!({}));
                }
                for f in files {
                    if let Err(e) = app.open_file("file.open", serde_json::json!({"path": f})) {
                        log::error!("Open {f}: {e}");
                    }
                }
                created.ready();
                Ok(Box::new(App {
                    app,
                    clipboard,
                    paste_requests,
                    paste_completions: Vec::new(),
                    #[cfg(target_os = "macos")]
                    menu: None,
                    startup: created,
                    frames: 0,
                    #[cfg(target_os = "macos")]
                    shutdown_windows: app_shutdown_windows,
                }))
            }),
        )
    }))
    .map_err(|payload| pagenor_engine::guard::panic_message(payload.as_ref()));
    #[cfg(target_os = "macos")]
    drop(shutdown_windows);
    // The window's end, or a graphics failure that starts the app again. A failure is logged:
    // standard error and the log file, since a Windows start has no console to show it.
    match gpu::finish(run, &startup) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            log::error!("Pagenor stopped: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn control_port_values_that_are_not_ports_keep_automation_disabled() {
        assert_eq!(super::control_port_from(Some("7979".into())), Some(7979));
        assert_eq!(super::control_port_from(Some(" 7979 ".into())), Some(7979));
        assert_eq!(super::control_port_from(None), None);
        for bad in ["", "abc", "-1", "65536", "99999999999999999999"] {
            assert_eq!(super::control_port_from(Some(bad.into())), None);
        }
    }

    #[test]
    fn relink_open_dialog_lists_graphics() {
        let filters = super::open_filters("relink");
        let exts: Vec<&str> = filters.iter().flat_map(|filter| filter.extensions.iter().copied()).collect();
        for ext in ["png", "jpg", "tif", "psd", "svg", "pdf", "ai", "eps"] {
            assert!(exts.contains(&ext), "{ext} is missing from the relink dialog: {exts:?}");
        }
        assert!(!exts.contains(&"pagenor") && !exts.contains(&"idml"));
    }

    #[test]
    fn data_merge_open_dialog_lists_table_extensions() {
        let filters = super::open_filters("dataMerge");
        let exts: Vec<&str> = filters.iter().flat_map(|filter| filter.extensions.iter().copied()).collect();
        for ext in ["csv", "tsv", "tab", "txt", "xlsx", "xls", "json"] {
            assert!(exts.contains(&ext), "{ext} is missing from the dataMerge dialog: {exts:?}");
        }
        assert!(
            !exts.iter().any(|ext| ["pgnr", "pagenor", "idml"].contains(ext)),
            "dataMerge must not fall through to the document filters: {exts:?}"
        );
        let place = super::open_filters("place");
        assert!(place.len() > 1, "the place dialog keeps a filter for each kind of file");
        assert!(place.iter().any(|filter| filter.extensions.contains(&"png")));
        let documents = super::open_filters("");
        assert!(documents.iter().any(|filter| filter.extensions.contains(&"pagenor")));
        assert!(documents.iter().any(|filter| filter.extensions.contains(&"pgnr")));
        for ext in ["p65", "t65", "pmd", "pmt", "pm7"] {
            assert!(documents.iter().any(|filter| filter.extensions.contains(&ext)));
        }
        assert!(place.iter().any(|filter| filter.extensions.contains(&"pgnr")));
    }
}

#[cfg(test)]
mod native_clipboard_output_tests {
    use super::NativeClipboardOutput;
    use std::sync::{Arc, Mutex};

    struct LateCopy(Option<String>);
    impl egui::Plugin for LateCopy {
        fn debug_name(&self) -> &'static str {
            "Late copy regression"
        }
        fn on_end_pass(&mut self, ui: &mut egui::Ui) {
            if let Some(text) = self.0.take() {
                ui.ctx().copy_text(text);
            }
        }
    }

    #[test]
    fn copies_late_text_at_output_handoff_without_consuming_backend_commands() {
        for late in ["selected label", ""] {
            let ctx = egui::Context::default();
            let writes = Arc::new(Mutex::new(Vec::new()));
            let sink = Arc::clone(&writes);
            ctx.add_plugin(NativeClipboardOutput(move |text: &str| sink.lock().unwrap().push(text.to_owned())));
            ctx.add_plugin(LateCopy(Some(late.into())));
            let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.ctx().copy_text("earlier field".into());
                assert!(writes.lock().unwrap().is_empty(), "copy must wait for completed output");
            });
            assert_eq!(*writes.lock().unwrap(), [late]);
            let texts: Vec<_> = output
                .platform_output
                .commands
                .iter()
                .filter_map(|command| match command {
                    egui::OutputCommand::CopyText(text) => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(texts, ["earlier field", late]);
            output.drop_without_applying_deltas();
            // A later external value must not be replaced on a frame with no copy.
            *writes.lock().unwrap() = vec!["external replacement".into()];
            ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
            assert_eq!(*writes.lock().unwrap(), ["external replacement"]);
        }
    }

    #[test]
    fn final_pass_copy_wins_when_an_earlier_pass_is_discarded() {
        let ctx = egui::Context::default();
        let writes = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&writes);
        ctx.add_plugin(NativeClipboardOutput(move |text: &str| sink.lock().unwrap().push(text.to_owned())));
        ctx.add_plugin(LateCopy(Some("late first pass".into())));
        let mut passes = 0;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            passes += 1;
            if passes == 1 {
                ui.ctx().copy_text("early first pass".into());
                ui.ctx().request_discard("clipboard ordering regression");
            } else {
                ui.ctx().copy_text("final pass".into());
            }
        });
        assert_eq!(passes, 2);
        assert_eq!(*writes.lock().unwrap(), ["late first pass", "final pass"]);
        let last = output.platform_output.commands.iter().rev().find_map(|command| match command {
            egui::OutputCommand::CopyText(text) => Some(text.as_str()),
            _ => None,
        });
        assert_eq!(last, Some("final pass"));
        output.drop_without_applying_deltas();
    }
}
