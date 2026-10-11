//! IDML round-trip tests (the sample magazine) and the interchange commands.

use pagenor_doc::{Document, ItemId};
use serde_json::json;

use crate::Session;

/// Every item of a spread list in order, flattened (pre-order).
fn items(spreads: &[std::sync::Arc<pagenor_doc::Spread>]) -> Vec<(ItemId, pagenor_geom::Rect)> {
    let mut v = Vec::new();
    for sp in spreads {
        for it in &sp.items {
            it.walk(&mut |i| v.push((i.id, i.bounds())));
        }
    }
    v
}

/// Thread order of every story as positions in the flattened item list.
fn positions(d: &Document) -> Vec<Vec<usize>> {
    let all: Vec<ItemId> = items(&d.spreads).into_iter().chain(items(&d.parents)).map(|(id, _)| id).collect();
    d.stories.values().map(|s| s.frames.iter().map(|f| all.iter().position(|i| i == f).expect("frame")).collect()).collect()
}

#[test]
fn magazine_round_trips_through_idml() {
    let d = crate::sample::magazine();
    let bytes = pagenor_idml::export_idml(&d);
    let back = pagenor_idml::import_idml(&bytes).unwrap();
    assert_eq!(back.page_count(), d.page_count());
    assert_eq!(back.spreads.len(), d.spreads.len());
    assert_eq!(back.parents.len(), d.parents.len());
    // Stories: same text in the same order, same styles applied.
    let ta: Vec<&str> = d.stories.values().map(|s| s.text.as_str()).collect();
    let tb: Vec<&str> = back.stories.values().map(|s| s.text.as_str()).collect();
    assert_eq!(ta, tb);
    for (a, b) in d.stories.values().zip(back.stories.values()) {
        let sa: Vec<&str> = a.paras.iter().map(|p| p.style.as_str()).collect();
        let sb: Vec<&str> = b.paras.iter().map(|p| p.style.as_str()).collect();
        assert_eq!(sa, sb);
        let ca: Vec<(usize, &str)> = a.chars.iter().map(|r| (r.len, r.format.style.as_str())).collect();
        let cb: Vec<(usize, &str)> = b.chars.iter().map(|r| (r.len, r.format.style.as_str())).collect();
        assert_eq!(ca, cb);
        for (ra, rb) in a.chars.iter().zip(&b.chars) {
            assert_eq!(ra.format.over, rb.format.over);
        }
        for (pa, pb) in a.paras.iter().zip(&b.paras) {
            assert_eq!(pa.para, pb.para);
        }
    }
    // Styles and swatches by name.
    for s in &d.styles.paragraph {
        let b = back.styles.para(&s.name).unwrap_or_else(|| panic!("paragraph style {}", s.name));
        if s.name != pagenor_doc::NO_PARA_STYLE {
            assert_eq!(b.chars, s.chars, "{}", s.name);
            assert_eq!(b.para, s.para, "{}", s.name);
        }
    }
    for s in &d.styles.character {
        let b = back.styles.char_style(&s.name).unwrap_or_else(|| panic!("character style {}", s.name));
        assert_eq!(b.chars, s.chars);
    }
    for s in &d.swatches {
        let b = back.swatch(&s.name).unwrap_or_else(|| panic!("swatch {}", s.name));
        assert_eq!(b.value, s.value, "{}", s.name);
    }
    // Frame geometry (document and parent spreads), pages.
    for (sa, sb) in [(&d.spreads, &back.spreads), (&d.parents, &back.parents)] {
        let (ia, ib) = (items(sa), items(sb));
        assert_eq!(ia.len(), ib.len());
        for ((_, a), (_, b)) in ia.iter().zip(&ib) {
            let close = (a.x0 - b.x0).abs() < 0.01 && (a.y0 - b.y0).abs() < 0.01 && (a.x1 - b.x1).abs() < 0.01 && (a.y1 - b.y1).abs() < 0.01;
            assert!(close, "{a:?} vs {b:?}");
        }
        for (pa, pb) in sa.iter().flat_map(|s| s.pages.iter()).zip(sb.iter().flat_map(|s| s.pages.iter())) {
            assert!((pa.x - pb.x).abs() < 0.01 && pa.side == pb.side && pa.margins == pb.margins && pa.columns == pb.columns);
            assert_eq!(pa.parent.is_some(), pb.parent.is_some());
        }
    }
    // Threads: the same frames in the same order.
    assert_eq!(positions(&d), positions(&back));
    assert!(back.stories.values().any(|s| s.frames.len() == 2));
    // Images are embedded.
    assert_eq!(back.assets.len(), d.assets.len());
    for (a, b) in d.assets.values().zip(back.assets.values()) {
        assert_eq!(a.data, b.data);
        assert_eq!(a.pixels, b.pixels);
    }
    // A second round trip is stable.
    let again = pagenor_idml::import_idml(&pagenor_idml::export_idml(&back)).unwrap();
    assert_eq!(items(&again.spreads).len(), items(&back.spreads).len());
}

