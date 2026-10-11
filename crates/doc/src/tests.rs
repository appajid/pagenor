use pagenor_geom::Rect;

use super::*;
use crate::build::NewDocument;

fn doc() -> Document {
    Document::new(&NewDocument { pages: 2, ..Default::default() })
}

#[test]
fn threading_moves_frames_and_text() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "first", ParaFormat::default()).unwrap();
    let (b, sb) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "second", ParaFormat::default()).unwrap();
    d.thread(a, b).unwrap();
    d.check().unwrap();
    assert!(d.story(sb).is_none());
    let st = d.story(sa).unwrap();
    assert_eq!(st.text, "first\nsecond");
    assert_eq!(st.frames, vec![a, b]);
    assert_eq!(d.next_frame(a), Some(b));
    assert_eq!(d.prev_frame(b), Some(a));
    d.unthread_after(a).unwrap();
    d.check().unwrap();
    assert_eq!(d.story(sa).unwrap().frames, vec![a]);
    assert_ne!(d.item(b).unwrap().text_frame().unwrap().story, sa);
}

#[test]
fn thread_to_empty_frame_converts_it() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "x", ParaFormat::default()).unwrap();
    let id = ItemId(d.alloc());
    let it = Item::new(id, lid, Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(0.0, 200.0, 50.0, 300.0)));
    d.insert_item(SpreadRef::Doc(0), it, None).unwrap();
    d.thread(a, id).unwrap();
    assert!(d.item(id).unwrap().is_text_frame());
    assert_eq!(d.story(sa).unwrap().frames, vec![a, id]);
    d.check().unwrap();
    assert!(d.thread(a, a).is_err());
}

#[test]
fn threading_merges_tables_notes_and_endnotes() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "first", ParaFormat::default()).unwrap();
    let (b, sb) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "second", ParaFormat::default()).unwrap();
    for (sid, note) in [(sa, "Note A"), (sb, "Note B")] {
        let st = d.story_mut(sid).unwrap();
        st.insert_note(3, note, ParaFormat::default());
        // Both stories hold a table with id 1.
        let n = st.len();
        st.insert_table(n, crate::table::Table::new(1, 2, 2, 0, 0, 80.0));
    }
    let st = d.story_mut(sb).unwrap();
    st.insert_endnote(2, "End B", ParaFormat::default());
    st.insert(1, &crate::NOTE_MARK.to_string());
    d.thread(a, b).unwrap();
    d.check().unwrap();
    let st = d.story(sa).unwrap();
    assert!(d.story(sb).is_none());
    assert_eq!(st.tables.len(), 2);
    assert_eq!(st.notes.len(), 2);
    assert_eq!(st.notes[1].text.text, "Note B");
    assert_eq!(st.endnotes.len(), 1);
    assert_eq!(st.endnotes[0].text.text, "End B");
    assert_eq!(st.editorial.len(), 1);
}

#[test]
fn empty_frame_threaded_before_a_story_joins_it_at_the_front() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "", ParaFormat::default()).unwrap();
    let (b, sb) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "Hello", ParaFormat::default()).unwrap();
    d.thread_keeping_story(a, b).unwrap();
    d.check().unwrap();
    assert!(d.story(sa).is_none());
    let st = d.story(sb).unwrap();
    assert_eq!(st.frames, vec![a, b]);
    assert_eq!(st.text, "Hello");
}

#[test]
fn empty_story_takes_the_threaded_text_without_a_blank_paragraph() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "", ParaFormat::default()).unwrap();
    let (a2, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 110.0, 100.0, 200.0), lid, "", ParaFormat::default()).unwrap();
    d.thread(a, a2).unwrap();
    let (b, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "Hello", ParaFormat::default()).unwrap();
    d.thread(a2, b).unwrap();
    d.check().unwrap();
    let st = d.story(d.item(a).unwrap().text_frame().unwrap().story).unwrap();
    assert_eq!(st.frames, vec![a, a2, b]);
    assert_eq!(st.text, "Hello");
}

