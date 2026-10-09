//! Native AI assistant window and background-task bridge.
use crate::{DesignApp, theme::Tokens};
use pagenor_doc::ai::Status;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowState {
    pub open: bool,
    pub floating: bool,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub tab: String,
}
impl Default for WindowState {
    fn default() -> Self {
        Self { open: false, floating: false, position: [700.0, 140.0], size: [390.0, 580.0], tab: "chat".into() }
    }
}
impl WindowState {
    fn normalize(&mut self) {
        let defaults = Self::default();
        if self.position.iter().any(|v| !v.is_finite() || v.abs() > 100_000.0) {
            self.position = defaults.position;
        }
        if self.size.iter().any(|v| !v.is_finite()) {
            self.size = defaults.size;
        }
        self.size[0] = self.size[0].clamp(320.0, 4096.0);
        self.size[1] = self.size[1].clamp(240.0, 4096.0);
        if !matches!(self.tab.as_str(), "chat" | "suggestions" | "settings") {
            self.tab = defaults.tab;
        }
    }
}
#[derive(Default)]
pub struct Assistant {
    pub drafts: std::collections::HashMap<u64, String>,
    pub notice: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub native: runtime::Runtime,
}
pub fn busy(app: &DesignApp) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.ai.native.task.is_some()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        false
    }
}
pub fn poll(app: &mut DesignApp, ctx: &egui::Context) {
    app.ui.ai.normalize();
    #[cfg(not(target_arch = "wasm32"))]
    runtime::poll(app, ctx);
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}
pub fn command(app: &mut DesignApp, id: &str, p: &Value) -> Result<Value, String> {
    match id {
        "window.ai" => {
            app.ui.ai.open = p["open"].as_bool().unwrap_or(true);
            Ok(json!(app.ui.ai))
        }
        "window.ai.layout" => {
            if let Some(f) = p["floating"].as_bool() {
                app.ui.ai.floating = f;
            }
            if let Some(t) = p["tab"].as_str() {
                if !matches!(t, "chat" | "suggestions" | "settings") {
                    return Err("invalid tab".into());
                }
                app.ui.ai.tab = t.into();
            }
            if let Some(pos) = p["position"].as_array()
                && pos.len() == 2
            {
                let x = pos[0].as_f64().filter(|x| x.is_finite() && x.abs() <= 100_000.0).ok_or("Invalid window position")?;
                let y = pos[1].as_f64().filter(|x| x.is_finite() && x.abs() <= 100_000.0).ok_or("Invalid window position")?;
                app.ui.ai.position = [x as f32, y as f32];
            }
            app.ui.ai.open = true;
            Ok(json!(app.ui.ai))
        }
        "ai.suggestion.show" => {
            let r = app.run("ai.suggestion.locate", p.clone())?;
            if let Some(page) = r["page"].as_u64() {
                crate::canvas::go_to_page(app, page as usize);
            } else if let Some(sid) = r["story"].as_u64() {
                app.story_editor = Some(pagenor_doc::StoryId(sid));
            }
            Ok(r)
        }
        _ => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                runtime::command(app, id, p)
            }
            #[cfg(target_arch = "wasm32")]
            {
                Err("AI networking and sign-in are available in the desktop version.".into())
            }
        }
    }
}
fn action(app: &mut DesignApp, id: &str, p: Value) {
    if let Err(e) = app.run(id, p) {
        app.ai.notice = e;
    }
}
pub fn docked(app: &mut DesignApp, ui: &mut egui::Ui) {
    if !app.ui.ai.open || app.ui.ai.floating {
        return;
    }
    let t = Tokens::get(ui.ctx());
    egui::Panel::right("ai_assistant_dock")
        .default_size(app.ui.ai.size[0])
        .size_range(320.0..=620.0)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10))
        .show(ui, |ui| {
            app.ui.ai.size[0] = ui.available_width();
            ui.horizontal(|ui| {
                ui.heading("AI Assistant");
                if ui.small_button("Float").clicked() {
                    action(app, "window.ai.layout", json!({"floating":true}));
                }
                if ui.small_button("Close").clicked() {
                    action(app, "window.ai", json!({"open":false}));
                }
            });
            body(app, ui);
        });
}
pub fn floating(app: &mut DesignApp, ctx: &egui::Context) {
    if !app.ui.ai.open || !app.ui.ai.floating {
        return;
    }
    let mut open = true;
    let at = app.ui.ai.position;
    let size = app.ui.ai.size;
    let r = egui::Window::new("AI Assistant")
        .id(egui::Id::new("ai_assistant_window"))
        .open(&mut open)
        .default_pos(egui::pos2(at[0], at[1]))
        .default_size(egui::vec2(size[0], size[1]))
        .min_width(320.0)
        .show(ctx, |ui| {
            if ui.small_button("Dock Right").clicked() {
                action(app, "window.ai.layout", json!({"floating":false}));
            }
            body(app, ui);
        });
    if let Some(r) = r {
        app.ui.ai.position = [r.response.rect.min.x, r.response.rect.min.y];
        app.ui.ai.size = [r.response.rect.width(), r.response.rect.height()];
    }
    if !open {
        action(app, "window.ai", json!({"open":false}));
    }
}
fn body(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let running = busy(app);
    ui.horizontal(|ui| {
        for (id, label) in [("chat", "Chat"), ("suggestions", "Suggestions"), ("settings", "Settings")] {
            if ui.selectable_label(app.ui.ai.tab == id, label).clicked() {
                action(app, "window.ai.layout", json!({"tab":id}));
            }
        }
    });
    ui.separator();
    ui.small("Chat can read document text. Proofread Selection sends the selected text. Review suggestions before accepting.");
    if running {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Working…");
            if ui.button("Stop").clicked() {
                action(app, "ai.task.stop", json!({}));
            }
        });
    }
    if !app.ai.notice.is_empty() {
        ui.label(egui::RichText::new(&app.ai.notice).color(t.accent));
    }
    if app.ui.ai.tab == "settings" {
        #[cfg(not(target_arch = "wasm32"))]
        runtime::settings(app, ui);
        #[cfg(target_arch = "wasm32")]
        {
            ui.label("AI services and sign-in are available in the desktop version.");
        }
        return;
    }
    if app.ui.ai.tab == "suggestions" && app.session.active().is_some() {
        let _ = app.run("ai.suggestion.list", json!({}));
    }
    let Some(st) = app.session.active() else {
        ui.label("Open a document to start proofreading.");
        return;
    };
    let uid = st.uid;
    let title = st.title();
    let data = st.doc.ai.clone().unwrap_or_default();
    ui.label(egui::RichText::new(title).strong());
    if app.ui.ai.tab == "suggestions" {
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!running, egui::Button::new("Accept All")).clicked() {
                action(app, "ai.suggestion.acceptAll", json!({}));
            }
            if ui.add_enabled(!running, egui::Button::new("Export HTML Report")).clicked() {
                action(app, "ai.report.export", json!({}));
            }
        });
        if data.suggestions.is_empty() {
            ui.label("No suggestions. Select body text and choose Proofread Selection.");
        }
        egui::ScrollArea::vertical().id_salt("ai_suggestions").auto_shrink([false, false]).show(ui, |ui| {
            for s in &data.suggestions {
                ui.push_id(s.id, |ui| {
                    ui.group(|ui| {
                        ui.label(format!("#{} · {} · {}", s.id, s.status.label(), s.page_label.as_deref().unwrap_or("Overset / unknown page")));
                        ui.label(egui::RichText::new(&s.original).strikethrough().color(t.text_dim));
                        ui.label(egui::RichText::new(&s.replacement).color(t.accent));
                        ui.label(&s.reason);
                        if s.status == Status::Pending {
                            ui.horizontal(|ui| {
                                if ui.button("Locate Original").clicked() {
                                    action(app, "ai.suggestion.show", json!({"id":s.id}));
                                }
                                if ui.add_enabled(!running, egui::Button::new("Accept")).clicked() {
                                    action(app, "ai.suggestion.accept", json!({"id":s.id}));
                                }
                                if ui.add_enabled(!running, egui::Button::new("Reject")).clicked() {
                                    action(app, "ai.suggestion.reject", json!({"id":s.id}));
                                }
                            });
                        }
                    });
                });
            }
        });
    } else {
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!running && cfg!(not(target_arch = "wasm32")), egui::Button::new("Proofread Selection")).clicked() {
                action(app, "ai.proofreadSelection", json!({}));
            }
            if ui.add_enabled(!running, egui::Button::new("New Conversation")).clicked() {
                action(app, "ai.chat.clear", json!({}));
            }
        });
        let draft = app.ai.drafts.entry(uid).or_default();
        ui.add(
            egui::TextEdit::multiline(draft)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text("Describe the proofreading or revision you need…"),
        );
        let text = draft.clone();
        if ui.add_enabled(!running && !text.trim().is_empty() && cfg!(not(target_arch = "wasm32")), egui::Button::new("Send")).clicked() {
            match app.run("ai.chat.send", json!({"text":text})) {
                Ok(_) => {
                    app.ai.drafts.remove(&uid);
                }
                Err(e) => app.ai.notice = e,
            }
        }
        ui.label(egui::RichText::new("Changes preserve formatting and paragraph breaks. Review and accept to apply them.").small().color(t.text_dim));
        ui.separator();
        egui::ScrollArea::vertical().id_salt(("ai_chat", uid)).auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
            if data.messages.is_empty() {
                ui.label("Select text for proofreading or ask a question.");
            }
            for m in &data.messages {
                ui.group(|ui| {
                    ui.label(egui::RichText::new(if m.role == "user" { "You" } else { "AI Assistant" }).strong());
                    ui.label(&m.text);
                });
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod runtime {
    use super::*;
    use pagenor_ai::native::{self, Cancellation, Settings};
    use std::{
        collections::HashSet,
        sync::mpsc::{self, Receiver, Sender},
        time::Duration,
    };
    pub enum Event {
        Tool { task: u64, name: String, params: Value, reply: Sender<Result<Value, String>> },
        Done { task: u64, result: Result<String, String> },
        Settings { task: u64, result: Result<Settings, String> },
        Models { task: u64, result: Result<Vec<native::Model>, String> },
    }
    pub struct Task {
        id: u64,
        doc: Option<u64>,
        cancel: Cancellation,
        allowed: Option<HashSet<u64>>,
        stories: Vec<(pagenor_doc::StoryId, String)>,
        initial_suggestions: usize,
    }
    pub struct Runtime {
        pub settings: Settings,
        saved: Settings,
        pub key: String,
        pub models: Vec<native::Model>,
        pub task: Option<Task>,
        tx: Sender<Event>,
        rx: Receiver<Event>,
        next: u64,
        seen: HashSet<u64>,
        workers: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        load_error: Option<String>,
    }
    impl Default for Runtime {
        fn default() -> Self {
            let (tx, rx) = mpsc::channel();
            let (settings, load_error) = match Settings::load() {
                Ok(s) => (s, None),
                Err(e) => (Settings::default(), Some(e)),
            };
            Self {
                saved: settings.clone(),
                settings,
                key: String::new(),
                models: vec![],
                task: None,
                tx,
                rx,
                next: 0,
                seen: HashSet::new(),
                workers: Default::default(),
                load_error,
            }
        }
    }
    struct WorkerPermit(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl WorkerPermit {
        fn acquire(workers: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Result<Self, String> {
            use std::sync::atomic::Ordering;
            let mut count = workers.load(Ordering::SeqCst);
            loop {
                if count >= 4 {
                    return Err("Previous AI requests are still finishing; retry shortly".into());
                }
                match workers.compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(actual) => count = actual,
                }
            }
            Ok(Self(workers))
        }
    }
    impl Drop for WorkerPermit {
        fn drop(&mut self) {
            self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    fn spawn(app: &mut DesignApp, f: impl FnOnce() + Send + 'static) -> Result<(), String> {
        let permit = match WorkerPermit::acquire(app.ai.native.workers.clone()) {
            Ok(p) => p,
            Err(e) => {
                stop(app, "Task could not start");
                return Err(e);
            }
        };
        match std::thread::Builder::new().name("pagenor-ai".into()).spawn(move || {
            let _permit = permit;
            f();
        }) {
            Ok(_) => Ok(()),
            Err(_) => {
                stop(app, "Task could not start");
                Err("Could not start the AI worker".into())
            }
        }
    }
    fn start(app: &mut DesignApp, doc: Option<u64>, allowed: Option<HashSet<u64>>) -> Result<(u64, Cancellation, Sender<Event>), String> {
        if busy(app) {
            return Err("Wait for the current task or stop it first.".into());
        }
        app.ai.native.next = app.ai.native.next.checked_add(1).filter(|n| *n < pagenor_doc::MAX_NEXT_ID).ok_or("AI task IDs exhausted")?;
        let id = app.ai.native.next;
        let cancel = Cancellation::default();
        let stories = if doc.is_some() {
            let st = app.session.active().ok_or("Document is closed")?;
            if Some(st.uid) != doc || st.doc.stories.len() > 4096 {
                return Err("AI document capture exceeds its limit".into());
            }
            let mut bytes = 0usize;
            for story in st.doc.stories.values() {
                bytes = bytes.saturating_add(story.text.len());
                if story.text.len() > pagenor_doc::ai::MAX_STORY_BYTES || bytes > 16 << 20 {
                    return Err("AI document capture exceeds its text limit".into());
                }
            }
            if let Some(a) = &st.doc.ai {
                a.check().map_err(|e| e.to_string())?;
            }
            st.doc.stories.iter().map(|(id, s)| (*id, s.text.clone())).collect()
        } else {
            vec![]
        };
        let initial_suggestions = app.session.active().and_then(|st| st.doc.ai.as_ref()).map_or(0, |a| a.suggestions.len());
        app.ai.native.task = Some(Task { id, doc, cancel: cancel.clone(), allowed, stories, initial_suggestions });
        app.ai.notice = "Working…".into();
        Ok((id, cancel, app.ai.native.tx.clone()))
    }
    pub fn poll(app: &mut DesignApp, ctx: &egui::Context) {
        app.ai.drafts.retain(|uid, _| app.session.documents().iter().any(|d| d.uid == *uid));
        app.ai.native.seen.retain(|uid| app.session.documents().iter().any(|d| d.uid == *uid));
        if let Some(e) = app.ai.native.load_error.take() {
            app.ai.notice = e;
        }
        if let Some(uid) = app.session.active().map(|s| s.uid)
            && app.ai.native.seen.insert(uid)
        {
            if app.session.active().and_then(|s| s.doc.ai.as_ref()).is_some_and(|a| a.task_status == "running") {
                let _ = app.run("ai.task.status", json!({"status":"interrupted"}));
            }
            let _ = app.run("ai.suggestion.list", json!({}));
        }
        let invalid = app.ai.native.task.as_ref().is_some_and(|t| {
            t.doc.is_some()
                && (t.doc != app.session.active().map(|s| s.uid)
                    || app.session.active().is_some_and(|st| t.stories.iter().any(|(id, text)| st.doc.story(*id).is_none_or(|s| s.text != *text))))
        });
        if invalid {
            stop(app, "Document switched, closed or text changed; the task was stopped.");
        }
        while let Ok(event) = app.ai.native.rx.try_recv() {
            let id = match &event {
                Event::Tool { task, .. } | Event::Done { task, .. } | Event::Settings { task, .. } | Event::Models { task, .. } => *task,
            };
            if app.ai.native.task.as_ref().is_none_or(|t| t.id != id) {
                continue;
            }
            match event {
                Event::Tool { name, params, reply, .. } => {
                    let allowed = app.ai.native.task.as_ref().and_then(|t| t.allowed.as_ref());
                    let permitted =
                        allowed.is_none_or(|ids| name == "propose_text_change" && params["snapshot"].as_u64().is_some_and(|id| ids.contains(&id)));
                    let result = if !permitted {
                        Err("Selection proofreading may only propose changes to its captured snapshots.".into())
                    } else {
                        match name.as_str() {
                            "get_text_frames" => app.run("ai.text.frames", params),
                            "read_text" => app.run("ai.text.read", params),
                            "propose_text_change" => app.run("ai.suggestion.propose", params),
                            _ => Err("Tool is not permitted".into()),
                        }
                    };
                    let _ = reply.send(result);
                }
                Event::Done { result, .. } => {
                    let is_doc = app.ai.native.task.as_ref().is_some_and(|t| t.doc.is_some());
                    let before = app.ai.native.task.as_ref().map_or(0, |t| t.initial_suggestions);
                    let after = app.session.active().and_then(|st| st.doc.ai.as_ref()).map_or(0, |a| a.suggestions.len());
                    let generated = after.saturating_sub(before);
                    app.ai.native.task = None;
                    if is_doc {
                        let _ = app.run("ai.task.status", json!({"status":if result.is_ok(){"completed"}else{"failed"}}));
                    }
                    match result {
                        Ok(text) => {
                            if is_doc {
                                let _ = app.run("ai.chat.append", json!({"role":"assistant","text":text}));
                            }
                            app.ai.notice = if !is_doc {
                                text
                            } else if generated == 0 {
                                "No changes were proposed. Read the response below.".into()
                            } else {
                                format!("Generated {generated} suggestions. Review before accepting.")
                            };
                        }
                        Err(e) => {
                            app.ai.notice = e;
                        }
                    }
                }
                Event::Settings { result, .. } => {
                    app.ai.native.task = None;
                    match result {
                        Ok(s) => {
                            app.ai.native.saved = s.clone();
                            app.ai.native.settings = s;
                            app.ai.native.models.clear();
                            app.ai.notice = if !app.ai.native.settings.account.is_empty() {
                                "Signed in. Pagenor will use your ChatGPT plan for AI requests. Load an available model.".into()
                            } else {
                                "Account settings updated.".into()
                            };
                        }
                        Err(e) => app.ai.notice = e,
                    }
                }
                Event::Models { result, .. } => {
                    app.ai.native.task = None;
                    match result {
                        Ok(m) => {
                            app.ai.native.models = m;
                            app.ai.notice = "Model catalogue updated.".into();
                        }
                        Err(e) => app.ai.notice = e,
                    }
                }
            }
        }
        if busy(app) {
            ctx.request_repaint_after(Duration::from_millis(80));
        }
    }
    fn stop(app: &mut DesignApp, notice: &str) {
        if let Some(task) = app.ai.native.task.take() {
            task.cancel.cancel();
            if let Some(uid) = task.doc {
                // Mark the original document, even after a tab switch. No selection switch required.
                let _ = app.run("ai.task.status", json!({"document":uid,"status":"interrupted"}));
            }
        }
        app.ai.notice = notice.into();
    }
    pub fn command(app: &mut DesignApp, id: &str, p: &Value) -> Result<Value, String> {
        if id == "ai.task.inspect" {
            return Ok(
                json!({"busy":busy(app),"notice":app.ai.notice,"models":app.ai.native.models,"task":app.ai.native.task.as_ref().map(|t|t.id)}),
            );
        }
        if id == "ai.task.stop" {
            stop(app, "Stopped; existing suggestions remain available.");
            return Ok(Value::Null);
        }
        if id == "ai.settings.get" {
            return Ok(json!(app.ai.native.saved));
        }
        if busy(app) {
            return Err("Wait for the current task or stop it first.".into());
        }
        match id {
            "ai.settings.save" => {
                let mut s = app.ai.native.settings.clone();
                for (key, target) in
                    [("provider", &mut s.provider), ("endpoint", &mut s.endpoint), ("model", &mut s.model), ("account", &mut s.account)]
                {
                    if let Some(v) = p[key].as_str() {
                        *target = v.into();
                    }
                }
                s.save(p["apiKey"].as_str())?;
                app.ai.native.saved = s.clone();
                app.ai.native.settings = s;
                app.ai.native.key.clear();
                app.ai.notice = "Settings saved.".into();
                Ok(Value::Null)
            }
            "ai.auth.usage" => {
                web_usage()?;
                Ok(Value::Null)
            }
            "ai.auth.login" | "ai.auth.logout" | "ai.models" | "ai.connection.test" => {
                let (task, cancel, tx) = start(app, None, None)?;
                let mut s = if id.starts_with("ai.auth.") { app.ai.native.settings.clone() } else { app.ai.native.saved.clone() };
                if p["newAccount"] == true {
                    s.account.clear();
                }
                let id = id.to_string();
                spawn(app, move || {
                    let e = match id.as_str() {
                        "ai.auth.login" => Event::Settings { task, result: native::login(s, &cancel) },
                        "ai.auth.logout" => Event::Settings { task, result: native::logout(s) },
                        "ai.models" => Event::Models { task, result: native::models(&s) },
                        _ => Event::Done { task, result: native::test_connection(&s, &cancel) },
                    };
                    let _ = tx.send(e);
                })?;
                Ok(json!({"task":task}))
            }
            "ai.chat.send" | "ai.proofreadSelection" => {
                let st = app.session.active().ok_or("Open a document first")?;
                let uid = st.uid;
                let settings = app.ai.native.saved.clone();
                if settings.model.trim().is_empty() {
                    return Err("Select a model in Settings and save it first.".into());
                }
                let proof = id == "ai.proofreadSelection";
                let history = if proof { vec![] } else { st.doc.ai.as_ref().map(|a| a.messages.clone()).unwrap_or_default() };
                let mut prompts = vec![];
                let mut allowed = None;
                if proof {
                    let t = st
                        .selection
                        .text
                        .filter(|t| !t.is_caret() && t.cell.is_none())
                        .ok_or("Select ordinary body text with the Type tool first.")?;
                    let captured = st.doc.story(t.story).and_then(|s| s.text.get(t.range())).ok_or("Invalid selection")?;
                    if captured.len() > 1_000_000 {
                        return Err("Selection exceeds the 1 MB proofreading limit".into());
                    }
                    let text = captured.to_string();
                    if text.trim().is_empty() {
                        return Err("Selection has no body text".into());
                    }
                    let mut ids = HashSet::new();
                    for range in pagenor_ai::batches(&text, 16000) {
                        let offset = text[..range.start].chars().count();
                        let length = text[range.clone()].chars().count();
                        let snap = app.run("ai.text.read", json!({"offset":offset,"length":length}))?;
                        ids.insert(snap["snapshot"].as_u64().ok_or("Missing text snapshot ID")?);
                        prompts.push(format!("Proofread punctuation and clear spelling errors in this selection snapshot only. Preserve meaning, numbers, names, paragraphs and language conventions. Explain uncertain corrections for human review. Do not read other content. Use propose_text_change; copy original exactly and include enough identical context in original and replacement to make the quote unique. The host computes positions; do not supply offsets. Snapshot: {}",snap));
                    }
                    allowed = Some(ids);
                } else {
                    let text = p["text"].as_str().filter(|s| !s.trim().is_empty()).ok_or("Enter a message")?;
                    prompts.push(text.to_string());
                }
                let (task, cancel, tx) = start(app, Some(uid), allowed)?;
                let user = if proof { "Proofread punctuation and spelling in the selection".to_string() } else { prompts[0].clone() };
                if let Err(e) =
                    app.run("ai.chat.append", json!({"role":"user","text":user})).and_then(|_| app.run("ai.task.status", json!({"status":"running"})))
                {
                    stop(app, "Task could not start");
                    return Err(e);
                }
                app.ui.ai.open = true;
                app.ui.ai.tab = "chat".into();
                spawn(app, move || {
                    let result = (|| {
                        let mut summaries = vec![];
                        for prompt in prompts {
                            cancel.check()?;
                            let summary = native::conversation(&settings, &history, &prompt, &cancel, proof, |name, params| {
                                let (reply, rx) = mpsc::channel();
                                tx.send(Event::Tool { task, name: name.into(), params, reply }).map_err(|_| "Window closed")?;
                                loop {
                                    cancel.check()?;
                                    match rx.recv_timeout(Duration::from_millis(100)) {
                                        Ok(v) => return v,
                                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                                        Err(_) => return Err("Task ended".into()),
                                    }
                                }
                            })?;
                            summaries.push(summary);
                            if summaries.iter().fold(0usize, |n, s| n.saturating_add(s.len())) > 1_000_000 {
                                return Err("Proofreading response exceeds the chat limit".into());
                            }
                        }
                        Ok(summaries.join("\n"))
                    })();
                    let _ = tx.send(Event::Done { task, result });
                })?;
                Ok(json!({"task":task}))
            }
            "ai.report.export" => {
                let html = app.run("ai.report.html", json!({}))?.as_str().ok_or("Could not generate the report")?.to_string();
                let st = app.session.active().ok_or("No document")?;
                let default =
                    st.path.as_ref().map(|p| std::path::Path::new(p).with_extension("proofreading-report.html").to_string_lossy().to_string());
                let path = p["path"]
                    .as_str()
                    .map(str::to_owned)
                    .or(default)
                    .or_else(|| app.services.pick_save.as_mut().and_then(|pick| pick("proofreading-report.html")))
                    .ok_or("No save path selected")?;
                use std::io::Write;
                let base = std::path::PathBuf::from(path);
                let mut path = base.clone();
                let mut n = 1usize;
                loop {
                    if n > 1000 {
                        return Err("Report filename collision limit reached".into());
                    }
                    match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                        Ok(mut f) => {
                            if let Err(e) = f.write_all(html.as_bytes()) {
                                drop(f);
                                let _ = std::fs::remove_file(&path);
                                return Err(e.to_string());
                            }
                            break;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            n += 1;
                            path = base.with_file_name(format!("{}_{}.html", base.file_stem().unwrap_or_default().to_string_lossy(), n));
                        }
                        Err(e) => return Err(e.to_string()),
                    }
                }
                app.ai.notice = format!("Report saved: {}", path.display());
                Ok(json!({"path":path}))
            }
            _ => Err(format!("unknown AI command: {id}")),
        }
    }
    fn web_usage() -> Result<(), String> {
        native::open_usage()
    }

    impl Drop for Runtime {
        fn drop(&mut self) {
            if let Some(t) = &self.task {
                t.cancel.cancel();
            }
        }
    }
    pub fn settings(app: &mut DesignApp, ui: &mut egui::Ui) {
        let running = busy(app);
        ui.add_enabled_ui(!running, |ui| {
            let s = &mut app.ai.native.settings;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut s.provider, "compatible".into(), "Compatible API");
                ui.selectable_value(&mut s.provider, "chatgpt".into(), "ChatGPT Sign-in");
            });
            if s.provider == "compatible" {
                ui.label("Endpoint");
                ui.text_edit_singleline(&mut s.endpoint);
                ui.label("API key (stored in system credentials)");
                ui.add(egui::TextEdit::singleline(&mut app.ai.native.key).password(true).hint_text("Leave blank to keep the existing key"));
            } else {
                ui.label("AI requests will use your ChatGPT plan after authorization.");
                if !s.account.is_empty() && s.accounts.iter().any(|a| a.id == s.account && a.sharing) {
                    ui.label("Using your ChatGPT plan");
                }
                let old = s.account.clone();
                egui::ComboBox::from_id_salt("ai_account")
                    .selected_text(s.accounts.iter().find(|a| a.id == s.account).map(|a| a.label.as_str()).unwrap_or("Select Account"))
                    .show_ui(ui, |ui| {
                        for a in &s.accounts {
                            ui.selectable_value(&mut s.account, a.id.clone(), &a.label);
                        }
                    });
                if old != s.account {
                    s.model.clear();
                    app.ai.native.models.clear();
                }
                if s.accounts.iter().any(|a| a.id == s.account && !a.sharing) {
                    ui.label("This account has not granted plan inference permission. Authorize again.");
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Continue with ChatGPT").clicked() {
                        action(app, "ai.auth.login", json!({}));
                    }
                    if ui.button("Add Account").clicked() {
                        action(app, "ai.auth.login", json!({"newAccount":true}));
                    }
                    if ui.button("Sign Out").clicked() {
                        action(app, "ai.auth.logout", json!({}));
                    }
                    if ui.button("Manage Usage").clicked() {
                        action(app, "ai.auth.usage", json!({}));
                    }
                });
            }
            ui.separator();
            ui.label("Model");
            ui.text_edit_singleline(&mut app.ai.native.settings.model);
            if !app.ai.native.models.is_empty() {
                egui::ComboBox::from_id_salt("ai_models").selected_text("Choose an available model").show_ui(ui, |ui| {
                    for m in &app.ai.native.models {
                        ui.selectable_value(&mut app.ai.native.settings.model, m.id.clone(), &m.label);
                    }
                });
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Save Settings").clicked() {
                    let mut p = json!({});
                    if !app.ai.native.key.is_empty() {
                        p["apiKey"] = json!(app.ai.native.key);
                    }
                    action(app, "ai.settings.save", p);
                }
                if ui.button("Load Models").clicked() {
                    action(app, "ai.models", json!({}));
                }
                if ui.button("Test Connection").clicked() {
                    action(app, "ai.connection.test", json!({}));
                }
            });
            ui.separator();
            ui.label("Chat and suggestions are saved in the document and included when you share it.");
            if ui.button("Clear document AI records").clicked() {
                action(app, "ai.records.clear", json!({}));
            }
        });
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn app() -> DesignApp {
            let mut app = DesignApp::new(pagenor_engine::Session::new(), Default::default());
            app.run("file.new", json!({})).unwrap();
            app.run("frame.create", json!({"rect":[0,0,200,200],"content":"text","text":"hello"})).unwrap();
            app
        }
        #[test]
        fn worker_limit_recovers_when_cancelled_workers_finish() {
            let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut permits = Vec::new();
            for _ in 0..4 {
                permits.push(WorkerPermit::acquire(count.clone()).unwrap());
            }
            assert!(WorkerPermit::acquire(count.clone()).is_err());
            permits.pop();
            assert!(WorkerPermit::acquire(count.clone()).is_ok());
            drop(permits);
            assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 0);
        }
        #[test]
        fn report_collision_preserves_existing_file() {
            let mut app = app();
            let dir = std::env::temp_dir().join(format!(
                "pagenor-ai-report-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ));
            std::fs::create_dir(&dir).unwrap();
            let path = dir.join("review.html");
            std::fs::write(&path, "existing report").unwrap();
            let result = command(&mut app, "ai.report.export", &json!({"path":path.to_str().unwrap()})).unwrap();
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "existing report");
            assert!(std::fs::read_to_string(result["path"].as_str().unwrap()).unwrap().contains("<!doctype html>"));
            std::fs::remove_dir_all(dir).unwrap();
        }
        #[test]
        fn closed_document_drafts_are_released() {
            let mut app = app();
            let uid = app.session.active().unwrap().uid;
            app.ai.drafts.insert(uid, "private draft".into());
            app.run("file.new", json!({})).unwrap();
            app.run("file.close", json!({"index":0})).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(!app.ai.drafts.contains_key(&uid));
        }
        #[test]
        fn close_keeps_task_but_switch_cancels_and_drops_late_results() {
            let mut app = app();
            let uid = app.session.active().unwrap().uid;
            let (task, cancel, tx) = start(&mut app, Some(uid), None).unwrap();
            app.run("window.ai", json!({"open":false})).unwrap();
            assert!(busy(&app));
            assert!(cancel.check().is_ok());
            app.run("file.new", json!({})).unwrap();
            tx.send(Event::Done { task, result: Ok("late response".into()) }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(!busy(&app));
            assert!(cancel.check().is_err());
            assert!(app.session.active().unwrap().doc.ai.is_none());
        }
        #[test]
        fn worker_cannot_accept_and_scope_rejects_other_snapshots() {
            let mut app = app();
            let uid = app.session.active().unwrap().uid;
            let (task, _, tx) = start(&mut app, Some(uid), Some(HashSet::from([42]))).unwrap();
            let (reply, rx) = mpsc::channel();
            tx.send(Event::Tool { task, name: "ai.suggestion.accept".into(), params: json!({"id":1}), reply }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(rx.try_recv().unwrap().is_err());
            let (reply, rx) = mpsc::channel();
            tx.send(Event::Tool { task, name: "propose_text_change".into(), params: json!({"snapshot":43}), reply }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(rx.try_recv().unwrap().is_err());
        }
        #[test]
        fn captured_selection_proposal_survives_review_and_undo() {
            let mut app = app();
            let story = app.session.active().unwrap().doc.stories.keys().next().unwrap().0;
            let snap = app.run("ai.text.read", json!({"story":story})).unwrap();
            let uid = app.session.active().unwrap().uid;
            let (task, _, tx) = start(&mut app, Some(uid), Some(HashSet::from([snap["snapshot"].as_u64().unwrap()]))).unwrap();
            let (reply, rx) = mpsc::channel();
            tx.send(Event::Tool {
                task,
                name: "propose_text_change".into(),
                params: json!({"snapshot":snap["snapshot"],"offset":0,"original":"hello","replacement":"Hello","reason":"首字母"}),
                reply,
            })
            .unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(rx.try_recv().unwrap().is_ok());
            tx.send(Event::Done { task, result: Ok("请审核".into()) }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(app.ai.notice.contains("1 suggestions"));
            app.run("ai.suggestion.acceptAll", json!({})).unwrap();
            assert_eq!(app.session.active().unwrap().doc.stories.values().next().unwrap().text, "Hello");
            app.run("edit.undo", json!({})).unwrap();
            assert_eq!(app.session.active().unwrap().doc.stories.values().next().unwrap().text, "hello");
            let (task, _, tx) = start(&mut app, Some(uid), None).unwrap();
            tx.send(Event::Done { task, result: Ok("没有建议".into()) }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(app.ai.notice.contains("No changes"));
        }
        #[test]
        fn external_edit_cancels_before_queued_tool_execution() {
            let mut app = app();
            let uid = app.session.active().unwrap().uid;
            let (task, cancel, tx) = start(&mut app, Some(uid), None).unwrap();
            app.run("text.insert", json!({"text":"changed"})).unwrap();
            let (reply, rx) = mpsc::channel();
            tx.send(Event::Tool { task, name: "read_text".into(), params: json!({}), reply }).unwrap();
            poll(&mut app, &egui::Context::default());
            assert!(cancel.check().is_err());
            assert!(rx.try_recv().is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_persisted_window_geometry_is_normalized() {
        let mut state = WindowState { position: [f32::NAN, 0.0], size: [f32::INFINITY, -20.0], tab: "invalid".into(), ..Default::default() };
        state.normalize();
        assert_eq!(state.position, WindowState::default().position);
        assert_eq!(state.size, WindowState::default().size);
        assert_eq!(state.tab, "chat");
        state.size = [-1.0, 1e30];
        state.normalize();
        assert_eq!(state.size, [320.0, 4096.0]);
    }
    #[test]
    fn window_state_survives_close_and_ui_serialization() {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), Default::default());
        app.run("window.ai", json!({})).unwrap();
        assert!(app.ui.ai.open);
        assert!(!app.ui.ai.floating);
        app.run("window.ai.layout", json!({"floating":true,"position":[120,160],"tab":"suggestions"})).unwrap();
        app.run("window.ai", json!({"open":false})).unwrap();
        let saved = serde_json::to_vec(&app.ui).unwrap();
        app.ui = serde_json::from_slice(&saved).unwrap();
        app.run("window.ai", json!({})).unwrap();
        assert!(app.ui.ai.floating);
        assert_eq!(app.ui.ai.position, [120.0, 160.0]);
        assert_eq!(app.ui.ai.tab, "suggestions");
        app.run("window.ai.layout", json!({"floating":false})).unwrap();
        assert!(!app.ui.ai.floating);
    }
}
