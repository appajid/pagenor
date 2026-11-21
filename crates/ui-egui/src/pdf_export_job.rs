//! Interactive PDF progress, adapted from upstream #1106/#2990. Engine/CLI exports
//! remain synchronous by default; native UI jobs retain document and font snapshots.
use crate::{DesignApp, dialogs::Dialog};
use serde_json::{Value, json};

#[derive(Default)]
pub(crate) struct State {
    #[cfg(not(target_arch = "wasm32"))]
    job: Option<Job>,
    pub(crate) last: Option<Value>,
    next: u64,
}
impl State {
    pub(crate) fn busy(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.job.is_some()
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }
    pub(crate) fn status(&self) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(job) = &self.job {
            let progress = *job.shared.progress.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let (phase, done, total) = match progress {
                pagenor_engine::PdfExportProgress::Preparing => ("preparing", 0, 0),
                pagenor_engine::PdfExportProgress::Pages { done, total } => ("pages", done, total),
                pagenor_engine::PdfExportProgress::Finishing => ("finishing", 0, 0),
            };
            let phase = match job.shared.phase.load(Ordering::Acquire) {
                SAVING => "saving",
                CANCELLED => "cancelling",
                _ => phase,
            };
            return json!({"busy":true,"operationId":self.next,"phase":phase,"done":done,"total":total,"last":self.last});
        }
        json!({"busy":false,"operationId":self.next,"last":self.last})
    }
}

pub(crate) fn completed(app: &mut DesignApp, path: &str, pages: &Value, warnings: &Value) {
    app.status("PDF export completed.");
    let warnings = warnings.as_array().into_iter().flatten().filter_map(Value::as_str).collect::<Vec<_>>().join("\n");
    let mut message = format!("PDF export completed.\n\n{path}\n{} page(s)", pages.as_u64().unwrap_or(0));
    if !warnings.is_empty() {
        message.push_str(&format!("\n\nWarnings:\n{warnings}"));
    }
    app.ui.dialog = Some(Dialog::new("alert", json!({"title":"PDF Export","message":message})));
}

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
    mpsc,
};
#[cfg(not(target_arch = "wasm32"))]
const GENERATING: u8 = 0;
#[cfg(not(target_arch = "wasm32"))]
const SAVING: u8 = 1;
#[cfg(not(target_arch = "wasm32"))]
const CANCELLED: u8 = 2;
#[cfg(not(target_arch = "wasm32"))]
struct Shared {
    phase: AtomicU8,
    progress: Mutex<pagenor_engine::PdfExportProgress>,
}
#[cfg(not(target_arch = "wasm32"))]
struct Done {
    report: pagenor_engine::PdfExportReport,
    written: bool,
}
#[cfg(not(target_arch = "wasm32"))]
fn save_report(
    shared: &Shared,
    writer: Option<&crate::BackgroundWriteFn>,
    path: &str,
    report: pagenor_engine::PdfExportReport,
) -> Result<Done, String> {
    shared.phase.compare_exchange(GENERATING, SAVING, Ordering::AcqRel, Ordering::Acquire).map_err(|_| "PDF export cancelled".to_string())?;
    if let Some(write) = writer {
        write(path, &report.bytes)?;
    }
    Ok(Done { report, written: writer.is_some() })
}