#[test]
fn threading_from_an_empty_frame_makes_it_a_text_frame() {
    let mut d = doc();
    let lid = d.default_layer();
    let id = ItemId(d.alloc());
    let it = Item::new(id, lid, Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(0.0, 200.0, 50.0, 300.0)));
    d.insert_item(SpreadRef::Doc(0), it, None).unwrap();
    let (b, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "Hello", ParaFormat::default()).unwrap();
    d.thread(id, b).unwrap();
    d.check().unwrap();
    let st = d.story(d.item(id).unwrap().text_frame().unwrap().story).unwrap();
    assert_eq!(st.frames, vec![id, b]);
    assert_eq!(st.text, "Hello");
}

#[test]
fn removing_a_frame_from_its_thread_keeps_the_text_in_the_others() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "words", ParaFormat::default()).unwrap();
    let (b, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(110.0, 10.0, 200.0, 100.0), lid, "", ParaFormat::default()).unwrap();
    let (c, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(210.0, 10.0, 300.0, 100.0), lid, "", ParaFormat::default()).unwrap();
    d.thread(a, b).unwrap();
    d.thread(b, c).unwrap();
    d.remove_from_thread(b).unwrap();
    d.check().unwrap();
    assert_eq!(d.story(sa).unwrap().frames, vec![a, c]);
    assert_eq!(d.story(sa).unwrap().text, "words");
    let own = d.item(b).unwrap().text_frame().unwrap().story;
    assert_ne!(own, sa);
    assert!(d.story(own).unwrap().text.is_empty());
    // A frame alone in its story has no thread to leave.
    assert!(d.remove_from_thread(b).is_err());
}

#[test]
fn frames_keep_their_story_direction_through_threading() {
    let mut d = doc();
    let lid = d.default_layer();
    let r = |x: f64| Rect::new(x, 10.0, x + 50.0, 100.0);
    let vertical = |d: &Document, f: ItemId| d.item(f).is_some_and(|i| d.frame_vertical(i));
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), r(0.0), lid, "縦", ParaFormat::default()).unwrap();
    let (b, _) = d.add_text_frame(SpreadRef::Doc(0), r(60.0), lid, "", ParaFormat::default()).unwrap();
    d.story_mut(sa).unwrap().vertical = true;
    // A horizontal frame threaded onto a vertical story turns vertical.
    d.thread(a, b).unwrap();
    assert!(vertical(&d, a) && vertical(&d, b));
    // Threading the vertical story's second frame after a horizontal frame: it turns horizontal,
    // and the frame left before it stays vertical.
    let (h, _) = d.add_text_frame(SpreadRef::Doc(0), r(120.0), lid, "横", ParaFormat::default()).unwrap();
    d.thread(h, b).unwrap();
    assert!(vertical(&d, a) && !vertical(&d, b) && !vertical(&d, h));
    // Breaking a vertical thread leaves the frames after the break vertical.
    let (c, _) = d.add_text_frame(SpreadRef::Doc(0), r(180.0), lid, "", ParaFormat::default()).unwrap();
    d.thread(a, c).unwrap();
    d.unthread_after(a).unwrap();
    assert!(vertical(&d, a) && vertical(&d, c));
    assert_ne!(d.item(c).unwrap().text_frame().unwrap().story, sa);
    d.check().unwrap();
}

