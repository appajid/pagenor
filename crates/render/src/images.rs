//! Decoded placed images, shared by every [`crate::Renderer`] in the process (canvas worker,
//! thumbnails, exports), so each asset is decoded once and each mip level built once.
//!
//! Entries are keyed by the address of the asset's encoded bytes (`Arc<Vec<u8>>`) and hold a
//! clone of that `Arc`, so the address can't be reused while cached. Least recently used levels
//! are evicted beyond a memory budget.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use vello_cpu::Pixmap;

use crate::{decode_pixmap_page_with_scope, halve};

// SVG pixels depend on the document scope and the current font catalogue.
type SourceKey = (usize, u32, u32, u64);
type Key = (SourceKey, u8);

/// Decoded pixels kept (bytes).
const BUDGET: usize = 768 << 20;

struct Entry {
    _bytes: Weak<Vec<u8>>,
    pm: Arc<Pixmap>,
    stamp: u64,
}

#[derive(Default)]
struct Cache {
    /// (bytes address, PDF page, font scope, font generation), mip level.
    map: HashMap<Key, Entry>,
    bytes: usize,
    clock: u64,
    /// Assets that failed to decode (don't retry every frame).
    bad: HashMap<SourceKey, Weak<Vec<u8>>>,
    dims: HashMap<SourceKey, (Weak<Vec<u8>>, (u32, u32))>,
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

fn size_of(pm: &Pixmap) -> usize {
    pm.width() as usize * pm.height() as usize * 4
}

fn with<R>(f: impl FnOnce(&mut Cache) -> R) -> R {
    let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    f(g.get_or_insert_with(Cache::default))
}

fn lookup(key: Key) -> Option<Arc<Pixmap>> {
    with(|c| {
        c.clock += 1;
        let clock = c.clock;
        if c.map.get(&key).is_some_and(|entry| entry._bytes.strong_count() == 0) {
            if let Some(entry) = c.map.remove(&key) {
                c.bytes = c.bytes.saturating_sub(size_of(&entry.pm));
            }
            return None;
        }
        c.map.get_mut(&key).map(|e| {
            e.stamp = clock;
            e.pm.clone()
        })
    })
}

fn store(bytes: &Arc<Vec<u8>>, source: SourceKey, level: u8, pm: Arc<Pixmap>) {
    if size_of(&pm) > BUDGET {
        return;
    }
    with(|c| {
        c.clock += 1;
        let key = (source, level);
        let n = size_of(&pm);
        if let Some(old) = c.map.insert(key, Entry { _bytes: Arc::downgrade(bytes), pm, stamp: c.clock }) {
            c.bytes -= size_of(&old.pm);
        }
        c.bytes += n;
        if c.clock % 64 == 0 {
            c.map.retain(|_, entry| entry._bytes.strong_count() > 0);
            c.bytes = c.map.values().map(|entry| size_of(&entry.pm)).sum();
        }
        if c.bytes > BUDGET || c.map.len() > 2048 {
            let mut v: Vec<(u64, Key)> = c.map.iter().map(|(k, e)| (e.stamp, *k)).collect();
            v.sort_unstable();
            for (_, k) in v {
                if c.bytes <= BUDGET * 3 / 4 && c.map.len() <= 1536 {
                    break;
                }
                if k == key {
                    continue;
                }
                if let Some(e) = c.map.remove(&k) {
                    c.bytes -= size_of(&e.pm);
                }
            }
        }
    });
}

fn source_key(bytes: &Arc<Vec<u8>>, page: u32, scope: u32) -> SourceKey {
    let (scope, generation) = if pagenor_images::is_svg(bytes) { (scope, pagenor_fonts::FontDb::global().scan_generation()) } else { (0, 0) };
    (Arc::as_ptr(bytes) as usize, page, scope, generation)
}

/// Full-resolution decode of `bytes` (cached), without document-local fonts.
pub fn decoded(bytes: &Arc<Vec<u8>>, page: u32) -> Option<Arc<Pixmap>> {
    decoded_with_key(bytes, source_key(bytes, page, 0))
}

fn decoded_with_key(bytes: &Arc<Vec<u8>>, source: SourceKey) -> Option<Arc<Pixmap>> {
    if let Some(pm) = lookup((source, 0)) {
        return Some(pm);
    }
    let pm = decode_full(bytes, source)?;
    store(bytes, source, 0, pm.clone());
    Some(pm)
}
fn decode_full(bytes: &Arc<Vec<u8>>, source: SourceKey) -> Option<Arc<Pixmap>> {
    if with(|c| c.bad.contains_key(&source)) {
        return None;
    }
    match decode_pixmap_page_with_scope(bytes, source.1, source.2) {
        Some(pm) => {
            let size = (u32::from(pm.width()), u32::from(pm.height()));
            with(|c| {
                if c.dims.len() >= 1024 {
                    c.dims.retain(|_, (bytes, _)| bytes.strong_count() > 0);
                }
                if c.dims.len() >= 1024
                    && let Some(key) = c.dims.keys().next().copied()
                {
                    c.dims.remove(&key);
                }
                c.dims.insert(source, (Arc::downgrade(bytes), size));
            });
            Some(Arc::new(pm))
        }
        None => {
            with(|c| {
                if c.bad.len() >= 64
                    && let Some(key) = c.bad.keys().next().copied()
                {
                    c.bad.remove(&key);
                }
                c.bad.insert(source, Arc::downgrade(bytes));
            });
            None
        }
    }
}
fn level_for(size: (u32, u32), width: f64, scale: f64) -> u8 {
    let ppt = f64::from(size.0) / width.max(1e-6);
    let (mut w, mut h, mut level) = (size.0, size.1, 0);
    while level < 8 && w > 64 && h > 64 && ppt / (1u64 << (level + 1)) as f64 >= scale * 1.2 {
        level += 1;
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    level
}
fn dimensions(bytes: &Arc<Vec<u8>>, source: SourceKey) -> Option<(u32, u32)> {
    with(|c| c.dims.get(&source).filter(|(live, _)| live.upgrade().is_some_and(|data| Arc::ptr_eq(&data, bytes))).map(|(_, size)| *size))
}

/// The mip level closest above `on_screen` device pixels per point.
pub fn mip(bytes: &Arc<Vec<u8>>, page: u32, width_pt: f64, on_screen: f64) -> Option<Arc<Pixmap>> {
    mip_with_scope(bytes, page, width_pt, on_screen, 0)
}

/// Document-aware SVG decode. Raster/PDF cache sharing is unchanged.
pub fn mip_with_scope(bytes: &Arc<Vec<u8>>, page: u32, width_pt: f64, on_screen: f64, scope: u32) -> Option<Arc<Pixmap>> {
    let source = source_key(bytes, page, scope);
    let desired = dimensions(bytes, source).map(|size| level_for(size, width_pt, on_screen));
    if let Some(level) = desired {
        if let Some(pm) = lookup((source, level)) {
            return Some(pm);
        }
        if let Some((from, mut pm)) = (0..level).rev().find_map(|from| lookup((source, from)).map(|pm| (from, pm))) {
            for _ in from..level {
                pm = Arc::new(halve(&pm));
            }
            store(bytes, source, level, pm.clone());
            return Some(pm);
        }
    }
    let mut pm = decode_full(bytes, source)?;
    let level = desired.unwrap_or_else(|| level_for((u32::from(pm.width()), u32::from(pm.height())), width_pt, on_screen));
    for _ in 0..level {
        pm = Arc::new(halve(&pm));
    }
    // Keep only the requested proxy, so a spread of large photos cannot evict itself.
    if source_key(bytes, page, scope) == source {
        store(bytes, source, level, pm.clone());
    }
    Some(pm)
}

static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
type Wake = Arc<dyn Fn() + Send + Sync>;
static WAKE: Mutex<Option<Wake>> = Mutex::new(None);
pub fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn set_waker(wake: impl Fn() + Send + Sync + 'static) {
    *WAKE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(wake));
}
fn wake() {
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let wake = WAKE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(wake) = wake {
        wake();
    }
}
#[cfg(not(target_arch = "wasm32"))]
struct Job {
    data: Weak<Vec<u8>>,
    source: SourceKey,
    width: f64,
    scale: f64,
    level: u8,
}
#[cfg(not(target_arch = "wasm32"))]
static PENDING: Mutex<Option<std::collections::HashSet<Key>>> = Mutex::new(None);
#[cfg(not(target_arch = "wasm32"))]
fn pending<R>(f: impl FnOnce(&mut std::collections::HashSet<Key>) -> R) -> R {
    f(PENDING.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default))
}
#[cfg(not(target_arch = "wasm32"))]
fn worker() -> Option<&'static std::sync::mpsc::SyncSender<Job>> {
    static WORKER: std::sync::OnceLock<Option<std::sync::mpsc::SyncSender<Job>>> = std::sync::OnceLock::new();
    WORKER
        .get_or_init(|| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<Job>(4);
            let rx = Arc::new(Mutex::new(rx));
            let mut started = 0;
            for _ in 0..2 {
                let rx = rx.clone();
                if std::thread::Builder::new()
                    .name("pagenor-image-proxy".into())
                    .spawn(move || {
                        loop {
                            let job = rx.lock().unwrap_or_else(|e| e.into_inner()).recv();
                            let Ok(job) = job else { break };
                            let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                if let Some(data) = job.data.upgrade()
                                    && source_key(&data, job.source.1, job.source.2) == job.source
                                {
                                    let _pixels = mip_with_scope(&data, job.source.1, job.width, job.scale, job.source.2);
                                }
                            })); // A corrupt decoder must not end the worker or strand the request.
                            if _result.is_err()
                                && let Some(bytes) = job.data.upgrade()
                            {
                                with(|cache| {
                                    if cache.bad.len() >= 64 {
                                        cache.bad.clear();
                                    }
                                    cache.bad.insert(job.source, Arc::downgrade(&bytes));
                                });
                            }
                            pending(|set| set.remove(&(job.source, job.level)));
                            wake();
                        }
                    })
                    .is_ok()
                {
                    started += 1;
                }
            }
            (started > 0).then_some(tx)
        })
        .as_ref()
}
/// Canvas-only, nonblocking image lookup. Export paths continue to use the synchronous functions.
pub fn mip_now_with_scope(bytes: &Arc<Vec<u8>>, page: u32, width: f64, scale: f64, scope: u32) -> Option<Arc<Pixmap>> {
    #[cfg(target_arch = "wasm32")]
    {
        mip_with_scope(bytes, page, width, scale, scope)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let source = source_key(bytes, page, scope);
        if with(|cache| cache.bad.contains_key(&source)) {
            return None;
        }
        let desired = dimensions(bytes, source).map_or(255, |size| level_for(size, width, scale));
        if desired != 255
            && let Some(pm) = lookup((source, desired))
        {
            return Some(pm);
        }
        let cached = with(|cache| {
            cache
                .map
                .iter()
                .filter(|((key, _), _)| *key == source)
                .min_by_key(|((_, level), _)| if desired == 255 { u16::from(*level) } else { u16::from(level.abs_diff(desired)) })
                .map(|(_, entry)| entry.pm.clone())
        });
        if let Some(worker) = worker() {
            let key = (source, desired);
            if pending(|set| set.len() < 6 && set.insert(key))
                && worker.try_send(Job { data: Arc::downgrade(bytes), source, width, scale, level: desired }).is_err()
            {
                pending(|set| set.remove(&key));
            }
        } else if cached.is_none() {
            return mip_with_scope(bytes, page, width, scale, scope);
        }
        cached
    }
}

