use std::sync::Arc;

use pagenor_doc::{Document, ItemId, StoryId};
use serde_json::json;

use super::{Session, compose};

#[derive(Clone, Copy, Debug)]
enum Removal {
    Frame,
    Page,
    Cut,
}

fn primary_chain() -> (Session, StoryId, ItemId) {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages": 1, "primaryTextFrame": true})).unwrap();
    let sid = s.doc().unwrap().doc.settings.primary_story.unwrap();
    s.execute("text.select", &json!({"story": sid.0, "anchor": 0, "focus": 0})).unwrap();
    s.execute("text.insert", &json!({"text": "Words keep coming and the page fills up quickly with them. ".repeat(250)})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let last = *d.story(sid).unwrap().frames.last().unwrap();
    let cs = s.cache.get(&d, sid, None);
    assert!(d.page_count() >= 3);
    assert!(!cs.is_overset());
    assert!(cs.frame(last).is_some_and(|f| !f.range.is_empty()));
    let mut probe = (*d).clone();
    probe.remove_item(last).unwrap();
    assert!(compose::compose_story(&probe, sid, &Default::default()).is_overset(), "removal must reduce occupied flow capacity");
    (s, sid, last)
}

fn remove(s: &mut Session, frame: ItemId, kind: Removal) {
    match kind {
        Removal::Frame => {
            s.execute("edit.clear", &json!({"ids": [frame.0]})).unwrap();
        }
        Removal::Page => {
            let page = s.doc().unwrap().doc.page_of_item(frame).unwrap();
            s.execute("layout.pages.delete", &json!({"pages": [page]})).unwrap();
        }
        Removal::Cut => {
            s.execute("selection.set", &json!({"ids": [frame.0]})).unwrap();
            s.execute("edit.cut", &json!({})).unwrap();
        }
    }
}

fn assert_removed(s: &Session, before: &Document, sid: StoryId, frame: ItemId, pages: usize) {
    let d = &s.doc().unwrap().doc;
    assert!(d.item(frame).is_none());
    assert_eq!(d.page_count(), pages);
    assert_eq!(d.settings.primary_story, Some(sid));
    let story = d.story(sid).unwrap();
    let old = before.story(sid).unwrap();
    assert_eq!(story.text, old.text, "removing a frame must retain its story text");
    let frames: Vec<_> = old.frames.iter().copied().filter(|f| *f != frame).collect();
    assert_eq!(story.frames, frames, "no replacement frame is added");
    assert!(s.cache.get(d, sid, None).is_overset());
    d.check().unwrap();
}

#[test]
fn explicit_primary_removal_survives_unrelated_edits_and_undo_redo() {
    for kind in [Removal::Frame, Removal::Page, Removal::Cut] {
        let (mut s, sid, last) = primary_chain();
        let before = s.doc().unwrap().doc.clone();
        let pages = before.page_count() - usize::from(matches!(kind, Removal::Page));
        remove(&mut s, last, kind);
        assert_removed(&s, &before, sid, last, pages);
        assert!(s.prefs.smart_text_reflow);
        let removed = s.doc().unwrap().doc.clone();
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before), "{kind:?}: undo restores the deleted frame/page");
        s.execute("edit.redo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &removed), "{kind:?}: redo does not reflow");

        // A cold cache must compare against the post-deletion document, not revive autoflow.
        s.cache.clear();
        s.execute("frame.create", &json!({"rect": [10, 10, 20, 20], "content": "unassigned"})).unwrap();
        assert_removed(&s, &before, sid, last, pages);
        s.execute("frame.create", &json!({"rect": [10, 30, 100, 60], "content": "text", "text": "Other story", "caret": false})).unwrap();
        assert_removed(&s, &before, sid, last, pages);
        let first = s.doc().unwrap().doc.story(sid).unwrap().frames[0];
        s.execute("object.primaryTextFrame", &json!({"ids": [first.0], "on": true})).unwrap();
        assert_removed(&s, &before, sid, last, pages);
        assert!(s.prefs.smart_text_reflow);
    }
}