#[cfg(not(target_arch = "wasm32"))]
struct Job {
    path: String,
    source: Option<Dialog>,
    settings: Option<crate::PdfExportSettings>,
    shared: Arc<Shared>,
    rx: mpsc::Receiver<Result<Done, String>>,
    shown: bool,
    started: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn start(app: &mut DesignApp, path: String, params: Value) -> Result<Value, String> {
    if app.pdf_export_job.busy() {
        return Err("A PDF export is already running.".into());
    }
    let mut snapshot = app.session.active().ok_or("no document")?.clone();
    snapshot.history = Default::default();
    snapshot.saved_doc = snapshot.doc.clone();
    snapshot.interaction = None;
    snapshot.preview_stash = None;
    let cache = app.session.cache.clone();
    let source = app.ui.dialog.clone().filter(|d| d.id == "pdfExport");
    let settings = source.as_ref().map(|d| serde_json::from_value(Value::Object((*d.fields).clone()))).transpose().map_err(|e| e.to_string())?;
    let id = app.pdf_export_job.next.checked_add(1).ok_or("Too many export operations")?;
    let shared = Arc::new(Shared { phase: AtomicU8::new(GENERATING), progress: Mutex::new(pagenor_engine::PdfExportProgress::Preparing) });
    let worker = shared.clone();
    let writer = app.services.background_write.clone();
    let target = path.clone();
    let wake = app.dialog_wake.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("pagenor-pdf-export".into())
        .spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut session = pagenor_engine::Session::new();
                session.cache = cache;
                session.add_document(snapshot);
                let report = pagenor_engine::pdf_export_report(&session, &params, &mut |p| {
                    *worker.progress.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = p;
                    worker.phase.load(Ordering::Acquire) != CANCELLED
                })
                .map_err(|e| e.to_string())?;
                // A cancel and the write claim are mutually exclusive. After SAVING, no
                // accepted cancellation can falsely promise that the old file is preserved.
                save_report(&worker, writer.as_ref(), &target, report)
            }))
            .unwrap_or_else(|_| Err("PDF export failed unexpectedly. Your document is unchanged.".into()));
            // The app can have exited; no UI receiver remains in that case.
            let _ = tx.send(outcome);
            if let Some(ctx) = wake.get() {
                ctx.request_repaint();
            }
        })
        .map_err(|e| format!("Could not start PDF export: {e}"))?;
    app.pdf_export_job.next = id;
    app.pdf_export_job.last = None;
    app.pdf_export_job.job = Some(Job { path: path.clone(), source, settings, shared, rx, shown: false, started: std::time::Instant::now() });
    app.ui.dialog = Some(Dialog::new("pdfExportProgress", json!({})));
    Ok(json!({"queued":true,"operationId":id,"path":path}))
}

/// Poll only after a progress frame was painted, including a short document that
/// already finished. The small minimum visibility never delays the actual work.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn poll(app: &mut DesignApp) {
    let Some(job) = app.pdf_export_job.job.as_ref() else { return };
    if !job.shown || job.started.elapsed() < std::time::Duration::from_millis(200) {
        return;
    }
    let result = match job.rx.try_recv() {
        Ok(r) => r,
        Err(mpsc::TryRecvError::Empty) => return,
        Err(mpsc::TryRecvError::Disconnected) => Err("PDF export was interrupted.".into()),
    };
    let Some(job) = app.pdf_export_job.job.take() else { return };
    let id = app.pdf_export_job.next;
    if job.shared.phase.load(Ordering::Acquire) == CANCELLED {
        app.status("PDF export cancelled.");
        app.pdf_export_job.last = Some(json!({"operationId":id,"path":job.path,"cancelled":true}));
        app.ui.dialog = job.source;
        return;
    }
    let result = result.and_then(|done| {
        if !done.written {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.services.write.as_mut().ok_or("no writer")?(&job.path, &done.report.bytes)))
                .unwrap_or_else(|_| Err("PDF could not be saved.".into()))?;
        }
        Ok(done.report)
    });
    match result {
        Ok(report) => {
            let result = json!({"operationId":id,"path":job.path,"pages":report.pages,"bytes":report.bytes.len(),"warnings":report.warnings});
            if let Some(settings) = job.settings {
                app.ui.pdf_export = settings;
            }
            completed(app, &job.path, &result["pages"], &result["warnings"]);
            app.pdf_export_job.last = Some(result);
        }
        Err(error) => {
            app.status(&error);
            app.pdf_export_job.last = Some(json!({"operationId":id,"path":job.path,"error":error}));
            app.ui.dialog = Some(if let Some(mut source) = job.source {
                source.fields.insert("error".into(), json!(error));
                source
            } else {
                Dialog::new("alert", json!({"title":"PDF Export Failed","message":error}))
            });
        }
    }
}

