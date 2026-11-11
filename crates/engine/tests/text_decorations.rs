use pagenor_compose::Cache;
use pagenor_doc::{CharAttrs, Document, ParaFormat, SpreadRef, build::NewDocument};
use pagenor_geom::{Affine, Rect};
use pagenor_render::{Placed, RenderOptions, Renderer};

/// An underline is one bar per stretch of text: a bar per glyph left lighter seams between the
/// letters where the anti-aliased edges met (at fractional positions).
#[test]
fn an_underline_has_no_seams_between_letters() {
    let mut d = Document::new(&NewDocument { width: 400.0, height: 120.0, facing_pages: false, ..Default::default() });
    let chars = CharAttrs {
        font_family: Some("Source Sans 3".into()),
        size: Some(48.0),
        fill: Some("[Black]".into()),
        underline: Some(true),
        underline_weight: Some(Some(6.0)),
        ..Default::default()
    };
    d.add_text_frame(SpreadRef::Doc(0), Rect::new(10.0, 10.0, 390.0, 110.0), d.default_layer(), "FERRUM", ParaFormat { chars, ..Default::default() })
        .unwrap();
    let mut renderer = Renderer::new();
    renderer.threads = 0;
    let placed = [Placed { spread: SpreadRef::Doc(0), xf: Affine::IDENTITY }];
    let options = RenderOptions { paper: false, background: Some([255; 4]), greek_below_px: 0.0, ..Default::default() };
    // A fractional scale puts the letters' edges between pixels.
    let (w, h) = (543u32, 163u32);
    let img = renderer.render(&d, &Cache::new(), &placed, w, h, Affine::scale(1.3579), &options);
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], img.pixels[i + 3]]
    };
    // The underline's middle row: the row with the longest run of dark pixels ([Black] shows as
    // about 52 grey), then nothing lighter than its darkest pixel along it.
    let row = (0..h).max_by_key(|&y| (0..w).filter(|&x| px(x, y)[0] < 110).count()).unwrap();
    let dark: Vec<u32> = (0..w).filter(|&x| px(x, row)[0] < 110).collect();
    let (x0, x1) = (dark[0], *dark.last().unwrap());
    assert!(x1 - x0 > 150, "the underline spans the word: {x0}..{x1}");
    // The bar's own anti-aliased ends aside.
    let inside = x0 + 2..=x1 - 2;
    let ink = inside.clone().map(|x| px(x, row)[0]).min().unwrap();
    let light: Vec<(u32, u8)> = inside.map(|x| (x, px(x, row)[0])).filter(|&(_, v)| v > ink + 12).collect();
    assert!(light.is_empty(), "seams in the underline at {light:?}");
}
