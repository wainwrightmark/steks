use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use http::header::HeaderMap;
use http::HeaderValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use resvg::usvg::{Tree, TreeParsing};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const WHITE: &str = "#f7f5f0";
const BLACK: &str = "#1f1b20";
const GRAY: &str = "#a1a9b0";

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
    let mut svg_text: String = "".to_string();
    svg_text.push_str(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 238.1 238.1">"#).as_str());
    svg_text.push('\n');

    svg_text.push_str(
        format!(
            r#"<path d="M0 0h238.1v238.1H0z" style="fill:{WHITE};stroke:{GRAY};stroke-width:4;" />"#
        )
        .as_str(),
    );
    svg_text.push('\n');

    svg_text.push_str("</svg>");

    return svg_text;
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

    use resvg::FitTo;
    resvg::render(
        &tree,
        FitTo::Size(WIDTH, HEIGHT),
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    pixmap.encode_png().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{draw_image, make_svg_text};

    #[test]
    fn test_svg() {
        let svg: String = make_svg_text("-1+4/78 5");
        std::fs::write("og_example.svg", svg).unwrap();
    }

    #[test]
    fn parse_test() {
        let data = draw_image("-1+4/78 5");
        std::fs::write("parse_test.png", data).unwrap();
    }

    #[test]
    fn unknown_test() {
        let data = draw_image("null");
        std::fs::write("unknown.png", data).unwrap();
    }
}
