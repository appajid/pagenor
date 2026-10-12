//! Master page settings and repeating header/footer/page-number frames.
use super::{CommandSpec, bad, cmd, has_doc, ok, str_param};
use crate::{Result, Session};
use pagenor_doc::{Align, Document, Margins, PageSide, ParaFormat, SpreadId, SpreadRef};
use pagenor_geom::Rect;
use serde_json::{Value, json};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "layout.parents.settings",
            "Master Page Settings…",
            [],
            None,
            "{parent: prefix, side?: all|left|right|single, margins?: number|{top,bottom,inside,outside}, columns?, gutter?, adjustLayout?: false}",
            has_doc,
            settings
        ),
        cmd!(
            "layout.parents.addText",
            "Add Master Header / Footer…",
            [],
            None,
            "{parent: prefix, side?: all|left|right|single, kind: header|footer|pageNumber, text?} → {items: [id]}",
            has_doc,
            add_text
        ),
    ]
}

pub(super) fn parent_id(d: &Document, value: Option<&Value>, allow_none: bool) -> Result<Option<SpreadId>> {
    if allow_none && value.is_some_and(Value::is_null) {
        return Ok(None);
    }
    let prefix = value.and_then(Value::as_str).ok_or_else(|| bad("master", "specify a master prefix"))?;
    d.parents
        .iter()
        .find(|p| p.parent.as_ref().is_some_and(|i| i.prefix == prefix || i.label() == prefix))
        .map(|p| Some(p.id))
        .ok_or_else(|| bad("master", "no such master"))
}

pub(super) fn document_pages(d: &Document, value: Option<&Value>) -> Result<Vec<usize>> {
    let Some(value) = value else { return Ok((0..d.page_count()).collect()) };
    let array = value.as_array().ok_or_else(|| bad("master", "pages must be an array of page indices"))?;
    let mut pages = std::collections::BTreeSet::new();
    for v in array {
        let page =
            v.as_u64().and_then(|n| usize::try_from(n).ok()).filter(|n| *n < d.page_count()).ok_or_else(|| bad("master", "no such document page"))?;
        pages.insert(page);
    }
    Ok(pages.into_iter().collect())
}

fn side(p: &Value) -> Result<Option<PageSide>> {
    match str_param(p, "side").unwrap_or("all") {
        "all" => Ok(None),
        "left" => Ok(Some(PageSide::Left)),
        "right" => Ok(Some(PageSide::Right)),
        "single" => Ok(Some(PageSide::Single)),
        _ => Err(bad("master", "side must be all, left, right or single")),
    }
}

pub(super) fn settings(s: &mut Session, p: &Value) -> Result<Value> {
    let d = &s.doc()?.doc;
    let id = parent_id(d, p.get("parent"), false)?.ok_or_else(|| bad("master", "missing master"))?;
    let index = d.parent_index(id).ok_or_else(|| bad("master", "missing master"))?;
    let side = side(p)?;
    if !d.parents[index].pages.iter().any(|page| side.is_none_or(|s| s == page.side)) {
        return Err(bad("master", "master has no page of this side"));
    }
    let margins = p
        .get("margins")
        .map(|v| {
            let m = if let Some(n) = v.as_f64() {
                Margins::uniform(n)
            } else {
                serde_json::from_value(v.clone()).map_err(|_| bad("master", "invalid margins"))?
            };
            if [m.top, m.bottom, m.inside, m.outside].iter().any(|v| !v.is_finite() || !(0.0..=14400.0).contains(v)) {
                return Err(bad("master", "margins must be between 0 and 14400 points"));
            }
            Ok(m)
        })
        .transpose()?;
    let columns = p
        .get("columns")
        .map(|v| v.as_u64().filter(|n| (1..=216).contains(n)).map(|n| n as u32).ok_or_else(|| bad("master", "columns must be between 1 and 216")))
        .transpose()?;
    let gutter = p
        .get("gutter")
        .map(|v| {
            v.as_f64()
                .filter(|n| n.is_finite() && (0.0..=14400.0).contains(n))
                .ok_or_else(|| bad("master", "gutter must be between 0 and 14400 points"))
        })
        .transpose()?;
    s.edit(|d, _| {
        let before = p.get("adjustLayout").and_then(Value::as_bool).unwrap_or(false).then(|| super::layout::margin_boxes(d));
        let sp = d.spread_mut(SpreadRef::Parent(index)).ok_or_else(|| bad("master", "missing master"))?;
        for page in &mut sp.pages {
            if side.is_some_and(|s| s != page.side) {
                continue;
            }
            if let Some(m) = margins {
                page.margins = m;
            }
            if let Some(n) = columns {
                page.columns.count = n;
                page.columns.positions = None;
            }
            if let Some(g) = gutter {
                page.columns.gutter = g;
            }
        }
        d.sync_parent_settings();
        if let Some(before) = before {
            super::layout::adjust_layout(d, &before);
        }
        ok()
    })
}

