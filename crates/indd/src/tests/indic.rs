use super::*;

fn chunks(parts: &[&[u16]]) -> Vec<u8> {
    let mut data = vec![0; 8];
    data.extend_from_slice(&(parts.len() as u16).to_le_bytes());
    for units in parts {
        data.extend_from_slice(&(6 + units.len() as u32 * 2).to_le_bytes());
        data.extend_from_slice(&(units.len() as u32).to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        for unit in *units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
    }
    data
}

#[test]
fn wide_story_preserves_indic_joiners_and_surrogates_across_chunks() {
    let text = "A😀 ఉదయం ఫాంట్‌ క్షి క్\u{200D}ష క్\u{200C}ష\rहिन्दी தமிழ் ಕನ್ನಡ മലയാളം";
    let units: Vec<_> = text.encode_utf16().collect();
    for cut in 0..=units.len() {
        let data = chunks(&[&units[..cut], &units[cut..]]);
        assert_eq!(parse_text(&data).unwrap(), text, "cut {cut}");
    }
    let xml = unzip(&to_idml(&crate::synthetic::unicode_sample(text).build()).unwrap());
    let story = &xml["Stories/Story_u50.xml"];
    for part in text.split('\r') {
        assert!(story.contains(part), "{story}");
    }
    assert!(!story.contains('\u{FFFD}'));
}

#[test]
fn malformed_wide_story_returns_error_instead_of_partial_unicode() {
    assert!(matches!(parse_text(&chunks(&[&[0xD800]])), Err(InddError::Damaged(_))));
    assert!(matches!(parse_text(&chunks(&[&[0xDC00]])), Err(InddError::Damaged(_))));
    let valid = chunks(&[&[0x0C15, 0x0C4D, 0x0C37]]);
    for len in 0..valid.len() {
        assert!(parse_text(&valid[..len]).is_err(), "truncated at {len}");
    }
    // Do not steal bytes from the following chunk to satisfy a corrupt character count.
    let mut data = chunks(&[&[0x0C15], &[0x0C37]]);
    data[14..18].copy_from_slice(&2u32.to_le_bytes());
    assert!(matches!(parse_text(&data), Err(InddError::Damaged(_))));
}

#[test]
fn unicode_font_strings_are_exact_and_reject_truncation() {
    let family = "ఉదయం Regular 😀";
    let units: Vec<_> = family.encode_utf16().collect();
    let mut bytes = (units.len() as u16).to_le_bytes().to_vec();
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    assert_eq!(crate::text::read_string(&bytes, 0), Some((family.into(), bytes.len())));
    for len in 0..bytes.len() {
        assert!(crate::text::read_string(&bytes[..len], 0).is_none());
    }
    assert!(crate::text::read_string(&bytes, usize::MAX).is_none());
}

#[test]
fn malformed_unicode_font_name_does_not_silently_substitute_another_family() {
    let mut sample = crate::synthetic::unicode_sample_with_font("ఉదయం", "ఉదయం", "Regular");
    let mut name = vec![0; 5];
    name.extend_from_slice(&2u16.to_le_bytes());
    name.extend_from_slice(&0u16.to_le_bytes());
    name.extend_from_slice(&0x0C09u16.to_le_bytes()); // header promises two units, only one is present
    sample.add(80, 0x3E03, crate::synthetic::object(&[(0x3E05, name)]));
    assert!(matches!(to_idml(&sample.build()), Err(InddError::Damaged(_))));
}

#[test]
fn source_utf16_run_lengths_do_not_move_indic_style_boundaries_after_emoji() {
    let text = "A😀ఉదయం";
    let bytes = crate::synthetic::unicode_sample(text).build();
    let c = Container::parse(&bytes).unwrap();
    let mut doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    let st = doc.stories.get_mut(&50).unwrap();
    st.chars = vec![
        crate::model::Run { length: 3, style: 71, attrs: Default::default() },
        crate::model::Run { length: "ఉదయం".encode_utf16().count(), style: 72, attrs: Default::default() },
    ];
    let xml = unzip(&crate::writer::write(&doc).unwrap());
    let story = &xml["Stories/Story_u50.xml"];
    assert!(story.contains("<Content>A😀</Content>"), "{story}");
    assert!(story.contains("<Content>ఉదయం</Content>"), "{story}");
    // A corrupt style boundary inside the emoji must not shift all subsequent text styles.
    doc.stories.get_mut(&50).unwrap().chars[0].length = 2;
    assert!(matches!(crate::writer::write(&doc), Err(InddError::Damaged(_))));
}
