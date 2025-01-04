use chrono_tz::Tz;
use lazy_static::lazy_static;
use rusqlite::Connection;
use rusqlite::Result;
use serde::Deserialize;
use tzf_rs::DefaultFinder;

lazy_static! {
    static ref FINDER: DefaultFinder = DefaultFinder::new();
}

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
            stations(noaa_id, name, lat, long, state, tz_name, get_for_dev) 
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
                Some(tz_name),
            ) = (
                station.id.as_ref(),
                station.name.as_ref(),
                station.lat.as_ref(),
                station.lng.as_ref(),
                station.state.as_ref(),
                //station.timezonecorr.as_ref(),
                Some(get_tz_name(station.lat.as_ref().unwrap(), station.lng.as_ref().unwrap())),
            ) {
                dbg!(tz_name);

                // First version without states had 2,642 entries
                // Including entries without states: 3,349
                let _ = conn.execute(insert_data, (noaa_id, name, lat, long, state, tz_name.to_string(), 0));
                // dbg!(noaa_id);
                // dbg!(state);
            }
        });
    };
    let _ = conn.close();
    Ok(())
}

fn get_tz_name(lat: &f64, long: &f64) -> Tz {
    let tz: Tz = FINDER.get_tz_name(*long, *lat).parse().unwrap();
    tz
}
