use rusqlite::Connection;
use rusqlite::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;

#[derive(Debug, Serialize)]
struct Payload {
    stations: Vec<Station>,
}

#[derive(Debug, Serialize)]
struct Station {
    noaa_id: String,
    name: String,
    lat: f64,
    long: f64,
    state: String,
}

#[derive(Debug, Serialize)]
struct PredictionId {
    noaa_id: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let payload = get_stations(&conn)?;
    let json_string = serde_json::to_string_pretty(&payload).unwrap();
    fs::write("./docs/data/stations.json", json_string).unwrap();
    Ok(())
}

fn get_stations(conn: &Connection) -> Result<Payload> {
    let mut payload = Payload { stations: vec![] };
    let mut sql1 = conn.prepare("SELECT DISTINCT noaa_id FROM predictions")?;
    let params = rusqlite::params![];

    let rows_iter = sql1.query_map(params, |row| {
        Ok(PredictionId {
            noaa_id: row.get(0)?,
        })
    })?;

    let mut noaa_ids = BTreeSet::new();

    for row in rows_iter {
        noaa_ids.insert(row.unwrap().noaa_id);
    }

    let mut stmt = conn.prepare(
        "SELECT noaa_id, name, lat, long, state, tz_offset 
            FROM stations
            WHERE noaa_id IS NOT NULL 
            ORDER BY state, name",
    )?;
    let params = rusqlite::params![];
    let rows_iter = stmt.query_map(params, |row| {
        Ok(Station {
            noaa_id: row.get(0)?,
            name: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
            state: row.get(4)?,
        })
    })?;
    for station in rows_iter {
        let item = station.unwrap();
        if noaa_ids.contains(&item.noaa_id) {
            payload.stations.push(item);
        }
    }
    Ok(payload)
}

// fn output_payload(payload: &Payload) {}
