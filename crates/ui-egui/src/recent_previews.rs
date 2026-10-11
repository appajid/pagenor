//! First-page Welcome previews. One native worker, bounded images, no file reads in painting.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use egui::TextureHandle;
use serde::{Deserialize, Deserializer, Serialize};

use crate::DesignApp;
use crate::recent_documents::Row;

const SIDE: u32 = 160;
const LIMIT: usize = 20;
const MAX_PNG: usize = (SIDE * SIDE * 4) as usize + 16_384;
#[cfg(not(target_arch = "wasm32"))]
const MAX_FILE: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Stamp {
    bytes: u64,
    modified: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedPreview {
    png: String,
    stamp: Option<Stamp>,
}

/// Preferences contain small PNGs only, capped at the size of the recent-file history.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct SavedThumbnails(BTreeMap<String, SavedPreview>);

impl<'de> Deserialize<'de> for SavedThumbnails {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let entries = BTreeMap::<String, SavedPreview>::deserialize(d)?;
        Ok(Self(entries.into_iter().filter(|(path, p)| path.len() <= 8192 && decoded_preview(&p.png).is_some()).take(LIMIT).collect()))
    }
}

fn decoded_preview(encoded: &str) -> Option<Vec<u8>> {
    if encoded.len() > MAX_PNG * 4 / 3 + 4 {
        return None;
    }
    let bytes = pagenor_engine::cmd::base64_decode(encoded);
    if bytes.len() > MAX_PNG || !bytes.starts_with(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR") {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    (width > 0 && height > 0 && width <= SIDE && height <= SIDE).then_some(bytes)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    File(String),
    Open(u64),
}

fn key(row: &Row) -> Option<Key> {
    row.path.as_ref().map(|p| Key::File(p.clone())).or_else(|| row.uid.map(Key::Open))
}

type Version = Option<(u64, u64)>;

struct Cached {
    version: Version,
    attempted: bool,
    texture: Option<TextureHandle>,
}

struct Snapshot {
    doc: Arc<pagenor_doc::Document>,
    cache: Arc<pagenor_compose::Cache>,
    _fonts: Option<Arc<pagenor_engine::FontScope>>,
}

struct Job {
    key: Key,
    version: Version,
    snapshot: Option<Snapshot>,
    #[cfg(not(target_arch = "wasm32"))]
    cached: Option<SavedPreview>,
    persist: bool,
}

struct Done {
    key: Key,
    version: Version,
    preview: Option<SavedPreview>,
    persist: bool,
}

#[derive(Default)]
pub(crate) struct Previews {
    entries: HashMap<Key, Cached>,
    #[cfg(not(target_arch = "wasm32"))]
    worker: Option<Worker>,
    #[cfg(not(target_arch = "wasm32"))]
    started: bool,
    busy: bool,
}

#[cfg(not(target_arch = "wasm32"))]
struct Worker {
    tx: std::sync::mpsc::SyncSender<Job>,
    rx: std::sync::mpsc::Receiver<Done>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Worker {
    fn spawn(ctx: egui::Context) -> Option<Self> {
        let (tx, jobs) = std::sync::mpsc::sync_channel::<Job>(1);
        let (done_tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("pagenor-recent-preview".into())
            .spawn(move || {
                while let Ok(job) = jobs.recv() {
                    let preview = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| preview(&job))).ok().flatten();
                    if done_tx.send(Done { key: job.key, version: job.version, preview, persist: job.persist }).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .ok()?;
        Some(Self { tx, rx })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn stamp(path: &str) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let modified = u64::try_from(meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos()).ok()?;
    Some(Stamp { bytes: meta.len(), modified })
}

fn render(snapshot: &Snapshot) -> Option<String> {
    let page = snapshot.doc.page(0)?;
    let side = page.width.max(page.height);
    if !side.is_finite() || side <= 0.0 {
        return None;
    }
    let mut renderer = pagenor_render::Renderer::new();
    renderer.threads = 0;
    let image = renderer.render_page(
        &snapshot.doc,
        &snapshot.cache,
        0,
        SIDE as f64 / side,
        false,
        &pagenor_render::RenderOptions { printing_only: true, ..Default::default() },
    )?;
    let png = pagenor_engine::cmd::base64_encode(&image.to_png());
    decoded_preview(&png)?;
    Some(png)
}

fn preview(job: &Job) -> Option<SavedPreview> {
    #[cfg(not(target_arch = "wasm32"))]
    let current = match &job.key {
        Key::File(path) => stamp(path),
        Key::Open(_) => None,
    };
    #[cfg(target_arch = "wasm32")]
    let current = None;
    if let Some(snapshot) = &job.snapshot {
        return Some(SavedPreview { png: render(snapshot)?, stamp: current });
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Key::File(path) = &job.key {
        let Some(current) = current else { return job.cached.clone() };
        if let Some(cached) = &job.cached
            && cached.stamp.as_ref() == Some(&current)
        {
            return Some(cached.clone());
        }
        if current.bytes > MAX_FILE {
            return None;
        }
        // Use the existing importer/font scopes. This private session never opens tabs in the UI.
        let mut session = pagenor_engine::Session::new();
        session.execute("file.open", &serde_json::json!({"path": path, "maxBytes": MAX_FILE})).ok()?;
        let doc = session.active()?;
        let snapshot = Snapshot { doc: doc.doc.clone(), cache: session.cache.clone(), _fonts: doc.fonts.clone() };
        let png = render(&snapshot)?;
        if stamp(path).as_ref() != Some(&current) {
            return None;
        }
        return Some(SavedPreview { png, stamp: Some(current) });
    }
    None
}

impl Previews {
    fn receive(&mut self, done: Done, ctx: &egui::Context, saved: &mut SavedThumbnails) {
        self.busy = false;
        let Some(entry) = self.entries.get_mut(&done.key).filter(|e| e.version == done.version) else { return };
        let Some(preview) = done.preview else { return };
        let Some(bytes) = decoded_preview(&preview.png) else { return };
        let Some(image) = pagenor_images::decode_rgba(&bytes) else { return };
        let color = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
        entry.texture = Some(ctx.load_texture("recent-document-preview", color, egui::TextureOptions::LINEAR));
        if done.persist
            && let Key::File(path) = done.key
            && (saved.0.len() < LIMIT || saved.0.contains_key(&path))
        {
            saved.0.insert(path, preview);
        }
    }
}

impl DesignApp {
    pub(crate) fn refresh_recent_preview_files(&mut self) {
        for entry in self.recent_previews.entries.values_mut().filter(|e| e.version.is_none()) {
            entry.attempted = false;
        }
    }

    pub(crate) fn retain_recent_thumbnails(&mut self, rows: &[Row]) {
        let keys: HashSet<_> = rows.iter().filter_map(key).collect();
        self.recent_previews.entries.retain(|k, _| keys.contains(k));
        self.ui.recent_thumbnails.0.retain(|path, _| keys.contains(&Key::File(path.clone())));
    }

    pub(crate) fn recent_thumbnail(&mut self, row: &Row, ctx: &egui::Context) -> Option<TextureHandle> {
        let key = key(row)?;
        let previews = &mut self.recent_previews;
        #[cfg(not(target_arch = "wasm32"))]
        if !previews.started {
            previews.worker = Worker::spawn(ctx.clone());
            previews.started = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(done) = previews.worker.as_ref().and_then(|w| w.rx.try_recv().ok()) {
            previews.receive(done, ctx, &mut self.ui.recent_thumbnails);
        }
        let doc = row.uid.and_then(|uid| self.session.documents().iter().find(|d| d.uid == uid));
        let version = doc.map(|d| (d.uid, d.revision));
        let entry = previews.entries.entry(key.clone()).or_insert(Cached { version, attempted: false, texture: None });
        if entry.version != version {
            entry.version = version;
            entry.attempted = false;
        }
        if !entry.attempted && !previews.busy {
            let snapshot = doc.map(|d| Snapshot {
                doc: d.preview_stash.clone().unwrap_or_else(|| d.doc.clone()),
                cache: self.session.cache.clone(),
                _fonts: d.fonts.clone(),
            });
            #[cfg(not(target_arch = "wasm32"))]
            let cached = match &key {
                Key::File(path) => self.ui.recent_thumbnails.0.get(path).cloned(),
                Key::Open(_) => None,
            };
            let job = Job {
                key: key.clone(),
                version,
                snapshot,
                #[cfg(not(target_arch = "wasm32"))]
                cached,
                persist: doc.is_none_or(|d| !d.is_dirty()),
            };
            entry.attempted = true;
            #[cfg(not(target_arch = "wasm32"))]
            {
                previews.busy = previews.worker.as_ref().is_some_and(|w| w.tx.try_send(job).is_ok());
            }
            #[cfg(target_arch = "wasm32")]
            {
                let image = preview(&job);
                previews.receive(
                    Done { key: job.key, version: job.version, preview: image, persist: job.persist },
                    ctx,
                    &mut self.ui.recent_thumbnails,
                );
            }
        }
        if previews.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
        previews.entries.get(&key).and_then(|e| e.texture.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn snapshot(session: &pagenor_engine::Session) -> Snapshot {
        let doc = session.doc().unwrap();
        Snapshot { doc: doc.doc.clone(), cache: session.cache.clone(), _fonts: doc.fonts.clone() }
    }

    #[test]
    fn preview_renders_real_first_page_and_preferences_round_trip_only_small_images() {
        let mut session = pagenor_engine::Session::new();
        session.execute("file.newSample", &json!({})).unwrap();
        let png = render(&snapshot(&session)).unwrap();
        let pixels = pagenor_images::decode_rgba(&decoded_preview(&png).unwrap()).unwrap();
        assert!(pixels.width() <= SIDE && pixels.height() <= SIDE);
        assert!(pixels.pixels().any(|p| p[0] < 180 || p[1] < 180 || p[2] < 180), "document artwork appears in the preview");
        let entries: BTreeMap<_, _> = (0..30).map(|n| (format!("{n}.pgnr"), SavedPreview { png: png.clone(), stamp: None })).collect();
        let mut saved: SavedThumbnails = serde_json::from_value(json!(entries)).unwrap();
        assert_eq!(saved.0.len(), LIMIT);
        let before = saved.0.values().next().unwrap().png.clone();
        saved.0.insert("invalid.pgnr".into(), SavedPreview { png: "corrupt".into(), stamp: None });
        let round_trip: SavedThumbnails = serde_json::from_value(json!(saved)).unwrap();
        assert!(!round_trip.0.contains_key("invalid.pgnr"));
        assert_eq!(round_trip.0.values().next().unwrap().png, before);
        let mut oversized = decoded_preview(&png).unwrap();
        oversized[16..20].copy_from_slice(&100_000_u32.to_be_bytes());
        assert!(decoded_preview(&pagenor_engine::cmd::base64_encode(&oversized)).is_none());
    }

    #[test]
    fn stale_background_result_cannot_replace_an_edited_document_preview() {
        let ctx = egui::Context::default();
        let key = Key::Open(1);
        let mut previews = Previews::default();
        previews.entries.insert(key.clone(), Cached { version: Some((1, 2)), attempted: false, texture: None });
        previews.busy = true;
        let mut saved = SavedThumbnails::default();
        previews.receive(
            Done { key, version: Some((1, 1)), preview: Some(SavedPreview { png: "old".into(), stamp: None }), persist: false },
            &ctx,
            &mut saved,
        );
        assert!(!previews.busy);
        assert!(previews.entries.values().all(|e| e.texture.is_none()));
        assert!(saved.0.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn closed_file_preview_is_cached_and_refreshes_after_save_without_opening_ui_tabs() {
        let dir = std::env::temp_dir().join(format!("pagenor-preview-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Preview.pgnr");
        let mut session = pagenor_engine::Session::new();
        session.execute("file.new", &json!({})).unwrap();
        session.execute("file.saveAs", &json!({"path": path})).unwrap();
        let mut job = Job { key: Key::File(path.to_string_lossy().into_owned()), version: None, snapshot: None, cached: None, persist: true };
        let first = preview(&job).unwrap();
        job.cached = Some(first.clone());
        assert_eq!(preview(&job).unwrap().png, first.png);
        session.execute("frame.create", &json!({"rect": [36, 36, 400, 150], "content": "text", "text": "VISIBLE PREVIEW"})).unwrap();
        session.execute("file.save", &json!({})).unwrap();
        let changed = preview(&job).unwrap();
        assert_ne!(changed.png, first.png);
        assert_ne!(changed.stamp, first.stamp);
        assert_eq!(session.documents().len(), 1);
        let before = session.doc().unwrap().doc.clone();
        assert!(session.execute("file.open", &json!({"path": path, "maxBytes": 1})).is_err());
        assert_eq!(session.documents().len(), 1);
        assert!(Arc::ptr_eq(&session.doc().unwrap().doc, &before));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
