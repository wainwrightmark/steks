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
use resvg::usvg::{AspectRatio, NodeExt, PathBbox, Tree, TreeParsing, ViewBox};
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

    let mut tree = match Tree::from_data(&svg_data.as_bytes(), &opt) {
        Ok(tree) => tree,
        Err(e) => panic!("{e}"),
    };

    let bbox = tree
        .root
        .calculate_bbox()
        .unwrap_or(PathBbox::new(0., 0., WIDTH as f64, HEIGHT as f64).unwrap());

    let bbox_size = bbox.to_rect().unwrap().size().to_screen_size(); // tree.size.to_screen_size();

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(RESOLUTION, RESOLUTION).expect("Could not create pixmap");

    let bc = color::background_color();
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(
        bc.red, bc.green, bc.blue, 255,
    ));

    let bbox_longest = bbox_size.width().max(bbox_size.height()) as f64;

    tree.view_box = ViewBox {
        rect: resvg::usvg::Rect::new(
            bbox.x() - ((bbox_longest - bbox.width()) * 0.75),
            bbox.y() - ((bbox_longest - bbox.height()) * 0.5),
            bbox_longest,
            bbox_longest,
        )
        .unwrap(),
        aspect: AspectRatio {
            defer: false,
            slice: false,
            align: resvg::usvg::Align::None,
        },
    };

    use resvg::FitTo;
    resvg::render(
        &tree,
        FitTo::Size(RESOLUTION, RESOLUTION),
        Default::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    pixmap.encode_png().unwrap()
}

#[cfg(test)]
mod tests {

    use crate::{draw_image, make_svg_text};
    const TEST_DATA: &'static str = "Dnsqa4DSEnvicbiXE3pKZTkeCHf5d22gFnVke6DcBpdpRoC0";

    #[test]
    fn generate_png_test() {
        let data = draw_image(TEST_DATA);
        std::fs::write("parse_test.png", data).unwrap();
    }

    #[test]
    fn generate_svg_test() {
        let svg: String = make_svg_text(TEST_DATA);
        std::fs::write("og_example.svg", svg).unwrap();
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null");
        std::fs::write("unknown.png", data).unwrap();
    }
}
