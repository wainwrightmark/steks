use std::fmt::Display;

use aws_lambda_events::query_map::QueryMap;
use base64::Engine;
pub use steks_common::images::prelude::*;
pub use steks_common::prelude::*;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};

use resvg::usvg::{fontdb, NodeKind, Tree, TreeTextToPath};

include!(concat!(env!("OUT_DIR"), "/level_stars.rs"));


#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

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

    let result_type: Command = Command::from_query_map(&e.payload.query_string_parameters);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", result_type.content_type_header_value());

    let dimensions = dimensions_from_query_map(&e.payload.query_string_parameters);

    let body = result_type.get_response_body(game, dimensions);

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
pub enum Command {
    #[default]
    Default,
    NoOverlay,
    SVG,
    //TODO level yaml
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Default => f.write_str("Default"),
            Command::NoOverlay => f.write_str("No_Overlay"),
            Command::SVG => f.write_str("SVG"),
        }
    }
}

impl Command {
    pub fn from_query_map(query_map: &QueryMap) -> Self {
        let s = query_map
            .iter()
            .filter(|x| x.0.eq_ignore_ascii_case("format"))
            .next()
            .unwrap_or_default();

        Self::from_str_ignore_case(s.1)
    }

    pub fn from_str_ignore_case(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "default" | "" => Self::Default,
            //spellchecker:disable-next-line
            "nooverlay" | "no_overlay" | "no-overlay" => Self::NoOverlay,
            "svg" => Self::SVG,
            _ => Self::Default,
        }
    }

    pub fn is_base64_encoded(&self) -> bool {
        match self {
            Command::Default => true,
            Command::NoOverlay => true,
            Command::SVG => false,
        }
    }

    pub fn content_type_header_value(&self) -> HeaderValue {
        match self {
            Command::Default => HeaderValue::from_static("image/png"),
            Command::NoOverlay => HeaderValue::from_static("image/png"),
            Command::SVG => HeaderValue::from_static("image/svg+xml"),
        }
    }

    pub fn get_response_body(&self, game: &str, dimensions: Dimensions) -> Body {
        let bytes = convert_game_to_bytes(game);
        let height_and_stars = HeightAndStars::from_bytes(bytes.as_slice());

        match self {
            Command::Default => {
                let data =
                    try_draw_image(&bytes, &text_overlay(), dimensions, height_and_stars).unwrap();
                Body::Binary(data)
            }
            Command::NoOverlay => {
                let data = try_draw_image(
                    &bytes,
                    &OverlayChooser::no_overlay(),
                    dimensions,
                    height_and_stars,
                )
                .unwrap();
                Body::Binary(data)
            }
            Command::SVG => {
                let data = make_svg_from_bytes(bytes.as_slice(), dimensions);

                Body::Text(data)
            }
        }
    }
}

pub fn dimensions_from_query_map(query_map: &aws_lambda_events::query_map::QueryMap) -> Dimensions {
    let mut width: u32 = 1024;
    let mut height: u32 = 1024;

    for (key, value) in query_map.iter() {
        if key.eq_ignore_ascii_case("width") {
            if let Ok(parsed_width) = value.parse::<u32>() {
                width = parsed_width;
            }
        }

        if key.eq_ignore_ascii_case("height") {
            if let Ok(parsed_height) = value.parse::<u32>() {
                height = parsed_height;
            }
        }
    }

    Dimensions { width, height }
}