/// A one-page IDML package: two threaded frames side by side, 19 lines each (10 pt on 12 pt),
/// paragraphs of `lines` forced-break lines in style `Body`, which carries `body_attrs`.
fn keep_lines_idml(body_attrs: &str, lines: &[usize]) -> Vec<u8> {
    use std::io::Write;
    let paras: String = lines
        .iter()
        .enumerate()
        .map(|(p, &n)| {
            let text = (0..n).map(|k| format!("P{p} line {k}")).collect::<Vec<_>>().join("\u{2028}");
            format!(
                r#"<ParagraphStyleRange AppliedParagraphStyle="ParagraphStyle/Body"><CharacterStyleRange AppliedCharacterStyle="CharacterStyle/$ID/[No character style]"><Content>{text}</Content>{}</CharacterStyleRange></ParagraphStyleRange>"#,
                if p + 1 < lines.len() { "<Br/>" } else { "" }
            )
        })
        .collect();
    let frame = |me: &str, x: f64, prev: &str, next: &str| {
        let pts: String = [(x, 36.0), (x, 276.0), (x + 180.0, 276.0), (x + 180.0, 36.0)]
            .iter()
            .map(|(x, y)| format!(r#"<PathPointType Anchor="{x} {y}" LeftDirection="{x} {y}" RightDirection="{x} {y}"/>"#))
            .collect();
        format!(
            r#"<TextFrame Self="{me}" ParentStory="s1" PreviousTextFrame="{prev}" NextTextFrame="{next}" ItemTransform="1 0 0 1 0 0"><Properties><PathGeometry><GeometryPathType PathOpen="false"><PathPointArray>{pts}</PathPointArray></GeometryPathType></PathGeometry></Properties></TextFrame>"#
        )
    };
    let designmap = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Document xmlns:idPkg="http://ns.adobe.com/AdobeInDesign/idml/1.0/packaging" DOMVersion="16.0" Self="d">
<DocumentPreference PageWidth="450" PageHeight="312" FacingPages="false"/>
<RootCharacterStyleGroup Self="rc"><CharacterStyle Self="CharacterStyle/$ID/[No character style]" Name="$ID/[No character style]"/></RootCharacterStyleGroup>
<RootParagraphStyleGroup Self="rp">
<ParagraphStyle Self="ParagraphStyle/$ID/[No paragraph style]" Name="$ID/[No paragraph style]" PointSize="12"/>
<ParagraphStyle Self="ParagraphStyle/Body" Name="Body" PointSize="10" {body_attrs}><Properties><BasedOn type="object">ParagraphStyle/$ID/[No paragraph style]</BasedOn><Leading type="unit">12</Leading></Properties></ParagraphStyle>
</RootParagraphStyleGroup>
<Story Self="s1">{paras}</Story>
<Spread Self="sp1" ItemTransform="1 0 0 1 0 0"><Page Self="p1" GeometricBounds="0 0 312 450" ItemTransform="1 0 0 1 0 0"/>{}{}</Spread>
</Document>"#,
        frame("t1", 36.0, "n", "t2"),
        frame("t2", 234.0, "t1", "n")
    );
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    w.start_file("mimetype", stored).unwrap();
    w.write_all(b"application/vnd.adobe.indesign-idml-package").unwrap();
    w.start_file("designmap.xml", zip::write::SimpleFileOptions::default()).unwrap();
    w.write_all(designmap.as_bytes()).unwrap();
    w.finish().unwrap().into_inner()
}

/// Lines of paragraph `para` in each frame of the story.
fn para_lines_per_frame(d: &Document, para: usize) -> Vec<usize> {
    let sid = *d.stories.keys().next().unwrap();
    let cs = pagenor_compose::compose_story(d, sid, &Default::default());
    cs.frames.iter().map(|f| f.lines.iter().filter(|l| l.para == para).count()).collect()
}

/// Keep Lines Together without a mode in the IDML: InDesign's default, at start/end of paragraph.
/// A 10-line paragraph starting 7 lines above the first frame's bottom (the last baseline may sit
/// on the bottom edge) splits 7/3.
#[test]
fn idml_keep_lines_without_mode_split_at_start_and_end() {
    let keep = r#"KeepLinesTogether="true" KeepFirstLines="2" KeepLastLines="2""#;
    let d = pagenor_idml::import_idml(&keep_lines_idml(keep, &[13, 10, 2])).unwrap();
    assert_eq!(para_lines_per_frame(&d, 0), [13, 0]);
    assert_eq!(para_lines_per_frame(&d, 1), [7, 3]);
    // All lines in paragraph, written explicitly: the paragraph moves whole.
    let d = pagenor_idml::import_idml(&keep_lines_idml(&format!(r#"{keep} KeepAllLinesTogether="true""#), &[13, 10, 2])).unwrap();
    assert_eq!(para_lines_per_frame(&d, 1), [0, 10]);
}

/// A document saved in format 1 with Keep Lines Together and no mode still keeps all lines
/// together after loading; saved now, it keeps lines at start/end of paragraph.
#[test]
fn format_1_keep_lines_without_mode_lays_out_all_lines() {
    use std::io::Write;
    let d = pagenor_idml::import_idml(&keep_lines_idml(r#"KeepLinesTogether="true""#, &[13, 10, 2])).unwrap();
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    w.start_file("meta.json", zip::write::SimpleFileOptions::default()).unwrap();
    w.write_all(br#"{"format": "pagenor", "version": 1}"#).unwrap();
    w.start_file("document.json", zip::write::SimpleFileOptions::default()).unwrap();
    w.write_all(&serde_json::to_vec(&d).unwrap()).unwrap();
    let v1 = pagenor_format::load(&w.finish().unwrap().into_inner()).unwrap();
    assert_eq!(para_lines_per_frame(&v1, 1), [0, 10]);
    let now = pagenor_format::load(&pagenor_format::save(&d).unwrap()).unwrap();
    assert_eq!(para_lines_per_frame(&now, 1), [7, 3]);
}

#[test]
fn idml_commands() {
    let mut s = Session::new();
    s.execute("file.newSample", &json!({})).unwrap();
    let r = s.execute("file.exportIdml", &json!({})).unwrap();
    let b64 = r["base64"].as_str().unwrap().to_string();
    let r = s.execute("file.openIdml", &json!({"base64": b64, "name": "copy.idml"})).unwrap();
    let i = r["index"].as_u64().unwrap() as usize;
    assert_eq!(s.doc().unwrap().doc.title, "copy");
    assert_eq!(s.doc().unwrap().doc.page_count(), 4);
    assert!(s.doc().unwrap().path.is_none());
    // openBytes recognises the package too.
    let r = s.execute("file.openBytes", &json!({"base64": b64, "name": "again.idml"})).unwrap();
    assert_eq!(r["index"].as_u64().unwrap() as usize, i + 1);
    assert!(s.execute("file.openIdml", &json!({})).is_err());
    let dir = std::env::temp_dir().join(format!("dc-idml-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("mag.idml").to_string_lossy().to_string();
    let r = s.execute("file.exportIdml", &json!({"path": p})).unwrap();
    assert!(r["bytes"].as_u64().unwrap() > 1000);
    s.execute("file.open", &json!({"path": p})).unwrap();
    assert_eq!(s.doc().unwrap().doc.title, "mag");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn gradient_vector_round_trips_through_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let id = s.execute("frame.create", &json!({"rect": [100, 100, 300, 200]})).unwrap()["id"].as_u64().unwrap();
    s.execute("object.gradient", &json!({"from": [120, 180], "to": [280, 120]})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let v = d.item(ItemId(id)).unwrap().fill.gradient_vector.unwrap();
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    let it = back.spreads.iter().flat_map(|sp| sp.items.iter()).find(|i| i.fill.gradient_vector.is_some()).expect("a gradient item");
    let w = it.fill.gradient_vector.unwrap();
    for k in 0..4 {
        assert!((v[k] - w[k]).abs() < 1e-3, "{v:?} vs {w:?}");
    }
}

#[test]
fn open_type_features_round_trip_through_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 300, 200], "content": "text", "text": "1/2 Office"})).unwrap();
    let sid = r["story"].as_u64().unwrap();
    s.execute("text.select", &json!({"story": sid, "anchor": 0, "focus": 3})).unwrap();
    s.execute("type.openType", &json!({"feature": "frac"})).unwrap();
    s.execute("type.openType", &json!({"figures": "tabularOldstyle", "stylisticSets": [3]})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    let st = back.stories.values().find(|st| st.text.starts_with("1/2")).unwrap();
    let f = st.format_after(0).over.otf_features.clone().unwrap_or_default();
    assert!(pagenor_doc::otf::is_on(&f, "frac"), "{f:?}");
    assert_eq!(pagenor_doc::otf::figures(&f), "tabularOldstyle");
    assert_eq!(pagenor_doc::otf::stylistic_sets(&f), 0b100);
    // Text after the selection has no features of its own.
    let after = st.format_after(5).over.otf_features.clone().unwrap_or_default();
    assert!(!pagenor_doc::otf::is_on(&after, "frac") && pagenor_doc::otf::stylistic_sets(&after) == 0, "{after:?}");
}

#[test]
fn underline_options_render_and_round_trip() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 200], "content": "text", "text": "Underlined words"})).unwrap();
    let sid = r["story"].as_u64().unwrap();
    s.execute("text.select", &json!({"story": sid, "anchor": 0, "focus": 10})).unwrap();
    s.execute("type.char", &json!({"attrs": {"underline": true, "underlineWeight": 3, "underlineOffset": 4, "underlineColor": "C=100 M=0 Y=0 K=0"}}))
        .unwrap();
    let d = s.doc().unwrap().doc.clone();
    let cs = pagenor_compose::compose_story(&d, pagenor_doc::StoryId(sid), &Default::default());
    let st = cs.styles.iter().find(|st| st.underline).unwrap();
    assert_eq!((st.underline_rule.weight, st.underline_rule.offset), (3.0, 4.0));
    assert_eq!(st.underline_rule.color, "C=100 M=0 Y=0 K=0");
    // Drawn in cyan, 4–7 pt under the baseline.
    let l = &cs.frames[0].lines[0];
    let mut rr = pagenor_render::Renderer::new();
    rr.threads = 0;
    let img = rr.render_page(&d, &s.cache, 0, 1.0, false, &Default::default()).unwrap();
    let x = l.glyphs[2].x as u32;
    let p = img.pixel(x, (l.baseline + 5.5) as u32);
    assert!(p[0] < 80 && p[2] > 150, "cyan underline: {p:?}");
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    let story = back.stories.values().find(|x| x.text.starts_with("Underlined")).unwrap();
    let f = story.format_after(0);
    assert_eq!(f.over.underline_weight, Some(Some(3.0)));
    assert_eq!(f.over.underline_offset, Some(Some(4.0)));
    assert_eq!(f.over.underline_color.as_deref(), Some("C=100 M=0 Y=0 K=0"));
}

#[test]
fn newer_features_round_trip_through_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    // Nested and GREP styles on a paragraph style.
    s.execute("style.character.create", &json!({"name": "Lead", "chars": {"fontStyle": "Bold"}})).unwrap();
    s.execute(
        "style.paragraph.create",
        &json!({"name": "Intro", "para": {
            "nestedStyles": [{"style": "Lead", "through": false, "count": 2, "until": {"kind": "chars", "chars": ":"}}],
            "grepStyles": [{"style": "Lead", "pattern": "\\d+"}]
        }}),
    )
    .unwrap();
    // Cell and table styles on a table.
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 500, 300], "content": "text", "text": ""})).unwrap();
    s.execute("text.select", &json!({"story": r["story"], "anchor": 0, "focus": 0})).unwrap();
    s.execute("table.insert", &json!({"rows": 2, "cols": 2, "headerRows": 1})).unwrap();
    s.execute("style.cell.create", &json!({"name": "Head", "fill": "[Black]", "insets": 5, "paragraphStyle": "Intro"})).unwrap();
    s.execute("style.table.create", &json!({"name": "Grid", "header": "Head", "border": {"weight": 3, "color": "[Black]"}})).unwrap();
    s.execute("style.table.apply", &json!({"name": "Grid"})).unwrap();
    // Type on a path.
    let line = s.execute("line.create", &json!({"a": [100, 500], "b": [400, 500]})).unwrap()["id"].as_u64().unwrap();
    s.execute("type.onPath", &json!({"id": line, "text": "On the line", "start": 12, "flip": true, "align": "center"})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    let intro = back.styles.para("Intro").unwrap();
    let ns = intro.para.nested_styles.clone().unwrap();
    assert_eq!(ns[0].style, "Lead");
    assert_eq!((ns[0].through, ns[0].count), (false, 2));
    assert_eq!(ns[0].until, pagenor_doc::NestedUntil::Chars(":".into()));
    assert_eq!(intro.para.grep_styles.clone().unwrap()[0].pattern, "\\d+");
    let head = back.styles.cell.iter().find(|c| c.name == "Head").unwrap();
    assert_eq!((head.fill.as_deref(), head.insets, head.paragraph_style.as_deref()), (Some("[Black]"), Some([5.0; 4]), Some("Intro")));
    let grid = back.styles.table.iter().find(|t| t.name == "Grid").unwrap();
    assert_eq!(grid.header.as_deref(), Some("Head"));
    assert_eq!(grid.border.as_ref().unwrap().weight, 3.0);
    let t = back.stories.values().flat_map(|st| st.tables.values()).next().unwrap();
    assert_eq!(t.style, "Grid");
    assert_eq!(t.cell(0, 0).unwrap().style, "Head");
    let pt = back
        .spreads
        .iter()
        .flat_map(|sp| sp.items.iter())
        .find_map(|it| match &it.content {
            pagenor_doc::Content::Text(tf) => tf.options.path.clone().map(|p| (p, tf.story)),
            _ => None,
        })
        .expect("type on a path");
    assert_eq!((pt.0.start, pt.0.flip, pt.0.align), (12.0, true, pagenor_doc::PathAlign::Center));
    assert_eq!(back.story(pt.1).unwrap().text, "On the line");
}

