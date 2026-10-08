//! Render branding with the workspace SVG renderer. SVG output converts text into paths.

pub fn run(args: &[&str]) -> Result<(), String> {
    let [input, output, side] = args else { return Err("usage: svg <in.svg> <out.png|out.svg> <max-side>".into()) };
    let side: u32 = side.parse().map_err(|e| format!("invalid size: {e}"))?;
    if !(1..=4096).contains(&side) {
        return Err("size must be 1..=4096".into());
    }
    let bytes = std::fs::read(input).map_err(|e| format!("read {input}: {e}"))?;
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(&bytes, &options).map_err(|e| format!("parse {input}: {e}"))?;
    if std::path::Path::new(output).extension().is_some_and(|e| e == "svg") {
        return std::fs::write(output, tree.to_string(&resvg::usvg::WriteOptions::default())).map_err(|e| format!("write {output}: {e}"));
    }
    let size = tree.size();
    let scale = side as f32 / size.width().max(size.height());
    let width = (size.width() * scale).round().max(1.0) as u32;
    let height = (size.height() * scale).round().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or("cannot allocate icon pixmap")?;
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.save_png(output).map_err(|e| format!("write {output}: {e}"))
}