#[test]
fn deleting_a_blank_page_does_not_reflow_primary_page_variables() {
    let (mut s, sid, _) = primary_chain();
    let end = s.doc().unwrap().doc.story(sid).unwrap().len();
    s.execute("text.select", &json!({"story": sid.0, "anchor": end, "focus": end})).unwrap();
    // A Last Page Number variable makes composition depend on the document page count.
    let index = s.doc().unwrap().doc.text_variables.iter().position(|v| v.name == "Last Page Number").unwrap();
    let marker = pagenor_doc::vars::var_char(index).unwrap().to_string();
    s.execute("text.insert", &json!({"text": marker})).unwrap();
    let before = s.doc().unwrap().doc.clone();
    let last = *before.story(sid).unwrap().frames.last().unwrap();
    assert_eq!(before.page_of_item(last), Some(before.page_count() - 1));
    remove(&mut s, last, Removal::Frame);
    assert_removed(&s, &before, sid, last, before.page_count());
    s.execute("layout.pages.delete", &json!({"pages": [before.page_count() - 1]})).unwrap();
    assert_removed(&s, &before, sid, last, before.page_count() - 1);
    assert!(s.prefs.smart_text_reflow);
    s.execute("frame.create", &json!({"rect": [10, 10, 20, 20], "content": "unassigned"})).unwrap();
    assert_removed(&s, &before, sid, last, before.page_count() - 1);
}

#[test]
fn primary_content_edit_resumes_reflow_after_explicit_removal() {
    for kind in [Removal::Frame, Removal::Page, Removal::Cut] {
        let (mut s, sid, last) = primary_chain();
        remove(&mut s, last, kind);
        let removed = s.doc().unwrap().doc.clone();
        let end = removed.story(sid).unwrap().len();
        s.execute("text.select", &json!({"story": sid.0, "anchor": end, "focus": end})).unwrap();
        s.execute("text.insert", &json!({"text": " More text."})).unwrap();
        let flowed = s.doc().unwrap().doc.clone();
        assert_eq!(flowed.settings.primary_story, Some(sid));
        assert!(s.prefs.smart_text_reflow);
        assert!(flowed.page_count() > removed.page_count());
        assert!(!s.cache.get(&flowed, sid, None).is_overset());
        assert!(flowed.item(last).is_none());
        assert!(flowed.story(sid).unwrap().frames.iter().any(|f| !removed.story(sid).unwrap().frames.contains(f)));
        assert_eq!(flowed.story(sid).unwrap().text, removed.story(sid).unwrap().text.clone() + " More text.");
        flowed.check().unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &removed), "one undo restores text and the explicitly reduced chain");
        s.execute("edit.redo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &flowed));
    }
}

#[test]
fn primary_frame_layout_edit_resumes_reflow_after_explicit_removal() {
    let (mut s, sid, last) = primary_chain();
    remove(&mut s, last, Removal::Page);
    let removed = s.doc().unwrap().doc.clone();
    let first = removed.story(sid).unwrap().frames[0];
    let inset = removed.item(first).unwrap().text_frame().unwrap().options.inset.map(|v| v + 1.0);
    s.execute("object.textFrameOptions", &json!({"ids": [first.0], "inset": inset})).unwrap();
    let flowed = s.doc().unwrap().doc.clone();
    assert!(flowed.page_count() > removed.page_count());
    assert!(!s.cache.get(&flowed, sid, None).is_overset());
    assert_eq!(flowed.settings.primary_story, Some(sid));
    assert!(s.prefs.smart_text_reflow);
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &removed));
    s.execute("edit.redo", &json!({})).unwrap();
    assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &flowed));
    flowed.check().unwrap();
}

#[test]
fn deleting_primary_text_still_prunes_empty_pages_including_clear_and_cut() {
    for command in ["text.delete", "edit.clear", "edit.cut"] {
        let (mut s, sid, _) = primary_chain();
        let before = s.doc().unwrap().doc.clone();
        let end = before.story(sid).unwrap().len();
        s.execute("text.select", &json!({"story": sid.0, "anchor": 0, "focus": end})).unwrap();
        let result = s.execute(command, &json!({})).unwrap();
        let empty = s.doc().unwrap().doc.clone();
        assert_eq!(empty.page_count(), 1, "{command}: empty trailing pages are pruned");
        assert_eq!(empty.settings.primary_story, Some(sid));
        assert_eq!(empty.story(sid).unwrap().frames.len(), 1);
        assert!(empty.story(sid).unwrap().text.is_empty());
        assert!(s.prefs.smart_text_reflow);
        if command == "edit.cut" {
            assert_eq!(result["text"], before.story(sid).unwrap().text);
        }
        empty.check().unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before));
        s.execute("edit.redo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &empty));
    }
}