/// Drop every decoded image.
pub fn clear() {
    with(|c| {
        c.map.clear();
        c.dims.clear();
        c.bad.clear();
        c.bytes = 0;
    });
    wake();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_cache_separates_document_fonts_and_refreshes_memory_fonts() {
        let fonts = pagenor_fonts::FontDb::global();
        let root = std::env::temp_dir().join(format!("pagenor-svg-cache-{}", std::process::id()));
        for (name, glyph) in [("one", 'X'), ("two", 'W')] {
            std::fs::create_dir_all(root.join(name)).unwrap();
            std::fs::write(root.join(name).join("font.ttf"), pagenor_fonts::testing::font_with_glyph("Source Sans 3", &['A'], glyph).unwrap())
                .unwrap();
        }
        // Publish the initial lazy catalogue before asserting cache hits in a stable generation.
        let _ = fonts.families();
        let one = fonts.load_document_fonts(&root.join("one")).scope;
        let two = fonts.load_document_fonts(&root.join("two")).scope;
        assert_ne!(one, 0);
        assert_ne!(two, 0);
        let data = Arc::new(br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text y="30" font-size="20" font-family="Source Sans 3">A</text></svg>"#.to_vec());
        let first = mip_with_scope(&data, 0, 100.0, 1.0, one).unwrap();
        let second = mip_with_scope(&data, 0, 100.0, 1.0, two).unwrap();
        assert_ne!(first.data(), second.data());
        assert!(Arc::ptr_eq(&first, &mip_with_scope(&data, 0, 100.0, 1.0, one).unwrap()));
        let refresh = Arc::new(br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text y="30" font-size="20" font-family="Pagenor SVG Refresh">A</text></svg>"#.to_vec());
        let before = decoded(&refresh, 0).unwrap();
        assert_eq!(fonts.add_font(pagenor_fonts::testing::font_with_glyph("Pagenor SVG Refresh", &['A'], 'X').unwrap()), 1);
        let after = decoded(&refresh, 0).unwrap();
        assert_ne!(before.data(), after.data());
        assert!(!Arc::ptr_eq(&before, &after));
        assert!(Arc::ptr_eq(&after, &decoded(&refresh, 0).unwrap()));
        fonts.close_scope(one);
        let closed = mip_with_scope(&data, 0, 100.0, 1.0, one).unwrap();
        assert_ne!(closed.data(), first.data(), "closed scopes must not reuse their former SVG glyphs");
        fonts.close_scope(two);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod proxy_review31_tests {
    use super::*;
    fn raster() -> Arc<Vec<u8>> {
        Arc::new(crate::Rendered { width: 512, height: 512, pixels: [40, 90, 180, 255].repeat(512 * 512) }.to_png())
    }
    #[test]
    fn requested_proxy_does_not_keep_the_full_decode() {
        let bytes = raster();
        let pm = mip(&bytes, 0, 64.0, 1.0).unwrap();
        assert!(pm.width() <= 128);
        let source = source_key(&bytes, 0, 0);
        assert!(!with(|cache| cache.map.contains_key(&(source, 0))));
        assert_eq!(decoded(&bytes, 0).unwrap().width(), 512, "export still obtains the full pixels");
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn canvas_proxy_finishes_off_thread_and_wakes() {
        let bytes = raster();
        let _first = mip_now_with_scope(&bytes, 0, 64.0, 1.0, 0);
        let mut result = None;
        for _ in 0..500 {
            result = mip_now_with_scope(&bytes, 0, 64.0, 1.0, 0);
            if result.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        assert!(result.is_some());
        assert!(result.unwrap().width() <= 128);
    }
}

#[cfg(test)]
mod ownership_review31_tests {
    use super::*;
    #[test]
    fn cached_proxy_does_not_retain_closed_document_encoded_bytes() {
        let bytes = Arc::new(crate::Rendered { width: 8, height: 8, pixels: [12, 40, 70, 255].repeat(64) }.to_png());
        let owner = Arc::downgrade(&bytes);
        let source = source_key(&bytes, 0, 0);
        let _image = decoded(&bytes, 0).unwrap();
        drop(bytes);
        assert!(owner.upgrade().is_none());
        assert!(lookup((source, 0)).is_none());
    }
}
