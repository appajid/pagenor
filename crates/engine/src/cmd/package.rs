//! File › Package and Links › Copy Links To: gather a document with the files it uses.

use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

use super::{CommandSpec, bad, cmd, has_doc, str_param};
use crate::{EngineError, Result, Session};
use pagenor_compose::ComposedStory;
use pagenor_doc::{AssetId, Document};
use pagenor_fonts::{FaceRef, FontFace, FontSource};
use serde_json::{Value, json};

// Reserve 24 bytes for an extension, numbering and a Windows device-name prefix.
const MAX_OUTPUT_COMPONENT_BYTES: usize = 200;

fn path_error(path: &Path, message: impl std::fmt::Display) -> EngineError {
    EngineError::Other(format!("{}: {message}", path.display()))
}

fn is_redirect(metadata: &Metadata) -> bool {
    // Rust reports Windows name-surrogate reparse points (including directory junctions) as
    // symlinks, while leaving non-redirecting reparse metadata such as cloud placeholders alone.
    metadata.file_type().is_symlink()
}

fn ensure_real_directory(path: &Path) -> Result<()> {
    // Walk even existing ancestors: an existing nested folder beneath a redirected parent
    // must not bypass the check. An iterative walk also avoids input-sized recursion.
    let ancestors: Vec<_> = path.ancestors().filter(|p| !p.as_os_str().is_empty()).collect();
    for directory in ancestors.into_iter().rev() {
        match std::fs::symlink_metadata(directory) {
            Ok(metadata) => {
                if is_redirect(&metadata) {
                    return Err(path_error(directory, "refusing a redirected output directory"));
                }
                if !metadata.is_dir() {
                    return Err(path_error(directory, "output path is not a directory"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if let Err(e) = std::fs::create_dir(directory)
                    && e.kind() != std::io::ErrorKind::AlreadyExists
                {
                    return Err(path_error(directory, e));
                }
                let metadata = std::fs::symlink_metadata(directory).map_err(|e| path_error(directory, e))?;
                if is_redirect(&metadata) || !metadata.is_dir() {
                    return Err(path_error(directory, "output directory was redirected while it was being created"));
                }
            }
            Err(e) => return Err(path_error(directory, e)),
        }
    }
    Ok(())
}

/// A selected output directory and its resolved identity. Every package path is checked against
/// it before use. Outputs are staged in the same directory and atomically persisted over an
/// existing regular file, preserving overwrite behavior without following file links.
struct OutputRoot {
    path: PathBuf,
    canonical: PathBuf,
}

impl OutputRoot {
    fn new(path: PathBuf) -> Result<Self> {
        ensure_real_directory(&path)?;
        let canonical = std::fs::canonicalize(&path).map_err(|e| path_error(&path, e))?;
        Ok(Self { path, canonical })
    }

    fn validate_directory(&self, path: &Path) -> Result<()> {
        if !path.starts_with(&self.path) {
            return Err(path_error(path, "output path is outside the selected directory"));
        }
        ensure_real_directory(path)?;
        let canonical = std::fs::canonicalize(path).map_err(|e| path_error(path, e))?;
        if !canonical.starts_with(&self.canonical) {
            return Err(path_error(path, "output directory resolves outside the selected directory"));
        }
        Ok(())
    }

    fn child(&self, name: &str) -> Result<PathBuf> {
        let path = self.path.join(name);
        if path.parent() != Some(self.path.as_path()) {
            return Err(path_error(&path, "invalid output directory name"));
        }
        Ok(path)
    }

    fn validate_file_entry(&self, path: &Path) -> Result<bool> {
        let parent = path.parent().ok_or_else(|| path_error(path, "output file has no parent directory"))?;
        self.validate_directory(parent)?;
        if !path.starts_with(&self.path) {
            return Err(path_error(path, "output file is outside the selected directory"));
        }
        match std::fs::symlink_metadata(path) {
            Ok(metadata) => {
                if is_redirect(&metadata) {
                    return Err(path_error(path, "refusing to replace a redirected output file"));
                }
                if !metadata.is_file() {
                    return Err(path_error(path, "output path is not a regular file"));
                }
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(path_error(path, e)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn temporary_file(&self, path: &Path, permissions: Option<std::fs::Permissions>) -> Result<tempfile::NamedTempFile> {
        let parent = path.parent().ok_or_else(|| path_error(path, "output file has no parent directory"))?;
        self.validate_directory(parent)?;
        let mut builder = tempfile::Builder::new();
        builder.prefix(".pagenor-package-").suffix(".tmp");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Match fs::write for a new output: 0666 restricted by the caller's umask.
            builder.permissions(std::fs::Permissions::from_mode(0o666));
        }
        let file = builder.tempfile_in(parent).map_err(|e| path_error(path, e))?;
        #[cfg(unix)]
        if let Some(permissions) = permissions {
            // Overwrites retain their prior mode; copied fonts retain their source mode.
            file.as_file().set_permissions(permissions).map_err(|e| path_error(path, e))?;
        }
        #[cfg(not(unix))]
        // The replacement inherits its directory's ACL. Rust's portable permission API cannot
        // reproduce a destination file's Windows DACL without platform-specific code.
        let _ = permissions;
        Ok(file)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (path, bytes);
            return Err(EngineError::Other("package files are unavailable on the web".into()));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let existed = self.validate_file_entry(path)?;
            let permissions = if existed { Some(std::fs::symlink_metadata(path).map_err(|e| path_error(path, e))?.permissions()) } else { None };
            let mut file = self.temporary_file(path, permissions)?;
            file.write_all(bytes).map_err(|e| path_error(path, e))?;
            // Recheck immediately before replacement. If a name is raced after this check,
            // persist replaces that directory entry rather than following it.
            self.validate_file_entry(path)?;
            file.persist(path).map_err(|e| path_error(path, e.error))?;
            Ok(())
        }
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        if self.validate_file_entry(to)?
            && std::fs::canonicalize(to).is_ok_and(|target| std::fs::canonicalize(from).is_ok_and(|source| source == target))
        {
            return Ok(());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = from;
            return Err(EngineError::Other("package files are unavailable on the web".into()));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut source = File::open(from).map_err(|e| path_error(from, e))?;
            let permissions = Some(source.metadata().map_err(|e| path_error(from, e))?.permissions());
            let mut target = self.temporary_file(to, permissions)?;
            std::io::copy(&mut source, &mut target).map_err(|e| path_error(to, e))?;
            self.validate_file_entry(to)?;
            target.persist(to).map_err(|e| path_error(to, e.error))?;
            Ok(())
        }
    }
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            noundo "file.package",
            "Package…",
            ["File"],
            None,
            "{dir: folder to create, idml?: true, pdf?: false, instructions?: text} — the document (its placed files relinked to Links/), Links/, the font files that draw its text in Document Fonts/ (fallback fonts included; unless their licence restricts it), an IDML copy, an optional PDF and a report → {dir, files, report}",
            has_doc,
            package
        ),
        cmd!(
            "links.copyTo",
            "Copy Link(s) To…",
            [],
            None,
            "{dir, assets?: [asset ids] (default: all)} — write the placed files to `dir` and relink to the copies → {copied}",
            has_doc,
            copy_links
        ),
    ]
}

fn is_reserved_windows_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end_matches([' ', '.']);
    ["con", "prn", "aux", "nul"].into_iter().any(|reserved| caseless::compatibility_caseless_match_str(stem, reserved))
        || (1..=9).any(|n| {
            caseless::compatibility_caseless_match_str(stem, &format!("com{n}"))
                || caseless::compatibility_caseless_match_str(stem, &format!("lpt{n}"))
        })
}

fn portable_names_equal(a: &str, b: &str) -> bool {
    caseless::canonical_caseless_match_str(a, b)
}

/// A portable single path component made from document metadata.
fn safe_component(name: &str, fallback: &str) -> String {
    let leaf = name.rsplit(['/', '\\']).find(|part| !part.is_empty()).unwrap_or("");
    let mut safe = String::new();
    for ch in leaf.trim().chars() {
        let replacement = ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '/' | '\\');
        let ch = if replacement { '_' } else { ch };
        if safe.len() + ch.len_utf8() > MAX_OUTPUT_COMPONENT_BYTES - 24 {
            break;
        }
        safe.push(ch);
    }
    safe.truncate(safe.trim_end_matches([' ', '.']).len());
    if safe.is_empty() || safe == "." || safe == ".." {
        safe = fallback.to_string();
    }
    if is_reserved_windows_name(&safe) {
        safe.insert(0, '_');
    }
    safe
}

/// Preserve a short file extension when shortening a long basename, so packaged font and
/// image files remain discoverable by readers that inspect the extension.
fn safe_file_component(name: &str, fallback: &str) -> String {
    let mut safe = safe_component(name, fallback);
    let leaf = name.rsplit(['/', '\\']).find(|part| !part.is_empty()).unwrap_or("");
    if let Some((_, extension)) = leaf.trim().trim_end_matches([' ', '.']).rsplit_once('.')
        && !extension.is_empty()
        && extension.len() <= 16
        && extension.chars().all(|c| c.is_ascii_alphanumeric())
    {
        let suffix = format!(".{extension}");
        if !safe.ends_with(&suffix) {
            let budget = MAX_OUTPUT_COMPONENT_BYTES - 24 - suffix.len();
            let end = safe.char_indices().map(|(i, _)| i).chain(std::iter::once(safe.len())).take_while(|i| *i <= budget).last().unwrap_or(0);
            safe.truncate(end);
            safe.push_str(&suffix);
        }
    }
    safe
}

fn package_output_path(dir: &Path, title: &str, extension: &str) -> Result<PathBuf> {
    let path = dir.join(format!("{title}.{extension}"));
    if path.parent() != Some(dir) {
        return Err(bad("file.package", "an output filename would be outside the selected directory"));
    }
    Ok(path)
}

/// A file name for asset `a` in a Links folder, unique among `taken`.
fn link_name(name: &str, id: AssetId, taken: &mut Vec<String>) -> String {
    let fallback = format!("asset-{}", id.0);
    unique_name(safe_file_component(name, &fallback), taken)
}

/// `base`, or `base` numbered ("logo 2.png"), unique under portable Unicode normalization and
/// case-insensitive comparison.
fn unique_name(base: String, taken: &mut Vec<String>) -> String {
    let (stem, ext) = match base.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (base.clone(), String::new()),
    };
    let mut n = base;
    let mut k = 2;
    while taken.iter().any(|t| portable_names_equal(t, &n)) {
        n = format!("{stem} {k}{ext}");
        k += 1;
    }
    taken.push(n.clone());
    n
}

