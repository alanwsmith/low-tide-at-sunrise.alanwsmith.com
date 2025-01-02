use rusqlite::Connection;
use rusqlite::Result;
use serde::Serialize;
// use std::fs;
use minijinja::{context, Environment, Error, Value};
// use sunrise::sunrise_sunset;
use chrono::{FixedOffset, NaiveDate};
use minijinja::value::Object;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct Station {
    noaa_id: String,
    name: String,
    lat: f64,
    long: f64,
    state: String,
    tz_offset: i32,
    low_tides: Vec<Value>,
}

#[derive(Copy, Clone, Debug, Serialize)]
struct Prediction {
    utc_year: i32,
    utc_month: u32,
    utc_day: u32,
    utc_hour: u32,
    utc_min: u32,
    value: f64,
    tz_offset: i32,
}

impl Prediction {
    pub fn output(&self) -> Value {
        let utc_dt = NaiveDate::from_ymd_opt(self.utc_year, self.utc_month, self.utc_day)
            .unwrap()
            .and_hms_opt(self.utc_hour, self.utc_min, 0)
            .unwrap()
            .and_utc();

        let tz_offset = FixedOffset::east_opt(self.tz_offset * 3600).unwrap();
        let adjusted_dt = utc_dt.with_timezone(&tz_offset);

        Value::from(format!("{}", adjusted_dt))
    }
}

impl Object for Prediction {
    fn call_method(
        self: &Arc<Prediction>,
        _state: &minijinja::State,
        name: &str,
        _args: &[Value],
    ) -> Result<Value, Error> {
        match name {
            "output" => Ok(self.output()),
            _ => Ok(Value::from("")),
        }
    }
}

enum Direction {
    Init,
    Up,
    Down,
}

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let mut stations = get_stations(&conn)?;
    let _ = find_low_tides(&conn, &mut stations);
    //let times = sunrise_sunset(21.3033332824707, -157.864532470703, 2025, 1, 1);
    //dbg!(times);
    //dbg!(stations);
    output_results(stations);
    Ok(())
}

fn find_low_tides(conn: &Connection, stations: &mut Vec<Station>) -> Result<()> {
    let mut sql = conn.prepare(
        "SELECT noaa_id, year, month, day, hour, min, value
            FROM predictions 
            WHERE noaa_id = ?1
            ORDER BY year, month, day, hour, min",
    )?;

    for station in stations {
        let params = rusqlite::params![station.noaa_id];
        let rows_iter = sql.query_map(params, |row| {
            Ok(Prediction {
                utc_year: row.get(1)?,
                utc_month: row.get(2)?,
                utc_day: row.get(3)?,
                utc_hour: row.get(4)?,
                utc_min: row.get(5)?,
                value: row.get(6)?,
                tz_offset: station.tz_offset,
            })
        })?;

        let mut direction: Direction = Direction::Init;
        let mut previous: Option<Prediction> = None;

        for row in rows_iter {
            let prediction = row.unwrap();
            match previous {
                Some(p) => {
                    previous = Some(p);
                    let current = prediction;

                    match direction {
                        Direction::Init => {
                            if current.value >= previous.unwrap().value {
                                direction = Direction::Up;
                            } else {
                                direction = Direction::Down;
                            }
                        }
                        Direction::Up => {
                            if current.value < previous.unwrap().value {
                                direction = Direction::Down;
                            }
                        }
                        Direction::Down => {
                            if current.value > previous.unwrap().value {
                                station.low_tides.push(Value::from_object(current.clone()));
                                direction = Direction::Up;
                            }
                        }
                    }
                }
                None => previous = Some(prediction),
            }
        }
    }
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
            low_tides: vec![],
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
        r#"
{%- for station in stations %}
{%- if station.low_tides %}
{{- station.state }} - {{ station.name }}{{ "\n" }}
{%- for low_tide in station.low_tides %}
{{- low_tide.output() }}{{ "\n" }}
{%- endfor %}
{%- endif %}
{%- endfor %}
"#
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