fn text_overlay() -> OverlayChooser<HeightAndStars> {
    OverlayChooser {
        options: vec![TEXT_RIGHT_OVERLAY, TEXT_BOTTOM_OVERLAY],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HeightAndStars {
    height: f32,
    stars: Option<StarType>,
}

impl HeightAndStars {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let shapes = decode_shapes(&bytes);
        let shapes_vec = ShapesVec(shapes);
        let height = shapes_vec.calculate_tower_height();
        let hash = shapes_vec.hash();

        let stars = get_level_stars(hash).map(|x|x.get_star(height));


        HeightAndStars {
            height,
            stars,
        }
    }
}

const TEXT_BOTTOM_OVERLAY: Overlay<HeightAndStars> = Overlay {
    h_placement: HorizontalPlacement::Centre,
    v_placement: VerticalPlacement::Bottom,
    ratio: Ratio::TallerThanWide(1.2),
    bytes: include_bytes!("text_bottom.svg"),
    modify_svg: &modify_svg, //modify_svg: Box<Fn(Tree)-> Tree>
};

const TEXT_RIGHT_OVERLAY: Overlay<HeightAndStars> = Overlay {
    h_placement: HorizontalPlacement::Right,
    v_placement: VerticalPlacement::Centre,
    ratio: Ratio::WiderThanTall(1.4),
    bytes: include_bytes!("text_right.svg"),
    modify_svg: &modify_svg,
};

fn modify_svg(mut tree: Tree, h: HeightAndStars) -> Tree {
    let HeightAndStars { height, stars } = h;
    if let Some(height_text_node) = tree.node_by_id("HeightText") {
        if let NodeKind::Text(ref mut text) = *height_text_node.borrow_mut() {
            text.chunks[0].text = format!("{height:.2}m");
        } else {
            println!("Height text node was not a text node");
        };
    } else {
        println!("Could not get height text node");
    }

    const STAR_IDS: [&str; 6] = ["GoldStar1", "GoldStar2", "GoldStar3", "BlackStar1", "BlackStar2", "BlackStar3"];

    let stars_to_remove : [bool; 6] = match stars{
        None=> [true, true, true,true,true,true],
        Some(StarType::Incomplete) => [true, true, true, false, false, false],
        Some(StarType::OneStar) => [false, true, true, true, false, false],
        Some(StarType::TwoStar) => [false, false,true, true, true, false],
        Some(StarType::ThreeStar) => [false, false,false,true, true, true],
    };

    for star in stars_to_remove.into_iter().zip(STAR_IDS.into_iter()).filter(|(remove, _)|*remove).map(|(_, star)|star){
        if let Some(star_node) = tree.node_by_id(star){
            star_node.detach();
        }
        else{
            println!("Could not get node: '{star}'");
        }
    }

    let mut font_database: fontdb::Database = fontdb::Database::new();
    let font_data = include_bytes!(r#"../../../../fonts/FiraMono-Medium.ttf"#).to_vec();

    font_database.load_font_data(font_data);

    tree.convert_text(&font_database);

    tree
}

pub fn convert_game_to_bytes(data: &str) -> Vec<u8> {
    base64::engine::general_purpose::URL_SAFE
        .decode(data)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use steks_common::prelude::{choose_color, Location, ALL_SHAPES, SHAPE_SIZE};

    use crate::*;
    use std::hash::{Hash, Hasher};
    use test_case::test_case;

    // spell-checker: disable-next-line
    const TEST_DATA: &'static str = "CQB7soTHrQwAfo2RbxMCAIPljd2LBQB8kF4O6RMAfQJmCOsIAIENmyiQBACA4qV-cg4AerRv1DcJAHq5eoKvCzCOqpxxPAIgi_-UJTwDIHlUV7Pp";

    fn test_image(data: &'static str, command: Command, dimensions: Dimensions) {
        let data_name = if data == TEST_DATA {
            "test_data"
        } else {
            "no_data"
        };
        let format = if command == Command::SVG {
            "svg"
        } else {
            "png"
        };
        let name = format!(
            "{data_name}_{command}_{width}x{height}.{format}",
            width = dimensions.width,
            height = dimensions.height
        );

        let bytes = convert_game_to_bytes(data);
        let height_and_stars = HeightAndStars::from_bytes(bytes.as_slice());

        let hash: u64 = match command {
            Command::SVG => {
                let svg: String = make_svg_from_bytes(&bytes, dimensions);
                let hash = calculate_hash(&svg);
                std::fs::write(name.clone(), svg).unwrap();
                hash
            }
            Command::NoOverlay | Command::Default => {
                let overlay_chooser = if command == Command::Default {
                    text_overlay()
                } else {
                    OverlayChooser::no_overlay()
                };
                let data =
                    try_draw_image(&bytes, &overlay_chooser, dimensions, height_and_stars).unwrap();
                let len = data.len();
                std::fs::write(name.clone(), data.as_slice()).unwrap();

                assert!(len < 300000, "Image is too big - {len} bytes");
                let hash = calculate_hash(&data);
                hash
            }
        };

        insta::assert_debug_snapshot!(name, hash);
    }

    #[test_case(true, "svg", 1024, 1024)]
    #[test_case(true, "svg", 512, 512)]
    #[test_case(true, "svg", 512, 1024)]
    #[test_case(true, "default", 1024, 1024)]
    #[test_case(true, "default", 512, 1024)]
    #[test_case(true, "default", 512, 512)]
    #[test_case(true, "default", 1024, 512)]
    #[test_case(false, "default", 1024, 1024)]
    #[test_case(true, "no_overlay", 1024, 1024)]
    #[test_case(true, "no_overlay", 512, 512)]
    #[test_case(true, "no_overlay", 512, 1024)]
    fn do_test(use_data: bool, command: &'static str, width: u32, height: u32) {
        let data = if use_data { TEST_DATA } else { "" };

        let command = Command::from_str_ignore_case(command);

        test_image(data, command, Dimensions { width, height });
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
                .as_svg(SHAPE_SIZE, Some(shape.fill(false).color), None);

            //println!("{shape_svg}");
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

                //println!("{shape_svg}");
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