/// Prepare relinking without changing the filesystem or the live document.
fn plan_links(d: &Document, dir: &Path, only: Option<&[AssetId]>) -> (Document, Vec<(AssetId, PathBuf)>) {
    let mut out = d.clone();
    let mut taken = Vec::new();
    let mut written = Vec::new();
    let mut ids: Vec<AssetId> = d.assets.keys().copied().collect();
    ids.sort_by_key(|i| i.0);
    for id in ids {
        if only.is_some_and(|o| !o.contains(&id)) {
            continue;
        }
        let a = &d.assets[&id];
        let name = link_name(a.link.as_deref().unwrap_or(&a.name), id, &mut taken);
        let path = dir.join(&name);
        let mut na = (**a).clone();
        na.link = Some(path.to_string_lossy().to_string());
        out.assets.insert(id, Arc::new(na));
        written.push((id, path));
    }
    (out, written)
}

fn write_planned_links(d: &Document, root: &OutputRoot, planned: &[(AssetId, PathBuf)]) -> Result<()> {
    for (id, path) in planned {
        let asset = d.assets.get(id).ok_or_else(|| path_error(path, "planned asset is missing"))?;
        root.write(path, asset.data.as_slice())?;
    }
    Ok(())
}

/// Validate every link destination before replacing any individual file.
fn write_links(d: &Document, root: &OutputRoot, dir: &Path, only: Option<&[AssetId]>) -> Result<(Document, Vec<PathBuf>)> {
    root.validate_directory(dir)?;
    let (out, planned) = plan_links(d, dir, only);
    for (_, path) in &planned {
        root.validate_file_entry(path)?;
    }
    write_planned_links(d, root, &planned)?;
    Ok((out, planned.into_iter().map(|(_, path)| path).collect()))
}

