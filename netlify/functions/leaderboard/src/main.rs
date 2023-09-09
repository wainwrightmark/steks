use std::env;
use std::fmt::Display;
use std::str::FromStr;

use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::http::{HeaderMap, HeaderValue};
use itertools::Itertools;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use planetscale_driver::PSConnection;
use planetscale_driver::{query, Database};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(logging_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

fn get_parameter<'a>(
    e: &'a LambdaEvent<ApiGatewayProxyRequest>,
    name: &'static str,
) -> Option<&'a str> {
    e.payload
        .query_string_parameters
        .iter()
        .filter(|x| x.0.eq_ignore_ascii_case(name))
        .map(|x| x.1)
        .next()
}

pub(crate) async fn logging_handler(
    e: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    match my_handler(e).await {
        Ok(r) => Ok(r),
        Err(e) => {
            println!("Error: {e}");
            Err(e)
        }
    }
}

pub(crate) async fn my_handler(
    e: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("*"),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST"),
    );

    let command = get_parameter(&e, "command").ok_or_else(|| "Could not get command")?;
    let command: Command = command.parse()?;

    match command {
        Command::Get => {
            let mut connection = connect_to_database();

            let rows: Vec<MiniRow> = query("select shapes_hash, max_height FROM tower_height;")
                .fetch_all(&mut connection)
                .await?;

            let data = Itertools::join(&mut rows.into_iter(), " ");

            let resp = ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Text(data)),
                is_base64_encoded: false,
            };
            return Ok(resp);
        }
        Command::GetRow => {
            let shapes_hash = get_parameter(&e, "hash").ok_or_else(|| "Could not get hash")?;
            let shapes_hash: u64 = shapes_hash.parse()?;

            let connection = connect_to_database();

            let row_result: anyhow::Result<FullRow> = query("select shapes_hash, max_height, image_blob FROM tower_height where shapes_hash = $0;")
                .bind(shapes_hash)
                .fetch_one(&connection)

                .await;

            match row_result {
                Ok(row) => {
                    let resp = ApiGatewayProxyResponse {
                        status_code: 200,
                        headers,
                        multi_value_headers: HeaderMap::new(),
                        body: Some(Body::Text(row.to_string())),
                        is_base64_encoded: false,
                    };
                    return Ok(resp);
                }
                Err(err) => {
                    if err.to_string().contains("No results found") {
                        let row = FullRow {
                            shapes_hash,
                            max_height: 0.0,
                            image_blob: "0".to_string(),
                        };
                        let resp = ApiGatewayProxyResponse {
                            status_code: 200,
                            headers,
                            multi_value_headers: HeaderMap::new(),
                            body: Some(Body::Text(row.to_string())),
                            is_base64_encoded: false,
                        };
                        return Ok(resp);
                    } else {
                        return Err(err.into());
                    }
                }
            }
        }
        Command::TrySet => {
            let hash = get_parameter(&e, "hash").ok_or_else(|| "Could not get hash")?;
            let hash: u64 = hash.parse()?;
            let height = get_parameter(&e, "height").ok_or_else(|| "Could not get height")?;
            let height: f32 = height.parse()?;
            let blob = get_parameter(&e, "blob").ok_or_else(|| "Could not get blob")?;

            try_set(height, hash, blob).await?;
            let resp = ApiGatewayProxyResponse {
                status_code: 202,
                headers,
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Empty),
                is_base64_encoded: false,
            };

            Ok(resp)
        }
    }
}

async fn try_set(height: f32, hash: u64, blob: &str) -> Result<(), Error> {
    let mut connection = connect_to_database();

    query(
        "
            Insert into tower_height (shapes_hash, max_height, image_blob) Values($0, $1, \"$2\")
            ON DUPLICATE KEY UPDATE
            max_height = IF (max_height > $1, max_height, $1),
            image_blob = IF (max_height > $1, image_blob, \"$2\");
            ",
    )
    .bind(hash)
    .bind(height)
    .bind(blob)
    .execute(&mut connection)
    .await?;

    Ok(())
}

fn connect_to_database() -> PSConnection {
    let host = env::var("DATABASE_HOST").expect("DATABASE_HOST not found");
    let username = env::var("DATABASE_USERNAME").expect("DATABASE_USERNAME not found");
    let password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not found");

    let conn = PSConnection::new(host.as_str(), username.as_str(), password.as_str());
    conn
}

#[derive(Debug, Clone, Copy, Database, PartialEq)]
pub struct MiniRow {
    shapes_hash: u64,
    max_height: f32,
}

impl Display for MiniRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.shapes_hash, self.max_height))
    }
}

#[derive(Debug, Clone, Database, PartialEq)]
pub struct FullRow {
    shapes_hash: u64,
    max_height: f32,
    image_blob: String,
}

impl Display for FullRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {}",
            self.shapes_hash, self.max_height, self.image_blob
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Get,
    GetRow,
    TrySet,
}

impl FromStr for Command {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("get") {
            Ok(Self::Get)
        } else if value.eq_ignore_ascii_case("getrow") {
            Ok(Self::GetRow)
        } else if value.eq_ignore_ascii_case("set") {
            Ok(Self::TrySet)
        } else {
            Err("Could not parse command")
        }
    }
}
