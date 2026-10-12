use super::*;
use crate::testing::font_with;

fn alias_face(family: &str, style: &str, source: FontSource) -> FontFace {
    make_face(FontBytes::Static(BUNDLED[0]), source, 0, family.into(), style.into(), Vec::new()).unwrap()
}

#[test]
fn canonical_style_wins_over_another_weights_regular_alias() {
    let db = FontDb::with_font_dirs(vec![]);
    let family = "Pagenor Style Precedence";
    db.insert_faces(vec![alias_face(family, "Light", FontSource::Memory), alias_face(family, "Regular", FontSource::Memory)]);
    assert_eq!(db.face(family, "Regular").style, "Regular");
    assert_eq!(db.face(family, "Light").style, "Light");
}

#[test]
fn document_alias_keeps_precedence_over_installed_canonical_style() {
    let db = FontDb::with_font_dirs(vec![]);
    let family = "Pagenor Scoped Style Precedence";
    db.insert_faces(vec![alias_face(family, "Regular", FontSource::Memory)]);
    let own = Arc::new(alias_face(family, "Book", FontSource::Document("document.ttf".into())));
    db.scopes.write().unwrap().insert(123, vec![own.clone()].into());
    assert!(Arc::ptr_eq(&own, &db.scoped(123).face(family, "Regular")));
    assert_eq!(db.face(family, "Regular").style, "Regular");
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn loaded_style_alias_cannot_hide_a_lazy_canonical_style() {
    let dir = std::env::temp_dir().join(format!("pagenor-canonical-style-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let family = "Pagenor Lazy Style Precedence";
    std::fs::write(dir.join("regular.ttf"), font_with(family, &['A']).unwrap()).unwrap();
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    db.insert_faces(vec![alias_face(family, "Light", FontSource::Memory)]);
    assert_eq!(db.face(family, "Regular").style, "Regular");
    assert!(matches!(db.face(family, "Regular").source, FontSource::Installed(_)));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn indic_fallback_requires_one_face_covering_the_whole_cluster() {
    let db = FontDb::with_font_dirs(vec![]);
    db.set_system_fallback(false);
    db.add_font(font_with("Pagenor Partial Telugu", &['క', 'ష']).unwrap());
    db.add_font(font_with("Pagenor Complete Telugu", &['క', '్', 'ష', 'ి']).unwrap());
    let primary = db.face("Pagenor Partial Telugu", "Regular");
    for text in ["కి", "క్షి", "క్\u{200D}షి", "క్\u{200C}షి"] {
        assert!(!primary.covers_cluster(text));
        let full = db.scoped(0).fallback_for_cluster(text, primary.id(), Some("te")).unwrap();
        assert_eq!(full.family, "Pagenor Complete Telugu");
        assert!(full.covers_cluster(text));
        assert!(!full.covers('\u{200D}'), "joiners do not need standalone glyphs");
    }
    assert!(db.scoped(0).fallback_for_cluster("క్షిఅ", primary.id(), Some("te")).is_none());
    assert!(db.scoped(0).fallback_for_cluster("", primary.id(), Some("te")).is_none());
    assert!(db.scoped(0).fallback_for_cluster("\u{200D}", primary.id(), Some("te")).is_none());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn lazy_native_cluster_fallback_does_not_accept_a_partial_face_and_refreshes_misses() {
    let dir = std::env::temp_dir().join(format!("pagenor-indic-fallback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("partial.ttf"), font_with("Pagenor Installed Partial", &['క']).unwrap()).unwrap();
    let db = FontDb::with_font_dirs(vec![dir.clone()]);
    let primary = db.face(FALLBACK_FAMILY, "Regular");
    assert!(db.scoped(0).fallback_for_cluster("కి", primary.id(), Some("te")).is_none());
    std::fs::write(dir.join("complete.ttf"), font_with("Pagenor Installed Complete", &['క', 'ి']).unwrap()).unwrap();
    db.load_system_fonts();
    let face = db.scoped(0).fallback_for_cluster("కి", primary.id(), Some("te")).unwrap();
    assert_eq!(face.family, "Pagenor Installed Complete");
    assert!(!db.is_loaded("Pagenor Installed Partial"), "partial files are not retained for the cluster");
    std::fs::remove_dir_all(dir).unwrap();
}