/// Copy the font file of `face` into `dir` (created when first needed) unless its licence
/// restricts it or it is already there (`copied`: the source files copied) → what the package
/// report adds after the font's name (nothing when copied).
fn copy_font(root: &OutputRoot, face: &FontFace, dir: &Path, copied: &mut Vec<PathBuf>, taken: &mut Vec<String>, files: &mut Vec<PathBuf>) -> String {
    let from = match &face.source {
        FontSource::Bundled => return " — included with Pagenor".into(),
        FontSource::Memory => return " — not copied: no font file".into(),
        FontSource::Native(_) => return " — not copied: supplied by the operating system".into(),
        FontSource::Installed(p) | FontSource::Document(p) => p,
    };
    if face.restricted_licence() {
        return " — not copied: its licence doesn't allow it".into();
    }
    if copied.contains(from) {
        return String::new();
    }
    let name = safe_file_component(&from.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(), "font");
    let to = dir.join(unique_name(name, taken));
    match root.validate_directory(dir).and_then(|_| root.copy(from, &to)) {
        Ok(()) => {
            copied.push(from.clone());
            files.push(to);
            String::new()
        }
        Err(e) => format!(" — not copied: {e}"),
    }
}

/// Add the faces that draw the visible glyphs of `cs` (table cells and footnotes included) to
/// `out`, each once.
fn drawn_faces(cs: &ComposedStory, depth: usize, out: &mut Vec<FaceRef>) {
    // Cells and footnotes hold composed stories of their own; tables nest only so deep.
    if depth > 16 {
        return;
    }
    for f in &cs.frames {
        for g in f.lines.iter().flat_map(|l| &l.glyphs).filter(|g| g.visible) {
            if !out.iter().any(|o| o.id() == g.face.id()) {
                out.push(g.face);
            }
        }
        for c in f.tables.iter().flat_map(|t| &t.cells) {
            drawn_faces(&c.text, depth + 1, out);
        }
        for n in &f.notes {
            drawn_faces(&n.text, depth + 1, out);
        }
    }
}

