//! Small Welcome-screen history. Persist paths only, never document contents or session IDs.

use pagenor_engine::{DocState, Session};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

use crate::DesignApp;

const LIMIT: usize = 20;
const MAX_PATH_BYTES: usize = 8192;

#[derive(Clone, Debug)]
struct Entry {
    path: Option<String>,
    uid: Option<u64>,
}

/// Newest first. File history survives restarts; unsaved documents are session-local.
#[derive(Clone, Debug, Default)]
pub struct RecentDocuments {
    entries: Vec<Entry>,
}

#[derive(Serialize)]
pub(crate) struct Row {
    pub name: String,
    pub path: Option<String>,
    pub uid: Option<u64>,
}

impl Serialize for RecentDocuments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.entries.iter().filter_map(|e| e.path.as_ref()).collect::<Vec<_>>().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RecentDocuments {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let paths = Vec::<String>::deserialize(deserializer)?;
        let mut history = Self::default();
        for path in paths {
            if history.entries.len() == LIMIT {
                break;
            }
            if valid_path(&path) && !history.entries.iter().any(|e| e.path.as_deref().is_some_and(|p| same_path(p, &path))) {
                history.entries.push(Entry { path: Some(path), uid: None });
            }
        }
        Ok(history)
    }
}

fn valid_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= MAX_PATH_BYTES && !path.contains('\0')
}

fn same_path(a: &str, b: &str) -> bool {
    if cfg!(windows) { a.eq_ignore_ascii_case(b) } else { a == b }
}

/// Run only at command boundaries. Rendering the history does no filesystem work.
fn normalized_path(path: &str) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let p = std::path::Path::new(path);
        let absolute = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().map_or_else(|_| p.to_path_buf(), |dir| dir.join(p)) };
        let resolved = std::fs::canonicalize(&absolute).unwrap_or(absolute);
        // Preserve exact UTF-8 paths; never substitute an invalid filename.
        resolved.to_str().unwrap_or(path).to_owned()
    }
    #[cfg(target_arch = "wasm32")]
    path.to_owned()
}

impl RecentDocuments {
    pub(crate) fn remember(&mut self, doc: &DocState, source_path: Option<&str>) {
        #[cfg(not(target_arch = "wasm32"))]
        let path = doc
            .path
            .as_deref()
            .or(source_path)
            .or_else(|| self.entries.iter().find(|e| e.uid == Some(doc.uid)).and_then(|e| e.path.as_deref()))
            .filter(|p| valid_path(p))
            .map(normalized_path);
        // A browser download's filename isn't a reopenable local path.
        #[cfg(target_arch = "wasm32")]
        let path = {
            let _ = source_path;
            None
        };
        // Save As retains the previous file in history but associates only the new file with
        // the open tab. Saving an untitled document replaces its unsaved entry.
        self.entries.retain(|e| !(e.uid == Some(doc.uid) && e.path.is_none()));
        for e in &mut self.entries {
            if e.uid == Some(doc.uid) {
                e.uid = None;
            }
        }
        self.entries.retain(|e| !path.as_deref().is_some_and(|p| e.path.as_deref().is_some_and(|old| same_path(old, p))));
        self.entries.insert(0, Entry { path, uid: Some(doc.uid) });
        self.entries.truncate(LIMIT);
    }

    pub(crate) fn rows(&self, session: &Session) -> Vec<Row> {
        self.entries
            .iter()
            .filter_map(|e| {
                let doc = e.uid.and_then(|uid| session.documents().iter().find(|d| d.uid == uid));
                let name = match &e.path {
                    Some(path) => std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path).to_owned(),
                    None => doc?.doc.title.clone(),
                };
                Some(Row { name, path: e.path.clone(), uid: doc.map(|d| d.uid) })
            })
            .collect()
    }
}

