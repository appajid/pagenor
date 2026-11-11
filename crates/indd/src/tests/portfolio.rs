use super::*;

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn layer_name(name: &str, internal: bool) -> Vec<u8> {
    let mut data = vec![u8::from(internal), 0, 0];
    data.extend_from_slice(&(0x4000 | name.len() as u16).to_le_bytes());
    data.extend_from_slice(name.as_bytes());
    data
}

#[test]
fn layer_stack_keeps_back_to_front_order_and_empty_user_layers() {
    let mut s = sample();
    s.add(1, 0xE01, object(&[(0x501, words(&[1, 20])), (0x301, words(&[4, 89, 10, 90, 91]))]));
    s.add(89, 0x302, object(&[(0x304, layer_name("Internal_pages_layer_name", true))]));
    s.add(90, 0x302, object(&[(0x304, layer_name("Title", false))]));
    s.add(91, 0x302, object(&[(0x304, layer_name("Page Number", false))]));
    s.add(20, 0x501, object(&[(0x503, words(&[20, 0, 2, 21, 92]))]));
    s.add(21, 0x301, object(&[(0x303, words(&[20, 20, 2, 22, 30])), (0x302, [words(&[10]), vec![0, 0]].concat())]));
    s.add(92, 0x301, object(&[(0x303, words(&[20, 20, 1, 40])), (0x302, [words(&[90]), vec![0, 0]].concat())]));
    let bytes = s.build();
    let c = Container::parse(&bytes).unwrap();
    let doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    assert_eq!(doc.layers.iter().map(|(u, _)| *u).collect::<Vec<_>>(), [10, 90, 91]);
    assert_eq!(doc.spreads[0].items.iter().map(|i| i.layer).collect::<Vec<_>>(), [Some(10), Some(90)]);
    let xml = unzip(&to_idml(&bytes).unwrap());
    let design = &xml["designmap.xml"];
    let positions: Vec<_> = [10, 90, 91].iter().map(|u| design.find(&format!(r#"<Layer Self="u{u}""#)).unwrap()).collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]));
    assert!(!design.contains("Internal_pages_layer_name"));
}

#[test]
fn raster_image_children_keep_proxy_link_bounds_and_transform() {
    let mut s = sample();
    s.add(30, 0x6201, object(&[(0x162B, rect_path(0.0, 0.0, 10.0, 15.0)), hierarchy(21, &[83])]));
    let xf: Vec<u8> = [0.5f64, 0.0, 0.0, 0.5, 2.0, 3.0].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bounds: Vec<u8> = [0.0f64, 0.0, 20.0, 30.0].iter().flat_map(|v| v.to_le_bytes()).collect();
    s.add(80, 0x129, b"\xff\xd8synthetic JPEG proxy".to_vec());
    s.add(81, 0x8C41, object(&[(0x8C92, b"file:/old/Links/portrait%20one.jpg\0".to_vec())]));
    s.add(82, 0x8C42, object(&[(0x8C9B, words(&[0, 0x101, 81]))]));
    s.add(84, 0x1708, object(&[(0x119, words(&[80]))]));
    s.add(83, 0x1702, object(&[(0x8CBC, words(&[0x101, 1, 82])), (0x170D, words(&[84])), (0x1633, bounds), (0x151, xf)]));
    let bytes = s.build();
    let c = Container::parse(&bytes).unwrap();
    let doc = crate::model::Builder::new(&c).unwrap().build().unwrap();
    let it = &doc.spreads[0].items[0];
    assert_eq!(it.kind, crate::model::Kind::Graphic);
    let graphic = it.graphic.as_ref().unwrap();
    assert_eq!(graphic.bounds, Some([0.0, 0.0, 20.0, 30.0]));
    assert_eq!(graphic.transform, [0.5, 0.0, 0.0, 0.5, 2.0, 3.0]);
    assert_eq!(graphic.proxy.as_deref(), Some(&b"\xff\xd8synthetic JPEG proxy"[..]));
    let xml = unzip(&to_idml(&bytes).unwrap());
    let spread = &xml["Spreads/Spread_u20.xml"];
    assert!(spread.contains(r#"<Image Self="u83" ItemTransform="0.5 0 0 0.5 2 3""#));
    assert!(spread.contains(r#"LinkResourceURI="file:/old/Links/portrait%20one.jpg""#));
    assert!(spread.contains("<Contents><![CDATA["));
}

#[test]
fn multi_column_preferences_preserve_count_gutter_and_bound_corrupt_values() {
    for (columns, gutter, expected_columns, expected_gutter) in [(3u32, 5.5f64, 3, "5.5"), (u32::MAX, f64::NAN, 1, "12")] {
        let mut s = sample();
        let prefs = [columns.to_le_bytes().to_vec(), gutter.to_le_bytes().to_vec()].concat();
        s.add(41, 0x263, object(&[hierarchy(40, &[42]), (0x2D1, prefs)]));
        let xml = unzip(&to_idml(&s.build()).unwrap());
        let spread = &xml["Spreads/Spread_u20.xml"];
        assert!(spread.contains(&format!(r#"TextColumnCount="{expected_columns}" TextColumnGutter="{expected_gutter}""#)));
    }
}