#[test]
fn named_lists_round_trip_through_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("list.define", &json!({"name": "Steps", "continueAcrossStories": false})).unwrap();
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": "One\nTwo"})).unwrap();
    s.execute("text.select", &json!({"story": r["story"], "anchor": 0, "focus": 3})).unwrap();
    s.execute("type.para", &json!({"attrs": {"listType": "numbers", "listName": "Steps", "startAt": 5}})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    assert_eq!(back.settings.lists, vec![pagenor_doc::NumberedList { name: "Steps".into(), continue_across_stories: false }]);
    let st = back.stories.values().find(|st| st.text.starts_with("One")).unwrap();
    assert_eq!(st.paras[0].para.list_name.as_deref(), Some("Steps"));
    assert_eq!(st.paras[0].para.start_at, Some(Some(5)));
}

#[test]
fn type_para_sets_a_drop_cap_that_survives_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("style.character.create", &json!({"name": "Initial", "chars": {"fontStyle": "Bold"}})).unwrap();
    let text = "Once upon a time there was a paragraph long enough to run beside its drop cap for a few lines and then some more.";
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 300, 400], "content": "text", "text": text})).unwrap();
    let sid = r["story"].as_u64().unwrap();
    s.execute("text.select", &json!({"story": sid, "anchor": 3, "focus": 3})).unwrap();
    s.execute("type.para", &json!({"attrs": {"dropCapLines": 3, "dropCapChars": 1, "dropCapStyle": "Initial", "dropCapAlignLeft": false}})).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let cs = pagenor_compose::compose_story(&d, pagenor_doc::StoryId(sid), &Default::default());
    let lines = &cs.frames[0].lines;
    let dc = &lines[0].glyphs[0];
    assert_eq!(dc.byte, 0);
    assert!((lines[0].baseline + dc.y - lines[2].baseline).abs() < 1e-6, "on line 3's baseline");
    assert!(lines[0].drop_cap.is_some_and(|b| b.end == 1));
    let p = &d.story(pagenor_doc::StoryId(sid)).unwrap().paras[0].para;
    assert_eq!((p.drop_cap_lines, p.drop_cap_chars, p.drop_cap_style.as_deref()), (Some(3), Some(1), Some("Initial")));
    let back = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
    let p = &back.stories.values().find(|st| st.text.starts_with("Once")).unwrap().paras[0].para;
    assert_eq!((p.drop_cap_lines, p.drop_cap_chars, p.drop_cap_align_left), (Some(3), Some(1), Some(false)));
    // IDML has no attribute for the drop cap's character style.
    assert_eq!(p.drop_cap_style, None);
}

