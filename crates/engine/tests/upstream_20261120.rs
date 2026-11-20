//! Synthetic regressions for the November upstream adaptations; no user documents or fonts.
use pagenor_doc::{Content, ItemId, OBJECT_MARK, StoryId};
use pagenor_engine::{
    Session,
    cmd::{base64_encode, pasted_png},
};
use serde_json::{Value, json};
use std::sync::Arc;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages":2})).unwrap();
    s
}
fn image() -> Value {
    let bytes = pasted_png(3, 2, vec![180; 24]).unwrap();
    json!({"base64":base64_encode(&bytes),"name":"Clipboard.png"})
}
fn text(s: &mut Session) -> StoryId {
    let f = s.execute("frame.create", &json!({"rect":[36,36,336,336],"content":"text","text":"OLD"})).unwrap();
    let sid = StoryId(f["story"].as_u64().unwrap());
    s.execute("text.select", &json!({"story":sid.0,"anchor":1,"focus":2})).unwrap();
    sid
}
#[test]
fn pasted_raster_replaces_selection_with_one_inline_object_and_one_undo() {
    let mut s = session();
    let sid = text(&mut s);
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    let result = s.execute("edit.pasteImage", &image()).unwrap();
    assert_eq!(result["anchored"], true);
    let st = s.doc().unwrap();
    let story = st.doc.story(sid).unwrap();
    assert_eq!(story.text, format!("O{OBJECT_MARK}D"));
    assert_eq!(story.objects.len(), 1);
    assert_eq!(st.selection.text.unwrap().focus, 1 + OBJECT_MARK.len_utf8());
    assert_eq!(st.history.undo.len(), undo + 1);
    let bytes = pagenor_format::save(&st.doc).unwrap();
    let loaded = pagenor_format::load(&bytes).unwrap();
    assert_eq!(loaded.story(sid).unwrap().objects.len(), 1);
    assert_eq!(loaded.assets.len(), 1);
    let idml = pagenor_idml::export_idml(&st.doc);
    let idml = pagenor_idml::import_idml(&idml).unwrap();
    assert!(idml.assets.values().any(|asset| asset.pixels == Some((3, 2)) && !asset.data.is_empty()));

    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(*s.doc().unwrap().doc, *before);
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.story(sid).unwrap().text, format!("O{OBJECT_MARK}D"));
}
#[test]
fn image_paste_fills_empty_frame_then_deduplicates_embedded_bytes() {
    let mut s = session();
    let frame = s.execute("frame.create", &json!({"rect":[36,36,136,136],"content":"unassigned"})).unwrap();
    let id = ItemId(frame["id"].as_u64().unwrap());
    s.execute("selection.set", &json!({"ids":[id.0]})).unwrap();
    let first = s.execute("edit.pasteImage", &image()).unwrap();
    assert_eq!(first["id"], id.0);
    assert!(matches!(s.doc().unwrap().doc.item(id).unwrap().content, Content::Graphic(_)));
    s.execute("selection.set", &json!({"ids":[]})).unwrap();
    let second = s.execute("edit.pasteImage", &image()).unwrap();
    assert_ne!(first["id"], second["id"]);
    assert_eq!(first["asset"], second["asset"]);
    assert_eq!(s.doc().unwrap().doc.assets.len(), 1);
}
#[test]
fn malformed_paste_is_atomic_and_never_removes_selected_text() {
    let mut s = session();
    text(&mut s);
    let valid = image();
    for change in [
        json!({"base64":"broken"}),
        json!({"name":"../bad.png"}),
        json!({"name":"bad.svg"}),
        json!({"at":[0]}),
        json!({"spread":-1}),
        json!({"at":[1e30,0]}),
    ] {
        let mut params = valid.clone();
        for (k, v) in change.as_object().unwrap() {
            params[k] = v.clone();
        }
        let before = s.doc().unwrap().doc.clone();
        let selection = s.doc().unwrap().selection.clone();
        let undo = s.doc().unwrap().history.undo.len();
        let journal = s.journal.len();
        assert!(s.execute("edit.pasteImage", &params).is_err(), "{params}");
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        assert_eq!(s.doc().unwrap().selection, selection);
        assert_eq!(s.doc().unwrap().history.undo.len(), undo);
        assert_eq!(s.journal.len(), journal);
    }
}
#[test]
fn omitted_alignment_and_malformed_page_lists_change_nothing() {
    let mut s = session();
    text(&mut s);
    s.execute("type.align", &json!({"align":"right"})).unwrap();
    for (command, params) in [
        ("type.align", json!({})),
        ("type.align", json!({"alignment":"center"})),
        ("type.align", json!({"align":"middle"})),
        ("layout.pageSize", json!({"pages":[1,"bad",2],"width":100})),
        ("layout.pageSize", json!({"pages":[1,-1],"width":100})),
    ] {
        let before = s.doc().unwrap().doc.clone();
        let undo = s.doc().unwrap().history.undo.len();
        assert!(s.execute(command, &params).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        assert_eq!(s.doc().unwrap().history.undo.len(), undo);
    }
    s.execute("type.align", &json!({"align":"center"})).unwrap();
    assert_eq!(s.execute("type.selectionAttrs", &json!({})).unwrap()["para"]["align"], "center");
    s.execute("layout.pageSize", &json!({"pages":[1,2],"width":200})).unwrap();
    assert_eq!(s.doc().unwrap().doc.page(1).unwrap().width, 200.0);
}
#[test]
fn child_story_paste_and_table_commands_cannot_modify_the_outer_story() {
    let mut s = session();
    let sid = text(&mut s);
    s.execute("footnote.insert", &json!({"text":"note"})).unwrap();
    // The footnote command leaves its own caret active.
    assert!(s.doc().unwrap().selection.text.unwrap().cell.unwrap().footnote_id().is_some());
    let before = s.doc().unwrap().doc.clone();
    for command in ["edit.pasteImage", "table.setCell"] {
        let params = if command == "edit.pasteImage" { image() } else { json!({"fill":"[Black]"}) };
        assert!(s.execute(command, &params).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
    }
    assert!(s.doc().unwrap().doc.story(sid).is_some());
}

#[test]
fn character_stroke_limits_apply_to_text_and_named_styles() {
    let mut s = session();
    text(&mut s);
    for weight in [-1.0, 800.5, 1e9] {
        let before = s.doc().unwrap().doc.clone();
        assert!(s.execute("type.char", &json!({"attrs":{"strokeWeight":weight}})).is_err());
        assert!(Arc::ptr_eq(&before, &s.doc().unwrap().doc));
        assert!(s.execute("style.character.create", &json!({"name":"Invalid","chars":{"strokeWeight":weight}})).is_err());
    }
    for weight in [0.0, 12.0, 800.0] {
        s.execute("type.char", &json!({"attrs":{"strokeWeight":weight}})).unwrap();
        assert_eq!(s.execute("type.selectionAttrs", &json!({})).unwrap()["chars"]["strokeWeight"], weight);
    }
}

#[test]
fn raster_paste_uses_stored_density_and_rejects_oversized_pixel_counts() {
    let png = pasted_png(40, 30, vec![255; 40 * 30 * 4]).unwrap();
    let mut data = 5669u32.to_be_bytes().repeat(2);
    data.push(1);
    let mut kind_and_data = b"pHYs".to_vec();
    kind_and_data.extend_from_slice(&data);
    let mut crc = 0xFFFF_FFFFu32;
    for b in &kind_and_data {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    let mut bytes = png[..33].to_vec();
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&kind_and_data);
    bytes.extend_from_slice(&(!crc).to_be_bytes());
    bytes.extend_from_slice(&png[33..]);
    let mut s = session();
    let pasted = s.execute("edit.pasteImage", &json!({"base64":base64_encode(&bytes),"at":[100,200]})).unwrap();
    let rect = s.doc().unwrap().doc.item(ItemId(pasted["id"].as_u64().unwrap())).unwrap().bounds();
    assert!((rect.width() - 20.0).abs() < 0.01 && (rect.height() - 15.0).abs() < 0.01, "{rect:?}");
    assert!((rect.center().x - 100.0).abs() < 1e-9 && (rect.center().y - 200.0).abs() < 1e-9);
    assert!(pagenor_engine::cmd::check_pasted_size(64_000_001, 1).is_err());
    assert!(pagenor_engine::cmd::check_pasted_size(0, 1).is_err());
    assert!(pagenor_engine::cmd::pasted_png(2, 2, vec![0; 3]).is_err());
}