pub(crate) fn show(app: &mut DesignApp, ctx: &egui::Context) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        poll(app);
        if let Some(job) = app.pdf_export_job.job.as_mut() {
            job.shown = true;
            let progress = *job.shared.progress.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let phase = job.shared.phase.load(Ordering::Acquire);
            egui::Modal::new(egui::Id::new("pdf-export-progress")).show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.heading("Exporting PDF");
                ui.label(std::path::Path::new(&job.path).file_name().unwrap_or_default().to_string_lossy());
                match progress {
                    pagenor_engine::PdfExportProgress::Pages { done, total } => {
                        ui.add(egui::ProgressBar::new(done as f32 / total.max(1) as f32).text(format!("{done} of {total} pages exported")));
                    }
                    _ => {
                        ui.spinner();
                    }
                }
                ui.label(if phase == CANCELLED {
                    "Cancelling…"
                } else if phase == SAVING {
                    "Saving PDF…"
                } else if matches!(progress, pagenor_engine::PdfExportProgress::Finishing) {
                    "Finishing PDF…"
                } else {
                    "Preparing and exporting pages…"
                });
                if ui.add_enabled(phase == GENERATING, egui::Button::new("Cancel")).clicked() {
                    // The worker can claim SAVING while this button is on screen.
                    // Refuse cancellation then; never claim a written file was cancelled.
                    let _ = job.shared.phase.compare_exchange(GENERATING, CANCELLED, Ordering::AcqRel, Ordering::Acquire);
                }
            });
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
            return true;
        }
    }
    let _ = (app, ctx);
    false
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn finish(app: &mut DesignApp) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while app.pdf_export_job.busy() && std::time::Instant::now() < deadline {
            if let Some(job) = app.pdf_export_job.job.as_mut() {
                job.shown = true;
                job.started = std::time::Instant::now() - std::time::Duration::from_secs(1);
            }
            poll(app);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(!app.pdf_export_job.busy(), "PDF worker must finish before test cleanup");
    }
    fn app(pages: usize) -> DesignApp {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"pages":pages})).unwrap();
        app
    }
    #[test]
    fn small_and_long_exports_show_progress_before_closing_with_completion() {
        for pages in [1, 48] {
            let mut app = app(pages);
            let writes = Arc::new(Mutex::new(Vec::new()));
            let sink = writes.clone();
            app.services.pick_save = Some(Box::new(|_| Some("test.pdf".into())));
            app.services.write = Some(Box::new(move |_, bytes| {
                sink.lock().unwrap().push(bytes.to_vec());
                Ok(())
            }));
            crate::dialogs::open_pdf_export(&mut app);
            let reply = crate::dialogs::confirm(&mut app).unwrap();
            assert_eq!(reply["queued"], true);
            assert!(app.pdf_export_job.busy());
            assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfExportProgress");
            assert!(writes.lock().unwrap().is_empty(), "UI result must not complete before a progress frame");
            // Even a finished one-page worker cannot dismiss an unpainted progress window.
            poll(&mut app);
            assert!(app.pdf_export_job.busy());
            assert!(start(&mut app, "other.pdf".into(), json!({})).unwrap_err().contains("already running"));
            finish(&mut app);
            assert_eq!(app.pdf_export_job.last.as_ref().unwrap()["pages"], pages);
            assert_eq!(writes.lock().unwrap().len(), 1);
            assert!(writes.lock().unwrap()[0].starts_with(b"%PDF"));
            assert_eq!(app.ui.dialog.as_ref().unwrap().id, "alert");
            assert!(app.ui.dialog.as_ref().unwrap().s("message").contains("PDF export completed."));
        }
    }
    #[test]
    fn deferred_native_save_answer_runs_from_its_original_export_options() {
        for cancelled in [false, true] {
            let mut app = app(2);
            let writes = Arc::new(Mutex::new(0));
            let sink = writes.clone();
            app.services.write = Some(Box::new(move |_, bytes| {
                assert!(bytes.starts_with(b"%PDF"));
                *sink.lock().unwrap() += 1;
                Ok(())
            }));
            app.answer_dialogs(
                move |_| if cancelled { craft_dialogs::Answer::Cancelled } else { craft_dialogs::Answer::Paths(vec!["deferred.pdf".into()]) },
            );
            crate::dialogs::open_pdf_export(&mut app);
            app.ui.dialog.as_mut().unwrap().fields.insert("marksCrop".into(), json!(true));
            assert!(crate::dialogs::confirm(&mut app).unwrap().is_null());
            assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfExport");
            assert!(app.file_dialogs_pending());
            app.settle_dialogs();
            assert!(!app.file_dialogs_pending(), "save answer must not wait forever behind its own options dialog");
            assert_eq!(app.pdf_export_job.busy(), !cancelled);
            if cancelled {
                assert_eq!(*writes.lock().unwrap(), 0);
                assert!(!app.ui.pdf_export.marks_crop);
                assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfExport");
            } else {
                finish(&mut app);
                assert_eq!(*writes.lock().unwrap(), 1);
                assert!(app.ui.pdf_export.marks_crop);
                assert_eq!(app.ui.dialog.as_ref().unwrap().id, "alert");
            }
        }
    }

    #[test]
    fn native_atomic_saving_is_off_thread_and_cannot_be_cancelled_late() {
        let mut app = app(2);
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("existing.pdf");
        std::fs::write(&path, b"previous PDF").unwrap();
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let release_rx = Mutex::new(release_rx);
        let ui_thread = std::thread::current().id();
        app.services.background_write = Some(Arc::new(move |path, bytes| {
            assert_ne!(ui_thread, std::thread::current().id());
            entered_tx.send(()).map_err(|e| e.to_string())?;
            release_rx.lock().unwrap().recv_timeout(std::time::Duration::from_secs(10)).map_err(|e| e.to_string())?;
            pagenor_engine::write_atomic(std::path::Path::new(path), bytes).map_err(|e| e.to_string())
        }));
        app.services.write = Some(Box::new(|_, _| panic!("UI writer must not run")));
        start(&mut app, path.to_string_lossy().into_owned(), json!({})).unwrap();
        entered_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"previous PDF");
        let shared = &app.pdf_export_job.job.as_ref().unwrap().shared;
        assert_eq!(shared.phase.compare_exchange(GENERATING, CANCELLED, Ordering::AcqRel, Ordering::Acquire), Err(SAVING));
        // Read-only commands remain usable; a second job cannot replace this one.
        assert_eq!(app.run("document.inspect", json!({})).unwrap()["pageCount"], 2);
        app.run("app.quit", json!({})).unwrap();
        assert!(!app.quit_requested && !app.quit_confirming);
        release_tx.send(()).unwrap();
        finish(&mut app);
        assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF"));
        assert!(app.pdf_export_job.last.as_ref().unwrap().get("error").is_none());
    }
    #[test]
    fn snapshot_export_survives_editing_and_closing_the_source_document() {
        let mut app = app(3);
        let before = app.session.active().unwrap().doc.clone();
        let output = Arc::new(Mutex::new(Vec::new()));
        let sink = output.clone();
        app.services.write = Some(Box::new(move |_, bytes| {
            *sink.lock().unwrap() = bytes.to_vec();
            Ok(())
        }));
        start(&mut app, "snapshot.pdf".into(), json!({})).unwrap();
        app.session.close_document(0);
        app.run("file.new", json!({"pages":1})).unwrap();
        finish(&mut app);
        assert_eq!(app.pdf_export_job.last.as_ref().unwrap()["pages"], 3);
        assert_eq!(before.page_count(), 3);
        assert_eq!(app.session.doc().unwrap().doc.page_count(), 1);
        assert!(output.lock().unwrap().starts_with(b"%PDF"));
    }
    #[test]
    fn accepted_cancellation_preserves_the_previous_output_and_never_calls_writer() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("existing.pdf");
        std::fs::write(&path, b"previous PDF").unwrap();
        let shared = Shared { phase: AtomicU8::new(CANCELLED), progress: Mutex::new(pagenor_engine::PdfExportProgress::Finishing) };
        let calls = Arc::new(Mutex::new(0));
        let count = calls.clone();
        let writer: crate::BackgroundWriteFn = Arc::new(move |path, bytes| {
            *count.lock().unwrap() += 1;
            pagenor_engine::write_atomic(std::path::Path::new(path), bytes).map_err(|e| e.to_string())
        });
        let report = pagenor_engine::PdfExportReport { bytes: b"replacement".to_vec(), pages: 1, warnings: vec![] };
        assert!(save_report(&shared, Some(&writer), &path.to_string_lossy(), report).is_err());
        assert_eq!(*calls.lock().unwrap(), 0);
        assert_eq!(std::fs::read(&path).unwrap(), b"previous PDF");
    }
    #[test]
    fn worker_failure_or_panic_restores_options_without_success_or_persisting_settings() {
        for panic in [false, true] {
            let mut app = app(1);
            app.services.background_write = Some(Arc::new(move |_, _| {
                if panic {
                    panic!("synthetic worker panic")
                }
                Err("disk full".into())
            }));
            app.services.pick_save = Some(Box::new(|_| Some("test.pdf".into())));
            crate::dialogs::open_pdf_export(&mut app);
            app.ui.dialog.as_mut().unwrap().fields.insert("marksCrop".into(), json!(true));
            assert_eq!(crate::dialogs::confirm(&mut app).unwrap()["queued"], true);
            finish(&mut app);
            assert!(!app.ui.pdf_export.marks_crop);
            assert_eq!(app.ui.dialog.as_ref().unwrap().id, "pdfExport");
            assert!(!app.ui.dialog.as_ref().unwrap().s("error").is_empty());
            assert!(!app.ui.status.contains("completed"));
            assert_eq!(app.session.doc().unwrap().doc.page_count(), 1);
        }
    }

    #[test]
    fn progress_modal_replaces_options_then_shows_completion_and_warnings() {
        use std::sync::atomic::AtomicBool;
        let mut app = app(48);
        let (tx, rx) = mpsc::sync_channel(1);
        app.pdf_export_job.next = 1;
        app.pdf_export_job.job = Some(Job {
            path: "progress-test.pdf".into(),
            source: None,
            settings: None,
            rx,
            shown: false,
            started: std::time::Instant::now(),
            shared: Arc::new(Shared {
                phase: AtomicU8::new(GENERATING),
                progress: Mutex::new(pagenor_engine::PdfExportProgress::Pages { done: 7, total: 48 }),
            }),
        });
        app.ui.dialog = Some(Dialog::new("pdfExportProgress", json!({})));
        let pictures = std::env::var_os("PAGENOR_PDF_PROGRESS_SHOTS").map(std::path::PathBuf::from);
        let ready = Arc::new(AtomicBool::new(false));
        let draw = ready.clone();
        let builder = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0));
        let builder = if pictures.is_some() { builder.wgpu() } else { builder };
        let mut h = builder.build_ui_state(
            move |ui, app: &mut DesignApp| {
                if !draw.load(Ordering::Acquire) {
                    return;
                }
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.input_mut().max_texture_side = Some(8192);
        ready.store(true, Ordering::Release);
        h.run_steps(6);
        assert!(h.state().pdf_export_job.busy());
        assert!(h.state().pdf_export_job.job.as_ref().unwrap().shown);
        assert_eq!(h.state().pdf_export_job.status()["done"], 7);
        assert_eq!(h.state().ui.dialog.as_ref().unwrap().id, "pdfExportProgress");
        if let Some(dir) = &pictures {
            std::fs::create_dir_all(dir).unwrap();
            h.render().unwrap().save(dir.join("pdf-progress.png")).unwrap();
        }
        h.state_mut().pdf_export_job.job.as_mut().unwrap().started = std::time::Instant::now() - std::time::Duration::from_secs(1);
        tx.send(Ok(Done {
            report: pagenor_engine::PdfExportReport { bytes: vec![], pages: 48, warnings: vec!["Some text is overset.".into()] },
            written: true,
        }))
        .unwrap();
        h.run_steps(4);
        assert!(!h.state().pdf_export_job.busy());
        let dialog = h.state().ui.dialog.as_ref().unwrap();
        assert_eq!(dialog.id, "alert");
        assert!(dialog.s("message").contains("PDF export completed."));
        assert!(dialog.s("message").contains("Some text is overset."));
        if let Some(dir) = &pictures {
            h.render().unwrap().save(dir.join("pdf-completed.png")).unwrap();
        }
    }

    #[test]
    fn explicit_path_exports_keep_the_synchronous_command_contract() {
        let mut app = app(1);
        app.services.write = Some(Box::new(|_, bytes| {
            assert!(bytes.starts_with(b"%PDF"));
            Ok(())
        }));
        let result = app.run("app.exportPdf", json!({"path":"test.pdf"})).unwrap();
        assert_eq!(result["pages"], 1);
        assert!(result["bytes"].as_u64().unwrap() > 0);
        assert!(!app.pdf_export_job.busy());
    }
}