#[test]
fn opens_indesign_documents() {
    let indd = pagenor_indd::synthetic::sample().build();
    let mut s = Session::new();
    let b64 = crate::cmd::base64_encode(&indd);
    let result = s.execute("file.openBytes", &json!({"base64": b64, "name": "card.indd"})).unwrap();
    assert!(result["importWarnings"][0].as_str().unwrap().contains("subset"));
    let d = &s.doc().unwrap().doc;
    assert_eq!(d.title, "card");
    assert_eq!(d.page_count(), 1);
    let texts: Vec<&str> = d.stories.values().map(|st| st.text.as_str()).collect();
    assert!(texts.iter().any(|t| t.contains("Hello") && t.contains("World")), "{texts:?}");
    // file.open routes the extension too.
    let dir = std::env::temp_dir().join(format!("pagenor-indd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("card.indd");
    std::fs::write(&p, &indd).unwrap();
    s.execute("file.open", &json!({"path": p.to_string_lossy()})).unwrap();
    assert_eq!(s.doc().unwrap().doc.title, "card");
    let _ = std::fs::remove_dir_all(&dir);
    // Damaged documents are an error, not a crash.
    let mut broken = indd.clone();
    broken.truncate(8192);
    let b64 = crate::cmd::base64_encode(&broken);
    let before = s.docs.len();
    assert!(s.execute("file.openBytes", &json!({"base64": b64, "name": "broken.indd"})).is_err());
    assert_eq!(s.docs.len(), before);
    let placed = s.execute("file.place", &json!({"base64":crate::cmd::base64_encode(&indd),"name":"placed.indd"})).unwrap();
    assert!(placed["importWarnings"].is_array());
}

#[test]
fn indic_idml_and_indd_open_native_reopen_and_pdf_preserve_text_and_font_identity() {
    use pagenor_doc::{ParaFormat, SpreadRef};
    use pagenor_geom::Rect;
    // ToUnicode-only extraction here; the importer tests also preserve ZWJ/ZWNJ, whose
    // invisible glyph substitutions can require PDF ActualText instead of ToUnicode.
    let text = "A😀 ఉదయం క్షి";
    let family = "Pagenor Import Indic Test";
    // Give every source character a distinct glyph so PDF ToUnicode can distinguish them.
    // The separate composition regressions test partial coverage and cluster fallback.
    let chars: std::collections::BTreeSet<_> = text.chars().collect();
    let mapping: Vec<_> = chars.into_iter().zip("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars()).collect();
    pagenor_fonts::FontDb::global().add_font(pagenor_fonts::testing::font_mapping(family, &mapping).unwrap());
    let mut source = Document::new(&pagenor_doc::build::NewDocument::default());
    let (_, sid) =
        source.add_text_frame(SpreadRef::Doc(0), Rect::new(36.0, 36.0, 300.0, 300.0), source.default_layer(), text, ParaFormat::default()).unwrap();
    source.story_mut(sid).unwrap().format_chars(0..text.len(), |f| {
        f.over.font_family = Some(family.into());
        f.over.font_style = Some("Regular".into());
        f.over.language = Some("Telugu".into());
    });
    let files = [
        ("indic.idml", pagenor_idml::export_idml(&source)),
        ("indic.indd", pagenor_indd::synthetic::unicode_sample_with_font(text, family, "Regular").build()),
    ];
    for (name, bytes) in files {
        let mut s = Session::new();
        s.execute("file.openBytes", &json!({"base64": crate::cmd::base64_encode(&bytes), "name": name})).unwrap();
        assert!(s.doc().unwrap().path.is_none(), "imports start unsaved");
        for native_reopen in [false, true] {
            let doc = &s.doc().unwrap().doc;
            let story = doc.stories.values().find(|story| story.text == text).unwrap();
            let (_, base) = doc.styles.resolve_para(&story.paras[0]);
            let props = doc.styles.resolve_char(&base, story.runs().next().unwrap().1);
            assert_eq!(props.font_family, family, "{name}: font identity");
            assert_eq!(props.font_style, "Regular", "{name}: font style");
            if name.ends_with(".idml") {
                assert_eq!(pagenor_doc::language_tag(&props.language), Some("te"));
            }
            let pdf = s.execute("file.exportPdf", &json!({})).unwrap();
            let pdf = crate::cmd::base64_decode(pdf["base64"].as_str().unwrap());
            let extracted = crate::tests::pdf_text(&pdf).concat();
            assert_eq!(extracted, text, "{name}, native reopen {native_reopen}");
            if !native_reopen {
                let native = pagenor_format::save(&s.doc().unwrap().doc).unwrap();
                s.execute("file.openBytes", &json!({"base64": crate::cmd::base64_encode(&native), "name": "indic.pgnr"})).unwrap();
            }
        }
    }
}

/// IDML says which page of a placed PDF shows and which box its graphic spans, so InDesign opens
/// it at the same size and position; importing it back gives the same page, crop and geometry.
#[test]
fn placed_pdf_page_and_crop_round_trip_through_idml() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages": 2})).unwrap();
    s.execute("layout.documentSetup", &json!({"bleed": 9})).unwrap();
    let b64 = s.execute("file.exportPdf", &json!({"bleed": true})).unwrap()["base64"].as_str().unwrap().to_string();
    let mut t = Session::new();
    t.execute("file.new", &json!({})).unwrap();
    // Page 2, cropped to its trim box: the frame is smaller than the page with its bleed.
    let place = json!({"base64": b64, "name": "a.pdf", "pdfPage": 2, "pdfCrop": "trim", "width": 315, "x": 20, "y": 30});
    let r = t.execute("file.place", &place).unwrap();
    let d = t.doc().unwrap().doc.clone();
    let item = d.item(ItemId(r["id"].as_u64().unwrap())).unwrap();
    let (bounds, g) = (item.bounds(), item.graphic().unwrap().clone());
    assert_eq!(d.assets[&g.asset].page, 1);
    assert_eq!(d.assets[&g.asset].pdf_crop, pagenor_doc::PdfCrop::Trim);

    let bytes = pagenor_idml::export_idml(&d);
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_slice())).unwrap();
    let mut spreads = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        if f.name().starts_with("Spreads/") {
            std::io::Read::read_to_string(&mut f, &mut spreads).unwrap();
        }
    }
    let attr = spreads.split("<PDFAttribute ").nth(1).and_then(|a| a.split("/>").next()).expect("a PDFAttribute");
    assert!(attr.contains(r#"PageNumber="2""#) && attr.contains(r#"PDFCrop="CropTrim""#), "{attr}");
    assert!(spreads.contains(r#"<GraphicBounds Left="0" Top="0" Right="612" Bottom="792"/>"#), "the trim box");

    let back = crate::cmd::interchange::import(&bytes, None).unwrap();
    let bi = back.spreads.iter().flat_map(|sp| sp.items.iter()).find(|i| i.graphic().is_some()).unwrap();
    let bg = bi.graphic().unwrap();
    let ba = &back.assets[&bg.asset];
    assert_eq!((ba.page, ba.pdf_crop), (1, pagenor_doc::PdfCrop::Trim));
    let (want, got) = (d.assets[&g.asset].shown_box().unwrap(), ba.shown_box().expect("the trim box found on the page"));
    assert!(want.iter().zip(got).all(|(a, b)| (a - b).abs() < 1e-9), "{got:?} vs {want:?}");
    let bb = bi.bounds();
    assert!([bb.x0 - bounds.x0, bb.y0 - bounds.y0, bb.x1 - bounds.x1, bb.y1 - bounds.y1].iter().all(|v| v.abs() < 1e-3), "{bb:?} vs {bounds:?}");
    assert!((bg.size.0 - g.size.0).abs() < 1e-3 && (bg.size.1 - g.size.1).abs() < 1e-3);
    let (a, b) = ((bi.xf * bg.xf).as_coeffs(), (item.xf * g.xf).as_coeffs());
    assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3), "{a:?} vs {b:?}");

    // A page the PDF doesn't have imports, and the PDF export leaves it out with a warning.
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_slice())).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut body = String::new();
        std::io::Read::read_to_string(&mut f, &mut body).unwrap();
        out.start_file(f.name(), zip::write::SimpleFileOptions::default()).unwrap();
        std::io::Write::write_all(&mut out, body.replace(r#"PageNumber="2""#, r#"PageNumber="4294967295""#).as_bytes()).unwrap();
    }
    let hostile = pagenor_idml::import_idml(&out.finish().unwrap().into_inner()).unwrap();
    assert_eq!(hostile.assets.values().map(|a| a.page).collect::<Vec<_>>(), [u32::MAX - 1]);
    t.edit(|doc, _| {
        *doc = hostile;
        Ok(())
    })
    .unwrap();
    let r = t.execute("file.exportPdf", &json!({})).unwrap();
    assert!(r["warnings"].to_string().contains("has no page 4294967295"), "{r}");
}

