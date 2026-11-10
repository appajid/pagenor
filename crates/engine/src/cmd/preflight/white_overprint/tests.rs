use std::sync::Arc;

use serde_json::json;

use crate::Session;

/// The `whiteOverprint` issues of the active document: (message, item).
fn white_overprints(s: &mut Session) -> Vec<(String, Option<u64>)> {
    white_overprints_on(s).into_iter().map(|(message, item, _)| (message, item)).collect()
}

/// The `whiteOverprint` issues of the active document: (message, item, page).
fn white_overprints_on(s: &mut Session) -> Vec<(String, Option<u64>, Option<u64>)> {
    let r = s.execute("preflight.run", &json!({})).unwrap();
    r["issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "whiteOverprint")
        .inspect(|i| assert_eq!(i["severity"], "warning"))
        .map(|i| (i["message"].as_str().unwrap().to_string(), i["item"].as_u64(), i["page"].as_u64()))
        .collect()
}

const PAPER_FILL: &str = "White or [Paper] fill set to overprint may disappear when overprint is honored";

/// A [Paper] fill set to overprint on `id`.
fn paper_overprint(s: &mut Session, id: u64) {
    s.execute("object.fill", &json!({"ids": [id], "swatch": "[Paper]"})).unwrap();
    s.execute("object.attributes", &json!({"ids": [id], "overprintFill": true})).unwrap();
}

fn rect(s: &mut Session, r: [f64; 4]) -> u64 {
    s.execute("frame.create", &json!({"rect": r})).unwrap()["id"].as_u64().unwrap()
}

/// A [Paper] fill set to overprint is gone on press: preflight said nothing about it.
#[test]
fn a_paper_fill_set_to_overprint_is_reported() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let id = rect(&mut s, [72.0, 72.0, 200.0, 200.0]);
    s.execute("object.fill", &json!({"ids": [id], "swatch": "[Paper]"})).unwrap();
    assert!(white_overprints(&mut s).is_empty(), "a white fill that knocks out prints");
    s.execute("object.attributes", &json!({"ids": [id], "overprintFill": true})).unwrap();
    assert_eq!(white_overprints(&mut s), [("White or [Paper] fill set to overprint may disappear when overprint is honored".to_string(), Some(id))]);
    // A coloured fill set to overprint is ordinary trapping.
    s.execute("object.fill", &json!({"ids": [id], "swatch": "[Black]"})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
    // Nonprinting: nothing of it prints anyway.
    s.execute("object.fill", &json!({"ids": [id], "swatch": "[Paper]"})).unwrap();
    s.execute("object.attributes", &json!({"ids": [id], "nonprinting": true})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
}

/// White is any paint that lays down no ink: C=0 M=0 Y=0 K=0, or a 0% tint.
#[test]
fn white_colours_and_strokes_are_reported() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    Arc::make_mut(&mut s.doc_mut().unwrap().doc).swatches.push(pagenor_color::swatch::Swatch::cmyk(0.0, 0.0, 0.0, 0.0));
    let a = rect(&mut s, [72.0, 72.0, 200.0, 200.0]);
    s.execute("object.fill", &json!({"ids": [a], "swatch": "[Black]", "tint": 0.0})).unwrap();
    s.execute("object.stroke", &json!({"ids": [a], "swatch": "C=0 M=0 Y=0 K=0", "weight": 4})).unwrap();
    s.execute("object.attributes", &json!({"ids": [a], "overprintFill": true, "overprintStroke": true})).unwrap();
    // A gap colour shows only between dashes: on a solid stroke it is not drawn.
    let b = rect(&mut s, [300.0, 72.0, 400.0, 200.0]);
    s.execute("object.stroke", &json!({"ids": [b], "swatch": "[Black]", "weight": 4, "gapSwatch": "[Paper]", "gapOverprint": true})).unwrap();
    assert_eq!(
        white_overprints(&mut s),
        [("White or [Paper] fill and stroke set to overprint may disappear when overprint is honored".to_string(), Some(a))]
    );
    s.execute("object.stroke", &json!({"ids": [b], "type": {"kind": "dotted"}})).unwrap();
    let found = white_overprints(&mut s);
    assert!(
        found.contains(&("White or [Paper] stroke gap set to overprint may disappear when overprint is honored".to_string(), Some(b))),
        "{found:?}"
    );
}

/// A spot ink prints on its own plate whatever colour stands in for it: a white ink or a varnish
/// set to overprint, as they usually are, is not white. A 0% tint of one lays down no ink.
#[test]
fn a_spot_ink_with_a_white_alternate_prints() {
    use pagenor_color::{Color, ColorType, Swatch, SwatchValue};
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let swatches = &mut Arc::make_mut(&mut s.doc_mut().unwrap().doc).swatches;
    for (name, color) in [("Opaque White", Color::cmyk(0.0, 0.0, 0.0, 0.0)), ("Varnish", Color::rgb(1.0, 1.0, 1.0))] {
        swatches.push(Swatch {
            name: name.into(),
            value: SwatchValue::Color { color, color_type: ColorType::Spot },
            locked: false,
            named: true,
            hidden: false,
        });
    }
    swatches.push(Swatch {
        name: "Varnish 50%".into(),
        value: SwatchValue::Tint { base: "Varnish".into(), tint: 0.5 },
        locked: false,
        named: true,
        hidden: false,
    });
    let a = rect(&mut s, [72.0, 72.0, 200.0, 200.0]);
    s.execute("object.fill", &json!({"ids": [a], "swatch": "Opaque White"})).unwrap();
    s.execute("object.stroke", &json!({"ids": [a], "swatch": "Varnish 50%", "weight": 4})).unwrap();
    s.execute("object.attributes", &json!({"ids": [a], "overprintFill": true, "overprintStroke": true})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
    s.execute("object.fill", &json!({"ids": [a], "swatch": "Opaque White", "tint": 0.0})).unwrap();
    assert_eq!(white_overprints(&mut s), [("White or [Paper] fill set to overprint may disappear when overprint is honored".to_string(), Some(a))]);
}

/// The issue's steps: text in [Paper], and Overprint Fill applied to its frame. Text has no
/// overprint of its own, and the frame's Overprint Fill is for the frame's fill, which is [None]:
/// the text knocks out and prints, so there is nothing to report. With a [Paper] fill on the
/// frame, the frame is what disappears.
#[test]
fn paper_text_in_a_frame_set_to_overprint() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 300, 200], "content": "text", "text": "White"})).unwrap();
    let id = r["id"].as_u64().unwrap();
    s.execute("text.select", &json!({"story": r["story"], "anchor": 0, "focus": 5})).unwrap();
    s.execute("type.char", &json!({"attrs": {"fill": "[Paper]"}})).unwrap();
    s.execute("object.attributes", &json!({"ids": [id], "overprintFill": true})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
    s.execute("object.fill", &json!({"ids": [id], "swatch": "[Paper]"})).unwrap();
    assert_eq!(white_overprints(&mut s), [("White or [Paper] fill set to overprint may disappear when overprint is honored".to_string(), Some(id))]);
}