impl DesignApp {
    /// Include documents supplied by the host (including restored recovery documents).
    pub fn sync_recent_documents(&mut self) {
        let docs = self.session.documents();
        self.ui.recent_documents.entries.retain(|e| e.path.is_some() || e.uid.is_some_and(|uid| docs.iter().any(|d| d.uid == uid)));
        for doc in docs {
            if !self.ui.recent_documents.entries.iter().any(|e| e.uid == Some(doc.uid)) {
                self.ui.recent_documents.remember(doc, None);
            }
        }
        self.refresh_recent_preview_files();
    }

    pub(crate) fn open_recent_document(&mut self, params: &Value) -> Result<Value, String> {
        let rows = self.ui.recent_documents.rows(&self.session);
        let uid = params["uid"].as_u64();
        let path = params["path"].as_str();
        let row = rows
            .into_iter()
            .find(|row| {
                uid.is_some_and(|uid| row.uid == Some(uid)) || path.is_some_and(|path| row.path.as_deref().is_some_and(|p| same_path(p, path)))
            })
            .ok_or_else(|| "This document is no longer in Recent Documents.".to_owned())?;
        let open = row.uid.and_then(|uid| self.session.documents().iter().position(|d| d.uid == uid)).or_else(|| {
            let path = row.path.as_deref()?;
            self.session.documents().iter().position(|d| d.path.as_deref().is_some_and(|p| same_path(&normalized_path(p), path)))
        });
        if let Some(index) = open {
            self.run("file.activate", json!({"index": index}))
        } else if let Some(path) = row.path {
            self.open_file("file.open", json!({"path": path}))
        } else {
            Err("This unsaved document has been closed.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Services, UiState, menus};

    fn app() -> DesignApp {
        DesignApp::new(Session::new(), Services::default())
    }

    fn recent(app: &mut DesignApp) -> Value {
        app.run("app.recentDocuments", json!({})).unwrap()
    }

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("pagenor-recent-{label}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_documents_are_newest_first_and_home_reactivates_without_duplicates() {
        let mut app = app();
        app.run("file.new", json!({"title": "First"})).unwrap();
        let first = app.session.doc().unwrap().uid;
        app.run("file.new", json!({"title": "Second"})).unwrap();
        assert_eq!(recent(&mut app)[0]["name"], "Second");
        app.run("app.home", json!({})).unwrap();
        assert!(app.ui.home);
        assert_eq!(app.session.documents().len(), 2);
        app.run("app.openRecentDocument", json!({"uid": first})).unwrap();
        assert!(!app.ui.home);
        assert_eq!(app.session.doc().unwrap().uid, first);
        assert_eq!(recent(&mut app)[0]["name"], "First");
        assert_eq!(app.session.documents().len(), 2);
        assert!(serde_json::to_value(&app.ui).unwrap()["recentDocuments"].as_array().unwrap().is_empty());
        app.run("file.close", json!({})).unwrap();
        assert!(app.run("app.openRecentDocument", json!({"uid": first})).is_err());
    }

    #[test]
    fn saved_paths_persist_reopen_and_save_as_keeps_both_files() {
        let dir = temp_dir("persist");
        let path = dir.join("हिन्दी document.pgnr");
        let other = dir.join("Second.pgnr");
        let mut app = app();
        app.run("file.new", json!({"title": "Book"})).unwrap();
        app.run("file.saveAs", json!({"path": path})).unwrap();
        assert_eq!(recent(&mut app).as_array().unwrap().len(), 1);
        app.run("file.saveAs", json!({"path": other})).unwrap();
        let rows = recent(&mut app);
        assert_eq!(rows.as_array().unwrap().len(), 2);
        assert_eq!(rows[0]["name"], "Second.pgnr");
        assert!(rows[1]["uid"].is_null(), "Save As doesn't associate the old file with the new tab");
        let prefs = serde_json::to_vec(&app.ui).unwrap();
        let mut restarted = self::app();
        restarted.ui = serde_json::from_slice(&prefs).unwrap();
        assert!(!restarted.ui.home);
        restarted.run("app.home", json!({})).unwrap();
        restarted.run("app.openRecentDocument", json!({"path": rows[1]["path"]})).unwrap();
        assert!(!restarted.ui.home);
        assert_eq!(recent(&mut restarted)[0]["name"], "हिन्दी document.pgnr");
        restarted.run("app.openRecentDocument", json!({"path": rows[1]["path"]})).unwrap();
        assert_eq!(restarted.session.documents().len(), 1, "reselecting activates its tab");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn imported_source_is_recent_and_reselecting_it_preserves_the_open_tab() {
        let dir = temp_dir("idml");
        let path = dir.join("Imported.idml");
        let mut app = app();
        app.run("file.new", json!({})).unwrap();
        app.run("file.exportIdml", json!({"path": path})).unwrap();
        app.run("file.close", json!({})).unwrap();
        app.run("file.open", json!({"path": path})).unwrap();
        assert!(app.session.doc().unwrap().path.is_none(), "an import must be saved as a Pagenor document");
        let row = recent(&mut app)[0].clone();
        app.run("app.home", json!({})).unwrap();
        app.run("app.openRecentDocument", row.clone()).unwrap();
        assert_eq!(app.session.documents().len(), 1);
        assert_eq!(recent(&mut app)[0]["path"], row["path"]);
        assert_eq!(recent(&mut app).as_array().unwrap().len(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failures_cancelled_pickers_and_missing_files_preserve_history_and_documents() {
        let mut app = app();
        app.run("file.new", json!({})).unwrap();
        let before = recent(&mut app);
        assert!(app.run("file.open", json!({"path": "/missing/pagenor-document.pgnr"})).is_err());
        assert!(app.run("file.saveAs", json!({"path": "/missing/pagenor-document.pgnr"})).is_err());
        app.run("app.openDialog", json!({})).unwrap();
        app.run("app.saveDialog", json!({})).unwrap();
        assert_eq!(recent(&mut app), before);
        app.ui.recent_documents = serde_json::from_value(json!(["/missing/pagenor-document.pgnr"])).unwrap();
        app.run("app.home", json!({})).unwrap();
        assert!(app.run("app.openRecentDocument", json!({"path": "/missing/pagenor-document.pgnr"})).is_err());
        assert!(app.ui.home);
        assert_eq!(app.session.documents().len(), 1);
        assert_eq!(app.ui.dialog.as_ref().unwrap().id, "alert");
    }

    #[test]
    fn history_is_bounded_deduplicated_and_old_preferences_still_load() {
        let history: RecentDocuments = serde_json::from_value(json!(["a.pgnr", "a.pgnr", "", "b.pgnr", "bad\0path"])).unwrap();
        assert_eq!(serde_json::to_value(&history).unwrap(), json!(["a.pgnr", "b.pgnr"]));
        let paths: Vec<_> = (0..100).map(|i| format!("{i}.pgnr")).collect();
        let history: RecentDocuments = serde_json::from_value(json!(paths)).unwrap();
        assert_eq!(history.entries.len(), LIMIT);
        let old: UiState = serde_json::from_value(json!({"language": "fr"})).unwrap();
        assert!(old.recent_documents.entries.is_empty());
        assert!(!old.home);
    }

    #[test]
    fn recovered_documents_are_included_and_file_menu_uses_the_same_history() {
        let mut session = Session::new();
        session.execute("file.new", &json!({"title": "Recovered"})).unwrap();
        let mut app = DesignApp::new(session, Services::default());
        app.run("app.home", json!({})).unwrap();
        let history = recent(&mut app);
        assert_eq!(history[0]["name"], "Recovered");
        let tree = menus::menu_tree_for(&app);
        let file = &tree.iter().find(|(name, _)| *name == "File").unwrap().1;
        let children = file
            .iter()
            .find_map(|item| match item {
                menus::Item::Sub(label, children) if label == "Open Recent" => Some(children),
                _ => None,
            })
            .unwrap();
        let menus::Item::Cmd { label, id, params, .. } = &children[0] else { panic!("recent command") };
        assert_eq!(label, "Recovered");
        app.run(id, params.clone()).unwrap();
        assert!(!app.ui.home);
        assert_eq!(app.session.documents().len(), 1);
    }
}
