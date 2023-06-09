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
    let func = service_fn(my_handler);
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

pub(crate) async fn my_handler(
    e: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));

    let command = get_parameter(&e, "command").ok_or_else(|| "Could not get command")?;
    let command: Command = command.parse()?;

    match command {
        Command::Get => {
            let mut connection = connect_to_database();

            let rows: Vec<Row> = query("select shapes_hash, max_height FROM tower_height;")
                .fetch_all(&mut connection)
                .await?;

            // .query_map(
            //     ,
            //     |(shapes_hash, max_height)| Row {
            //         shapes_hash,
            //         max_height,
            //     },
            // )?;

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
        Command::TrySet => {
            let hash = get_parameter(&e, "hash").ok_or_else(|| "Could not get hash")?;
            let hash: u64 = hash.parse()?;
            let height = get_parameter(&e, "height").ok_or_else(|| "Could not get height")?;
            let height: f32 = height.parse()?;

            try_set(height, hash).await?;
            let resp = ApiGatewayProxyResponse {
                status_code: 200,
                headers,
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Empty),
                is_base64_encoded: false,
            };

            Ok(resp)
        }
    }
}

async fn try_set(height: f32, hash: u64) -> Result<(), Error> {
    let mut connection = connect_to_database();

    query(
        "

            Insert into tower_height (shapes_hash, max_height) Values($0, $1)
            ON DUPLICATE KEY UPDATE
            max_height = IF (max_height > $1, max_height, $1);",
    )
    .bind(hash)
    .bind(height)
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
pub struct Row {
    shapes_hash: u64,
    max_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Get,
    TrySet,
}

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.shapes_hash, self.max_height))
    }
}

impl FromStr for Command {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("get") {
            Ok(Self::Get)
        } else if value.eq_ignore_ascii_case("set") {
            Ok(Self::TrySet)
        } else {
            Err("Could not parse command")
        }
    }
}
