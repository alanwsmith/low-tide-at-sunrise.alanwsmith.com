use rusqlite::Connection;
use rusqlite::Result;
use serde::Serialize;
// use std::fs;
use minijinja::{context, Environment};
// use sunrise::sunrise_sunset;

#[derive(Debug, Serialize)]
struct Station {
    noaa_id: String,
    name: String,
    lat: f64,
    long: f64,
    state: String,
    tz_offset: i32,
}

// #[derive(Debug)]
// struct LowTide {
//     date: String,
// }

// impl Station {
//     pub fn low_tides(&self, conn: &Connection) -> Vec<LowTide> {
//         vec![]
//     }
// }

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let stations = get_stations(&conn)?;

    //let times = sunrise_sunset(21.3033332824707, -157.864532470703, 2025, 1, 1);
    //dbg!(times);
    //dbg!(stations);
    output_results(stations);
    Ok(())
}

fn get_stations(conn: &Connection) -> Result<Vec<Station>> {
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
            tz_offset: row.get(5)?,
        })
    })?;
    let mut stations: Vec<Station> = vec![];
    for station in rows_iter {
        stations.push(station.unwrap());
    }
    Ok(stations)
}

fn output_results(stations: Vec<Station>) {
    let mut env = Environment::new();
    env.add_template_owned(
        "output_template",
        "
{% for station in stations %}
{{ station.state }} - {{ station.name }}
{% endfor %}
"
        .to_string(),
    )
    .unwrap();
    match env.get_template("output_template") {
        Ok(template) => match template.render(context!(stations => stations)) {
            Ok(output) => {
                println!("{}", output);
            }
            Err(e) => {
                dbg!(e);
                ()
            }
        },
        Err(e) => {
            dbg!(e);
            ()
        }
    }
}