fn package(s: &mut Session, p: &Value) -> Result<Value> {
    let dir = PathBuf::from(str_param(p, "dir").ok_or_else(|| bad("file.package", "missing `dir`"))?);
    let st = s.doc()?;
    let root = OutputRoot::new(dir.clone())?;
    let display_title = st.doc.title.clone();
    let title = safe_component(&display_title, "Untitled");
    let fonts = s.execute("font.list", &json!({}))?;
    let links = s.execute("links.list", &json!({}))?;
    let pre = s.execute("preflight.run", &json!({})).unwrap_or(Value::Null);
    let d = s.doc()?.doc.clone();
    let links_dir = root.child("Links")?;
    let (packed, planned) = plan_links(&d, &links_dir, None);
    // Prepare required exports before any output is replaced. A refused native save or PDF
    // export leaves prior package files intact. This is per-file atomic output, not a
    // transaction across the entire folder if a later storage operation fails.
    let doc_path = package_output_path(&dir, &title, pagenor_doc::file_type::EXTENSION)?;
    let native_bytes = super::to_bytes(&packed)?;
    let idml = p.get("idml").and_then(Value::as_bool).unwrap_or(true).then(|| pagenor_idml::export_idml(&packed));
    let pdf = if p.get("pdf").and_then(Value::as_bool).unwrap_or(false) {
        Some(pagenor_pdf::export_pdf(&packed, &s.cache, &Default::default()).map_err(|e| EngineError::Other(e.to_string()))?)
    } else {
        None
    };
    let idml_path = package_output_path(&dir, &title, "idml")?;
    let pdf_path = package_output_path(&dir, &title, "pdf")?;
    let report_path = root.child("Instructions.txt")?;
    root.validate_directory(&links_dir)?;
    for path in std::iter::once(&doc_path)
        .chain(std::iter::once(&report_path))
        .chain(idml.as_ref().map(|_| &idml_path))
        .chain(pdf.as_ref().map(|_| &pdf_path))
        .chain(planned.iter().map(|(_, path)| path))
    {
        root.validate_file_entry(path)?;
    }
    write_planned_links(&d, &root, &planned)?;
    root.write(&doc_path, &native_bytes)?;
    let mut files = vec![doc_path];
    files.extend(planned.into_iter().map(|(_, path)| path));
    // The font files, and what the report says about each font.
    let db = pagenor_fonts::FontDb::global().scoped(d.font_scope);
    let fonts_dir = root.child(pagenor_fonts::DOCUMENT_FONTS_FOLDER)?;
    let (mut copied, mut taken) = (Vec::new(), Vec::new());
    let (mut font_lines, mut named) = (Vec::new(), Vec::new());
    for f in fonts.as_array().into_iter().flatten() {
        let (family, style) = (f["family"].as_str().unwrap_or(""), f["style"].as_str().unwrap_or(""));
        let missing = f["missing"].as_bool().unwrap_or(false) || f["styleMissing"].as_bool().unwrap_or(false);
        // A missing font's text is drawn in its substitute, which isn't a fallback font.
        let face = db.face(family, style);
        named.push(face.id());
        let note = if missing { " — MISSING".to_string() } else { copy_font(&root, &face, &fonts_dir, &mut copied, &mut taken, &mut files) };
        font_lines.push(format!("  {family} {style}{note}\n"));
    }
    // Fallback fonts: the faces that draw characters the named fonts lack.
    let mut drawn = Vec::new();
    for sid in d.stories.keys() {
        drawn_faces(&s.cache.get(&d, *sid, None), 0, &mut drawn);
    }
    drawn.retain(|f| !named.contains(&f.id()));
    drawn.sort_by(|a, b| (&a.family, &a.style).cmp(&(&b.family, &b.style)));
    let fallback_lines: Vec<String> = drawn
        .iter()
        .map(|f| format!("  {} {}{}\n", f.family, f.style, copy_font(&root, f, &fonts_dir, &mut copied, &mut taken, &mut files)))
        .collect();
    if let Some(bytes) = idml {
        root.write(&idml_path, &bytes)?;
        files.push(idml_path);
    }
    if let Some(bytes) = pdf {
        root.write(&pdf_path, &bytes)?;
        files.push(pdf_path);
    }
    // The report: fonts (missing first), links, preflight, the user's instructions.
    let mut r = format!("Package report: {display_title}\n\n");
    if let Some(t) = str_param(p, "instructions").filter(|t| !t.trim().is_empty()) {
        r += &format!("Instructions\n{t}\n\n");
    }
    r += "Fonts\n";
    r += &font_lines.concat();
    if !fallback_lines.is_empty() {
        r += "\nFallback fonts (for characters the fonts above lack)\n";
        r += &fallback_lines.concat();
    }
    r += "\nLinks\n";
    for l in links.as_array().into_iter().flatten() {
        r += &format!("  {} ({})\n", l["name"].as_str().unwrap_or(""), l["status"].as_str().unwrap_or(""));
    }
    if let Some(issues) = pre.get("issues").and_then(Value::as_array) {
        r += &format!("\nPreflight: {} issue(s)\n", issues.len());
        for i in issues {
            r += &format!("  {}\n", i.get("message").and_then(Value::as_str).unwrap_or(""));
        }
    }
    let report = report_path;
    root.write(&report, r.as_bytes())?;
    files.push(report);
    Ok(json!({"dir": dir.to_string_lossy(), "files": files.iter().map(|f| f.to_string_lossy().to_string()).collect::<Vec<_>>(), "report": r}))
}

