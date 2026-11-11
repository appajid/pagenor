//! Friendly master-page dialogs; all changes go through engine commands.
use super::{Dialog, text_field};
use crate::DesignApp;
use serde_json::{Value, json};

pub(crate) fn open(app: &mut DesignApp, id: &str, p: &Value) -> Result<Value, String> {
    let st = app.session.active().ok_or("no document open")?;
    if id == "app.newMaster" {
        app.ui.dialog = Some(Dialog::new("masterNew", json!({"prefix":st.doc.next_parent_prefix(), "name":"Master", "applyAll":true})));
        return Ok(Value::Null);
    }
    let index = p.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
    let sp = st.doc.parents.get(index).ok_or("no such master")?;
    let parent = sp.parent.as_ref().ok_or("no such master")?;
    let mut fields = json!({"index":index,"parent":parent.prefix,"side":p.get("side").and_then(Value::as_str).unwrap_or("all"),"kind":p.get("kind").and_then(Value::as_str).unwrap_or("pageNumber"),"text":""});
    if id == "app.masterText" && fields["kind"] == "pageNumber" {
        return app.run("layout.parents.addText", fields);
    }
    let page = sp
        .pages
        .iter()
        .find(|pg| match fields["side"].as_str() {
            Some("left") => pg.side == pagenor_doc::PageSide::Left,
            Some("right") => pg.side == pagenor_doc::PageSide::Right,
            _ => true,
        })
        .ok_or("master has no page of this side")?;
    fields["top"] = json!(page.margins.top);
    fields["bottom"] = json!(page.margins.bottom);
    fields["inside"] = json!(page.margins.inside);
    fields["outside"] = json!(page.margins.outside);
    fields["columns"] = json!(page.columns.count);
    fields["gutter"] = json!(page.columns.gutter);
    fields["adjustLayout"] = json!(false);
    app.ui.dialog = Some(Dialog::new(if id == "app.masterSettings" { "masterSettings" } else { "masterText" }, fields));
    Ok(Value::Null)
}

pub(super) fn form(app: &DesignApp, ui: &mut egui::Ui, d: &mut Dialog) {
    if d.id == "masterNew" {
        ui.label("A facing-page master contains a left and a right page for book layouts.");
        egui::Grid::new("master_name").num_columns(2).show(ui, |ui| {
            ui.label("Prefix");
            text_field(ui, d, "prefix", 80.0);
            ui.end_row();
            ui.label("Name");
            text_field(ui, d, "name", 260.0);
            ui.end_row();
        });
        let mut all = d.b("applyAll");
        ui.checkbox(&mut all, "Apply to all pages and use for new pages");
        d.fields.insert("applyAll".into(), json!(all));
        if let Some(status) = d.fields.get("status").and_then(Value::as_str) {
            ui.label(status);
        }
        return;
    }
    ui.label(format!("{} master", d.s("parent")));
    let mut side = d.s("side");
    let old_side = side.clone();
    ui.horizontal(|ui| {
        ui.label("Apply to");
        egui::ComboBox::from_id_salt("master_side")
            .selected_text(match side.as_str() {
                "left" => "Left master",
                "right" => "Right master",
                "single" => "Single master",
                _ => "Both masters",
            })
            .show_ui(ui, |ui| {
                let facing =
                    app.session.active().and_then(|st| st.doc.parents.get(d.n("index").unwrap_or(0.0) as usize)).is_some_and(|sp| sp.pages.len() > 1);
                if facing {
                    ui.selectable_value(&mut side, "all".into(), "Both masters");
                    ui.selectable_value(&mut side, "left".into(), "Left master");
                    ui.selectable_value(&mut side, "right".into(), "Right master");
                } else {
                    ui.selectable_value(&mut side, "single".into(), "Single master");
                }
            });
    });
    d.fields.insert("side".into(), json!(side));
    if side != old_side
        && d.id == "masterSettings"
        && let Some(pg) = app.session.active().and_then(|st| st.doc.parents.get(d.n("index").unwrap_or(0.0) as usize)).and_then(|sp| {
            sp.pages.iter().find(|pg| match side.as_str() {
                "left" => pg.side == pagenor_doc::PageSide::Left,
                "right" => pg.side == pagenor_doc::PageSide::Right,
                _ => true,
            })
        })
    {
        for (k, v) in [
            ("top", pg.margins.top),
            ("bottom", pg.margins.bottom),
            ("inside", pg.margins.inside),
            ("outside", pg.margins.outside),
            ("gutter", pg.columns.gutter),
        ] {
            d.fields.insert(k.into(), json!(v));
        }
        d.fields.insert("columns".into(), json!(pg.columns.count));
    }
    if d.id == "masterText" {
        ui.label(if d.s("kind") == "header" { "Header text" } else { "Footer text" });
        text_field(ui, d, "text", 400.0);
        ui.label("The text repeats on every page using this master. Edit its frame on the master to change formatting.");
    } else {
        ui.label("Margins and column gutter are in points. Shared settings update every page using this master.");
        egui::Grid::new("master_settings").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            for (key, label) in [
                ("top", "Top margin"),
                ("bottom", "Bottom margin"),
                ("inside", "Inside / left margin"),
                ("outside", "Outside / right margin"),
                ("columns", "Columns"),
                ("gutter", "Column gutter"),
            ] {
                ui.label(label);
                text_field(ui, d, key, 100.0);
                ui.end_row();
            }
        });
        let mut adjust = d.b("adjustLayout");
        ui.checkbox(&mut adjust, "Adjust existing objects to the new margins");
        d.fields.insert("adjustLayout".into(), json!(adjust));
    }
    if let Some(status) = d.fields.get("status").and_then(Value::as_str) {
        ui.label(status);
    }
}

pub(super) fn confirm(app: &mut DesignApp, d: &Dialog) -> Result<Value, String> {
    let result = (|| {
        if d.id == "masterNew" {
            let r = app.run("layout.parents.new", json!({"prefix":d.s("prefix"),"name":d.s("name"),"applyAll":d.b("applyAll")}))?;
            let index = app
                .session
                .active()
                .and_then(|st| st.doc.parents.iter().position(|p| p.parent.as_ref().is_some_and(|i| i.prefix == d.s("prefix"))))
                .ok_or("new master missing")?;
            app.run("app.masterPages", json!({}))?;
            crate::canvas::go_to_master(app, index, None)?;
            return Ok(r);
        }
        let mut p = json!({"parent":d.s("parent"),"side":d.s("side")});
        if d.id == "masterText" {
            p["kind"] = json!(d.s("kind"));
            p["text"] = json!(d.s("text"));
            app.run("layout.parents.addText", p)
        } else {
            let number = |key: &str| d.n(key).ok_or_else(|| format!("Enter a number for {key}"));
            p["margins"] = json!({"top":number("top")?,"bottom":number("bottom")?,"inside":number("inside")?,"outside":number("outside")?});
            let columns = number("columns")?;
            if columns.fract() != 0.0 || !(1.0..=216.0).contains(&columns) {
                return Err("Columns must be a whole number from 1 to 216".into());
            }
            p["columns"] = json!(columns as u32);
            p["gutter"] = json!(number("gutter")?);
            p["adjustLayout"] = json!(d.b("adjustLayout"));
            app.run("layout.parents.settings", p)
        }
    })();
    if let Err(e) = &result {
        let mut d = d.clone();
        d.fields.insert("status".into(), json!(e));
        app.ui.dialog = Some(d);
    }
    result
}
