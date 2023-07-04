pub mod svg;
use base64::Engine;
pub use steks_common::prelude::*;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::tiny_skia::Transform;
use resvg::usvg::{AspectRatio, NodeExt, NonZeroRect, Tree, TreeParsing, ViewBox};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

const RESOLUTION: u32 = 1024;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;

pub(crate) async fn my_handler(
    e: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));

    let game = e
        .payload
        .query_string_parameters
        .iter()
        .filter(|x| x.0.eq_ignore_ascii_case("game"))
        .map(|x| x.1)
        .next()
        .unwrap_or_else(|| "myriad123");

    let data = draw_image(game);

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Binary(data)),
        is_base64_encoded: true,
    };

    Ok(resp)
}

fn make_svg_from_data(data: &str) -> String {
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(data) else {return "".to_string();};

    let shapes = decode_shapes(&bytes);

    let svg = svg::create_svg(shapes.into_iter());
    svg
}

fn draw_image(game: &str) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();
    let svg_data = make_svg_from_data(game);

    let mut game_tree = match Tree::from_data(&svg_data.as_bytes(), &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let logo_bytes = include_bytes!("logo_monochrome.svg");

    let logo_tree = match Tree::from_data(logo_bytes, &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let bbox = game_tree
        .root
        .calculate_bbox()
        .unwrap_or(resvg::usvg::Rect::from_xywh(0., 0., WIDTH as f32, HEIGHT as f32).unwrap());

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(RESOLUTION, RESOLUTION).expect("Could not create pixmap");

    let [r,g,b,a] = steks_common::color::BACKGROUND_COLOR.as_rgba_u32().to_le_bytes();
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(
        r,g,b,a
    ));

    const SPACE_RATIO: f32 = 1.1;

    let length_to_use = (bbox.width().max(bbox.height())) * SPACE_RATIO;

    game_tree.view_box = ViewBox {
        rect: NonZeroRect::from_xywh(
            bbox.x() - ((length_to_use - bbox.width()) * 0.75),
            bbox.y() - ((length_to_use - bbox.height()) * 0.75),
            length_to_use,
            length_to_use,
        )
        .unwrap(),
        aspect: AspectRatio {
            defer: false,
            slice: true,
            align: resvg::usvg::Align::XMidYMid,
        },
    };

    let game_scale = (HEIGHT as f32 / game_tree.size.height() as f32)
        .min(WIDTH as f32 / game_tree.size.width() as f32);

    resvg::Tree::render(
        &resvg::Tree::from_usvg(&game_tree),
        Transform::from_scale(game_scale, game_scale),
        &mut pixmap.as_mut(),
    );

    let logo_scale = WIDTH as f32 / logo_tree.size.width() as f32;
    resvg::Tree::render(
        &resvg::Tree::from_usvg(&logo_tree),
        Transform::from_scale(logo_scale, logo_scale),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().expect("Could not encode png")
}

#[cfg(test)]
mod tests {
    use crate::{draw_image, make_svg_from_data};
    use std::hash::{Hash, Hasher};
    const TEST_DATA: &'static str = "EjCDVGhLHgogf_9ccQAUAHrTZvd3DgB9g332dgkAel51we8MAHuiiKV3";

    #[test]
    fn generate_png_test() {
        let data = draw_image(TEST_DATA);
        let len = data.len();
        std::fs::write("parse_test.png", data.as_slice()).unwrap();

        assert!(len < 300000, "Image is too big - {len} bytes");
        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);
    }

    #[test]
    fn generate_svg_test() {
        let svg: String = make_svg_from_data(TEST_DATA);
        std::fs::write("og_example.svg", svg).unwrap();
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null");
        let len = data.len();
        std::fs::write("unknown.png", data.as_slice()).unwrap();

        assert!(len < 300000, "Image is too big - {len} bytes");
        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);
    }

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}
