use reqwest::Error;
use reqwest::header::USER_AGENT;

#[derive(Debug)]
struct TleEntry {
    name: String,
    line1: String,
    line2: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // fetch TLEs for stations
    let url = "https://celestrak.org/NORAD/elements/stations.txt";

    let response = reqwest::Client::new()
        .get(url)
        .header(USER_AGENT, "tle-fetcher")
        .send()
        .await?
        .text()
        .await?;

    // parse
    let lines: Vec<&str> = response.lines().collect();
    let mut tle_entries: Vec<TleEntry> = Vec::new();

    // 3 lines is 1 entry
    for chunk in lines.chunks(3) {
        if chunk.len() == 3 {
            let entry = TleEntry {
                name: chunk[0].trim().to_string(),
                line1: chunk[1].trim().to_string(),
                line2: chunk[2].trim().to_string(),
            };
            tle_entries.push(entry);
        }
    }

    for tle in tle_entries.iter().take(6) {
        println!("{:#?}", tle);
    }

    Ok(())
}