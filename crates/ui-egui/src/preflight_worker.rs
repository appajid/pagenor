//! Live preflight must not compose every story on the input/paint thread.
//! One bounded native worker owns its composition cache; obsolete results never
//! become the current document's verdict. Explicit preflight.run stays synchronous.
use pagenor_doc::Document;
#[cfg(not(target_arch = "wasm32"))]
use pagenor_engine::DocState;
use pagenor_engine::{Session, cmd::preflight::Issue};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Report {
    pub issues: Arc<Vec<Issue>>,
    pub checking: bool,
    pub unavailable: bool,
}
#[derive(Default)]
pub(crate) struct State {
    report: Option<(Arc<Document>, u64, Arc<Vec<Issue>>)>,
    #[cfg(not(target_arch = "wasm32"))]
    failed: Option<(std::sync::Weak<Document>, u64)>,
    #[cfg(not(target_arch = "wasm32"))]
    cleared: bool,
    #[cfg(not(target_arch = "wasm32"))]
    worker: Option<Worker>,
    #[cfg(not(target_arch = "wasm32"))]
    submitted: Option<(std::sync::Weak<Document>, u64)>,
}
#[cfg(not(target_arch = "wasm32"))]
struct Job {
    doc: Arc<Document>,
    fonts: Option<Arc<pagenor_engine::FontScope>>,
    generation: u64,
}
#[cfg(not(target_arch = "wasm32"))]
struct Done {
    doc: Arc<Document>,
    generation: u64,
    issues: Option<Vec<Issue>>,
}
#[cfg(not(target_arch = "wasm32"))]
enum Request {
    Check(Job),
    Clear,
}
#[cfg(not(target_arch = "wasm32"))]
struct Worker {
    tx: std::sync::mpsc::SyncSender<Request>,
    rx: std::sync::mpsc::Receiver<Done>,
}
#[cfg(not(target_arch = "wasm32"))]
impl Worker {
    fn spawn(ctx: egui::Context) -> Option<Self> {
        Self::spawn_with(ctx, Box::new(|session| pagenor_engine::cmd::preflight::check(session, 150.0)))
    }
    fn spawn_with(ctx: egui::Context, mut check: Box<dyn FnMut(&Session) -> Vec<Issue> + Send>) -> Option<Self> {
        let (tx, jobs) = std::sync::mpsc::sync_channel::<Request>(1);
        let (done, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("pagenor-preflight".into())
            .spawn(move || {
                let cache = Arc::new(pagenor_compose::Cache::new());
                let mut generation = None;
                while let Ok(request) = jobs.recv() {
                    let Request::Check(job) = request else {
                        cache.clear();
                        generation = None;
                        ctx.request_repaint();
                        continue;
                    };
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        if generation != Some(job.generation) {
                            cache.clear();
                            generation = Some(job.generation);
                        }
                        let mut session = Session::new();
                        let mut state = DocState::new(Document::new(&Default::default()), None);
                        state.doc = job.doc.clone();
                        state.saved_doc = job.doc.clone();
                        state.fonts = job.fonts;
                        session.cache = cache.clone();
                        session.add_document(state);
                        check(&session)
                    }))
                    .ok();
                    if done.send(Done { doc: job.doc, generation: job.generation, issues: result }).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .ok()?;
        Some(Self { tx, rx })
    }
}
impl State {
    pub(crate) fn clear(&mut self, ctx: &egui::Context) {
        self.report = None;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.submitted = None;
            self.failed = None;
            if let Some(worker) = &self.worker {
                while worker.rx.try_recv().is_ok() {}
                if !self.cleared {
                    match worker.tx.try_send(Request::Clear) {
                        Ok(()) => self.cleared = true,
                        Err(std::sync::mpsc::TrySendError::Full(_)) => ctx.request_repaint_after(std::time::Duration::from_millis(50)),
                        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => self.worker = None,
                    }
                }
            }
        }
        let _ = ctx;
    }
    pub(crate) fn get(&mut self, session: &Session, ctx: &egui::Context) -> Report {
        self.get_with(session, ctx, cfg!(test), pagenor_fonts::FontDb::global().scan_generation())
    }
    fn get_with(&mut self, session: &Session, ctx: &egui::Context, synchronous: bool, generation: u64) -> Report {
        let Some(st) = session.active() else {
            self.clear(ctx);
            return Report { issues: Arc::default(), checking: false, unavailable: false };
        };
        #[cfg(not(target_arch = "wasm32"))]
        if !synchronous {
            self.cleared = false;
            if self.worker.is_none() {
                self.worker = Worker::spawn(ctx.clone());
            }
            let Some(worker) = self.worker.as_ref() else {
                return Report { issues: Arc::default(), checking: false, unavailable: true };
            };
            let mut failed = self.failed.as_ref().is_some_and(|(doc, gen_)| doc.ptr_eq(&Arc::downgrade(&st.doc)) && *gen_ == generation);
            while let Ok(done) = worker.rx.try_recv() {
                if Arc::ptr_eq(&done.doc, &st.doc) && done.generation == generation {
                    if let Some(issues) = done.issues {
                        self.report = Some((done.doc, generation, Arc::new(issues)));
                    } else {
                        self.failed = Some((Arc::downgrade(&done.doc), done.generation));
                        failed = true;
                    }
                }
            }
            let current = self.report.as_ref().is_some_and(|(doc, gen_, _)| Arc::ptr_eq(doc, &st.doc) && *gen_ == generation);
            let submitted = self.submitted.as_ref().is_some_and(|(doc, gen_)| doc.ptr_eq(&Arc::downgrade(&st.doc)) && *gen_ == generation);
            if !current && !submitted && !failed {
                match worker.tx.try_send(Request::Check(Job { doc: st.doc.clone(), fonts: st.fonts.clone(), generation })) {
                    Ok(()) => self.submitted = Some((Arc::downgrade(&st.doc), generation)),
                    Err(std::sync::mpsc::TrySendError::Full(_)) => ctx.request_repaint_after(std::time::Duration::from_millis(50)),
                    Err(std::sync::mpsc::TrySendError::Disconnected(_)) => failed = true,
                }
            }
            return Report {
                issues: if current { self.report.as_ref().map(|(_, _, issues)| issues.clone()).unwrap_or_default() } else { Arc::default() },
                checking: !current && !failed,
                unavailable: failed,
            };
        }
        // WASM cannot use threads; cache exact snapshots, never an unpinned address.
        let _ = (ctx, synchronous);
        if self.report.as_ref().is_none_or(|(doc, gen_, _)| !Arc::ptr_eq(doc, &st.doc) || *gen_ != generation) {
            self.report = Some((st.doc.clone(), generation, Arc::new(pagenor_engine::cmd::preflight::check(session, 150.0))));
        }
        Report { issues: self.report.as_ref().map(|(_, _, issues)| issues.clone()).unwrap_or_default(), checking: false, unavailable: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn snapshot_cache_reuses_navigation_checks_and_invalidates_edits_and_documents() {
        let ctx = egui::Context::default();
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let mut state = State::default();
        let first = state.get_with(&session, &ctx, true, 0);
        assert!(!first.checking && !first.unavailable);
        assert!(Arc::ptr_eq(&first.issues, &state.get_with(&session, &ctx, true, 0).issues));
        session.execute("frame.create", &json!({"rect":[20,20,40,30],"content":"text","text":"overset ".repeat(300)})).unwrap();
        let edited = state.get_with(&session, &ctx, true, 0);
        assert!(!Arc::ptr_eq(&first.issues, &edited.issues));
        assert!(edited.issues.iter().any(|i| i.kind == "overset"));
        session.execute("file.new", &json!({})).unwrap();
        assert!(!state.get_with(&session, &ctx, true, 0).issues.iter().any(|i| i.kind == "overset"));
        let before_fonts = state.get_with(&session, &ctx, true, 0);
        let new_fonts = state.get_with(&session, &ctx, true, 1);
        assert!(!Arc::ptr_eq(&before_fonts.issues, &new_fonts.issues));
        assert!(Arc::ptr_eq(&new_fonts.issues, &state.get_with(&session, &ctx, true, 1).issues));
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_background_check_stays_unavailable_instead_of_showing_no_errors() {
        let ctx = egui::Context::default();
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let doc = session.active().unwrap().doc.clone();
        let generation = 0;
        let (tx, _jobs) = std::sync::mpsc::sync_channel(1);
        let (done, rx) = std::sync::mpsc::sync_channel(1);
        assert!(done.send(Done { doc, generation, issues: None }).is_ok());
        let mut state = State { worker: Some(Worker { tx, rx }), ..Default::default() };
        for _ in 0..3 {
            let report = state.get_with(&session, &ctx, false, 0);
            assert!(report.unavailable && !report.checking && report.issues.is_empty());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_checks_do_not_block_input_or_publish_obsolete_results() {
        use std::sync::mpsc;
        use std::time::Duration;
        let ctx = egui::Context::default();
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let original = session.active().unwrap().doc.clone();
        let (started_tx, started) = mpsc::channel();
        let (release, resume) = mpsc::channel();
        let worker = Worker::spawn_with(
            ctx.clone(),
            Box::new(move |_| {
                started_tx.send(()).unwrap();
                resume.recv().unwrap();
                vec![Issue { severity: "error", kind: "test", message: "synthetic result".into(), item: None, page: None }]
            }),
        )
        .unwrap();
        let mut state = State { worker: Some(worker), ..Default::default() };
        assert!(state.get_with(&session, &ctx, false, 0).checking);
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        // The worker is deliberately blocked; the UI can still change documents.
        session.execute("file.new", &json!({})).unwrap();
        for _ in 0..100 {
            let report = state.get_with(&session, &ctx, false, 0);
            assert!(report.checking && report.issues.is_empty());
        }
        release.send(()).unwrap();
        started.recv_timeout(Duration::from_secs(10)).unwrap();
        let report = state.get_with(&session, &ctx, false, 0);
        assert!(report.checking && report.issues.is_empty(), "old document's error must never be shown");
        release.send(()).unwrap();
        for _ in 0..1000 {
            let report = state.get_with(&session, &ctx, false, 0);
            if !report.checking {
                assert_eq!(report.issues[0].kind, "test");
                assert!(!Arc::ptr_eq(&original, &state.report.as_ref().unwrap().0));
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("background check did not complete");
    }
    #[test]
    fn closing_the_last_document_releases_cached_report_ownership() {
        let ctx = egui::Context::default();
        let mut session = Session::new();
        session.execute("file.new", &json!({})).unwrap();
        let weak = Arc::downgrade(&session.active().unwrap().doc);
        let mut state = State::default();
        state.get_with(&session, &ctx, true, 0);
        session.close_document(0);
        state.clear(&ctx);
        assert!(weak.upgrade().is_none());
        assert!(state.report.is_none());
    }
}
