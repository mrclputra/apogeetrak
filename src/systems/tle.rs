use reqwest::Error;
use reqwest::header::USER_AGENT;

#[derive(Debug)]
struct Satellite {
    name: String,
    norad_id: u32,
    intl_id: String,
    launch_year: u16,
    launch_number: u16,
    epoch_year: u16,
    epoch_day: f64,
    mean_motion: f64,
    inclination: f64,

    // keep raw TLE lines for reference if needed
    line1: String,
    line2: String
}

impl Satellite {
    fn parse(name: &str, line1: &str, line2: &str) -> Option<Self> {
        if line1.len() < 69 || line2.len() < 69 {
            return None;
        }

        // note that in TLE format, positions are fixed
        // https://en.wikipedia.org/wiki/Two-line_element_set

        // extract data from line 1
        let norad_id: u32 = line1[2..7].trim().parse().ok()?;
        let intl_id = line1[9..17].trim().to_string();
        let launch_year: u16 = line1[9..11].parse().ok()?;
        let launch_number: u16 = line1[11..14].trim().parse().ok()?;
        let epoch_year: u16 = line1[18..20].parse().ok()?;
        let epoch_day: f64 = line1[20..32].trim().parse().ok()?;

        // extract data from line 2
        let inclination: f64 = line2[8..16].trim().parse().ok()?;
        let mean_motion: f64 = line2[52..63].trim().parse().ok()?;

        Some(Satellite {
            name: name.trim().to_string(),
            norad_id,
            intl_id,
            launch_year: if launch_year < 57 { 2000 + launch_year } else { 1900 + launch_year },
            launch_number,
            epoch_year: if epoch_year < 57 { 2000 + epoch_year } else { 1900 + epoch_year },
            epoch_day,
            mean_motion,
            inclination,
            line1: line1.to_string(),
            line2: line2.to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // call API here
    let url = "https://celestrak.org/NORAD/elements/gnss.txt";

    let response = reqwest::Client::new()
        .get(url)
        .header(USER_AGENT, "gnss-satellite-tracker")
        .send()
        .await?
        .text()
        .await?;

    // parse
    let lines: Vec<&str> = response.lines().collect();
    let mut satellites: Vec<Satellite> = Vec::new();

    for chunk in lines.chunks(3) {
        if chunk.len() == 3 {
            if let Some(satellite) = Satellite::parse(chunk[0], chunk[1], chunk[2]) {
                satellites.push(satellite);
            }
        }
    }

    println!("Found {} navigation satellites in orbit!\n", satellites.len());

    for (i, satellite) in satellites.iter().take(10).enumerate() {
        println!("{}. {}", i + 1, satellite.name);
        println!("   NORAD ID: {}", satellite.norad_id);
        println!("   Launch: {}-{:03} (Year-Number)", satellite.launch_year, satellite.launch_number);
        println!("   Inclination: {:.2}°", satellite.inclination);
        println!("   Mean Motion: {:.2} rev/day", satellite.mean_motion);
        println!("   Data Epoch: Year {} Day {:.2}", satellite.epoch_year, satellite.epoch_day);
        println!();
    }

    if satellites.len() > 10 {
        println!("... and {} more entries", satellites.len() - 10);
    }

    Ok(())
}