fn copy_links(s: &mut Session, p: &Value) -> Result<Value> {
    let dir = PathBuf::from(str_param(p, "dir").ok_or_else(|| bad("links.copyTo", "missing `dir`"))?);
    let only: Option<Vec<AssetId>> = p.get("assets").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_u64).map(AssetId).collect());
    let d = s.doc()?.doc.clone();
    let root = OutputRoot::new(dir.clone())?;
    let (relinked, written) = write_links(&d, &root, &dir, only.as_deref())?;
    s.edit(|doc, _| {
        doc.assets = relinked.assets;
        Ok(())
    })?;
    Ok(json!({"copied": written.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocated_packages_rebind_embedded_image_links_on_open() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-relocation-{}-{nonce}", std::process::id()));
        let original = dir.join("original");
        let moved = dir.join("moved");
        let png = pagenor_render::Rendered { width: 4, height: 3, pixels: [30, 90, 180, 255].repeat(12) }.to_png();
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "OwnedPackage", "width": 72, "height": 72})).unwrap();
        s.execute("file.place", &json!({"base64": super::super::base64_encode(&png), "name": "owned.png", "x": 5, "y": 5, "width": 16})).unwrap();
        s.execute("file.package", &json!({"dir": original.to_string_lossy(), "idml": true, "pdf": false})).unwrap();
        std::fs::rename(&original, &moved).unwrap();
        let expected = std::path::absolute(moved.join("Links").join("owned.png")).unwrap();
        for (command, extension) in [("file.open", "pgnr"), ("file.openIdml", "idml")] {
            let mut opened = Session::new();
            opened.execute(command, &json!({"path": moved.join(format!("OwnedPackage.{extension}")).to_string_lossy()})).unwrap();
            let asset = opened.doc().unwrap().doc.assets.values().next().unwrap();
            assert_eq!(*asset.data, png, "{extension}: preserve embedded pixels");
            assert_eq!(asset.link.as_deref(), expected.to_str(), "{extension}: use the relocated Links file");
            let links = opened.execute("links.list", &json!({})).unwrap();
            assert_eq!(links[0]["status"], "ok", "{extension}");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn package_writes_document_links_and_report() {
        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let svg = dir.join("logo.svg");
        std::fs::write(&svg, r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40"><rect width="40" height="40"/></svg>"##).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("file.place", &json!({"path": svg.to_string_lossy(), "x": 100, "y": 100})).unwrap();
        s.execute("frame.create", &json!({"rect": [72, 300, 400, 400], "content": "text", "text": "Hello"})).unwrap();
        let out = dir.join("Pkg");
        let r = s.execute("file.package", &json!({"dir": out.to_string_lossy(), "pdf": true, "instructions": "Print on matte."})).unwrap();
        let files: Vec<String> = r["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap().to_string()).collect();
        assert!(files.iter().any(|f| f.ends_with(".pgnr")));
        assert!(files.iter().any(|f| f.ends_with(".idml")));
        assert!(files.iter().any(|f| f.ends_with(".pdf")));
        assert!(out.join("Links").join("logo.svg").exists());
        let report = std::fs::read_to_string(out.join("Instructions.txt")).unwrap();
        assert!(report.contains("Print on matte.") && report.contains("logo.svg"), "{report}");
        assert!(report.contains("Preflight:"), "{report}");
        // The packaged document links to its Links folder.
        let doc_file = files.iter().find(|f| f.ends_with(".pgnr")).unwrap();
        let packed = super::super::from_bytes(&std::fs::read(doc_file).unwrap()).unwrap();
        let link = packed.assets.values().next().unwrap().link.clone().unwrap();
        assert!(link.contains("Links"), "{link}");
        // Copy Links To relinks the open document.
        let r = s.execute("links.copyTo", &json!({"dir": dir.join("Copies").to_string_lossy()})).unwrap();
        assert_eq!(r["copied"], 1);
        assert!(s.doc().unwrap().doc.assets.values().next().unwrap().link.as_deref().unwrap().contains("Copies"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_copies_the_fonts_their_licences_allow() {
        use pagenor_fonts::DOCUMENT_FONTS_FOLDER;
        use pagenor_fonts::testing::{font_with, with_fs_type};
        const FREE: &str = "DocFont Package Free";
        const RESTRICTED: &str = "DocFont Package Restricted";
        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-fonts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let fonts = dir.join("src").join(DOCUMENT_FONTS_FOLDER);
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::write(fonts.join("free.ttf"), font_with(FREE, &['F']).unwrap()).unwrap();
        std::fs::write(fonts.join("restricted.otf"), with_fs_type(font_with(RESTRICTED, &['R']).unwrap(), 0x0002).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(fonts.join("free.ttf"), std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        // Text in both fonts, and in the bundled default font.
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": "Free Restricted Default"})).unwrap();
        for (a, b, family) in [(0, 4, FREE), (5, 15, RESTRICTED)] {
            s.execute("text.select", &json!({"story": r["story"], "anchor": a, "focus": b})).unwrap();
            s.execute("type.char", &json!({"fontFamily": family, "fontStyle": "Regular"})).unwrap();
        }
        let src = dir.join("src").join("Fonts.pgnr");
        s.execute("file.save", &json!({"path": src.to_string_lossy()})).unwrap();
        let mut s = Session::new();
        assert_eq!(s.execute("file.open", &json!({"path": src.to_string_lossy()})).unwrap()["documentFonts"], 2);

        let out = dir.join("Pkg");
        let r = s.execute("file.package", &json!({"dir": out.to_string_lossy(), "idml": false})).unwrap();
        let packed_fonts = out.join(DOCUMENT_FONTS_FOLDER);
        assert!(packed_fonts.join("free.ttf").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(
                std::fs::metadata(packed_fonts.join("free.ttf")).unwrap().permissions().mode() & 0o777,
                std::fs::metadata(fonts.join("free.ttf")).unwrap().permissions().mode() & 0o777
            );
        }
        assert!(!packed_fonts.join("restricted.otf").exists(), "a restricted licence keeps the font out");
        assert!(r["files"].to_string().contains("free.ttf"), "{}", r["files"]);
        let report = r["report"].as_str().unwrap();
        assert!(report.contains(&format!("{RESTRICTED} Regular — not copied: its licence doesn't allow it")), "{report}");
        assert!(report.contains(&format!("{FREE} Regular\n")), "{report}");
        assert!(report.contains(&format!("{} Regular — included with Pagenor", pagenor_fonts::DEFAULT_FAMILY)), "{report}");
        // The packaged document opens with its fonts.
        let doc = r["files"][0].as_str().unwrap().to_string();
        assert_eq!(Session::new().execute("file.open", &json!({"path": doc})).unwrap()["documentFonts"], 1);
        // Packaging into the folder the fonts come from leaves them whole.
        let len = std::fs::metadata(fonts.join("free.ttf")).unwrap().len();
        s.execute("file.package", &json!({"dir": dir.join("src").to_string_lossy(), "idml": false})).unwrap();
        assert_eq!(std::fs::metadata(fonts.join("free.ttf")).unwrap().len(), len);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_copies_the_fallback_fonts_that_draw_text() {
        use pagenor_fonts::DOCUMENT_FONTS_FOLDER;
        use pagenor_fonts::testing::{font_with, with_fs_type};
        const NAMED: &str = "DocFont Fallback Named";
        const HELPER: &str = "DocFont Fallback Helper";
        const RESTRICTED: &str = "DocFont Fallback Restricted";
        // Characters no other font has: the named font lacks both, the document's other fonts
        // draw them.
        let (helped, restricted) = ('\u{F0A41}', '\u{F0A42}');
        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-fallback-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let fonts = dir.join("src").join(DOCUMENT_FONTS_FOLDER);
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::write(fonts.join("named.ttf"), font_with(NAMED, &['N']).unwrap()).unwrap();
        std::fs::write(fonts.join("helper.ttf"), font_with(HELPER, &[helped]).unwrap()).unwrap();
        std::fs::write(fonts.join("restricted.otf"), with_fs_type(font_with(RESTRICTED, &[restricted]).unwrap(), 0x0002).unwrap()).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("document.preferences", &json!({"glyphFallback": true})).unwrap();
        let text = format!("N{helped}{restricted}");
        let r = s.execute("frame.create", &json!({"rect": [72, 72, 400, 300], "content": "text", "text": text})).unwrap();
        s.execute("text.select", &json!({"story": r["story"], "anchor": 0, "focus": text.len()})).unwrap();
        s.execute("type.char", &json!({"fontFamily": NAMED, "fontStyle": "Regular"})).unwrap();
        let src = dir.join("src").join("Fallback.pgnr");
        s.execute("file.save", &json!({"path": src.to_string_lossy()})).unwrap();
        let mut s = Session::new();
        assert_eq!(s.execute("file.open", &json!({"path": src.to_string_lossy()})).unwrap()["documentFonts"], 3);

        let out = dir.join("Pkg");
        let r = s.execute("file.package", &json!({"dir": out.to_string_lossy(), "idml": false})).unwrap();
        let packed_fonts = out.join(DOCUMENT_FONTS_FOLDER);
        assert!(packed_fonts.join("named.ttf").exists());
        assert!(packed_fonts.join("helper.ttf").exists(), "the fallback font that draws text is copied");
        assert!(!packed_fonts.join("restricted.otf").exists(), "a restricted licence keeps a fallback font out too");
        let report = r["report"].as_str().unwrap();
        let fallback = report.split("Fallback fonts").nth(1).unwrap_or_else(|| panic!("{report}"));
        assert!(fallback.contains(&format!("{HELPER} Regular\n")), "{report}");
        assert!(fallback.contains(&format!("{RESTRICTED} Regular — not copied: its licence doesn't allow it")), "{report}");
        assert!(!fallback.contains(NAMED), "{report}");
        // The packaged document draws the same text with its fonts.
        let doc = r["files"][0].as_str().unwrap().to_string();
        assert_eq!(Session::new().execute("file.open", &json!({"path": doc})).unwrap()["documentFonts"], 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_uses_portable_names_inside_the_selected_directory() {
        assert_eq!(safe_component("Archive/Annual: Review", "Untitled"), "Annual_ Review");
        assert_eq!(safe_component(".", "Untitled"), "Untitled");
        assert_eq!(safe_component("CON", "Untitled"), "_CON");
        assert_eq!(safe_component("COM¹.txt", "Untitled"), "_COM¹.txt");
        assert_eq!(safe_component("LPT³", "Untitled"), "_LPT³");
        for name in ["COM¹", "COM²", "COM³", "LPT¹", "LPT²", "LPT³"] {
            assert!(is_reserved_windows_name(name), "{name}");
        }
        let mut taken = Vec::new();
        assert_eq!(unique_name("Logo.svg".into(), &mut taken), "Logo.svg");
        assert_eq!(unique_name("logo.svg".into(), &mut taken), "logo 2.svg");
        assert_eq!(unique_name("Résumé.pdf".into(), &mut taken), "Résumé.pdf");
        assert_eq!(unique_name("Re\u{301}sume\u{301}.pdf".into(), &mut taken), "Re\u{301}sume\u{301} 2.pdf");
        assert_eq!(unique_name("ΟΣ.txt".into(), &mut taken), "ΟΣ.txt");
        assert_eq!(unique_name("ος.txt".into(), &mut taken), "ος 2.txt");

        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-safe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Archive/Annual: Review"})).unwrap();
        let r = s.execute("file.package", &json!({"dir": dir.to_string_lossy(), "idml": false})).unwrap();
        let files = r["files"].as_array().unwrap();
        assert!(files.iter().all(|path| Path::new(path.as_str().unwrap()).starts_with(&dir)));
        assert!(dir.join("Annual_ Review.pgnr").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_overwrites_existing_regular_outputs() {
        let dir = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-overwrite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let document = dir.join("Annual.pgnr");
        std::fs::write(&document, b"previous package").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&document, std::fs::Permissions::from_mode(0o640)).unwrap();
        }

        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Annual"})).unwrap();
        s.execute("file.package", &json!({"dir": dir.to_string_lossy(), "idml": false})).unwrap();
        s.execute("file.package", &json!({"dir": dir.to_string_lossy(), "idml": false})).unwrap();
        assert!(super::super::from_bytes(&std::fs::read(&document).unwrap()).is_ok());
        assert!(!dir.join(pagenor_fonts::DOCUMENT_FONTS_FOLDER).exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(std::fs::metadata(&document).unwrap().permissions().mode() & 0o777, 0o640);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_accepts_a_relative_destination() {
        let dir = PathBuf::from(format!(".pagenor-package-relative-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Relative"})).unwrap();
        s.execute("file.package", &json!({"dir": dir.to_string_lossy(), "idml": false})).unwrap();
        assert!(dir.join("Relative.pgnr").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn package_refuses_redirected_destinations() {
        use std::os::unix::fs::symlink;

        let base = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-redirects-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let outside = base.join("separate");
        std::fs::create_dir_all(&outside).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Annual"})).unwrap();

        let redirected_root = base.join("redirected-package");
        symlink(&outside, &redirected_root).unwrap();
        assert!(s.execute("file.package", &json!({"dir": redirected_root.to_string_lossy(), "idml": false})).is_err());
        assert!(!outside.join("Annual.pgnr").exists());
        std::fs::remove_file(&redirected_root).unwrap();

        let redirected_parent = base.join("redirected-parent");
        symlink(&outside, &redirected_parent).unwrap();
        let nested_package = redirected_parent.join("nested").join("Package");
        assert!(s.execute("file.package", &json!({"dir": nested_package.to_string_lossy(), "idml": false})).is_err());
        assert!(!outside.join("nested").exists());
        std::fs::create_dir_all(outside.join("nested/Package")).unwrap();
        assert!(s.execute("file.package", &json!({"dir": nested_package.to_string_lossy(), "idml": false})).is_err());
        assert!(!outside.join("nested/Package/Annual.pgnr").exists(), "existing descendants cannot bypass parent checks");
        std::fs::remove_file(&redirected_parent).unwrap();

        let package = base.join("Package");
        std::fs::create_dir_all(&package).unwrap();
        symlink(&outside, package.join("Links")).unwrap();
        assert!(s.execute("file.package", &json!({"dir": package.to_string_lossy(), "idml": false})).is_err());
        assert!(!outside.join("Annual.pgnr").exists());
        std::fs::remove_file(package.join("Links")).unwrap();
        std::fs::create_dir(package.join("Links")).unwrap();

        let kept = outside.join("kept.pgnr");
        std::fs::write(&kept, b"kept").unwrap();
        symlink(&kept, package.join("Annual.pgnr")).unwrap();
        assert!(s.execute("file.package", &json!({"dir": package.to_string_lossy(), "idml": false})).is_err());
        assert_eq!(std::fs::read(&kept).unwrap(), b"kept");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(windows)]
    #[test]
    fn package_refuses_windows_directory_redirects() {
        use std::os::windows::fs::symlink_dir;

        let base = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-windows-redirect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let outside = base.join("separate");
        std::fs::create_dir_all(&outside).unwrap();
        let redirected = base.join("Package");
        match symlink_dir(&outside, &redirected) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                let _ = std::fs::remove_dir_all(&base);
                return;
            }
            Err(e) => panic!("could not create directory redirect: {e}"),
        }
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Annual"})).unwrap();
        assert!(s.execute("file.package", &json!({"dir": redirected.to_string_lossy(), "idml": false})).is_err());
        assert!(!outside.join("Annual.pgnr").exists());
        std::fs::remove_dir(&redirected).unwrap();
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn package_title_cannot_replace_a_file_outside_output() {
        let base = std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("pagenor-package-title-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let outside = base.join("outside.pgnr");
        std::fs::write(&outside, b"keep").unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": base.join("outside").to_string_lossy()})).unwrap();
        s.execute("file.package", &json!({"dir": base.join("Pkg").to_string_lossy(), "idml": false})).unwrap();
        assert_eq!(std::fs::read(&outside).unwrap(), b"keep");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn package_preparation_failure_preserves_previous_outputs() {
        use pagenor_doc::Asset;
        let base = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let out = base.path().join("Pkg");
        std::fs::create_dir_all(out.join("Links")).unwrap();
        std::fs::write(out.join("Annual.pgnr"), b"old document").unwrap();
        std::fs::write(out.join("Links/asset-0"), b"old link").unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Annual"})).unwrap();
        let d = Arc::make_mut(&mut s.doc_mut().unwrap().doc);
        for i in 0..4094 {
            let id = AssetId(d.alloc());
            d.assets.insert(
                id,
                Arc::new(Asset {
                    id,
                    page: 0,
                    name: format!("asset-{i}"),
                    mime: "application/octet-stream".into(),
                    link: None,
                    data: Arc::new(vec![1]),
                    pixels: None,
                }),
            );
        }
        let before = s.doc().unwrap().doc.clone();
        assert!(s.execute("file.package", &json!({"dir": out.to_string_lossy(), "idml": false})).is_err());
        assert_eq!(std::fs::read(out.join("Annual.pgnr")).unwrap(), b"old document");
        assert_eq!(std::fs::read(out.join("Links/asset-0")).unwrap(), b"old link");
        assert_eq!(std::fs::read_dir(out.join("Links")).unwrap().count(), 1);
        assert!(Arc::ptr_eq(&s.doc().unwrap().doc, &before));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_font_copy_keeps_existing_file_and_removes_staging_file() {
        let base = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let root = OutputRoot::new(base.path().to_path_buf()).unwrap();
        let target = base.path().join("font.ttf");
        std::fs::write(&target, b"keep font").unwrap();
        // A directory cannot supply font bytes; failure must not truncate the destination.
        assert!(root.copy(base.path(), &target).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"keep font");
        assert_eq!(std::fs::read_dir(base.path()).unwrap().count(), 1);
    }

    #[test]
    fn portable_names_leave_room_for_numbering_and_extension() {
        let mut taken = Vec::new();
        let long = format!("{}.svg", "தமிழ்".repeat(80));
        for _ in 0..12 {
            let name = link_name(&long, AssetId(1), &mut taken);
            assert!(name.len() <= MAX_OUTPUT_COMPONENT_BYTES);
            assert!(name.ends_with(".svg"));
            assert_eq!(Path::new(&name).components().count(), 1);
        }
        assert_eq!(safe_component(r"C:\Fonts\NUL.ttf", "font"), "_NUL.ttf");
        assert_eq!(safe_component("name... ", "asset"), "name");
    }

    #[cfg(unix)]
    #[test]
    fn report_redirect_is_refused_before_replacing_the_document() {
        use std::os::unix::fs::symlink;
        let base = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let out = base.path().join("Pkg");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("Annual.pgnr"), b"keep document").unwrap();
        let outside = base.path().join("outside.txt");
        std::fs::write(&outside, b"keep report").unwrap();
        symlink(&outside, out.join("Instructions.txt")).unwrap();
        let mut s = Session::new();
        s.execute("file.new", &json!({"title": "Annual"})).unwrap();
        assert!(s.execute("file.package", &json!({"dir": out.to_string_lossy(), "idml": false})).is_err());
        assert_eq!(std::fs::read(out.join("Annual.pgnr")).unwrap(), b"keep document");
        assert_eq!(std::fs::read(&outside).unwrap(), b"keep report");
    }
}
