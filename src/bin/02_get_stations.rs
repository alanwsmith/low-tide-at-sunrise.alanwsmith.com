use rusqlite::Connection;
use rusqlite::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
struct Station {
    id: Option<String>,
    lat: Option<f64>,
    lng: Option<f64>,
    name: Option<String>,
    state: Option<String>,
    timezonecorr: Option<i64>,
}

fn main() {
    let url = "https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations.json?type=tidepredictions&units=english";
    match get_json(url) {
        Ok(()) => println!("got it"),
        Err(e) => {
            dbg!(e);
            ()
        }
    }
}

fn get_json(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/data.sqlite")?;
    let insert_data = "
        INSERT OR IGNORE INTO 
            stations(noaa_id, name, lat, long, state, tz_offset, get_for_dev) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";
    let res = reqwest::blocking::get(url)?;
    if res.status() == 200 {
        let body = res.text()?;
        let data: Data = serde_json::from_str(&body)?;
        data.stations.iter().for_each(|station| {
            if let (
                Some(noaa_id),
                Some(name),
                Some(lat),
                Some(long),
                Some(state),
                Some(tz_offset),
            ) = (
                station.id.as_ref(),
                station.name.as_ref(),
                station.lat.as_ref(),
                station.lng.as_ref(),
                station.state.as_ref(),
                station.timezonecorr.as_ref(),
            ) {
                // First version without states had 2,642 entries
                // Including entries without states: 3,349
                let _ = conn.execute(insert_data, (noaa_id, name, lat, long, state, tz_offset, 0));
                // dbg!(noaa_id);
                // dbg!(state);
            }
        });
    };
    let _ = conn.close();
    Ok(())
}
