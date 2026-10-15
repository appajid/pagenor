//! Window › Utilities › Scripts: run a command script (see [`crate::script`]) as one undo step.

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, cmd, str_param};
use crate::Result;

fn run(s: &mut crate::Session, p: &Value) -> Result<Value> {
    let text = str_param(p, "text").ok_or_else(|| bad("script.run", "`text` required"))?;
    let steps = crate::script::parse(text).map_err(|e| bad("script.run", e))?;
    let docs = s.docs.clone();
    let active = s.active;
    let journal = s.journal.len();
    let typing = s.typing_format.clone();
    let requests = s.ui_requests.clone();
    let loaded = s.loaded;
    // Steps own their intermediate undo entries; successful scripts collapse to one step.
    s.in_undo_step = false;
    let uid = s.active().map(|d| d.uid);
    let before = s.active().map_or(0, |d| d.history.undo.len());
    let report = s.run_script(&steps);
    if report.failed.is_some() {
        s.docs = docs;
        s.active = active;
        s.journal.truncate(journal);
        s.typing_format = typing;
        s.ui_requests = requests;
        s.loaded = loaded;
    } else if let Some(st) = s.active_mut()
        && Some(st.uid) == uid
        && st.history.undo.len() > before
    {
        st.history.undo.truncate(before);
    }
    let out = json!({"results": report.results, "steps": steps.len()});
    match report.failed {
        Some((i, c, e)) => Err(bad("script.run", format!("step {i} ({c}): {e}"))),
        None => Ok(out),
    }
}

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "script.run",
        "Run Script",
        [],
        None,
        "{text} — command script lines (`command.id {json}`, JSON lines or a JSON array; `$N.path` refers to earlier results) → {results, steps}",
        always,
        run
    )]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Session;

    #[test]
    fn scripts_run_as_one_undo_step() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let text = "frame.create {\"rect\": [72, 72, 200, 200]}\nobject.fill {\"swatch\": \"[Black]\", \"ids\": [\"$0.id\"]}\nframe.create {\"rect\": [300, 72, 400, 200]}";
        let r = s.execute("script.run", &json!({"text": text})).unwrap();
        assert_eq!(r["steps"], 3);
        let count = |s: &Session| s.doc().unwrap().doc.spreads[0].items.len();
        assert_eq!(count(&s), 2);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(count(&s), 0, "one undo removes the whole script's work");
        assert!(s.execute("script.run", &json!({"text": "nope.command"})).is_err());
    }

    /// A script that fails after a successful step leaves the document and its history exactly as
    /// they were: the half-run prefix is not stranded without an undo entry (#709).
    #[test]
    fn a_failed_script_rolls_the_successful_prefix_back() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let count = |s: &Session| s.doc().unwrap().doc.spreads[0].items.len();
        let err = s.execute("script.run", &json!({"text": "frame.create {\"rect\": [72, 72, 200, 200]}\nnope.command"})).unwrap_err();
        assert!(err.to_string().contains("step 1") && err.to_string().contains("nope.command"), "{err}");
        assert_eq!(count(&s), 0, "the successful prefix is rolled back");
        assert!(s.execute("edit.undo", &json!({})).is_err(), "nothing was left to undo");
        // The session keeps working and its undo history is as it was.
        s.execute("frame.create", &json!({"rect": [72, 72, 200, 200]})).unwrap();
        assert_eq!(count(&s), 1);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(count(&s), 0);
    }
    #[test]
    fn failed_script_restores_document_set_active_document_and_redo() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"title":"Original"})).unwrap();
        s.execute("frame.create", &json!({"rect":[0,0,100,100]})).unwrap();
        let redo_doc = s.doc().unwrap().doc.clone();
        s.execute("edit.undo", &json!({})).unwrap();
        let doc = s.doc().unwrap().doc.clone();
        let journal = s.journal.clone();
        let script = "file.new {\"title\":\"Temporary\"}\nframe.create {\"rect\":[0,0,100,100]}\nfile.activate {\"index\":0}\nframe.create {\"rect\":[0,0,100,100]}\nnope.command";
        let error = s.execute("script.run", &json!({"text":script})).unwrap_err();
        assert!(error.to_string().contains("step 4") && error.to_string().contains("nope.command"), "{error}");
        assert_eq!(s.documents().len(), 1);
        assert_eq!(s.active_index(), Some(0));
        assert!(std::sync::Arc::ptr_eq(&s.doc().unwrap().doc, &doc));
        assert_eq!(s.journal, journal);
        assert_eq!(s.doc().unwrap().history.redo.len(), 1);
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc, redo_doc);
    }
}
