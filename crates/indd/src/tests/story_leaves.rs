use super::*;

fn mixed_chunk(segments: &[(bool, &[u8])], count: u32) -> Vec<u8> {
    let mut body = count.to_le_bytes().to_vec();
    for (wide, bytes) in segments {
        let units = bytes.len() / if *wide { 2 } else { 1 };
        body.extend_from_slice(&((if *wide { 0x8000 } else { 0x4000 }) | units as u16).to_le_bytes());
        body.extend_from_slice(bytes);
    }
    let mut leaf = vec![0; 8];
    leaf.extend_from_slice(&1u16.to_le_bytes());
    leaf.extend_from_slice(&(body.len() as u32).to_le_bytes());
    leaf.extend_from_slice(&body);
    leaf
}

fn wide(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

fn index(refs: &[(u32, u32)]) -> Vec<u8> {
    let mut data = (refs.len() as u16).to_le_bytes().to_vec();
    for (length, uid) in refs {
        data.extend_from_slice(&length.to_le_bytes());
        data.extend_from_slice(&uid.to_le_bytes());
    }
    object(&[(0x261, data)])
}

fn run(length: u32, style: u32) -> Vec<u8> {
    let mut data = vec![0; 8];
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&10u32.to_le_bytes());
    data.extend_from_slice(&length.to_le_bytes());
    data.extend_from_slice(&style.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data
}

#[test]
fn mixed_encoding_segments_are_text_not_unicode_characters() {
    let telugu = wide("మొండిచెయ్యి");
    let hindi = wide("हिन्दी");
    let expected = "1. మొండిచెయ్యి हिन्दी";
    let count = expected.encode_utf16().count() as u32;
    let leaf = mixed_chunk(&[(false, b"1. "), (true, &telugu), (false, b" "), (true, &hindi)], count);
    assert_eq!(parse_text(&leaf).unwrap(), expected);
    for cut in 0..leaf.len() {
        assert!(parse_text(&leaf[..cut]).is_err(), "cut {cut}");
    }
    let mut wrong_count = leaf.clone();
    wrong_count[14..18].copy_from_slice(&(count + 1).to_le_bytes());
    assert!(parse_text(&wrong_count).is_err());
    let mut invalid_tag = leaf;
    invalid_tag[18..20].copy_from_slice(&0xC003u16.to_le_bytes());
    assert!(parse_text(&invalid_tag).is_err());
}

#[test]
fn mixed_encoding_retains_surrogate_pairs_across_segments() {
    let leaf = mixed_chunk(&[(true, &0xD83Du16.to_le_bytes()), (true, &0xDE00u16.to_le_bytes()), (false, &[0x80])], 3);
    assert_eq!(parse_text(&leaf).unwrap(), "😀€");
}

fn multiple_leaves() -> crate::synthetic::Synth {
    let mut s = sample();
    s.add(51, 0x234, index(&[(6, 61), (4, 90)]));
    s.add(61, 0x27E, object(&[(0x262, mixed_chunk(&[(false, b"Hello ")], 6))]));
    s.add(90, 0x27E, object(&[(0x262, mixed_chunk(&[(true, &wide("ఉదయం"))], 4))]));
    s.add(52, 0x236, index(&[(6, 62), (4, 91)]));
    s.add(62, 0xCA17, object(&[(0x262, run(6, 70))]));
    s.add(91, 0xCA17, object(&[(0x262, run(4, 70))]));
    s.add(54, 0x235, index(&[(6, 64), (4, 92)]));
    s.add(64, 0xCA18, object(&[(0x262, run(6, 71))]));
    s.add(92, 0xCA18, object(&[(0x262, run(4, 72))]));
    s
}

#[test]
fn stories_and_style_runs_read_every_leaf_in_source_order() {
    let bytes = multiple_leaves().build();
    let c = Container::parse(&bytes).unwrap();
    let doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    let st = &doc.stories[&50];
    assert_eq!(st.text, "Hello ఉదయం");
    assert_eq!(st.paras.len(), 2);
    assert_eq!(st.chars.len(), 2);
    assert_eq!(st.chars[1].style, 72);
    let xml = unzip(&to_idml(&bytes).unwrap());
    let story = &xml["Stories/Story_u50.xml"];
    assert!(story.contains("<Content>Hello </Content>"));
    assert!(story.contains("<Content>ఉదయం</Content>"));
    assert!(story.contains("CharacterStyle/Accent"));
}

#[test]
fn damaged_leaf_indexes_cannot_silently_truncate_or_repeat_a_story() {
    for refs in [vec![(6, 61), (4, 999)], vec![(6, 61), (6, 61)]] {
        let mut s = multiple_leaves();
        s.add(51, 0x234, index(&refs));
        assert!(matches!(to_idml(&s.build()), Err(InddError::Damaged(_))));
    }
    let mut s = multiple_leaves();
    s.add(51, 0x234, object(&[(0x261, vec![2, 0, 6, 0, 0, 0, 61, 0, 0, 0])]));
    assert!(matches!(to_idml(&s.build()), Err(InddError::Damaged(_))));
}

#[test]
fn paragraph_spacing_and_left_justified_last_lines_are_preserved() {
    let bytes = sample().build();
    let c = Container::parse(&bytes).unwrap();
    let mut doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    let attrs = &mut doc.stories.get_mut(&50).unwrap().paras[0].attrs;
    attrs.insert(crate::model::TA_JUSTIFY, vec![(0x1B20, 4u16.to_le_bytes().to_vec())]);
    for (id, value) in [(crate::model::TA_SPACE_BEFORE, 2.0f64), (crate::model::TA_SPACE_AFTER, 8.503937007874017)] {
        attrs.insert(id, vec![(0x1BA8, value.to_le_bytes().to_vec())]);
    }
    let xml = unzip(&crate::writer::write(&doc).unwrap());
    let story = &xml["Stories/Story_u50.xml"];
    assert!(story.contains(r#"Justification="LeftJustified""#));
    assert!(story.contains(r#"SpaceBefore="2""#));
    assert!(story.contains(r#"SpaceAfter="8.503937""#));
}

#[test]
fn page_numbers_continue_across_all_spreads() {
    let bytes = sample().build();
    let c = Container::parse(&bytes).unwrap();
    let mut doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    let original = doc.spreads[0].clone();
    doc.spreads.clear();
    for i in 0..33 {
        let mut sp = original.clone();
        sp.uid = 1000 + i;
        sp.pages[0].uid = 2000 + i;
        sp.items.clear();
        doc.spreads.push(sp);
    }
    let xml = unzip(&crate::writer::write(&doc).unwrap());
    for i in 0..33 {
        let spread = &xml[&format!("Spreads/Spread_u{}.xml", 1000 + i)];
        assert!(spread.contains(&format!(r#"Self="u{}" Name="{}""#, 2000 + i, i + 1)), "{spread}");
    }
}
