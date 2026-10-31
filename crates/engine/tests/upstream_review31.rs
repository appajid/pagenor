#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use pagenor_doc::build::NewDocument;
use pagenor_doc::{Document, ItemId, ParaFormat, SpreadRef, Table};
use pagenor_engine::{DocState, Session};
use pagenor_geom::Rect;
use serde_json::json;
fn reopens(session: &Session) {
    let doc = &session.doc().unwrap().doc;
    doc.check().unwrap();
    let bytes = pagenor_format::save(doc).unwrap();
    pagenor_format::load(&bytes).unwrap();
}
#[test]
fn review31_copy_nested_tables_has_fresh_ids_and_reopens() {
    let mut doc = Document::new(&NewDocument::default());
    let layer = doc.default_layer();
    let (frame, sid) = doc.add_text_frame(SpreadRef::Doc(0), Rect::new(36., 36., 400., 500.), layer, "Host", ParaFormat::default()).unwrap();
    let mut outer = Table::new(doc.alloc(), 1, 1, 0, 0, 180.);
    let inner = Table::new(doc.alloc(), 1, 1, 0, 0, 120.);
    outer.cells[0].text.insert_table(0, inner);
    doc.story_mut(sid).unwrap().insert_table(0, outer);
    let mut session = Session::new();
    session.add_document(DocState::new(doc, None));
    reopens(&session);
    session.execute("transform.move", &json!({"ids":[frame.0],"copy":true,"dx":10,"dy":10})).unwrap();
    reopens(&session);
    let doc = &session.doc().unwrap().doc;
    let mut ids = Vec::new();
    fn collect(story: &pagenor_doc::Story, ids: &mut Vec<u64>) {
        for (id, table) in &story.tables {
            ids.push(*id);
            for cell in &table.cells {
                collect(&cell.text, ids);
            }
        }
    }
    for story in doc.stories.values() {
        collect(story, &mut ids);
    }
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(ids.len(), 4);
    assert_eq!(unique.len(), 4);
}
#[test]
fn review31_duplicate_layer_keeps_threading_settings_and_undo() {
    let mut session = Session::new();
    session.execute("file.new", &json!({"pages":2})).unwrap();
    let layer = session.execute("layer.new", &json!({"name":"Text"})).unwrap()["id"].clone();
    session.execute("layer.set", &json!({"id":layer,"color":"Red","printable":false})).unwrap();
    let a = session.execute("frame.create", &json!({"rect":[36,36,220,220],"content":"text","text":"Threaded Telugu తెలుగు"})).unwrap()["id"].clone();
    let b = session.execute("frame.create", &json!({"rect":[250,36,450,220],"content":"text"})).unwrap()["id"].clone();
    session.execute("text.thread", &json!({"from":a,"to":b})).unwrap();
    let before = session.doc().unwrap().doc.clone();
    let new = session.execute("layer.duplicate", &json!({"id":layer})).unwrap()["id"].as_u64().unwrap();
    let doc = &session.doc().unwrap().doc;
    let copy = doc.layers.iter().find(|l| l.id.0 == new).unwrap();
    assert!(!copy.printable);
    assert_eq!(copy.name, "Text copy");
    let frames: Vec<_> = doc.spreads.iter().flat_map(|sp| &sp.items).filter(|it| it.layer.0 == new).collect();
    assert_eq!(frames.len(), 2);
    let sid = frames[0].text_frame().unwrap().story;
    assert_eq!(frames[1].text_frame().unwrap().story, sid);
    assert_eq!(doc.story(sid).unwrap().frames.len(), 2);
    reopens(&session);
    session.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(session.doc().unwrap().doc.layers.len(), before.layers.len());
}
#[test]
fn review31_master_resize_targets_one_side_with_nine_anchors() {
    for anchor in 0..9 {
        let mut doc = Document::new(&NewDocument::default());
        let parent = doc.parents[0].id;
        let old = doc.parents[0].pages[0].clone();
        let other = doc.parents[0].pages.get(1).cloned();
        let pages: Vec<_> = doc.spreads.iter().flat_map(|sp| sp.pages.iter().map(|p| (p.id, p.width, p.height))).collect();
        let (item, _) = doc.add_text_frame(SpreadRef::Parent(0), old.margin_rect(), doc.default_layer(), "Master", ParaFormat::default()).unwrap();
        let bounds = doc.item(item).unwrap().bounds();
        let mut session = Session::new();
        session.add_document(DocState::new(doc, None));
        session
            .execute("layout.parentPageSize", &json!({"parent":parent.0,"page":0,"width":old.width+36.,"height":old.height+72.,"anchor":anchor}))
            .unwrap();
        let doc = &session.doc().unwrap().doc;
        let new = &doc.parents[0].pages[0];
        let after = doc.item(item).unwrap().bounds();
        let dx = new.x - old.x + 36. * (anchor % 3) as f64 / 2.;
        let dy = 72. * (anchor / 3) as f64 / 2.;
        assert!((after.x0 - bounds.x0 - dx).abs() < 0.01);
        assert!((after.y0 - bounds.y0 - dy).abs() < 0.01);
        if let Some(other) = other {
            assert_eq!(doc.parents[0].pages[1].width, other.width);
        }
        assert_eq!(doc.spreads.iter().flat_map(|sp| sp.pages.iter().map(|p| (p.id, p.width, p.height))).collect::<Vec<_>>(), pages);
        reopens(&session);
        session.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(session.doc().unwrap().doc.parents[0].pages[0].width, old.width);
    }
}
#[test]
fn review31_new_cover_and_bad_dimensions_are_atomic() {
    let mut session = Session::new();
    let count = session.documents().len();
    assert!(session.execute("file.newCover", &json!({"printer":"other","trimWidth":432,"trimHeight":648,"spine":-1})).is_err());
    assert_eq!(session.documents().len(), count);
    let created = session.execute("file.newCover", &json!({"printer":"lulu","trimWidth":432,"trimHeight":648,"pages":224})).unwrap();
    assert!((created["spine"].as_f64().unwrap() - 40.644).abs() < 0.001);
    let doc = &session.doc().unwrap().doc;
    assert_eq!(doc.page_count(), 1);
    assert_eq!(doc.page(0).unwrap().guides.len(), 2);
    reopens(&session);
    let _ = ItemId(0);
}