/// A parent page's item prints on every page that shows it: it was never looked at (noted in
/// review). It is reported once, on the first page that shows it, and not when no page does.
#[test]
fn a_parent_page_item_is_reported_on_the_pages_that_show_it() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"pages": 2, "facingPages": false})).unwrap();
    let parent = json!({"kind": "parent", "index": 0});
    let id = s.execute("frame.create", &json!({"spread": parent, "rect": [72, 72, 200, 200]})).unwrap()["id"].as_u64().unwrap();
    paper_overprint(&mut s, id);
    assert_eq!(white_overprints_on(&mut s), [(PAPER_FILL.to_string(), Some(id), Some(0))]);
    for page in &mut Arc::make_mut(&mut s.doc_mut().unwrap().doc).spreads {
        Arc::make_mut(page).pages[0].show_parent_items = false;
    }
    assert!(white_overprints(&mut s).is_empty(), "no page shows the parent's items");
}

/// An anchored object prints in the frame that places it: it was never looked at (noted in
/// review).
#[test]
fn an_anchored_object_is_reported() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 400], "content": "text", "text": "Before after."})).unwrap();
    let id = rect(&mut s, [450.0, 450.0, 474.0, 490.0]);
    paper_overprint(&mut s, id);
    let d = s.doc().unwrap().doc.clone();
    let sid = d.item(pagenor_doc::ItemId(r["id"].as_u64().unwrap())).unwrap().text_frame().unwrap().story;
    s.execute("anchored.insert", &json!({"ids": [id], "story": sid.0, "pos": 7})).unwrap();
    assert_eq!(white_overprints_on(&mut s), [(PAPER_FILL.to_string(), Some(id), Some(0))]);
    s.execute("object.attributes", &json!({"ids": [r["id"]], "nonprinting": true})).unwrap();
    assert!(white_overprints(&mut s).is_empty(), "its frame doesn't print");
}