#[test]
fn frames_keep_their_column_direction_through_threading() {
    use crate::TextDirection::{LeftToRight, RightToLeft};
    let mut d = doc();
    let lid = d.default_layer();
    let r = |x: f64| Rect::new(x, 10.0, x + 50.0, 100.0);
    let direction = |d: &Document, f: ItemId| d.story(d.item(f).unwrap().text_frame().unwrap().story).unwrap().direction;
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), r(0.0), lid, "نص", ParaFormat::default()).unwrap();
    let (b, _) = d.add_text_frame(SpreadRef::Doc(0), r(60.0), lid, "", ParaFormat::default()).unwrap();
    d.story_mut(sa).unwrap().direction = RightToLeft;
    d.thread(a, b).unwrap();
    assert_eq!((direction(&d, a), direction(&d, b)), (RightToLeft, RightToLeft));
    // Threading the right-to-left story's second frame after a left-to-right frame: the frame
    // left before it stays right to left.
    let (h, _) = d.add_text_frame(SpreadRef::Doc(0), r(120.0), lid, "abc", ParaFormat::default()).unwrap();
    d.thread(h, b).unwrap();
    assert_eq!((direction(&d, a), direction(&d, b), direction(&d, h)), (RightToLeft, LeftToRight, LeftToRight));
    // Breaking a right-to-left thread leaves the frames after the break right to left.
    let (c, _) = d.add_text_frame(SpreadRef::Doc(0), r(180.0), lid, "", ParaFormat::default()).unwrap();
    d.thread(a, c).unwrap();
    d.unthread_after(a).unwrap();
    assert_ne!(d.item(c).unwrap().text_frame().unwrap().story, sa);
    assert_eq!((direction(&d, a), direction(&d, c)), (RightToLeft, RightToLeft));
    d.check().unwrap();
}

#[test]
fn removing_last_frame_deletes_story() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, sa) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "x", ParaFormat::default()).unwrap();
    d.remove_item(a).unwrap();
    assert!(d.story(sa).is_none());
    d.check().unwrap();
}

#[test]
fn hit_testing_respects_layers() {
    let mut d = doc();
    let lid = d.default_layer();
    let (a, _) = d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "x", ParaFormat::default()).unwrap();
    assert_eq!(d.hit_item(0, pagenor_geom::Point::new(50.0, 50.0), 2.0), Some(a));
    assert_eq!(d.hit_item(0, pagenor_geom::Point::new(300.0, 300.0), 2.0), None);
    d.layer_mut(lid).unwrap().locked = true;
    assert_eq!(d.hit_item(0, pagenor_geom::Point::new(50.0, 50.0), 2.0), None);
}

#[test]
fn page_names_follow_sections() {
    let mut d = Document::new(&NewDocument { pages: 6, ..Default::default() });
    assert_eq!(d.page_name(0), "1");
    d.sections.push(Section {
        start: 2,
        start_number: Some(1),
        style: NumberStyle::LowerRoman,
        prefix: "A-".into(),
        marker: String::new(),
        include_prefix: true,
    });
    assert_eq!(d.page_name(1), "2");
    assert_eq!(d.page_name(2), "A-i");
    assert_eq!(d.page_name(4), "A-iii");
    // A section that continues numbering.
    d.sections.push(Section {
        start: 5,
        start_number: None,
        style: NumberStyle::Arabic,
        prefix: String::new(),
        marker: String::new(),
        include_prefix: false,
    });
    assert_eq!(d.page_name(5), "4");
}

#[test]
fn hostile_section_start_numbers_saturate() {
    let mut d = Document::new(&NewDocument { pages: 3, ..Default::default() });
    let first = d.sections.iter_mut().find(|s| s.start == 0).unwrap();
    first.start_number = Some(u32::MAX);
    let continuing = Section { start: 2, start_number: None, ..first.clone() };
    d.sections.push(continuing);
    assert_eq!(d.page_number(1), u32::MAX);
    assert_eq!(d.page_number(2), u32::MAX);
    assert_eq!(d.page_name(2), u32::MAX.to_string());
    for section in &mut d.sections {
        section.style = NumberStyle::Symbols;
    }
    assert_eq!(d.page_name(2), u32::MAX.to_string());
    d.sections.clear();
    assert_eq!(d.page_number(usize::MAX), u32::MAX);
    assert_eq!(d.page_name(usize::MAX), u32::MAX.to_string());
}

