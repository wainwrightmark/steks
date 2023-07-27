pub mod svg;

use aws_lambda_events::query_map::QueryMap;
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


    let game = e
        .payload
        .query_string_parameters
        .iter()
        .filter(|x| x.0.eq_ignore_ascii_case("game"))
        .map(|x| x.1)
        .next()
        .unwrap_or_else(|| "");

    let result_type: ResultType = ResultType::from_query_map(&e.payload.query_string_parameters);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", result_type.content_type_header_value());

    let body = result_type.get_response_body(game);

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(body),
        is_base64_encoded: result_type.is_base64_encoded(),
    };

    Ok(resp)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ResultType {
    #[default]
    Default,
    NoOverlay,
    SVG,
    //TODO level yaml
}

impl ResultType {
    pub fn from_query_map(query_map: &QueryMap) -> Self {
        let r = query_map
            .iter()
            .filter(|x| x.0.eq_ignore_ascii_case("format"))
            .map(|x| x.1.to_ascii_lowercase())
            .next()
            .unwrap_or_default();


        match r.as_str() {
            "default" | "" => Self::Default,
            //spellchecker:disable-next-line
            "nooverlay" => Self::NoOverlay,
            "svg" => Self::SVG,
            _=> Self::Default
        }
    }

    pub fn is_base64_encoded(&self)-> bool{
        match self{
            ResultType::Default => true,
            ResultType::NoOverlay => true,
            ResultType::SVG => false,
        }
    }

    pub fn content_type_header_value(&self)-> HeaderValue{
        match self{
            ResultType::Default => HeaderValue::from_static("image/png"),
            ResultType::NoOverlay => HeaderValue::from_static("image/png"),
            ResultType::SVG => HeaderValue::from_static("image/svg+xml"),
        }
    }

    pub fn get_response_body(&self, game: &str) -> Body {
        match self {
            ResultType::Default => {
                let data = draw_image(game, true);
                Body::Binary(data)
            }
            ResultType::NoOverlay => {
                let data = draw_image(game, false);
                Body::Binary(data)
            }
            ResultType::SVG => {
                let data = make_svg_from_data(game);

                Body::Text(data)
            }
        }
    }
}

fn make_svg_from_data(data: &str) -> String {
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(data) else {return "".to_string();};

    let shapes = decode_shapes(&bytes);

    let svg = svg::create_svg(shapes.into_iter());
    svg
}

fn draw_image(game: &str, include_overlay: bool) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();
    let svg_data = make_svg_from_data(game);

    let mut game_tree = match Tree::from_data(&svg_data.as_bytes(), &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let bbox = game_tree
        .root
        .calculate_bbox()
        .unwrap_or(resvg::usvg::Rect::from_xywh(0., 0., WIDTH as f32, HEIGHT as f32).unwrap());

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(RESOLUTION, RESOLUTION).expect("Could not create pixmap");

    let [r, g, b, a] = steks_common::color::BACKGROUND_COLOR
        .as_rgba_u32()
        .to_le_bytes();
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(r, g, b, a));

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

    if include_overlay {
        let logo_bytes = include_bytes!("logo_monochrome.svg");

        let logo_tree = match Tree::from_data(logo_bytes, &opt) {
            Ok(tree) => tree,
            Err(e) => panic!("{e}"),
        };

        let logo_scale = WIDTH as f32 / logo_tree.size.width() as f32;
        resvg::Tree::render(
            &resvg::Tree::from_usvg(&logo_tree),
            Transform::from_scale(logo_scale, logo_scale),
            &mut pixmap.as_mut(),
        );
    }

    pixmap.encode_png().expect("Could not encode png")
}

#[cfg(test)]
mod tests {
    use steks_common::prelude::{choose_color, Location, ALL_SHAPES, SHAPE_SIZE};

    use crate::{draw_image, make_svg_from_data};
    use std::hash::{Hash, Hasher};

    // spell-checker: disable-next-line
    const TEST_DATA: &'static str = "CACFeXBT7wUAgklpIncOAHeZj7uyBQB0VZzfMw4Ae3iaYLUEAH8Cd_3tAwBzjIdNPQwAfgiHhssJAH2ipQ-1AyCDVF7QAAYgdf9__wASMHX_tCUAEjCHVIXsAA==";

    #[test]
    fn generate_png_test() {
        let data = draw_image(TEST_DATA, true);
        let len = data.len();
        std::fs::write("parse_test.png", data.as_slice()).unwrap();

        assert!(len < 300000, "Image is too big - {len} bytes");
        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);
    }

    #[test]
    fn generate_png_no_overlay() {
        let data = draw_image(TEST_DATA, false);
        let len = data.len();
        std::fs::write("parse_test_no_overlay.png", data.as_slice()).unwrap();

        assert!(len < 300000, "Image is too big - {len} bytes");
        let hash = calculate_hash(&data);
        insta::assert_debug_snapshot!(hash);
    }

    #[test]
    fn generate_svg_test() {
        let svg: String = make_svg_from_data(TEST_DATA);
        let hash = calculate_hash(&svg);
        std::fs::write("og_example.svg", svg).unwrap();


        insta::assert_debug_snapshot!(hash);
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null", true);
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

    #[test]
    pub fn all_shapes_svg() {
        let mut svg = String::new();

        svg.push_str(r#"<svg width="500" height="500" xmlns="http://www.w3.org/2000/svg">"#);

        for (index, shape) in ALL_SHAPES.iter().enumerate() {
            svg.push('\n');

            let x = ((index % 5) as f32 * 100.) + 50.;
            let y = ((index / 5) as f32 * 100.) + 50.;

            let location = Location::new(x, y, 0.0);

            let transform = location.svg_transform();

            svg.push_str(format!(r#"<g transform="{transform}">"#).as_str());

            svg.push('\n');
            let shape_svg = shape
                .body
                .as_svg(SHAPE_SIZE, Some(shape.fill().color), None);

            println!("{shape_svg}");
            svg.push_str(shape_svg.as_str());

            svg.push('\n');

            svg.push_str("</g>");
        }

        svg.push_str(r#"</svg>"#);

        let hash = calculate_hash(&svg);
        std::fs::write("all_shapes.svg", svg).unwrap();

        insta::assert_debug_snapshot!(hash);
    }

    #[test]
    pub fn all_colors_svg() {
        let mut svg = String::new();

        svg.push_str(r#"<svg width="500" height="1000" xmlns="http://www.w3.org/2000/svg">"#);

        for alt in [false, true] {
            for (index, shape) in ALL_SHAPES.iter().enumerate() {
                svg.push('\n');

                let x = ((index % 5) as f32 * 100.) + 50.;
                let y = ((index / 5) as f32 * 100.) + 50. + if alt { 500. } else { 0. };

                let location = Location::new(x, y, 0.0);

                let transform = location.svg_transform();

                svg.push_str(format!(r#"<g transform="{transform}">"#).as_str());

                svg.push('\n');

                let color = choose_color(index, alt);

                let shape_svg = shape.body.as_svg(SHAPE_SIZE, Some(color), None);

                println!("{shape_svg}");
                svg.push_str(shape_svg.as_str());

                svg.push('\n');

                svg.push_str("</g>");
            }
        }

        svg.push_str(r#"</svg>"#);

        let hash = calculate_hash(&svg);
        std::fs::write("all_colors.svg", svg).unwrap();

        insta::assert_debug_snapshot!(hash);
    }
}