/// A multi-state object prints its active state only, but a frame made from one (Flex Layout's
/// convert keeps the states) prints every object in it, as PDF export draws a frame's pasted-in
/// objects: the inactive state's white overprint was left out (noted in review).
#[test]
fn a_frame_with_states_is_looked_at_as_pdf_export_draws_it() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let a = rect(&mut s, [72.0, 72.0, 200.0, 200.0]);
    paper_overprint(&mut s, a);
    let b = rect(&mut s, [300.0, 72.0, 400.0, 200.0]);
    let ms = s.execute("states.create", &json!({"ids": [a, b]})).unwrap()["id"].as_u64().unwrap();
    let at = s.doc().unwrap().doc.item(pagenor_doc::ItemId(ms)).unwrap().children().iter().position(|c| c.id.0 == a).unwrap();
    s.execute("states.show", &json!({"id": ms, "index": 1 - at})).unwrap();
    assert!(white_overprints(&mut s).is_empty(), "an inactive state doesn't print");
    // Synthetic imported frame containing state-tagged pasted objects; no Flex UI is required.
    s.edit(|d, _| {
        let it = d.item_mut(pagenor_doc::ItemId(ms)).unwrap();
        let bounds = it.inner_bounds();
        it.shape = pagenor_doc::Shape::Rectangle;
        it.path = pagenor_geom::shapes::rectangle(bounds);
        Ok(())
    })
    .unwrap();
    assert!(!s.doc().unwrap().doc.item(pagenor_doc::ItemId(ms)).unwrap().states.is_empty(), "the frame keeps its states");
    assert_eq!(white_overprints(&mut s), [(PAPER_FILL.to_string(), Some(a))]);
}
#[test]
fn hidden_nonprinting_layers_and_groups_are_excluded() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let layer = s.execute("layer.new", &json!({"name":"Notes"})).unwrap()["id"].as_u64().unwrap();
    s.execute("layer.activate", &json!({"id":layer})).unwrap();
    let a = rect(&mut s, [72.0, 72.0, 200.0, 200.0]);
    paper_overprint(&mut s, a);
    assert_eq!(white_overprints(&mut s).len(), 1);
    for key in ["visible", "printable"] {
        s.execute("layer.set", &json!({"id":layer,key:false})).unwrap();
        assert!(white_overprints(&mut s).is_empty());
        s.execute("layer.set", &json!({"id":layer,key:true})).unwrap();
    }
    let b = rect(&mut s, [250.0, 72.0, 400.0, 200.0]);
    let g = s.execute("object.group", &json!({"ids":[a,b]})).unwrap()["id"].as_u64().unwrap();
    s.execute("object.attributes", &json!({"ids":[g],"nonprinting":true})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
    s.execute("object.attributes", &json!({"ids":[g],"nonprinting":false})).unwrap();
    s.execute("object.hide", &json!({"ids":[g]})).unwrap();
    assert!(white_overprints(&mut s).is_empty());
}
