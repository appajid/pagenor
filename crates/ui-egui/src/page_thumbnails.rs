//! Bounded, document-owned page/master thumbnails. Native rendering never holds the UI cache lock.
use pagenor_doc::Document;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub(crate) enum Target {
    Page(usize),
    Master(usize, usize),
}
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct Key {
    uid: u64,
    doc: usize,
    target: Target,
    height: u32,
    fonts: u64,
}
struct Entry {
    stamp: u64,
    bytes: usize,
    texture: Option<egui::TextureHandle>,
}
#[derive(Default)]
struct State {
    entries: HashMap<Key, Entry>,
    current: HashMap<u64, Weak<Document>>,
    pending: HashSet<Key>,
    clock: u64,
    bytes: usize,
}
#[derive(Default)]
pub(crate) struct Thumbnails {
    state: Arc<Mutex<State>>,
}
const MAX_ENTRIES: usize = 512;
const MAX_BYTES: usize = 40 << 20;
const MAX_PENDING: usize = 4;
impl Thumbnails {
    pub(crate) fn retain_documents(&self, live: impl Iterator<Item = u64>) {
        let live: HashSet<_> = live.collect();
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.current.retain(|uid, _| live.contains(uid));
        state.entries.retain(|key, _| live.contains(&key.uid));
        state.bytes = state.entries.values().map(|e| e.bytes).sum();
    }
    pub(crate) fn get(
        &self,
        ctx: &egui::Context,
        uid: u64,
        doc: Arc<Document>,
        cache: Arc<pagenor_compose::Cache>,
        target: Target,
        height: f32,
    ) -> Option<egui::TextureHandle> {
        let height = (height * ctx.pixels_per_point()).round().clamp(1.0, 512.0) as u32;
        let key = Key { uid, doc: Arc::as_ptr(&doc) as usize, target, height, fonts: pagenor_fonts::FontDb::global().scan_generation() };
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.current.insert(uid, Arc::downgrade(&doc));
        state.entries.retain(|entry, _| entry.uid != uid || entry.doc == key.doc && entry.fonts == key.fonts);
        state.bytes = state.entries.values().map(|entry| entry.bytes).sum();
        state.clock = state.clock.wrapping_add(1);
        let stamp = state.clock;
        if let Some(entry) = state.entries.get_mut(&key) {
            entry.stamp = stamp;
            return entry.texture.clone();
        }
        if state.pending.contains(&key) || state.pending.len() >= MAX_PENDING {
            return None;
        }
        state.pending.insert(key);
        drop(state);
        let state = Arc::downgrade(&self.state);
        let ctx = ctx.clone();
        let job = move || {
            let image = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(&doc, &cache, target, height))).ok().flatten();
            let bytes = image.as_ref().map_or(0, |image| image.pixels.len());
            let texture = image.map(|image| {
                let pixels = egui::ColorImage::from_rgba_premultiplied([image.width as usize, image.height as usize], &image.pixels);
                crate::widgets::load_texture(&ctx, format!("page-preview-{}-{:?}", uid, target), pixels, egui::TextureOptions::LINEAR)
            });
            if let Some(state) = state.upgrade() {
                let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
                state.pending.remove(&key);
                if state.current.get(&uid).is_some_and(|current| Weak::ptr_eq(current, &Arc::downgrade(&doc))) {
                    state.clock = state.clock.wrapping_add(1);
                    let stamp = state.clock;
                    state.entries.insert(key, Entry { stamp, bytes, texture });
                    state.bytes += bytes;
                    while state.entries.len() > MAX_ENTRIES || state.bytes > MAX_BYTES {
                        let victim = state.entries.iter().min_by_key(|(_, entry)| entry.stamp).map(|(key, _)| *key);
                        let Some(victim) = victim else { break };
                        if let Some(entry) = state.entries.remove(&victim) {
                            state.bytes = state.bytes.saturating_sub(entry.bytes);
                        }
                    }
                }
            }
            ctx.request_repaint();
        };
        #[cfg(target_arch = "wasm32")]
        job();
        #[cfg(not(target_arch = "wasm32"))]
        {
            if cfg!(test) || std::env::var_os("PAGENOR_SYNC_RENDER").is_some() {
                job();
            } else if let Err(error) = std::thread::Builder::new().name("pagenor-page-preview".into()).spawn(job) {
                self.state.lock().unwrap_or_else(|e| e.into_inner()).pending.remove(&key);
                log::warn!("Could not draw page preview: {error}");
            }
        }
        self.state.lock().unwrap_or_else(|e| e.into_inner()).entries.get(&key).and_then(|entry| entry.texture.clone())
    }
}
fn render(doc: &Document, cache: &pagenor_compose::Cache, target: Target, height: u32) -> Option<pagenor_render::Rendered> {
    let page = match target {
        Target::Page(abs) => doc.page(abs)?,
        Target::Master(index, page) => doc.parents.get(index)?.pages.get(page)?,
    };
    let scale = (f64::from(height) / page.height.max(1.0)).min(1024.0 / page.width.max(1.0));
    let mut renderer = pagenor_render::Renderer::new();
    renderer.threads = 0;
    let options = pagenor_render::RenderOptions { printing_only: true, ..Default::default() };
    match target {
        Target::Page(abs) => renderer.render_page(doc, cache, abs, scale, false, &options),
        Target::Master(index, _) => Some(renderer.render(
            doc,
            cache,
            &[pagenor_render::Placed { spread: pagenor_doc::SpreadRef::Parent(index), xf: pagenor_geom::Affine::IDENTITY }],
            (page.width * scale).ceil() as u32,
            (page.height * scale).ceil() as u32,
            pagenor_geom::Affine::scale(scale) * pagenor_geom::Affine::translate((-page.x, 0.0)),
            &options,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn previews_do_not_keep_documents_and_close_releases_textures() {
        let doc = Arc::new(Document::new(&pagenor_doc::build::NewDocument::default()));
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        let cache = Thumbnails::default();
        let texture = cache.get(&ctx, 42, doc.clone(), Arc::new(pagenor_compose::Cache::new()), Target::Page(0), 24.0);
        assert!(texture.is_some());
        assert_eq!(Arc::strong_count(&doc), 1);
        cache.retain_documents(std::iter::empty());
        let state = cache.state.lock().unwrap();
        assert!(state.entries.is_empty());
        assert_eq!(state.bytes, 0);
        drop(state);
        drop(texture);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
    }
}