#[test]
fn document_serde_roundtrip() {
    let mut d = doc();
    let lid = d.default_layer();
    d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), lid, "héllo\nworld", ParaFormat::default()).unwrap();
    let s = serde_json::to_string(&d).unwrap();
    let back: Document = serde_json::from_str(&s).unwrap();
    assert_eq!(d, back);
    back.check().unwrap();
}

/// A file can set `next_id` to `u64::MAX`; the next allocated id then overflowed (IDML export
/// adds to it too). Such a document is rejected when it is checked on load.
#[test]
fn next_id_near_the_top_is_rejected() {
    let mut d = Document::new(&crate::build::NewDocument::default());
    d.check().unwrap();
    d.next_id = u64::MAX;
    assert!(d.check().is_err());
    d.next_id = crate::MAX_NEXT_ID;
    d.check().unwrap();
}

#[test]
fn find_many_agrees_with_find_across_groups_and_parent_spreads() {
    let mut d = doc();
    let lid = d.default_layer();
    let mut ids = Vec::new();
    for spread in [SpreadRef::Doc(0), SpreadRef::Doc(1), SpreadRef::Parent(0)] {
        let id = ItemId(d.alloc());
        let child = Item::new(id, lid, Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(0.0, 0.0, 20.0, 20.0)));
        let group_id = ItemId(d.alloc());
        let mut group = Item::new(group_id, lid, Shape::Group, child.path.clone());
        group.content = Content::Group { items: vec![Arc::new(child)] };
        d.insert_item(spread, group, None).unwrap();
        ids.extend([id, group_id]);
    }
    let absent = ItemId(999_999);
    let mut requested = ids.clone();
    requested.extend([absent, ids[0], ids[2]]);
    let before = d.clone();
    let found = d.find_many(&requested);
    for id in &ids {
        assert_eq!(found.get(id), d.find(*id).as_ref());
        assert_eq!(d.item_at(found.get(id).unwrap()).unwrap().id, *id);
        assert_eq!(d.find_many(&[*id, *id]).get(id), found.get(id));
    }
    assert_eq!(found.len(), ids.len());
    assert!(!found.contains_key(&absent));
    assert!(d.find_many(&[absent]).is_empty());
    assert!(d.find_many(&[]).is_empty());
    assert_eq!(d, before, "lookup must not alter the document");
}

#[test]
fn find_many_keeps_first_match_precedence_for_duplicate_item_ids() {
    // Malformed documents are rejected by check(), but lookups still agree before validation.
    let mut d = doc();
    let id = ItemId(d.alloc());
    let other = ItemId(d.alloc());
    let item = Item::new(id, d.default_layer(), Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(0.0, 0.0, 20.0, 20.0)));
    let mut group = item.clone();
    group.id = other;
    group.content = Content::Group { items: vec![Arc::new(item.clone())] };
    d.insert_item(SpreadRef::Doc(0), group, None).unwrap();
    d.insert_item(SpreadRef::Doc(0), item.clone(), None).unwrap();
    d.insert_item(SpreadRef::Parent(0), item, None).unwrap();
    let found = d.find_many(&[id, other]);
    assert_eq!(found.get(&id), d.find(id).as_ref());
    assert_eq!(found[&id].spread, SpreadRef::Doc(0));
    assert_eq!(found[&id].path, vec![0, 0]);
    assert!(d.check().is_err());
}

