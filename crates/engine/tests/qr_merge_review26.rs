//! QR merge target regressions: generated paths cannot be filled as frame contents.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use pagenor_engine::{Session, cmd::base64_encode};
use serde_json::{Value, json};

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"facingPages": false})).unwrap();
    let csv = "Link,#Code\nhttps://a.example,https://a.example\nhttps://b.example,https://b.example\nhttps://c.example,https://c.example\n";
    s.execute("data.source.select", &json!({"bytes": base64_encode(csv.as_bytes()), "name": "links.csv"})).unwrap();
    s
}

fn refuse_without_edit(s: &mut Session, command: &str, params: Value) {
    let before = serde_json::to_value(&s.doc().unwrap().doc).unwrap();
    let selection = s.doc().unwrap().selection.clone();
    let undo = s.doc().unwrap().history.undo.len();
    let revision = s.doc().unwrap().revision;
    let documents = s.documents().len();
    let err = s.execute(command, &params).unwrap_err().to_string();
    assert!(err.contains("empty graphic frame"), "{err}");
    assert_eq!(serde_json::to_value(&s.doc().unwrap().doc).unwrap(), before);
    assert_eq!(s.doc().unwrap().selection, selection);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
    assert_eq!(s.doc().unwrap().revision, revision);
    assert_eq!(s.documents().len(), documents);
}

fn existing_code(s: &mut Session) -> Value {
    s.execute("object.qrCode", &json!({"content": "https://old.example", "rect": [72, 72, 216, 216]})).unwrap()["id"].clone()
}

fn check_merged_codes(s: &mut Session) {
    s.execute("data.merge", &json!({})).unwrap();
    let doc = &s.doc().unwrap().doc;
    doc.check().unwrap();
    let mut codes = Vec::new();
    for sp in &doc.spreads {
        for item in &sp.items {
            item.walk(&mut |i| {
                if let pagenor_doc::Content::Group { items } = &i.content {
                    for child in items {
                        if let Some(payload) = child.alt_text.strip_prefix("QR code: ") {
                            let expected = pagenor_geom::qr::qr_path(payload, i.inner_bounds()).unwrap();
                            assert_eq!(child.path, expected, "the visible vector code must match the record");
                        }
                    }
                }
                if i.alt_text.starts_with("QR code: ") {
                    codes.push(i.alt_text.clone());
                }
            });
        }
    }
    codes.sort();
    assert_eq!(codes, ["QR code: https://a.example", "QR code: https://b.example", "QR code: https://c.example"]);
}

#[test]
fn existing_code_is_refused_by_explicit_and_selected_qr_placeholder() {
    let mut s = session();
    let id = existing_code(&mut s);
    refuse_without_edit(&mut s, "data.placeholder.add", json!({"field":"Link", "role":"qr", "item":id}));
    refuse_without_edit(&mut s, "data.placeholder.add", json!({"field":"Link", "role":"qr"}));
    refuse_without_edit(&mut s, "data.placeholder.add", json!({"field":"Code", "item":id}));
}

#[test]
fn qr_generator_cannot_bind_a_new_or_existing_path_as_a_placeholder() {
    let mut s = session();
    refuse_without_edit(&mut s, "object.qrCode", json!({"field":"Link", "content":"https://old.example", "rect":[72,72,216,216]}));
    existing_code(&mut s);
    refuse_without_edit(&mut s, "object.qrCode", json!({"field":"Link"}));
    refuse_without_edit(&mut s, "object.qrCode", json!({"field":"Link", "content":"https://changed.example"}));
}

#[test]
fn empty_frame_placeholder_merges_every_record_and_undo_keeps_template() {
    let mut s = session();
    let id = s.execute("frame.create", &json!({"rect":[72,72,216,216], "content":"unassigned"})).unwrap()["id"].clone();
    let before = serde_json::to_value(&s.doc().unwrap().doc).unwrap();
    s.execute("data.placeholder.add", &json!({"field":"Link", "role":"qr", "item":id})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(serde_json::to_value(&s.doc().unwrap().doc).unwrap(), before);
    s.execute("edit.redo", &json!({})).unwrap();
    let template = serde_json::to_value(&s.doc().unwrap().doc).unwrap();
    check_merged_codes(&mut s);
    assert_eq!(serde_json::to_value(&s.documents()[0].doc).unwrap(), template);
}

#[test]
fn qr_generator_can_bind_an_empty_frame() {
    let mut s = session();
    s.execute("object.qrCode", &json!({"field":"Link", "rect":[72,72,216,216]})).unwrap();
    check_merged_codes(&mut s);
}

#[test]
fn a_frame_containing_a_qr_code_remains_a_valid_placeholder() {
    let mut s = session();
    let id = s.execute("frame.create", &json!({"rect":[72,72,216,216], "content":"unassigned"})).unwrap()["id"].clone();
    s.execute("object.qrCode", &json!({"content":"https://old.example"})).unwrap();
    s.execute("data.placeholder.add", &json!({"field":"Link", "role":"qr", "item":id})).unwrap();
    check_merged_codes(&mut s);
}

#[test]
fn an_existing_qr_code_can_still_have_a_hyperlink_placeholder() {
    let mut s = session();
    let id = existing_code(&mut s);
    s.execute("data.placeholder.add", &json!({"field":"Link", "role":"hyperlink", "item":id})).unwrap();
    assert_eq!(s.doc().unwrap().doc.data_merge.placeholders.len(), 1);
}
