//! OS text composition for the custom document canvas. Preedit never edits the document.

use egui::{Context, Event, ImeEvent};
use pagenor_doc::TextSel;
use serde_json::json;

use crate::DesignApp;

const MAX_PREEDIT_BYTES: usize = 16 * 1024;
const MAX_COMMIT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Target {
    uid: u64,
    revision: u64,
    selection: TextSel,
    pane: u8,
    viewport: egui::ViewportId,
}

#[derive(Default)]
pub(crate) struct Composition {
    target: Option<Target>,
    pub text: String,
    inline: Option<Inline>,
}

impl Composition {
    pub fn active(&self) -> bool {
        !self.text.is_empty()
    }

    fn clear(&mut self) {
        self.target = None;
        self.text.clear();
        self.inline = None;
    }
}

fn target(app: &DesignApp, ctx: &Context) -> Option<Target> {
    let st = app.session.active()?;
    Some(Target { uid: st.uid, revision: st.revision, selection: st.selection.text?, pane: app.pane, viewport: ctx.viewport_id() })
}

impl DesignApp {
    /// Apply already-delivered canvas text before Quit checkpoints the document. Native
    /// menu delivery precedes the canvas paint, so a final IME commit can share this pass.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn finish_canvas_input_before_quit(&mut self, ctx: &Context) {
        if !self.session.wants_text()
            || self.ui.home
            || self.ui.dialog.is_some()
            || self.ui.palette.is_some()
            || ctx.text_edit_focused()
            || self.pane != self.focus_pane
            || !ctx.input(|i| i.focused)
        {
            return;
        }
        self.validate_canvas_composition(ctx);
        let events = ctx.input_mut(|i| {
            let text = i.events.iter().filter(|e| matches!(e, Event::Ime(_) | Event::Text(_))).cloned().collect::<Vec<_>>();
            i.events.retain(|e| !matches!(e, Event::Ime(_) | Event::Text(_)));
            text
        });
        for event in events {
            match event {
                Event::Ime(event) => self.canvas_ime_event(ctx, &event),
                Event::Text(text) if !self.canvas_ime.active() => {
                    let _ = self.run("text.insert", json!({"text":text}));
                }
                _ => {}
            }
        }
    }

    /// Invalidate composition before global shortcuts, including when Home/dialogs take focus.
    pub(crate) fn validate_canvas_composition(&mut self, ctx: &Context) {
        if self.canvas_ime.target.is_some()
            && (self.canvas_ime.target != target(self, ctx)
                || self.ui.home
                || self.ui.dialog.is_some()
                || self.ui.palette.is_some()
                || ctx.text_edit_focused()
                || !ctx.input(|i| i.focused))
        {
            self.cancel_canvas_composition(ctx);
            ctx.input_mut(|i| i.events.retain(|e| !matches!(e, Event::Ime(_))));
        }
    }

    pub(crate) fn cancel_canvas_composition(&mut self, ctx: &Context) {
        if self.canvas_ime.active() {
            ctx.memory_mut(|m| m.interrupt_ime());
        }
        self.canvas_ime.clear();
    }

    /// Returns true while composition owns editing keys (candidate selection/confirmation).
    pub(crate) fn canvas_composing(&self, ctx: &Context) -> bool {
        self.canvas_ime.active()
            || ctx.input(|i| {
                i.events.iter().any(|e| matches!(e, Event::Ime(ImeEvent::Preedit { text, .. } | ImeEvent::Commit(text)) if !text.is_empty()))
            })
    }

    pub(crate) fn canvas_ime_event(&mut self, ctx: &Context, event: &ImeEvent) {
        let current = target(self, ctx);
        if self.canvas_ime.target.is_some() && self.canvas_ime.target != current {
            self.cancel_canvas_composition(ctx);
            return; // A queued commit belongs to the previous caret/document.
        }
        match event {
            ImeEvent::Preedit { text, .. } => {
                if text.len() > MAX_PREEDIT_BYTES {
                    self.cancel_canvas_composition(ctx);
                    self.status("Input-method composition exceeds the size limit");
                } else if text.is_empty() {
                    self.canvas_ime.clear();
                } else {
                    self.canvas_ime.target = current;
                    if self.canvas_ime.text != *text {
                        self.canvas_ime.inline = None;
                        self.canvas_ime.text.clone_from(text);
                    }
                }
            }
            ImeEvent::Commit(text) => {
                self.canvas_ime.clear();
                if text.len() > MAX_COMMIT_BYTES {
                    self.status("Input-method text exceeds the size limit");
                } else if current.is_some() && !text.is_empty() {
                    // One existing command: selection replacement, typing style, undo and recovery.
                    let _ = self.run("text.insert", json!({"text": text}));
                }
            }
            ImeEvent::DeleteSurrounding { before_chars, after_chars } => {
                let Some(st) = self.session.active() else { return };
                let Some(mut sel) = st.selection.text else { return };
                let Some(story) = st.doc.text_story(sel.story, sel.cell) else { return };
                let at = pagenor_doc::story::floor_char_boundary(&story.text, sel.focus);
                let start = story.text.get(..at).and_then(|s| s.char_indices().rev().nth(before_chars.saturating_sub(1))).map_or(0, |(i, _)| i);
                let start = if *before_chars == 0 { at } else { start };
                let end = story.text.get(at..).and_then(|s| s.char_indices().nth(*after_chars)).map_or(story.len(), |(i, _)| at + i);
                if start < end {
                    self.cancel_canvas_composition(ctx);
                    sel.anchor = start;
                    sel.focus = end;
                    // Preserve cell/note addressing; undo uses the existing text-edit command.
                    if let Ok(st) = self.session.doc_mut() {
                        st.selection.text = Some(sel);
                    }
                    let _ = self.run("text.insert", json!({"text": "", "raw": true}));
                }
            }
            _ => {}
        }
    }
}

