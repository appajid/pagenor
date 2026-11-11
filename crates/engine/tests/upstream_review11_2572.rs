//! A numeric `perPage` or `arrange` is refused, not treated as absent.
use pagenor_engine::Session;
use pagenor_engine::cmd::base64_encode;
use serde_json::json;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"facingPages": false})).unwrap();
    let csv = "Name\nAda\nGrace\nLin\nMay\n";
    s.execute("data.source.select", &json!({"bytes": base64_encode(csv.as_bytes()), "name": "people.csv"})).unwrap();
    s.execute("frame.create", &json!({"rect": [36, 36, 100, 60], "content": "text", "text": "<<Name>>"})).unwrap();
    s
}

#[test]
fn numeric_per_page_is_a_type_error() {
    let mut s = session();
    let err = s.execute("data.merge", &json!({"perPage": 4})).unwrap_err().to_string();
    assert!(err.contains("perPage"), "{err}");
    assert_eq!(s.doc().unwrap().doc.spreads.len(), 1, "a refused merge adds no pages");
}

#[test]
fn numeric_arrange_is_a_type_error() {
    let mut s = session();
    let err = s.execute("data.merge", &json!({"perPage": "multiple", "arrange": 1})).unwrap_err().to_string();
    assert!(err.contains("arrange"), "{err}");
}

#[test]
fn string_values_still_work() {
    let mut s = session();
    assert!(s.execute("data.merge", &json!({"perPage": "bogus"})).is_err());
    s.execute("data.merge", &json!({"perPage": "multiple", "arrange": "columns"})).unwrap();
}

#[test]
fn new_and_saved_preset_gutter_is_nonnegative() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"columns":2,"gutter":-50})).unwrap();
    assert_eq!(s.doc().unwrap().doc.page(0).unwrap().columns.gutter, 0.0);
    s.execute("file.savePreset", &json!({"name":"Tight","columns":2,"gutter":-50})).unwrap();
    assert_eq!(s.prefs.document_presets[0].gutter, 0.0);
}
#[test]
fn custom_variable_scalar_and_invalid_values() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    for text in [json!(2026), json!(0.5), json!(true), json!("తెలుగు")] {
        s.execute("variables.define", &json!({"name":"Scalar","text":text})).unwrap();
        let doc = &s.doc().unwrap().doc;
        let v = &doc.text_variables[doc.variable("Scalar").unwrap()];
        assert!(matches!(&v.kind,pagenor_doc::vars::VarKind::Custom{text:actual}if actual==text.as_str().unwrap_or(&text.to_string())));
        let before = doc.clone();
        assert!(s.execute("variables.define", &json!({"name":"Scalar","text":[]})).is_err());
        assert_eq!(s.doc().unwrap().doc.text_variables, before.text_variables);
    }
}