/// An IDML placed PDF cropped to its media box, without `GraphicBounds`: the graphic takes the
/// media box (the page with its 9 pt bleed).
#[test]
fn idml_placed_pdf_cropped_to_its_media_box_gets_media_box_bounds() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("layout.documentSetup", &json!({"bleed": 9})).unwrap();
    let b64 = s.execute("file.exportPdf", &json!({"bleed": true})).unwrap()["base64"].as_str().unwrap().to_string();
    let mut t = Session::new();
    t.execute("file.new", &json!({})).unwrap();
    t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "pdfCrop": "trim", "width": 630, "x": 0, "y": 0})).unwrap();
    let bytes = pagenor_idml::export_idml(&t.doc().unwrap().doc);

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_slice())).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut edited = false;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut body).unwrap();
        if f.name().starts_with("Spreads/") {
            let text = String::from_utf8(body).unwrap().replace(r#"PDFCrop="CropTrim""#, r#"PDFCrop="CropMedia""#);
            let start = text.find("<GraphicBounds ").unwrap();
            let end = start + text[start..].find("/>").unwrap() + 2;
            body = format!("{}{}", &text[..start], &text[end..]).into_bytes();
            edited = true;
        }
        out.start_file(f.name(), zip::write::SimpleFileOptions::default()).unwrap();
        std::io::Write::write_all(&mut out, &body).unwrap();
    }
    assert!(edited);
    let back = crate::cmd::interchange::import(&out.finish().unwrap().into_inner(), None).unwrap();
    let g = back.spreads.iter().flat_map(|sp| sp.items.iter()).find_map(|i| i.graphic().cloned()).unwrap();
    let a = &back.assets[&g.asset];
    assert_eq!(a.pdf_crop, pagenor_doc::PdfCrop::Media);
    assert!((g.size.0 - 630.0).abs() < 1e-3 && (g.size.1 - 810.0).abs() < 1e-3, "{:?}", g.size);
    let b = a.shown_box().unwrap_or([0.0, 0.0, 1.0, 1.0]);
    assert!(b.iter().zip([0.0, 0.0, 1.0, 1.0]).all(|(x, y)| (x - y).abs() < 1e-6), "{b:?}");
}

