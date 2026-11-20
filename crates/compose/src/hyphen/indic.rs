//! Indic syllable patterns from hyph-utf8 through hypher; English retains Pagenor's dictionaries.
use super::Limits;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn language(tag: &str) -> Option<hypher::Lang> {
    let code = pagenor_doc::language_subtag(tag);
    let bytes: [u8; 2] = code.as_bytes().try_into().ok()?;
    hypher::Lang::from_iso(bytes)
}

pub(crate) fn joiner(c: char) -> bool {
    matches!(
        c,
        '\u{094d}'
            | '\u{09cd}'
            | '\u{0a4d}'
            | '\u{0acd}'
            | '\u{0b4d}'
            | '\u{0bcd}'
            | '\u{0c4d}'
            | '\u{0ccd}'
            | '\u{0d4d}'
            | '\u{0d3b}'
            | '\u{0d3c}'
            | '\u{093c}'
            | '\u{09bc}'
            | '\u{0a3c}'
            | '\u{0abc}'
            | '\u{0b3c}'
            | '\u{0c3c}'
            | '\u{0cbc}'
            | '\u{200c}'
            | '\u{200d}'
    )
}

pub(crate) fn points(word: &str, lang: hypher::Lang, limits: &Limits) -> Vec<usize> {
    if word.len() > 16_384 || word.chars().filter(|c| c.is_alphabetic()).count() < limits.min_word.max(2) {
        return Vec::new();
    }
    // Pattern points are char indices. Require a grapheme boundary as well, so a virama,
    // dependent vowel, nukta or joiner can never be separated from its shaping cluster.
    let boundaries: std::collections::HashSet<usize> = word.grapheme_indices(true).map(|(i, _)| i).collect();
    let mut bytes = 0;
    let mut chars = 0;
    let mut out = Vec::new();
    for syllable in hypher::hyphenate_bounded(word, lang, limits.after_first.max(1), limits.before_last.max(1)) {
        bytes += syllable.len();
        chars += syllable.chars().count();
        if bytes < word.len() && boundaries.contains(&bytes) {
            out.push(chars);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indian_languages_are_available_without_changing_english() {
        for tag in ["hi", "mr", "bn", "as", "gu", "pa", "or", "ta", "te", "kn", "ml"] {
            assert!(language(tag).is_some(), "{tag}");
        }
        for tag in ["en", "fr", "sa", "xx", ""] {
            assert!(language(tag).is_none(), "{tag}");
        }
        assert_eq!(language("te-IN"), language("te"));
    }
    #[test]
    fn patterns_break_indic_words_only_at_complete_graphemes() {
        for (word, tag) in [
            ("విశ్వవిద్యాలయం", "te"),
            ("विद्यार्थी", "hi"),
            ("महाराष्ट्र", "mr"),
            ("തിരുവനന്തപുരം", "ml"),
            ("தொழில்நுட்பம்", "ta"),
            ("বিশ্ববিদ্যালয়", "bn"),
            ("ವಿಶ್ವವಿದ್ಯಾಲಯ", "kn"),
            ("વિશ્વવિદ્યાલય", "gu"),
        ] {
            let points = points(word, language(tag).unwrap(), &Limits { min_word: 2, ..Limits::default() });
            assert!(!points.is_empty(), "{word}");
            let boundaries: Vec<usize> = word.grapheme_indices(true).map(|(i, _)| i).collect();
            for p in points {
                let byte = word.char_indices().nth(p).unwrap().0;
                assert!(boundaries.contains(&byte), "{word}: {p}");
            }
        }
    }
    #[test]
    fn conjuncts_joiners_limits_and_oversized_words_stay_whole() {
        let language = language("te").unwrap();
        let limits = Limits { min_word: 100, ..Limits::default() };
        assert!(points("విశ్వవిద్యాలయం", language, &limits).is_empty());
        assert!(points(&"క".repeat(6000), language, &Limits::default()).is_empty());
        for c in ['\u{0c4d}', '\u{093c}', '\u{200c}', '\u{200d}'] {
            assert!(joiner(c));
        }
        for word in ["అవన్\u{200d}ట", "పడ్\u{200c}ట"] {
            for p in points(word, language, &Limits { min_word: 2, ..Limits::default() }) {
                let at = word.char_indices().nth(p).unwrap().0;
                assert!(word.grapheme_indices(true).any(|(i, _)| i == at));
            }
        }
    }
}