fn add_text(s: &mut Session, p: &Value) -> Result<Value> {
    let d = &s.doc()?.doc;
    let id = parent_id(d, p.get("parent"), false)?.ok_or_else(|| bad("master", "missing master"))?;
    let index = d.parent_index(id).ok_or_else(|| bad("master", "missing master"))?;
    let side = side(p)?;
    let kind = str_param(p, "kind").unwrap_or("pageNumber");
    if !matches!(kind, "header" | "footer" | "pageNumber") {
        return Err(bad("master", "kind must be header, footer or pageNumber"));
    }
    let text = if kind == "pageNumber" {
        "\u{e000}"
    } else {
        str_param(p, "text").filter(|t| !t.trim().is_empty()).ok_or_else(|| bad("master", "enter header or footer text"))?
    }
    .to_string();
    let pages: Vec<_> = d.parents[index].pages.iter().filter(|pg| side.is_none_or(|s| s == pg.side)).cloned().collect();
    if pages.is_empty() {
        return Err(bad("master", "master has no page of this side"));
    }
    if pages.iter().any(|pg| if kind == "header" { pg.margins.top < 18.0 } else { pg.margins.bottom < 18.0 }) {
        return Err(bad("master", "a header or footer needs at least 18 points of margin"));
    }
    let layer = s.doc()?.active_layer;
    s.edit(|d, _| {
        let mut items = Vec::new();
        for page in pages {
            let margin = page.margin_rect();
            if margin.width() <= 0.0 {
                return Err(bad("master", "margins leave no room for text"));
            }
            let y = if kind == "header" { (page.margins.top - 18.0) / 2.0 } else { page.height - page.margins.bottom / 2.0 - 9.0 };
            let rect = Rect::new(margin.x0, y, margin.x1, y + 18.0);
            let mut format = ParaFormat::default();
            format.para.align =
                Some(if kind == "pageNumber" { if page.side == PageSide::Left { Align::Left } else { Align::Right } } else { Align::Center });
            let (item, _) = d.add_text_frame(SpreadRef::Parent(index), rect, layer, &text, format)?;
            items.push(item.0);
        }
        Ok(json!({"items": items}))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pagenor_doc::{ItemId, Orientation};

    fn book() -> Session {
        let mut s = Session::new();
        s.execute("file.new", &json!({"pages":6,"facingPages":true})).unwrap();
        s
    }

    #[test]
    fn master_settings_follow_sides_new_pages_and_undo() {
        let mut s = book();
        s.execute("layout.parents.settings", &json!({"parent":"A","side":"left","margins":54,"columns":2,"gutter":10})).unwrap();
        s.execute("layout.parents.settings", &json!({"parent":"A","side":"right","margins":42,"columns":3,"gutter":8})).unwrap();
        s.execute("layout.pages.insert", &json!({"count":2,"after":0})).unwrap();
        for pg in s.doc().unwrap().doc.spreads.iter().flat_map(|sp| &sp.pages) {
            assert!(pg.inherit_parent_settings);
            assert_eq!((pg.margins.top, pg.columns.count), if pg.side == PageSide::Left { (54.0, 2) } else { (42.0, 3) });
        }
        s.execute("layout.documentSetup", &json!({"binding":"rightToLeft"})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page(0).unwrap().margins.top, 54.0);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page(0).unwrap().margins.top, 42.0);
        // Explicit page settings are kept until its master is reapplied.
        s.execute("layout.marginsAndColumns", &json!({"pages":[0],"margins":27})).unwrap();
        s.execute("layout.parents.settings", &json!({"parent":"A","side":"right","margins":48})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page(0).unwrap().margins.top, 27.0);
        s.execute("layout.pages.applyParent", &json!({"parent":"A","pages":[0]})).unwrap();
        assert_eq!(s.doc().unwrap().doc.page(0).unwrap().margins.top, 48.0);
    }

    #[test]
    fn repeating_headers_footers_and_numbers_survive_native_round_trip() {
        let mut s = book();
        for (side, text) in [("left", "Left chapter"), ("right", "Right chapter")] {
            s.execute("layout.parents.addText", &json!({"parent":"A","side":side,"kind":"header","text":text})).unwrap();
        }
        s.execute("layout.parents.addText", &json!({"parent":"A","kind":"footer","text":"Pagenor Publishing"})).unwrap();
        let numbers = s.execute("layout.parents.addText", &json!({"parent":"A","kind":"pageNumber"})).unwrap();
        s.execute("guide.add", &json!({"spread":{"kind":"parent","index":0},"page":1,"orientation":"vertical","position":712})).unwrap();
        let d = s.doc().unwrap().doc.clone();
        let native = pagenor_format::load(&pagenor_format::save(&d).unwrap()).unwrap();
        let idml = pagenor_idml::import_idml(&pagenor_idml::export_idml(&d)).unwrap();
        for d in [&*d, &native, &idml] {
            d.check().unwrap();
            assert_eq!(d.parents[0].items.len(), 6);
            assert!(d.spreads.iter().all(|sp| sp.items.is_empty()), "shared items are not copied to pages");
            assert!(d.page(0).unwrap().inherit_parent_settings);
            for abs in 0..6 {
                let mut test = Session::new();
                test.add_document(crate::DocState::new(d.clone(), None));
                let applied = test.execute("layout.overrideParentItems", &json!({"page":abs})).unwrap();
                assert_eq!(applied["ids"].as_array().unwrap().len(), 3);
                let texts: Vec<_> = applied["ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|id| {
                        let it = test.doc().unwrap().doc.item(ItemId(id.as_u64()?))?;
                        Some(test.doc().unwrap().doc.story(it.text_frame()?.story)?.text.clone())
                    })
                    .collect();
                assert!(texts.iter().any(|t| t == if d.page(abs).unwrap().side == PageSide::Left { "Left chapter" } else { "Right chapter" }));
            }
        }
        // The same marker story resolves to the actual document number per page.
        for abs in 0..6 {
            let (_, pi) = native.parent_page_for(abs).unwrap();
            let item = native.item(ItemId(numbers["items"][pi].as_u64().unwrap())).unwrap();
            let composed = s.cache.get(&native, item.text_frame().unwrap().story, Some(abs));
            assert!(!composed.is_overset());
            let text: String =
                composed.frames.iter().flat_map(|f| &f.lines).flat_map(|l| &l.glyphs).filter_map(|g| g.generated_text.as_deref()).collect();
            assert_eq!(text, native.page_name(abs));
        }
        assert_eq!(
            native
                .inherited_guides(SpreadRef::Doc(0), 0)
                .iter()
                .filter(|g| g.orientation == Orientation::Vertical)
                .map(|g| g.position)
                .collect::<Vec<_>>(),
            vec![100.0]
        );
    }

    #[test]
    fn choosing_a_default_master_persists_for_document_growth() {
        let mut s = book();
        s.execute("layout.parents.new", &json!({"prefix":"B","name":"Book","applyAll":true})).unwrap();
        s.execute("layout.parents.settings", &json!({"parent":"B","margins":60})).unwrap();
        let d = pagenor_format::load(&pagenor_format::save(&s.doc().unwrap().doc).unwrap()).unwrap();
        let id = d.parents[1].id;
        let mut s = Session::new();
        s.add_document(crate::DocState::new(d, None));
        s.execute("layout.pages.insert", &json!({"count":1})).unwrap();
        s.execute("layout.documentSetup", &json!({"pages":9})).unwrap();
        assert_eq!(s.doc().unwrap().doc.default_parent(), Some(id));
        for pg in s.doc().unwrap().doc.spreads.iter().flat_map(|sp| &sp.pages) {
            assert_eq!(pg.parent, Some(id));
            assert_eq!(pg.margins.top, 60.0);
        }
    }

    #[test]
    fn inherited_guides_follow_side_translation_and_stop_at_cycles() {
        let mut s = book();
        s.execute("layout.parents.new", &json!({"prefix":"B","name":"Base"})).unwrap();
        for (parent, page, orientation, position, spread) in
            [(0, 0, "horizontal", 100, true), (0, 1, "vertical", 712, false), (1, 1, "horizontal", 150, false)]
        {
            s.execute(
                "guide.add",
                &json!({"spread":{"kind":"parent","index":parent},"page":page,"orientation":orientation,"position":position,"spreadGuide":spread}),
            )
            .unwrap();
        }
        s.edit(|d, _| {
            let (a, b) = (d.parents[0].id, d.parents[1].id);
            d.spread_mut(SpreadRef::Parent(0)).unwrap().parent.as_mut().unwrap().based_on = Some(b);
            d.spread_mut(SpreadRef::Parent(1)).unwrap().parent.as_mut().unwrap().based_on = Some(a);
            Ok(Value::Null)
        })
        .unwrap();
        let d = &s.doc().unwrap().doc;
        let right = d.inherited_guides(SpreadRef::Doc(0), 0);
        assert_eq!(right.len(), 3);
        assert!(right.iter().any(|g| g.orientation == Orientation::Vertical && g.position == 100.0));
        assert!(right.iter().all(|g| g.locked && !g.spread));
        let left = d.inherited_guides(SpreadRef::Doc(1), 0);
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].position, 100.0);
        assert!(d.spreads.iter().flat_map(|sp| &sp.pages).all(|p| p.guides.is_empty()));
    }

    #[test]
    fn invalid_master_parameters_do_not_modify_document() {
        let mut s = book();
        let before = s.doc().unwrap().doc.clone();
        for (cmd, p) in [
            ("layout.parents.new", json!({"prefix":"A","name":"Duplicate"})),
            ("layout.parents.settings", json!({"parent":"missing","margins":50})),
            ("layout.parents.settings", json!({"parent":"A","margins":-1})),
            ("layout.parents.settings", json!({"parent":"A","columns":0})),
            ("layout.parents.settings", json!({"parent":"A","side":"back"})),
            ("layout.pages.applyParent", json!({"parent":"missing"})),
            ("layout.pages.applyParent", json!({"parent":"A","pages":[0,9999]})),
            ("layout.pages.applyParent", json!({"parent":"A","pages":[0],"setDefault":true})),
        ] {
            assert!(s.execute(cmd, &p).is_err(), "{cmd} {p}");
            assert_eq!(*s.doc().unwrap().doc, *before);
        }
    }
}