/// A Bounding Box crop needs the page rendered to find its box. IDML import leaves that until
/// the graphic is drawn or exported (its `GraphicBounds` give its size), so opening a document
/// doesn't render every placed PDF; drawing and PDF export find the same box placing it did.
#[test]
fn idml_bounding_box_crop_is_measured_when_drawn_not_on_import() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("frame.create", &json!({"rect": [100, 200, 300, 260], "content": "unassigned"})).unwrap();
    s.execute("object.fill", &json!({"swatch": "[Black]"})).unwrap();
    let b64 = s.execute("file.exportPdf", &json!({})).unwrap()["base64"].as_str().unwrap().to_string();
    let mut t = Session::new();
    t.execute("file.new", &json!({})).unwrap();
    t.execute("file.place", &json!({"base64": b64, "name": "a.pdf", "pdfCrop": "contentVisible", "width": 200, "x": 20, "y": 30})).unwrap();
    let placed = t.doc().unwrap().doc.assets.values().next().unwrap().shown_box().expect("placing measures the box");
    assert!(placed[0] > 0.1 && placed[2] < 0.6, "the drawn rectangle, not the page: {placed:?}");

    let bytes = pagenor_idml::export_idml(&t.doc().unwrap().doc);
    let back = crate::cmd::interchange::import(&bytes, None).unwrap();
    let a = back.assets.values().next().unwrap();
    assert_eq!(a.pdf_crop, pagenor_doc::PdfCrop::ContentVisible);
    assert!(a.content_box_pending(), "import didn't render the page: {:?}", a.pdf_box);
    let close = |b: [f64; 4]| b.iter().zip(placed).all(|(x, y)| (x - y).abs() < 1e-9);
    let drawn = pagenor_render::shown_box(a).expect("drawing measures the box");
    assert!(close(drawn), "{drawn:?} vs {placed:?}");
    let boxed = crate::cmd::interchange::with_pdf_boxes(&back);
    let exported = boxed.assets.values().next().unwrap().shown_box().expect("PDF export gets the box");
    assert!(close(exported), "{exported:?} vs {placed:?}");
}

