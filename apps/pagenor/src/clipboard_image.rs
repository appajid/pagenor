//! One bounded, nonblocking native image read. Worker requests never retain a document.
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};

pub const WAIT: std::time::Duration = std::time::Duration::from_millis(1500);
pub struct ImageFile {
    pub name: String,
    pub bytes: Vec<u8>,
}
type Reply = (u64, Result<Option<ImageFile>, String>);
pub struct Reader {
    send: SyncSender<(u64, egui::Context)>,
    recv: Receiver<Reply>,
    busy: bool,
    next: u64,
}
impl Reader {
    pub fn new() -> Result<Self, String> {
        Self::with_read(read)
    }
    fn with_read(read: impl Fn() -> Result<Option<ImageFile>, String> + Send + 'static) -> Result<Self, String> {
        let (send, requests) = mpsc::sync_channel::<(u64, egui::Context)>(1);
        let (replies, recv) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("clipboard images".into())
            .spawn(move || {
                while let Ok((id, ctx)) = requests.recv() {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(&read))
                        .unwrap_or_else(|_| Err("The clipboard image could not be read.".into()));
                    if replies.send((id, result)).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .map_err(|e| format!("Could not start clipboard reading: {e}"))?;
        Ok(Self { send, recv, busy: false, next: 0 })
    }
    pub fn request(&mut self, ctx: &egui::Context) -> Option<u64> {
        if self.busy {
            return None;
        }
        self.next = self.next.checked_add(1)?;
        self.send.try_send((self.next, ctx.clone())).ok()?;
        self.busy = true;
        ctx.request_repaint_after(WAIT);
        Some(self.next)
    }
    pub fn poll(&mut self) -> Option<Reply> {
        match self.recv.try_recv() {
            Ok(reply) => {
                self.busy = false;
                Some(reply)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.busy = false;
                None
            }
        }
    }
}

pub fn image_address(text: &str) -> bool {
    pagenor_ui_egui::clipboard_image_address(text)
}

fn read() -> Result<Option<ImageFile>, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Could not read the clipboard: {e}"))?;
    if let Ok(image) = clipboard.get_image() {
        let width = u32::try_from(image.width).map_err(|_| "The clipboard image is too wide.")?;
        let height = u32::try_from(image.height).map_err(|_| "The clipboard image is too tall.")?;
        pagenor_engine::cmd::check_pasted_size(width, height)?;
        let bytes = pagenor_engine::cmd::pasted_png(width, height, image.bytes.into_owned())?;
        return Ok(Some(ImageFile { name: "Pasted image.png".into(), bytes }));
    }
    if let Ok(files) = clipboard.get().file_list() {
        for path in files.iter().take(1024) {
            let image = path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| pagenor_engine::cmd::IMAGE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
            if !image {
                continue;
            }
            if let Ok(bytes) = read_capped(path) {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Pasted image.png".into());
                return Ok(Some(ImageFile { name, bytes }));
            }
        }
    }
    Ok(None)
}

fn read_capped(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    const MAX: u64 = 256 << 20;
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > MAX {
        return Err(std::io::Error::other("The clipboard image exceeds 256 MiB."));
    }
    let mut bytes = Vec::new();
    file.take(MAX + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX {
        return Err(std::io::Error::other("The clipboard image exceeds 256 MiB."));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_blocked_clipboard_never_blocks_a_frame_or_starts_another_read() {
        let (release, blocked) = mpsc::sync_channel::<()>(1);
        let mut reader = Reader::with_read(move || {
            blocked.recv().map_err(|e| e.to_string())?;
            Ok(None)
        })
        .unwrap();
        let ctx = egui::Context::default();
        let ticket = reader.request(&ctx).unwrap();
        assert!(reader.poll().is_none());
        assert!(reader.request(&ctx).is_none());
        release.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some((got, result)) = reader.poll() {
                assert_eq!(got, ticket);
                assert!(result.unwrap().is_none());
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    }
    #[test]
    fn only_a_single_address_uses_image_fallback() {
        assert!(image_address("https://example.com/photo.png"));
        assert!(!image_address("text https://example.com/photo.png"));
        assert!(!image_address("ఉదయం"));
        assert!(!image_address(""));
    }
}
