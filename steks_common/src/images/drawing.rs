use crate::images::prelude::*;
pub use crate::prelude::*;
use resvg::tiny_skia::Transform;
use resvg::usvg::{AspectRatio, NodeExt, NonZeroRect, Tree, TreeParsing, ViewBox};

pub fn make_svg_from_bytes(bytes: &[u8], dimensions: Dimensions)-> String{
    let shapes = decode_shapes(&bytes);
    let svg = create_svg(shapes.into_iter(), dimensions);
    svg
}



pub fn draw_image(bytes: &[u8], overlay_chooser: &OverlayChooser, dimensions: Dimensions) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();
    let svg_data = make_svg_from_bytes(bytes, dimensions);

    let mut game_tree = match Tree::from_data(&svg_data.as_bytes(), &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let bbox = game_tree.root.calculate_bbox().unwrap_or(
        resvg::usvg::Rect::from_xywh(0., 0., dimensions.width as f32, dimensions.height as f32)
            .unwrap(),
    );

    let mut pixmap = resvg::tiny_skia::Pixmap::new(dimensions.width, dimensions.height)
        .expect("Could not create pixmap");

    let [r, g, b, a] = crate::color::BACKGROUND_COLOR.as_rgba_u32().to_le_bytes();
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(r, g, b, a));

    let h_scale = bbox.width() / dimensions.width as f32;
    let v_scale = bbox.height() / dimensions.height as f32;
    let (ratio, overlay) = overlay_chooser.choose_scale_and_overlay(h_scale, v_scale);

    //let ratio_to_use = h_scale.max(v_scale) * SPACE_RATIO;

    let w = ratio * (dimensions.width as f32);
    let h = ratio * (dimensions.height as f32);

    //let length_to_use = (bbox.width().max(bbox.height())) * SPACE_RATIO;

    game_tree.view_box = ViewBox {
        rect: NonZeroRect::from_xywh(
            bbox.x() - ((w - bbox.width()) * 0.5),
            bbox.y() - ((h - bbox.height()) * 0.5),
            w,
            h,
        )
        .unwrap(),
        aspect: AspectRatio {
            defer: false,
            slice: true,
            align: resvg::usvg::Align::XMidYMid,
        },
    };

    let game_scale = (dimensions.height as f32 / game_tree.size.height() as f32)
        .min(dimensions.width as f32 / game_tree.size.width() as f32);

    resvg::Tree::render(
        &resvg::Tree::from_usvg(&game_tree),
        Transform::from_scale(game_scale, game_scale),
        &mut pixmap.as_mut(),
    );

    if let Some(overlay) = overlay {
        overlay.include(&mut pixmap, &opt, dimensions);
    }

    pixmap.encode_png().expect("Could not encode png")
}