/// A copy-on-write document and visual caret containing unfinished composition.
/// It is rendered in the normal text flow and never enters the live session.
#[derive(Clone)]
pub(crate) struct Inline {
    pub doc: std::sync::Arc<pagenor_doc::Document>,
    pub selection: TextSel,
}

impl DesignApp {
    pub(crate) fn inline_composition(&mut self) -> Option<Inline> {
        if !self.canvas_ime.active() {
            return None;
        }
        if self.canvas_ime.inline.is_none() {
            let (doc, selection) = self.session.transient_text_insertion(&self.canvas_ime.text).ok()?;
            self.canvas_ime.inline = Some(Inline { doc, selection: selection.text? });
        }
        self.canvas_ime.inline.clone()
    }

    pub(crate) fn composition_selection(&self) -> Option<TextSel> {
        self.canvas_ime.inline.as_ref().map(|p| p.selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TELUGU: &str = "ఉదయం ఫాంట్\u{200c}";

    fn frame(app: &mut DesignApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1440.0, 900.0))),
                events,
                ..Default::default()
            },
            |ui| {
                app.logic(ctx);
                app.ui(ui);
            },
        );
        out.textures_delta.clear();
        out
    }

    fn setup(text: &str) -> (DesignApp, Context, pagenor_doc::StoryId) {
        let mut app = DesignApp::new(pagenor_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();
        let r = app.run("frame.create", json!({"rect":[72,72,450,220],"content":"text","text":text,"caret":true})).unwrap();
        app.session.set_tool("type");
        let sid = pagenor_doc::StoryId(r["story"].as_u64().unwrap());
        let ctx = Context::default();
        for _ in 0..4 {
            frame(&mut app, &ctx, vec![]);
        }
        (app, ctx, sid)
    }

    fn preedit(text: &str) -> Event {
        Event::Ime(ImeEvent::Preedit { text: text.into(), active_range_chars: Some(0..text.chars().count()) })
    }

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> Event {
        Event::Key { key, physical_key: Some(key), pressed: true, repeat: false, modifiers }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_quit_key_release_after_ime_commit_prompts_and_preserves_text_once() {
        let (mut app, ctx, sid) = setup("Saved");
        let dir = std::env::temp_dir().join(format!("pagenor-ime-quit-shortcut-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        app.session.recovery_dir = Some(dir.clone());
        app.run("file.saveAs", json!({"path":dir.join("saved.pgnr")})).unwrap();
        app.native_menu = true;
        app.native_shortcuts.insert("app.quit".into());
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        let release = Event::Key {
            key: egui::Key::Q,
            physical_key: Some(egui::Key::Q),
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers { mac_cmd: true, command: true, ..Default::default() },
        };
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit(TELUGU.into())), release]);
        assert_eq!(app.ui.dialog.as_ref().map(|d| d.id.as_str()), Some("closeDocument"), "a swallowed key-down must not swallow Quit");
        assert_eq!(text(&app, sid).matches(TELUGU).count(), 1);
        let mut reopened = pagenor_engine::Session::new();
        assert_eq!(pagenor_engine::recovery::open(&mut reopened, &dir).unwrap().len(), 1);
        assert_eq!(reopened.doc().unwrap().doc.story(sid).unwrap().text, text(&app, sid));
        app.ui.dialog.as_mut().unwrap().fields.insert("discard".into(), json!(true));
        app.run("app.quit", json!({})).unwrap(); // A later native menu notification for the same shortcut.
        assert_eq!(app.ui.dialog.as_ref().unwrap().fields["discard"], true, "duplicate delivery must not reset the user's save choice");
        app.cancel_quit();
        app.ui.dialog = None;
        frame(&mut app, &ctx, vec![Event::Text("Z".into())]);
        assert!(text(&app, sid).ends_with('Z'));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn quit_shortcut_during_preedit_keeps_candidate_and_layout_keys_owned_by_input() {
        let (mut app, ctx, sid) = setup("Saved");
        let dir = std::env::temp_dir().join(format!("pagenor-ime-quit-key-ownership-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        app.run("file.saveAs", json!({"path":dir.join("saved.pgnr")})).unwrap();
        frame(&mut app, &ctx, vec![preedit(TELUGU)]);
        for event in [
            key(egui::Key::ArrowDown, egui::Modifiers::NONE),
            key(egui::Key::Enter, egui::Modifiers::NONE),
            key(egui::Key::Q, egui::Modifiers::NONE),
            key(egui::Key::Q, egui::Modifiers::ALT),
        ] {
            let mut release = event.clone();
            if let Event::Key { pressed, .. } = &mut release {
                *pressed = false;
            }
            let output = frame(&mut app, &ctx, vec![event, release]);
            assert!(output.platform_output.ime.is_some());
            assert!(!output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::Close));
            assert_eq!(text(&app, sid), "Saved");
        }
        let output = frame(&mut app, &ctx, vec![key(egui::Key::Q, egui::Modifiers::COMMAND)]);
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::Close));
        assert!(output.platform_output.ime.is_none());
        assert_eq!(text(&app, sid), "Saved", "unfinished preedit is not silently committed by Quit");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_quit_menu_checkpoints_same_pass_text_and_preserves_undo() {
        let (mut app, ctx, sid) = setup("Saved");
        let dir = std::env::temp_dir().join(format!("pagenor-ime-native-quit-commit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        app.session.recovery_dir = Some(dir.clone());
        let path = dir.join("saved.pgnr");
        app.run("file.saveAs", json!({"path":path})).unwrap();
        let saved = std::fs::read(&path).unwrap();
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1440.0, 900.0))),
            events: vec![preedit(""), Event::Ime(ImeEvent::Commit(TELUGU.into()))],
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            crate::menus::activate_native(&mut app, &ctx, "app.quit", &serde_json::Value::Null);
            app.logic(&ctx);
            app.ui(ui);
        });
        output.textures_delta.clear();
        assert_eq!(text(&app, sid).matches(TELUGU).count(), 1);
        assert_eq!(app.ui.dialog.as_ref().unwrap().id, "closeDocument");
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        let mut reopened = pagenor_engine::Session::new();
        assert_eq!(pagenor_engine::recovery::open(&mut reopened, &dir).unwrap().len(), 1);
        assert_eq!(reopened.doc().unwrap().doc.story(sid).unwrap().text, text(&app, sid));
        app.cancel_quit();
        app.ui.dialog = None;
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(text(&app, sid), "Saved");
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn text(app: &DesignApp, sid: pagenor_doc::StoryId) -> &str {
        &app.session.doc().unwrap().doc.story(sid).unwrap().text
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn approved_quit_disables_native_input_before_window_teardown() {
        let (mut app, ctx, sid) = setup("Committed text");
        let dir = std::env::temp_dir().join(format!("pagenor-ime-quit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("saved.pgnr");
        app.run("file.saveAs", json!({"path":path})).unwrap();
        let saved = std::fs::read(&path).unwrap();
        assert!(frame(&mut app, &ctx, vec![preedit("ఉ")]).platform_output.ime.is_some());
        app.run("app.quit", json!({})).unwrap();
        let output = frame(&mut app, &ctx, vec![]);
        assert!(output.platform_output.ime.is_none(), "a closing canvas must not re-enable the native input context");
        assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.contains(&egui::ViewportCommand::IMEAllowed(false)));
        assert_eq!(text(&app, sid), "Committed text");
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert_eq!(text(&app, sid), "Committed text", "a late native commit must not edit a closing document");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn clean_window_close_disables_input_and_cancelled_quit_resumes_typing() {
        let (mut app, ctx, sid) = setup("Committed text");
        let dir = std::env::temp_dir().join(format!("pagenor-ime-window-close-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        app.session.recovery_dir = Some(dir.clone());
        app.run("app.quit", json!({})).unwrap();
        assert!(app.ui.dialog.is_some(), "unsaved work must still prompt before quitting");
        assert!(frame(&mut app, &ctx, vec![]).platform_output.ime.is_none());
        app.cancel_quit();
        app.ui.dialog = None;
        let output = frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert!(output.platform_output.ime.is_some(), "cancelling quit must leave typing available");
        assert!(text(&app, sid).contains(TELUGU));
        app.run("file.saveAs", json!({"path": dir.join("saved.pgnr")})).unwrap();
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        let mut raw =
            egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1440.0, 900.0))), ..Default::default() };
        raw.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().events.push(egui::ViewportEvent::Close);
        let mut output = ctx.run_ui(raw, |ui| {
            app.logic(&ctx);
            app.ui(ui);
        });
        output.textures_delta.clear();
        assert!(output.platform_output.ime.is_none());
        let commands = &output.viewport_output[&egui::ViewportId::ROOT].commands;
        assert!(commands.contains(&egui::ViewportCommand::IMEAllowed(false)));
        assert!(commands.contains(&egui::ViewportCommand::CancelClose));
        assert!(app.session.documents().is_empty());
        assert!(app.ui.home && !app.closing, "a document close returns to Welcome without shutting down");
        assert!(!app.canvas_ime.active());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn canvas_enables_os_text_service_and_commits_telugu_once_with_undo() {
        let (mut app, ctx, sid) = setup("Old");
        app.run("text.select", json!({"story":sid,"anchor":0,"focus":3})).unwrap();
        let before = app.session.doc().unwrap().doc.clone();
        let output = frame(&mut app, &ctx, vec![preedit("ఉ")]);
        assert!(output.platform_output.ime.is_some(), "the canvas must enable native composition");
        assert_eq!(app.session.doc().unwrap().doc, before, "preedit is never saved or journalled");
        frame(&mut app, &ctx, vec![preedit(TELUGU)]);
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit(TELUGU.into())), key(egui::Key::Enter, egui::Modifiers::NONE)]);
        assert_eq!(text(&app, sid), TELUGU, "confirmation must not also insert a paragraph");
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, before);
        app.run("edit.redo", json!({})).unwrap();
        assert_eq!(text(&app, sid), TELUGU);
    }

    #[test]
    fn composition_keys_cancellation_and_language_switch_preserve_the_document() {
        let (mut app, ctx, sid) = setup("Original");
        let before = app.session.doc().unwrap().doc.clone();
        app.ui.shortcuts.insert("view.guides".into(), "Alt+G".into());
        let guides = app.ui.guides;
        frame(&mut app, &ctx, vec![preedit("ఉ"), key(egui::Key::Backspace, egui::Modifiers::NONE)]);
        frame(&mut app, &ctx, vec![key(egui::Key::G, egui::Modifiers::ALT), key(egui::Key::Escape, egui::Modifiers::NONE)]);
        assert_eq!(app.ui.guides, guides);
        assert_eq!(app.session.doc().unwrap().doc, before);
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit(String::new()))]);
        assert_eq!(app.session.doc().unwrap().doc, before, "cancel must not delete selected text");
        frame(&mut app, &ctx, vec![Event::Text("é العربية 日本語".into())]);
        assert!(text(&app, sid).contains("é العربية 日本語"), "direct layout text remains supported");
    }

    #[test]
    fn stale_commit_cannot_enter_a_different_document_or_caret() {
        let (mut app, ctx, sid) = setup("Original");
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        app.run("text.select", json!({"story":sid,"anchor":0,"focus":0})).unwrap();
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert_eq!(text(&app, sid), "Original");
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        app.run("file.new", json!({})).unwrap();
        let r = app.run("frame.create", json!({"rect":[72,72,450,220],"content":"text","text":"Other","caret":true})).unwrap();
        let other = pagenor_doc::StoryId(r["story"].as_u64().unwrap());
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert_eq!(text(&app, other), "Other");
    }

    #[test]
    fn option_or_altgr_layout_text_does_not_fire_an_alt_shortcut() {
        let (mut app, ctx, sid) = setup("");
        app.ui.shortcuts.insert("view.guides".into(), "Alt+G".into());
        let guides = app.ui.guides;
        frame(&mut app, &ctx, vec![key(egui::Key::G, egui::Modifiers::ALT), Event::Text("ం".into())]);
        assert_eq!(text(&app, sid), "ం");
        assert_eq!(app.ui.guides, guides);
        frame(
            &mut app,
            &ctx,
            vec![Event::Key { key: egui::Key::G, physical_key: Some(egui::Key::G), pressed: false, repeat: false, modifiers: egui::Modifiers::ALT }],
        );
        frame(&mut app, &ctx, vec![key(egui::Key::G, egui::Modifiers::ALT)]);
        assert_ne!(app.ui.guides, guides, "the shortcut still works when no layout character was produced");
    }

    #[test]
    fn surrounding_deletion_uses_unicode_characters_and_preserves_undo() {
        let (mut app, ctx, sid) = setup("AఉదZ");
        app.run("text.select", json!({"story":sid,"anchor":4,"focus":4})).unwrap();
        let before = app.session.doc().unwrap().doc.clone();
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::DeleteSurrounding { before_chars: 1, after_chars: 1 })]);
        assert_eq!(text(&app, sid), "AZ");
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, before);
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::DeleteSurrounding { before_chars: usize::MAX, after_chars: usize::MAX })]);
        assert_eq!(text(&app, sid), "");
    }

    #[test]
    fn split_canvas_commits_once_and_home_disables_composition() {
        let (mut app, ctx, sid) = setup("");
        app.split = true;
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert_eq!(text(&app, sid), TELUGU);
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        app.run("app.home", json!({})).unwrap();
        let output = frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit("unwanted".into()))]);
        assert!(output.platform_output.ime.is_none());
        assert_eq!(text(&app, sid), TELUGU);
    }

    #[test]
    fn oversized_preedit_is_rejected_without_document_changes() {
        let (mut app, ctx, sid) = setup("Original");
        frame(&mut app, &ctx, vec![preedit(&"x".repeat(MAX_PREEDIT_BYTES + 1))]);
        assert!(!app.canvas_ime.active());
        assert_eq!(text(&app, sid), "Original");
    }

    #[test]
    fn focused_ui_text_field_owns_its_composition_instead_of_the_canvas() {
        let (mut app, ctx, sid) = setup("Original");
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        app.ui.palette = Some(String::new());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert!(ctx.text_edit_focused());
        assert!(!app.canvas_ime.active());
        frame(&mut app, &ctx, vec![preedit("ఉ")]);
        frame(&mut app, &ctx, vec![Event::Ime(ImeEvent::Commit(TELUGU.into()))]);
        assert_eq!(app.ui.palette.as_deref(), Some(TELUGU));
        assert_eq!(text(&app, sid), "Original", "a retained canvas caret must not steal a text field's commit");
    }

    #[test]
    fn continuous_dead_key_typing_keeps_native_composition_and_advances_the_caret() {
        let (mut app, ctx, sid) = setup("");
        let output = frame(&mut app, &ctx, vec![preedit("ల")]);
        assert!(!output.platform_output.ime.unwrap().should_interrupt_composition, "repainting must not reset the OS dead-key state");
        let output = frame(&mut app, &ctx, vec![]);
        assert!(!output.platform_output.ime.unwrap().should_interrupt_composition);
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit("ల".into())), preedit("త")]);
        assert_eq!(text(&app, sid), "ల");
        assert_eq!(app.session.doc().unwrap().selection.text.unwrap().focus, "ల".len());
        assert_eq!(app.canvas_ime.text, "త");
        let output = frame(&mut app, &ctx, vec![key(egui::Key::ArrowRight, egui::Modifiers::NONE)]);
        assert!(!output.platform_output.ime.unwrap().should_interrupt_composition, "candidate navigation must retain native focus");
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit("త".into()))]);
        assert_eq!(text(&app, sid), "లత");
        assert_eq!(app.session.doc().unwrap().selection.text.unwrap().focus, "లత".len());
    }

    #[test]
    fn unfinished_text_is_inline_at_the_typing_size_and_commit_does_not_move_it() {
        let (mut app, ctx, sid) = setup("A");
        app.run("type.char", json!({"attrs":{"size":36}})).unwrap();
        let original = app.session.doc().unwrap().doc.clone();
        let journal = app.session.journal.clone();
        let output = frame(&mut app, &ctx, vec![preedit("త")]);
        let caret = output.platform_output.ime.unwrap().cursor_rect;
        let inline = app.canvas.shown_doc.clone().unwrap();
        assert_eq!(inline.story(sid).unwrap().text, "Aత", "the canvas texture must include the current character in the actual line");
        assert_eq!(app.session.doc().unwrap().doc, original);
        assert_eq!(app.session.journal, journal);
        let style = inline.story(sid).unwrap().format_after(1);
        assert_eq!(style.over.size, Some(36.0), "preedit must use the pending typing style, not a fixed small font");
        let repeated = frame(&mut app, &ctx, vec![]);
        assert!(std::sync::Arc::ptr_eq(&inline, app.canvas.shown_doc.as_ref().unwrap()), "unchanged composition reuses its copy-on-write view");
        assert_eq!(repeated.platform_output.ime.unwrap().cursor_rect, caret);
        let committed = frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit("త".into()))]);
        assert_eq!(app.session.doc().unwrap().doc, inline, "native commit uses the same formatting and layout as the inline text");
        assert_eq!(committed.platform_output.ime.unwrap().cursor_rect, caret, "the character must not jump when committed");
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, original);
    }

    #[test]
    fn inline_replacement_wraps_with_the_paragraph_and_cancellation_restores_the_canvas() {
        let (mut app, ctx, sid) = setup("Keep Old text");
        app.run("text.select", json!({"story":sid,"anchor":5,"focus":8})).unwrap();
        let original = app.session.doc().unwrap().doc.clone();
        let phrase = "An unfinished phrase that is long enough to wrap over several lines in this text frame. ".repeat(5);
        frame(&mut app, &ctx, vec![preedit(&phrase)]);
        let inline = app.canvas.shown_doc.clone().unwrap();
        assert_eq!(inline.story(sid).unwrap().text, format!("Keep {phrase} text"));
        assert!(app.session.cache.get(&inline, sid, None).line_count() > 1);
        assert_eq!(app.session.doc().unwrap().doc, original);
        frame(&mut app, &ctx, vec![preedit(""), Event::Ime(ImeEvent::Commit(String::new()))]);
        assert!(
            std::sync::Arc::ptr_eq(&original, app.canvas.shown_doc.as_ref().unwrap()),
            "cancellation removes inline text from the normal canvas texture"
        );
        assert_eq!(text(&app, sid), "Keep Old text");
    }
}
