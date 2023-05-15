pub mod color;
pub mod encoding;
pub mod fixed_shape;
pub mod game_shape;
pub mod screenshots;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use base64::Engine;
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::usvg::{Tree, TreeParsing, Color};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    x: f32,
    y: f32,
}
impl Vec2 {
    fn new(x: f32, y: f32) -> Vec2 {
        Self { x, y }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

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
        is_base64_encoded: Some(true),
    };

    Ok(resp)
}

fn make_svg_text(data: &str) -> String {
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(data) else {return "".to_string();};

    let shapes = encoding::decode_shapes(&bytes);

    let svg_text = screenshots::create_svg(shapes.into_iter());
    svg_text
}

fn draw_image(game: &str) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();
    let svg_data = make_svg_text(game);

    //println!("{svg_data}");

    let tree = match Tree::from_data(&svg_data.as_bytes(), &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new(WIDTH, HEIGHT).unwrap();

    let bc = color::background_color();
     pixmap.fill(resvg::tiny_skia::Color::from_rgba8(bc.red, bc.green, bc.blue, 255) );

    use resvg::FitTo;
    resvg::render(
        &tree,
        FitTo::Size(WIDTH, HEIGHT),
        resvg::tiny_skia::Transform::from_translate((WIDTH as f32) *0.0, (HEIGHT as f32) * 0.0),
        pixmap.as_mut(),
    )
    .unwrap();

    pixmap.encode_png().unwrap()
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::{draw_image, make_svg_text};

    #[test]
    fn test_svg() {
        let svg: String = make_svg_text("BczJ2_d4CkEj2myzFs6m2xo7Bs5o2XJ2JkY32IvuCsVN1vm0ClBJ1XruBs4y1Q95Hsn40Ht5KNBuvOQ9IF_L32S0Ft8p34g8BsieUHA9");
        std::fs::write("og_example.svg", svg).unwrap();
    }

    #[test]
    fn parse_test() {
        let data = draw_image("BczJ2_d4CkEj2myzFs6m2xo7Bs5o2XJ2JkY32IvuCsVN1vm0ClBJ1XruBs4y1Q95Hsn40Ht5KNBuvOQ9IF_L32S0Ft8p34g8BsieUHA9");
        std::fs::write("parse_test.png", data).unwrap();
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null");
        std::fs::write("unknown.png", data).unwrap();
    }
}