#[test]
fn find_many_handles_deep_groups_on_a_small_thread_stack() {
    let mut d = doc();
    let lid = d.default_layer();
    let leaf = ItemId(d.alloc());
    let mut item = Item::new(leaf, lid, Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(0.0, 0.0, 20.0, 20.0)));
    let depth = 2048;
    for _ in 0..depth {
        let mut parent = Item::new(ItemId(d.alloc()), lid, Shape::Group, item.path.clone());
        parent.content = Content::Group { items: vec![Arc::new(item)] };
        item = parent;
    }
    d.insert_item(SpreadRef::Doc(0), item, None).unwrap();
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            let single = d.find(leaf).unwrap();
            assert_eq!(single.path.len(), depth + 1);
            let many = d.find_many(&[leaf, ItemId(999_999)]);
            assert_eq!(many.get(&leaf), Some(&single));
            // Drop this deliberately deep synthetic tree iteratively too. Ordinary Arc teardown
            // is outside the locator's scope and would recursively drop the only child.
            let mut pending = std::mem::take(&mut Arc::make_mut(&mut d.spreads[0]).items);
            while let Some(item) = pending.pop() {
                let mut item = Arc::try_unwrap(item).unwrap();
                if let Content::Group { items } = &mut item.content {
                    pending.append(items);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn chapter_settings_legacy_defaults_and_labels_round_trip() {
    let mut d = doc();
    let old: DocSettings = serde_json::from_value(serde_json::json!({"chapterNumber": 7})).unwrap();
    assert_eq!(old.chapter_style, NumberStyle::Arabic);
    assert_eq!(old.chapter_source, ChapterSource::Automatic);
    d.settings = old;
    let (_, sid) =
        d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 100.0, 100.0), d.default_layer(), "Heading", ParaFormat::default()).unwrap();
    let anchor = d.paragraph_anchor(sid, 0).unwrap();
    for (style, label) in [
        (NumberStyle::Arabic, "7"),
        (NumberStyle::UpperRoman, "VII"),
        (NumberStyle::LowerRoman, "vii"),
        (NumberStyle::UpperLetters, "G"),
        (NumberStyle::LowerLetters, "g"),
        (NumberStyle::ArabicLeadingZero, "07"),
        (NumberStyle::ArabicThreeDigits, "007"),
        (NumberStyle::ArabicFourDigits, "0007"),
    ] {
        d.settings.chapter_style = style;
        d.settings.chapter_source = ChapterSource::UserDefined;
        let restored: Document = serde_json::from_slice(&serde_json::to_vec(&d).unwrap()).unwrap();
        assert_eq!(restored.settings, d.settings);
        assert_eq!(restored.chapter_label(), label);
        assert_eq!(restored.xref_values(anchor).unwrap().chapter, label);
    }
    d.settings.chapter_style = NumberStyle::Symbols;
    d.settings.chapter_number = u32::MAX;
    assert_eq!(d.chapter_label(), "99999");
    d.settings.chapter_style = NumberStyle::UpperRoman;
    assert!(d.chapter_label().len() < 200);
}

/// A placed PDF saved before its crop choice was stored loads with the crop box (the whole page),
/// and that default isn't written out.
#[test]
fn placed_pdf_without_a_crop_choice_loads_with_the_crop_box() {
    let a: Asset = serde_json::from_str(r#"{"id": 7, "name": "a.pdf", "mime": "application/pdf", "page": 1}"#).unwrap();
    assert_eq!((a.pdf_crop, a.pdf_box, a.shown_box()), (PdfCrop::Crop, None, None));
    let json = serde_json::to_string(&a).unwrap();
    assert!(!json.contains("pdfCrop") && !json.contains("pdfBox"), "{json}");
    let trim = Asset { pdf_crop: PdfCrop::Trim, pdf_box: Some([0.1, 0.1, 0.9, 0.9]), ..a };
    let back: Asset = serde_json::from_str(&serde_json::to_string(&trim).unwrap()).unwrap();
    assert_eq!((back.pdf_crop, back.shown_box()), (PdfCrop::Trim, Some([0.1, 0.1, 0.9, 0.9])));
    // A box with no area shows the whole page.
    assert_eq!(Asset { pdf_box: Some([0.5, 0.5, 0.5, 0.9]), ..back }.shown_box(), None);
}
