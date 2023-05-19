pub mod color;
pub mod encoding;
pub mod fixed_shape;
pub mod game_shape;
pub mod point;
pub mod svg;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use base64::Engine;
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::tiny_skia::Transform;
use resvg::usvg::{AspectRatio, NodeExt, PathBbox, Tree, TreeParsing, ViewBox};
use resvg::FitTo;

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

fn make_svg_from_data(data: &str) -> String {
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(data) else {return "".to_string();};

    let shapes = encoding::decode_shapes(&bytes);

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
        .unwrap_or(PathBbox::new(0., 0., WIDTH as f64, HEIGHT as f64).unwrap());

    let bbox_size = bbox.to_rect().unwrap().size().to_screen_size(); // tree.size.to_screen_size();

    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(RESOLUTION, RESOLUTION).expect("Could not create pixmap");

    let bc = color::background_color();
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(
        bc.red, bc.green, bc.blue, 255,
    ));

    let length_to_use = (bbox_size.width().max(bbox_size.height()) as f64) * 1.1;

    game_tree.view_box = ViewBox {
        rect: resvg::usvg::Rect::new(
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

    resvg::render(
        &game_tree,
        FitTo::Size(RESOLUTION, RESOLUTION),
        Default::default(),
        pixmap.as_mut(),
    )
    .expect("Could not render game svg");

    resvg::render(
        &logo_tree,
        FitTo::Size(RESOLUTION, RESOLUTION),
        Transform::from_translate(0.0, 200.0),
        pixmap.as_mut(),
    )
    .expect("Could not render logo svg");

    pixmap.encode_png().expect("Could not encode png")
}

#[cfg(test)]
mod tests {

    use crate::{draw_image, make_svg_from_data};
    const TEST_DATA: &'static str =
        "EHWEbQIBEIBBdrntBIM1ZLTwA38jYMQeHoeAaM12CHQ3ctMBBoijcwmyCIfGbUV2EHRSZmp5AoA9fGKT";

    #[test]
    fn generate_png_test() {
        let data = draw_image(TEST_DATA);
        std::fs::write("parse_test.png", data).unwrap();
    }

    #[test]
    fn generate_svg_test() {
        let svg: String = make_svg_from_data(TEST_DATA);
        std::fs::write("og_example.svg", svg).unwrap();
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null");
        std::fs::write("unknown.png", data).unwrap();
    }
}