#[test]
fn indd_layer_stack_survives_native_import_and_save_without_covering_text() {
    use pagenor_indd::synthetic::object;
    let words = |values: &[u32]| values.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
    let name = |name: &str| {
        let mut b = vec![0; 3];
        b.extend_from_slice(&(0x4000 | name.len() as u16).to_le_bytes());
        b.extend_from_slice(name.as_bytes());
        b
    };
    let mut source = pagenor_indd::synthetic::sample();
    source.add(1, 0xE01, object(&[(0x501, words(&[1, 20])), (0x301, words(&[3, 10, 90, 91]))]));
    source.add(90, 0x302, object(&[(0x304, name("Title"))]));
    source.add(91, 0x302, object(&[(0x304, name("Page Number"))]));
    source.add(20, 0x501, object(&[(0x503, words(&[20, 0, 2, 21, 92]))]));
    source.add(21, 0x301, object(&[(0x303, words(&[20, 20, 2, 22, 30])), (0x302, [words(&[10]), vec![0, 0]].concat())]));
    source.add(92, 0x301, object(&[(0x303, words(&[20, 20, 1, 40])), (0x302, [words(&[90]), vec![0, 0]].concat())]));
    let mut session = Session::new();
    session.execute("file.openBytes", &json!({"base64": crate::cmd::base64_encode(&source.build()), "name": "layered.indd"})).unwrap();
    let doc = &session.doc().unwrap().doc;
    assert_eq!(doc.layers.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), ["Page Number", "Title", "Layer 1"]);
    let text = doc.spreads[0].items.iter().find(|i| matches!(i.content, pagenor_doc::Content::Text(_))).unwrap();
    let background = doc.spreads[0].items.iter().find(|i| !matches!(i.content, pagenor_doc::Content::Text(_))).unwrap();
    assert!(doc.layer_rank(text.layer) > doc.layer_rank(background.layer), "text must paint above background");
    let reopened = pagenor_format::load(&pagenor_format::save(doc).unwrap()).unwrap();
    assert_eq!(reopened.layers, doc.layers);
    assert_eq!(
        reopened.spreads[0].items.iter().map(|i| i.layer).collect::<Vec<_>>(),
        doc.spreads[0].items.iter().map(|i| i.layer).collect::<Vec<_>>()
    );
}
