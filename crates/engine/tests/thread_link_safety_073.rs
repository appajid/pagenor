//! Linked-story threading and text-color transaction safety.
use pagenor_doc::StoryId;
use pagenor_engine::Session;
use serde_json::{Value, json};
use std::sync::Arc;

fn setup() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s
}
fn frame(s: &mut Session, text: &str) -> (u64, StoryId) {
    let f = s.execute("frame.create", &json!({"rect":[36,36,200,100],"content":"text","text":text,"caret":false})).unwrap();
    (f["id"].as_u64().unwrap(), StoryId(f["story"].as_u64().unwrap()))
}
fn child(s: &mut Session, parent: StoryId) -> (u64, StoryId) {
    let f = s.execute("story.placeAndLink", &json!({"story":parent.0,"rect":[36,150,200,210]})).unwrap();
    (f["id"].as_u64().unwrap(), StoryId(f["story"].as_u64().unwrap()))
}
fn state(s: &Session) -> Value {
    let d = s.doc().unwrap();
    json!({"doc":d.doc,"selection":d.selection,"revision":d.revision,"dirty":d.is_dirty(),"undo":format!("{:?}",d.history),"journal":s.journal})
}
fn link(s: &mut Session, sid: StoryId) -> Value {
    s.execute("story.links", &json!({})).unwrap().as_array().unwrap().iter().find(|l| l["story"] == sid.0).unwrap().clone()
}

#[test]
fn safety_linked_parent_follows_merge_and_updates_after_native_reopen() {
    let mut s = setup();
    let (first, survivor) = frame(&mut s, "PREFIX");
    let (incoming, parent) = frame(&mut s, "PARENT");
    let (_, linked) = child(&mut s, parent);
    // A coincident old parent revision must not hide the change of parent identity/content.
    let future_rev = s.doc().unwrap().doc.story(survivor).unwrap().rev + 2;
    s.edit(|d, _| {
        d.story_mut(linked).unwrap().link = Some((parent, future_rev));
        Ok(())
    })
    .unwrap();
    s.execute("text.thread", &json!({"from":first,"to":incoming})).unwrap();
    assert_eq!(link(&mut s, linked)["parent"], survivor.0);
    assert_eq!(link(&mut s, linked)["outOfDate"], true);
    assert_eq!(s.doc().unwrap().doc.story(linked).unwrap().text, "PARENT");
    let bytes = s.execute("file.serialize", &json!({})).unwrap();
    let mut reopened = Session::new();
    reopened.execute("file.openBytes", &json!({"name":"links.pgnr","base64":bytes["base64"]})).unwrap();
    assert_eq!(link(&mut reopened, linked)["parent"], survivor.0);
    reopened.execute("story.updateLink", &json!({"story":linked.0})).unwrap();
    assert_eq!(reopened.doc().unwrap().doc.story(linked).unwrap().text, "PREFIX\nPARENT");
    assert_eq!(link(&mut reopened, linked)["outOfDate"], false);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(link(&mut s, linked)["parent"], parent.0);
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(link(&mut s, linked)["parent"], survivor.0);
}

#[test]
fn safety_linked_child_cannot_merge_body_text_in_either_direction() {
    for reverse in [false, true] {
        let mut s = setup();
        let (_, parent) = frame(&mut s, "SOURCE");
        let (linked, child) = child(&mut s, parent);
        let (body, _) = frame(&mut s, "BODY");
        let (from, to) = if reverse { (body, linked) } else { (linked, body) };
        let before = state(&s);
        let error = s.execute("text.thread", &json!({"from":from,"to":to})).unwrap_err();
        assert!(error.to_string().contains("linked story"));
        assert_eq!(state(&s), before);
        s.execute("story.updateLink", &json!({"story":child.0})).unwrap();
        assert!(s.doc().unwrap().doc.stories.values().any(|st| st.text == "BODY"));
    }
}