fn image_source(color: u8) -> serde_json::Value {
    let bytes = pagenor_render::Rendered { width: 4, height: 4, pixels: [color, 30, 50, 255].repeat(16) }.to_png();
    json!({"name":format!("{color}.png"),"base64":pagenor_engine::cmd::base64_encode(&bytes)})
}
#[test]
fn review31_place_queue_is_ordered_atomic_bound_and_cancelled() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let sources = vec![image_source(10), image_source(20), image_source(30)];
    s.execute("place.load", &json!({"files":sources})).unwrap();
    assert_eq!(s.loaded_queue.len(), 2);
    let first = s.loaded.unwrap().0;
    assert_eq!(s.doc().unwrap().doc.assets[&first].name, "10.png");
    let before = s.doc().unwrap().doc.clone();
    assert!(s.execute("place.grid", &json!({"rect":[0,0,20,20],"columns":2,"rows":2,"gutter":100})).is_err());
    assert!(std::sync::Arc::ptr_eq(&before, &s.doc().unwrap().doc));
    assert_eq!(s.loaded.unwrap().0, first);
    assert!(s.execute("place.load", &json!({"files":[image_source(40),{"name":"bad","base64":"AA=="}]})).is_err());
    assert_eq!(s.loaded.unwrap().0, first);
    s.execute("place.drop", &json!({"x":36,"y":36})).unwrap();
    assert_eq!(s.doc().unwrap().doc.assets[&s.loaded.unwrap().0].name, "20.png");
    let placed = s.execute("place.grid", &json!({"rect":[36,100,300,300],"columns":2,"rows":1})).unwrap();
    assert_eq!(placed["ids"].as_array().unwrap().len(), 2);
    assert!(s.loaded.is_none());
    reopens(&s);
    s.execute("place.load", &image_source(50)).unwrap();
    s.execute("file.new", &json!({})).unwrap();
    assert!(s.loaded.is_none() && s.loaded_queue.is_empty());
    s.execute("place.load", &image_source(60)).unwrap();
    s.execute("place.cancel", &json!({})).unwrap();
    assert!(s.loaded.is_none() && s.loaded_queue.is_empty());
}
#[test]
fn review31_contact_sheet_places_every_image_once_and_undoes() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages":1})).unwrap();
    s.execute("place.load", &json!({"files":[image_source(10),image_source(20),image_source(30)]})).unwrap();
    let before = s.doc().unwrap().doc.clone();
    assert_eq!(s.execute("place.contactSheet", &json!({"columns":2,"rows":1})).unwrap()["images"], 3);
    assert_eq!(s.doc().unwrap().doc.page_count(), 3);
    assert!(s.loaded.is_none());
    reopens(&s);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.page_count(), before.page_count());
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn review31_entire_book_toc_refresh_export_freshness_and_reorder() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.pgnr");
    let b = dir.path().join("b.pgnr");
    let manifest = dir.path().join("Book.pbook");
    let mut s = Session::new();
    for (path, title) in [(&a, "తెలుగు First"), (&b, "Second")] {
        s.execute("file.new", &json!({"pages":2,"title":title})).unwrap();
        s.execute("style.paragraph.create", &json!({"name":"Heading","chars":{"size":16}})).unwrap();
        let id = s.execute("frame.create", &json!({"rect":[36,36,450,120],"content":"text","text":title})).unwrap()["id"].clone();
        s.execute("selection.set", &json!({"ids":[id]})).unwrap();
        s.execute("style.paragraph.apply", &json!({"name":"Heading"})).unwrap();
        s.execute("file.saveAs", &json!({"path":path})).unwrap();
        s.execute("file.close", &json!({})).unwrap();
    }
    s.execute("book.new", &json!({"path":manifest})).unwrap();
    for path in [&a, &b] {
        s.execute("book.add", &json!({"path":path})).unwrap();
    }
    s.execute("book.paginate", &json!({})).unwrap();
    s.execute("file.open", &json!({"path":a})).unwrap();
    let result = s.execute("toc.generate", &json!({"styles":["Heading"],"scope":"entireBook","page":2})).unwrap();
    assert_eq!(result["entries"], 2);
    let sid = pagenor_doc::StoryId(result["story"].as_u64().unwrap());
    assert!(s.doc().unwrap().doc.story(sid).unwrap().text.contains("Second\t3"));
    reopens(&s);
    s.execute("file.save", &json!({})).unwrap();
    assert!(s.execute("book.refreshToc", &json!({})).is_err(), "close participants before disk refresh");
    s.execute("file.close", &json!({})).unwrap();
    s.execute("book.refreshToc", &json!({})).unwrap();
    assert!(s.execute("book.exportPdf", &json!({})).is_ok());
    s.execute("book.move", &json!({"from":1,"to":0})).unwrap();
    assert!(s.execute("book.exportPdf", &json!({})).is_err());
    s.execute("book.refreshToc", &json!({})).unwrap();
    let doc = pagenor_format::load(&std::fs::read(&a).unwrap()).unwrap();
    assert!(doc.story(sid).unwrap().text.contains("Second\t1"));
    assert_eq!(doc.toc.unwrap().scope, pagenor_doc::TocScope::EntireBook);
    // Changed on-disk headings must invalidate even a persisted checkpoint.
    let mut editor = Session::new();
    editor.execute("file.open", &json!({"path":b})).unwrap();
    let story = editor.doc().unwrap().doc.stories.iter().find(|(_, st)| st.text.starts_with("Second")).map(|(id, _)| *id).unwrap();
    editor.execute("story.replaceRange", &json!({"story":story.0,"start":0,"end":6,"text":"Changed"})).unwrap();
    editor.execute("file.save", &json!({})).unwrap();
    editor.execute("file.close", &json!({})).unwrap();
    assert!(s.execute("book.exportPdf", &json!({})).is_err());
    s.execute("book.refreshToc", &json!({})).unwrap();
    assert!(s.execute("book.exportPdf", &json!({})).is_ok());
    let original_a = std::fs::read(&a).unwrap();
    let original_b = std::fs::read(&b).unwrap();
    let mut oversized = pagenor_format::load(&original_a).unwrap();
    oversized.styles_mut().paragraph.iter_mut().find(|p| p.name == "TOC Level 1").unwrap().chars.size = Some(5000.0);
    std::fs::write(&a, pagenor_format::save(&oversized).unwrap()).unwrap();
    assert!(s.execute("book.exportPdf", &json!({})).is_err(), "a TOC-only style change must not export clipped entries");
    std::fs::write(&a, &original_a).unwrap();
    let mut changed: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    changed["documents"].as_array_mut().unwrap().reverse();
    std::fs::write(&manifest, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(s.execute("book.refreshToc", &json!({})).is_err(), "refuse an externally reordered manifest");
    assert!(s.execute("book.exportPdf", &json!({})).is_err());
    assert_eq!(std::fs::read(&a).unwrap(), original_a);
    assert_eq!(std::fs::read(&b).unwrap(), original_b);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn review31_relative_book_paths_survive_reopening() {
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir_in(&cwd).unwrap();
    let relative = dir.path().strip_prefix(&cwd).unwrap();
    let chapter = relative.join("chapter.pgnr");
    let book = relative.join("Book.pbook");
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("file.saveAs", &json!({"path":chapter})).unwrap();
    s.execute("file.close", &json!({})).unwrap();
    s.execute("book.new", &json!({"path":book})).unwrap();
    s.execute("book.add", &json!({"path":chapter})).unwrap();
    s.execute("book.close", &json!({})).unwrap();
    s.execute("book.open", &json!({"path":book})).unwrap();
    assert_eq!(s.execute("book.list", &json!({})).unwrap()["documents"][0]["pages"], 1);
    assert!(s.execute("book.paginate", &json!({})).is_ok());
    s.execute("file.open", &json!({"path":chapter})).unwrap();
    assert!(s.execute("toc.styles", &json!({"scope":"entireBook"})).is_ok());
}
