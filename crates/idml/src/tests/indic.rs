use super::*;

#[test]
fn indic_text_font_style_and_language_survive_independent_idml_round_trips() {
    for (language, tag, text) in [
        ("Telugu", "te", "ఉదయం ఫాంట్‌ క్షి క్\u{200D}ష క్\u{200C}ష"),
        ("Hindi", "hi", "हिन्दी क्षि क्\u{200D}ष क्\u{200C}ष"),
        ("Tamil", "ta", "தமிழ் ஸ்ரீ"),
        ("Kannada", "kn", "ಕನ್ನಡ ಕ್ಷಿ"),
        ("Malayalam", "ml", "മലയാളം ക്ഷി"),
        ("Bengali", "bn", "বাংলা ক্ষি"),
        ("Gujarati", "gu", "ગુજરાતી ક્ષિ"),
        ("Punjabi", "pa", "ਪੰਜਾਬੀ"),
        ("Oriya", "or", "ଓଡ଼ିଆ"),
    ] {
        let first = "A😀 ";
        let story = format!(
            r#"<idPkg:Story xmlns:idPkg="http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging"><Story Self="s1">
            <ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/$ID/[No paragraph style]">
            <CharacterStyleRange AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" PointSize="12"><Content>{first}</Content></CharacterStyleRange>
            <CharacterStyleRange AppliedCharacterStyle="CharacterStyle/$ID/[No character style]" FontStyle="Regular" PointSize="36" AppliedLanguage="$ID/{language}">
            <Properties><AppliedFont type="string">$ID/ఉదయం&#9;Regular</AppliedFont></Properties><Content>{text}</Content></CharacterStyleRange>
            </ParagraphStyleRange></Story></idPkg:Story>"#
        );
        let mut d = import_idml(&fixture_with_story(&story)).unwrap();
        for _ in 0..3 {
            let st = d.stories.values().find(|s| s.text.contains(text)).unwrap();
            assert_eq!(st.text, format!("{first}{text}"));
            let (range, format) = st.runs().find(|(range, _)| range.start == first.len()).unwrap();
            assert_eq!(range, first.len()..st.text.len(), "styles use UTF-8 byte offsets");
            assert_eq!(format.over.font_family.as_deref(), Some("ఉదయం"));
            assert_eq!(format.over.font_style.as_deref(), Some("Regular"));
            assert_eq!(format.over.size, Some(36.0));
            assert_eq!(format.over.language.as_deref(), Some(language));
            assert_eq!(pagenor_doc::language_tag(format.over.language.as_deref().unwrap()), Some(tag));
            d = import_idml(&export_idml(&d)).unwrap();
        }
    }
}