fn check_indirect_linked_ancestor_rejection(ancestor_first: bool) {
    let mut s = setup();
    let (ancestor_frame, ancestor) = frame(&mut s, "");
    let (middle_frame, middle) = child(&mut s, ancestor);
    s.execute("story.setText", &json!({"story": middle.0, "text": "MIDDLE"})).unwrap();
    let (leaf_frame, leaf) = child(&mut s, middle);
    s.execute("selection.set", &json!({"ids": [middle_frame]})).unwrap();
    // Retain a real redo entry so rejection cannot silently clear history.
    s.execute("story.setText", &json!({"story": middle.0, "text": "REDO"})).unwrap();
    let redo_doc = s.doc().unwrap().doc.clone();
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.story(middle).unwrap().text, "MIDDLE");
    assert_eq!(s.doc().unwrap().doc.story(middle).unwrap().link.map(|(id, _)| id), Some(ancestor));
    assert_eq!(s.doc().unwrap().doc.story(leaf).unwrap().link.map(|(id, _)| id), Some(middle));
    assert_eq!(link(&mut s, middle)["outOfDate"], false);
    assert_eq!(link(&mut s, leaf)["outOfDate"], false);
    s.doc().unwrap().doc.check().unwrap();
    assert!(!s.doc().unwrap().history.redo.is_empty());
    let before_doc = s.doc().unwrap().doc.clone();
    let saved_doc = s.doc().unwrap().saved_doc.clone();
    let before = state(&s);
    let (from, to) = if ancestor_first { (ancestor_frame, leaf_frame) } else { (leaf_frame, ancestor_frame) };

    let error = s.execute("text.thread", &json!({"from": from, "to": to})).unwrap_err();

    assert!(error.to_string().contains("linked story"));
    assert_eq!(s.doc().unwrap().doc, before_doc);
    assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before_doc));
    assert!(Arc::ptr_eq(&s.doc().unwrap().saved_doc, &saved_doc));
    assert_eq!(state(&s), before, "rejection must preserve selection, history, revision, dirty state and journal");
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc, redo_doc);
}

#[test]
fn safety_thread_rejects_empty_indirect_ancestor_before_descendant() {
    check_indirect_linked_ancestor_rejection(true);
}

#[test]
fn safety_thread_rejects_empty_indirect_ancestor_after_descendant() {
    check_indirect_linked_ancestor_rejection(false);
}

#[test]
fn safety_thread_allows_unrelated_empty_frame_beside_linked_descendant() {
    for empty_first in [true, false] {
        let mut s = setup();
        let (_, ancestor) = frame(&mut s, "");
        let (_, middle) = child(&mut s, ancestor);
        s.execute("story.setText", &json!({"story": middle.0, "text": "MIDDLE"})).unwrap();
        let (leaf_frame, leaf) = child(&mut s, middle);
        let (empty_frame, empty_story) = frame(&mut s, "");
        let before = s.doc().unwrap().doc.clone();
        let undo_len = s.doc().unwrap().history.undo.len();
        let old_rev = before.story(leaf).unwrap().rev;
        let (from, to) = if empty_first { (empty_frame, leaf_frame) } else { (leaf_frame, empty_frame) };
        let frames = vec![pagenor_doc::ItemId(from), pagenor_doc::ItemId(to)];
        let mut expected = (*before).clone();
        expected.stories.remove(&empty_story);
        let st = expected.story_mut(leaf).unwrap();
        st.frames = frames;
        st.rev = old_rev.wrapping_add(1);
        expected.item_mut(pagenor_doc::ItemId(empty_frame)).unwrap().text_frame_mut().unwrap().story = leaf;

        s.execute("text.thread", &json!({"from": from, "to": to})).unwrap();

        assert_eq!(*s.doc().unwrap().doc, expected);
        assert_eq!(s.doc().unwrap().selection, pagenor_doc::Selection::items(vec![pagenor_doc::ItemId(to)]));
        assert_eq!(s.doc().unwrap().history.undo.len(), undo_len + 1);
        assert_eq!(link(&mut s, middle)["outOfDate"], false);
        assert_eq!(link(&mut s, leaf)["outOfDate"], false);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc, before);
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(*s.doc().unwrap().doc, expected);
    }
}