#[test]
fn reflow_off_preserves_removal_and_does_not_resume_on_primary_edits() {
    for kind in [Removal::Frame, Removal::Page, Removal::Cut] {
        let (mut s, sid, last) = primary_chain();
        s.execute("prefs.set", &json!({"smartTextReflow": false})).unwrap();
        let before = s.doc().unwrap().doc.clone();
        let pages = before.page_count() - usize::from(matches!(kind, Removal::Page));
        remove(&mut s, last, kind);
        assert_removed(&s, &before, sid, last, pages);
        let frames = s.doc().unwrap().doc.story(sid).unwrap().frames.clone();
        let end = s.doc().unwrap().doc.story(sid).unwrap().len();
        s.execute("text.select", &json!({"story": sid.0, "anchor": end, "focus": end})).unwrap();
        s.execute("text.insert", &json!({"text": " More text."})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page_count(), pages);
        assert_eq!(s.doc().unwrap().doc.story(sid).unwrap().frames, frames);
        assert!(s.cache.get(&s.doc().unwrap().doc, sid, None).is_overset());
        let end = s.doc().unwrap().doc.story(sid).unwrap().len();
        s.execute("text.select", &json!({"story": sid.0, "anchor": 0, "focus": end})).unwrap();
        s.execute("text.delete", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page_count(), pages, "OFF does not prune empty pages");
        assert_eq!(s.doc().unwrap().doc.story(sid).unwrap().frames, frames);
        assert_eq!(s.doc().unwrap().doc.settings.primary_story, Some(sid));
        assert!(!s.prefs.smart_text_reflow);
        s.doc().unwrap().doc.check().unwrap();
    }
}

#[test]
fn unused_or_shadowed_grep_conditions_preserve_deleted_primary_pages() {
    for shadowed in [false, true] {
        let (mut s, sid, _) = primary_chain();
        s.execute("condition.new", &json!({"name": "Dormant", "visible": true})).unwrap();
        s.execute("style.character.create", &json!({"name": "Conditional", "chars": {"conditions": ["Dormant"]}})).unwrap();
        s.execute("style.character.create", &json!({"name": "Clear", "chars": {"conditions": []}})).unwrap();
        let len = s.doc().unwrap().doc.story(sid).unwrap().len();
        s.execute("text.select", &json!({"story": sid.0, "anchor": 0, "focus": len})).unwrap();
        let rules = if shadowed {
            json!([{"style": "Conditional", "pattern": ".+"}, {"style": "Clear", "pattern": ".+"}])
        } else {
            json!([{"style": "Conditional", "pattern": "ZZZ_NEVER_PRESENT"}])
        };
        s.execute("type.para", &json!({"attrs": {"grepStyles": rules}})).unwrap();
        let last = *s.doc().unwrap().doc.story(sid).unwrap().frames.last().unwrap();
        remove(&mut s, last, Removal::Page);
        let before = s.doc().unwrap().doc.clone();
        let reference = compose::compose_story(&before, sid, &Default::default());
        assert!(reference.is_overset());
        s.execute("condition.options", &json!({"name": "Dormant", "visible": false})).unwrap();
        let after = s.doc().unwrap().doc.clone();
        assert_eq!(after.page_count(), before.page_count(), "shadowed: {shadowed}");
        assert_eq!(after.story(sid).unwrap().frames, before.story(sid).unwrap().frames);
        let current = compose::compose_story(&after, sid, &Default::default());
        assert_eq!(format!("{reference:?}"), format!("{current:?}"));
        after.check().unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before));
        s.execute("edit.redo", &json!({})).unwrap();
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &after));
    }
}

#[test]
fn used_condition_in_overset_tail_resumes_primary_growth() {
    let (mut s, sid, _) = primary_chain();
    s.execute("condition.new", &json!({"name": "Printed tail", "visible": false})).unwrap();
    let len = s.doc().unwrap().doc.story(sid).unwrap().len();
    let start = len - 200;
    s.execute("text.select", &json!({"story": sid.0, "anchor": start, "focus": len})).unwrap();
    s.execute("condition.apply", &json!({"name": "Printed tail", "only": true})).unwrap();
    let last = *s.doc().unwrap().doc.story(sid).unwrap().frames.last().unwrap();
    remove(&mut s, last, Removal::Page);
    let before = s.doc().unwrap().doc.clone();
    assert!(s.cache.get(&before, sid, None).overset_at.is_some_and(|at| at < start));
    s.execute("condition.options", &json!({"name": "Printed tail", "visible": true})).unwrap();
    let after = s.doc().unwrap().doc.clone();
    assert!(after.page_count() > before.page_count());
    assert!(!s.cache.get(&after, sid, None).is_overset());
    after.check().unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before));
    s.execute("edit.redo", &json!({})).unwrap();
    assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &after));
}
