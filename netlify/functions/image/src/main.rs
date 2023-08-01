pub mod placement;
pub mod svg;

use std::fmt::Display;

use aws_lambda_events::query_map::QueryMap;
use base64::Engine;
pub use steks_common::prelude::*;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{AspectRatio, NodeExt, NonZeroRect, Options, Tree, TreeParsing, ViewBox};

use crate::placement::{HorizontalPlacement, VerticalPlacement};

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

    let dimensions = Dimensions::from_query_map(&e.payload.query_string_parameters);

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
        }
    }
}

impl Dimensions {
    pub fn from_query_map(query_map: &QueryMap) -> Self {
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
        match self {
            Command::Default => {
                let data = draw_image(game, &OverlayChooser::text_overlay(), dimensions);
                Body::Binary(data)
            }
            Command::NoOverlay => {
                let data = draw_image(game, &OverlayChooser::no_overlay(), dimensions);
                Body::Binary(data)
            }
            Command::SVG => {
                let data = make_svg_from_data(game, dimensions);

                Body::Text(data)
            }
        }
    }
}

fn make_svg_from_data(data: &str, dimensions: Dimensions) -> String {
    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(data) else {return "".to_string();};

    let shapes = decode_shapes(&bytes);

    let svg = svg::create_svg(shapes.into_iter(), dimensions);
    svg
}

fn draw_image(game: &str, overlay_chooser: &OverlayChooser, dimensions: Dimensions) -> Vec<u8> {
    let opt: resvg::usvg::Options = Default::default();
    let svg_data = make_svg_from_data(game, dimensions);

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

    let [r, g, b, a] = steks_common::color::BACKGROUND_COLOR
        .as_rgba_u32()
        .to_le_bytes();
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

pub struct OverlayChooser {
    pub options: Vec<Overlay>,
}

pub const DEFAULT_SCALE_MULTIPLIER: f32 = 1.1;
impl OverlayChooser {
    pub fn no_overlay() -> Self {
        Self { options: vec![] }
    }

    pub fn text_overlay() -> Self {
        Self {
            options: vec![Overlay::TEXT_RIGHT_OVERLAY, Overlay::TEXT_BOTTOM_OVERLAY],
        }
    }

    pub fn choose_scale_and_overlay(&self, h_scale: f32, v_scale: f32) -> (f32, Option<&Overlay>) {
        println!("h: {h_scale} v: {v_scale}");
        if self.options.is_empty() {
            return (h_scale.max(v_scale) * DEFAULT_SCALE_MULTIPLIER, None);
        }

        let mut result = (f32::MAX, None);

        for ov in self.options.iter() {
            let scale = match ov.ratio {
                Ratio::WiderThanTall(r) => (r * h_scale).max(v_scale),
                Ratio::TallerThanWide(r) => (r * v_scale).max(h_scale),
            };

            println!("Scale: {scale}");

            let scale = scale * DEFAULT_SCALE_MULTIPLIER;

            if scale < result.0 {
                result = (scale, Some(ov))
            }
        }

        return result;
    }
}

#[derive(Debug, Clone)]
pub struct Overlay {
    pub h_placement: HorizontalPlacement,
    pub v_placement: VerticalPlacement,

    pub ratio: Ratio,

    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, Copy)]
pub enum Ratio {
    WiderThanTall(f32),
    TallerThanWide(f32),
}

impl Overlay {
    // pub const DEFAULT_OVERLAY: Overlay = Overlay {
    //     h_placement: HorizontalPlacement::Centre,
    //     v_placement: VerticalPlacement::Centre,
    //     min_h: DEFAULT_SCALE_MULTIPLIER,
    //     min_v: DEFAULT_SCALE_MULTIPLIER,
    //     bytes: include_bytes!("logo_monochrome.svg"),
    // };

    pub const TEXT_BOTTOM_OVERLAY: Overlay = Overlay {
        h_placement: HorizontalPlacement::Centre,
        v_placement: VerticalPlacement::Bottom,
        ratio: Ratio::TallerThanWide(1.2),
        bytes: include_bytes!("text_bottom.svg"),
    };

    pub const TEXT_RIGHT_OVERLAY: Overlay = Overlay {
        h_placement: HorizontalPlacement::Right,
        v_placement: VerticalPlacement::Centre,
        ratio: Ratio::WiderThanTall(1.4),
        bytes: include_bytes!("text_right.svg"),
    };

    pub fn include(&self, pixmap: &mut Pixmap, opt: &Options, dimensions: Dimensions) {
        let logo_tree = match Tree::from_data(self.bytes, &opt) {
            Ok(tree) => tree,
            Err(e) => panic!("{e}"),
        };

        let logo_scale = (dimensions.width as f32 / logo_tree.size.width() as f32)
            .min(dimensions.height as f32 / logo_tree.size.height() as f32);

        let x_offset = self
            .h_placement
            .get_x(dimensions.width as f32, logo_tree.size.width() * logo_scale);
        let y_offset = self.v_placement.get_y(
            dimensions.height as f32,
            logo_tree.size.height() * logo_scale,
        );
        let transform =
            Transform::from_scale(logo_scale, logo_scale).post_translate(x_offset, y_offset);

        resvg::Tree::render(
            &resvg::Tree::from_usvg(&logo_tree),
            transform,
            &mut pixmap.as_mut(),
        );
    }
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

        let hash: u64 = match command {
            Command::SVG => {
                let svg: String = make_svg_from_data(data, dimensions);
                let hash = calculate_hash(&svg);
                std::fs::write(name.clone(), svg).unwrap();
                hash
            }
            Command::NoOverlay | Command::Default => {
                let overlay_chooser = if command == Command::Default {
                    OverlayChooser::text_overlay()
                } else {
                    OverlayChooser::no_overlay()
                };
                let data = draw_image(data, &overlay_chooser, dimensions);
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
                .as_svg(SHAPE_SIZE, Some(shape.fill().color), None);

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
