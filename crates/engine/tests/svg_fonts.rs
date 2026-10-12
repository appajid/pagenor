//! Document-font scope must agree between screen rendering and vector SVG export.
use pagenor_compose::Cache;
use pagenor_doc::{Document, Item, ItemId, SpreadRef, build::NewDocument};
use pagenor_geom::Rect;
use pagenor_pdf::{PdfOptions, export_pdf_with_report};

pub(crate) fn extract_text(bytes: &[u8]) -> Vec<String> {
    use hayro_interpret::font::Glyph;
    use hayro_interpret::{
        BlendMode, ClipPath, Context, Device, GlyphDrawMode, Image, InterpreterCache, InterpreterSettings, Paint, PathDrawMode, SoftMask,
        interpret_page,
    };
    use hayro_syntax::Pdf;
    struct Ex(String);
    impl Device<'_> for Ex {
        fn set_soft_mask(&mut self, _: Option<SoftMask<'_>>) {}
        fn set_blend_mode(&mut self, _: BlendMode) {}
        fn draw_path(&mut self, _: &kurbo::BezPath, _: kurbo::Affine, _: &Paint<'_>, _: &PathDrawMode) {}
        fn push_clip_path(&mut self, _: &ClipPath) {}
        fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}
        fn draw_glyph(&mut self, g: &Glyph<'_>, _: kurbo::Affine, _: kurbo::Affine, _: &Paint<'_>, _: &GlyphDrawMode) {
            match g.as_unicode() {
                Some(hayro_cmap::BfString::Char(c)) => self.0.push(c),
                Some(hayro_cmap::BfString::String(s)) => self.0.push_str(&s),
                None => self.0.push('\u{FFFD}'),
            }
        }
        fn draw_image(&mut self, _: Image<'_, '_>, _: kurbo::Affine) {}
        fn pop_clip_path(&mut self) {}
        fn pop_transparency_group(&mut self) {}
    }
    let pdf = Pdf::new(bytes.to_vec()).expect("parse");
    let cache = InterpreterCache::new();
    pdf.pages()
        .iter()
        .map(|page| {
            let mut ctx =
                Context::new(kurbo::Affine::IDENTITY, kurbo::Rect::new(0.0, 0.0, 1.0, 1.0), &cache, pdf.xref(), InterpreterSettings::default());
            let mut ex = Ex(String::new());
            interpret_page(page, &mut ctx, &mut ex);
            ex.0
        })
        .collect()
}

fn place(d: &mut Document, data: Vec<u8>, px: (u32, u32)) {
    let asset = pagenor_doc::AssetId(d.alloc());
    let mime = pagenor_images::mime(&data).into();
    d.assets.insert(
        asset,
        std::sync::Arc::new(pagenor_doc::Asset {
            id: asset,
            name: "scan.tif".into(),
            mime,
            link: None,
            data: data.into(),
            pixels: Some(px),
            page: 0,
            ..Default::default()
        }),
    );
    let lid = d.default_layer();
    let id = ItemId(d.alloc());
    let mut it = Item::new(id, lid, pagenor_doc::Shape::Rectangle, pagenor_geom::shapes::rectangle(Rect::new(72.0, 72.0, 272.0, 172.0)));
    it.content = pagenor_doc::Content::Graphic(pagenor_doc::Graphic {
        asset,
        size: (px.0 as f64, px.1 as f64),
        xf: pagenor_geom::Affine::translate((72.0, 72.0)) * pagenor_geom::Affine::scale(50.0),
        auto_fit: pagenor_doc::Fitting::FillProportionally,
        fit_align: 4,
        crop: [0.0; 4],
        wrap: Default::default(),
    });
    d.insert_item(SpreadRef::Doc(0), it, None).unwrap();
}

#[test]
fn placed_svg_screen_and_pdf_use_the_document_font_scope() {
    let fonts = pagenor_fonts::FontDb::global();
    let root = std::env::temp_dir().join(format!("pagenor-svg-pdf-{}", std::process::id()));
    let mut pixels = Vec::new();
    for (name, glyph) in [("one", 'X'), ("two", 'W')] {
        std::fs::create_dir_all(root.join(name)).unwrap();
        std::fs::write(root.join(name).join("font.ttf"), pagenor_fonts::testing::font_with_glyph("Pagenor SVG PDF Scope", &['A'], glyph).unwrap())
            .unwrap();
        let mut d = Document::new(&NewDocument { width: 320.0, height: 240.0, facing_pages: false, ..Default::default() });
        d.font_scope = fonts.load_document_fonts(&root.join(name)).scope;
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text y="30" font-size="20" font-family="Pagenor SVG PDF Scope">A</text></svg>"#;
        place(&mut d, svg.to_vec(), (100, 40));
        let spread = std::sync::Arc::make_mut(&mut d.spreads[0]);
        for item in &mut spread.items {
            if let pagenor_doc::Content::Graphic(g) = &mut std::sync::Arc::make_mut(item).content {
                g.xf = pagenor_geom::Affine::translate((72.0, 72.0));
            }
        }
        let report = export_pdf_with_report(&d, &Cache::new(), &PdfOptions::default()).unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(extract_text(&report.bytes), ["A"]);
        let raster = pagenor_render::render_pdf_page(&report.bytes, 0, 640).unwrap();
        pixels.push(raster.data().to_vec());
        if let Some(folder) = std::env::var_os("PAGENOR_SVG_PROOF_DIR") {
            let folder = std::path::PathBuf::from(folder);
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join(format!("{name}.pdf")), &report.bytes).unwrap();
            let shown = pagenor_render::Renderer::new().render_page(&d, &Cache::new(), 0, 2.0, false, &Default::default()).unwrap();
            std::fs::write(folder.join(format!("{name}-canvas.png")), shown.to_png()).unwrap();
        }
        fonts.close_scope(d.font_scope);
    }
    assert_ne!(pixels[0], pixels[1], "same SVG family must embed the right document's glyphs");
    std::fs::remove_dir_all(root).unwrap();
}
