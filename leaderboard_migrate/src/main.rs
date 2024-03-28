use itertools::Itertools;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    for line in include_str!("hashes.csv").lines() {
        let hash: u64 = line.parse().unwrap();

        let data = get_leaderboard_data(hash).await;

        let (hash, height, blob) = data.deconstruct();

        //println!("Updating {hash} {height} {blob}");

        update_wrs_async(hash, height, blob).await?;
    }

    Ok(())
}

async fn get_leaderboard_data(hash: u64) -> LeaderboardDataEvent {
    let client = reqwest::Client::new();
    let url =
        format!("https://steks.net/.netlify/functions/leaderboard?command=getrow&hash={hash}");
    let res = client.get(url).send().await;

    match res {
        Ok(response) => match response.text().await {
            Ok(text) => LeaderboardDataEvent::Success { text },
            Err(err) => LeaderboardDataEvent::Failure {
                error: err.to_string(),
                hash,
            },
        },
        Err(err) => LeaderboardDataEvent::Failure {
            error: err.to_string(),
            hash,
        },
    }
}

async fn update_wrs_async(hash: u64, height: f32, blob: String) -> Result<(), reqwest::Error> {


    let url = format!("https://steks.net/.netlify/functions/leaderboard2?command=set&hash={hash}&height={height:.2}&blob={blob}");

    println!("{url}");

    let client = reqwest::Client::new();
    let res = client
            .post(url)
            .send()
            .await?;

    res.error_for_status().map(|_| ())
}

#[derive(Debug, Clone)]
pub enum LeaderboardDataEvent {
    Success { text: String },
    Failure { hash: u64, error: String },
}

impl LeaderboardDataEvent {
    pub fn deconstruct(self) -> (u64, f32, String) {
        let text = match self {
            LeaderboardDataEvent::Success { text } => text,
            LeaderboardDataEvent::Failure { hash, error } => {
                panic!("Leaderboard event failure {error}")
            }
        };

        let Some((hash, written_height, image_blob)) = text.split_ascii_whitespace().next_tuple()
        else {
            panic!("Could not parse wr row: {text}");
        };

        let hash: u64 = hash.parse().unwrap();

        let height :f32 = written_height.parse().unwrap();

        (hash, height, image_blob.to_string())


    }
}
