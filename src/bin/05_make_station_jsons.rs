use chrono::{DateTime, FixedOffset, NaiveDate};
use chrono::{Datelike, Timelike};
use indicatif::ProgressBar;
use rusqlite::Connection;
use rusqlite::Result;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use sunrise::sunrise_sunset;

#[derive(Debug, Serialize)]
struct Station {
    noaa_id: String,
    name: String,
    lat: f64,
    long: f64,
    state: String,
    tz_offset: i32,
    tides: Vec<Tide>,
}

// #[derive(Debug, Serialize)]
// struct InitialTide {
//     high_or_low: String,
//     value: f64,
//     tide_utc_year: i32,
//     tide_utc_month: u32,
//     tide_utc_day: u32,
//     tide_utc_hour: u32,
//     tide_utc_min: u32,
// }

#[derive(Debug, Serialize)]
struct Tide {
    high_low: String,
    value: f64,
    tide_local_year: i32,
    tide_local_month: u32,
    tide_local_day: u32,
    tide_local_hour: u32,
    tide_local_minute: u32,
    sunrise_local_year: i32,
    sunrise_local_month: u32,
    sunrise_local_day: u32,
    sunrise_local_hour: u32,
    sunrise_local_minute: u32,
    sunset_local_year: i32,
    sunset_local_month: u32,
    sunset_local_day: u32,
    sunset_local_hour: u32,
    sunset_local_minute: u32,
    sunrise_delta_hour: i64,
    sunrise_delta_minute: i64,
    sunrise_delta_minutes_raw: i64,
    sunset_delta_hour: i64,
    sunset_delta_minute: i64,
    sunset_delta_minutes_raw: i64,
}

impl Tide {
    pub fn new(
        high_low: String,
        value: f64,
        utc_year: i32,
        utc_month: u32,
        utc_day: u32,
        utc_hour: u32,
        utc_minute: u32,
        lat: f64,
        long: f64,
        tz_offset: i32,
    ) -> Tide {
        let tz = FixedOffset::east_opt(tz_offset * 3600).unwrap();

        let utc_low_tide = NaiveDate::from_ymd_opt(utc_year, utc_month, utc_day)
            .unwrap()
            .and_hms_opt(utc_hour, utc_minute, 0)
            .unwrap()
            .and_utc();
        let local_low_tide = utc_low_tide.with_timezone(&tz);

        let sun_times = sunrise_sunset(lat, long, utc_year, utc_month, utc_day);
        let utc_sunrise = DateTime::from_timestamp(sun_times.0, 0).unwrap();
        let utc_sunset = DateTime::from_timestamp(sun_times.1, 0).unwrap();

        // remove seconds since only minutes are used in output
        let utc_sunrise = utc_sunrise.with_second(0).unwrap();

        let local_sunrise = utc_sunrise.with_timezone(&tz);
        let local_sunset = utc_sunset.with_timezone(&tz);
        let delta_sunrise = utc_low_tide - utc_sunrise;
        let delta_sunset = utc_low_tide - utc_sunset;

        let tide_local_year = local_low_tide.year();
        let tide_local_month = local_low_tide.month();
        let tide_local_day = local_low_tide.day();
        let tide_local_hour = local_low_tide.hour();
        let tide_local_minute = local_low_tide.minute();
        let sunrise_local_year = local_sunrise.year();
        let sunrise_local_month = local_sunrise.month();
        let sunrise_local_day = local_sunrise.day();
        let sunrise_local_hour = local_sunrise.hour();
        let sunrise_local_minute = local_sunrise.minute();
        let sunset_local_year = local_sunset.year();
        let sunset_local_month = local_sunset.month();
        let sunset_local_day = local_sunset.day();
        let sunset_local_hour = local_sunset.hour();
        let sunset_local_minute = local_sunset.minute();
        let sunrise_delta_hour = delta_sunrise.num_hours();
        let sunrise_delta_minute = (delta_sunrise.num_minutes() % 60).abs();
        let sunrise_delta_minutes_raw = delta_sunrise.num_minutes();
        let sunset_delta_hour = delta_sunset.num_hours();
        let sunset_delta_minute = (delta_sunset.num_minutes() % 60).abs();
        let sunset_delta_minutes_raw = delta_sunset.num_minutes();

        Tide {
            high_low,
            value,
            tide_local_year,
            tide_local_month,
            tide_local_day,
            tide_local_hour,
            tide_local_minute,
            sunrise_local_year,
            sunrise_local_month,
            sunrise_local_day,
            sunrise_local_hour,
            sunrise_local_minute,
            sunset_local_year,
            sunset_local_month,
            sunset_local_day,
            sunset_local_hour,
            sunset_local_minute,
            sunrise_delta_hour,
            sunrise_delta_minute,
            sunset_delta_hour,
            sunset_delta_minute,
            sunrise_delta_minutes_raw,
            sunset_delta_minutes_raw,
        }
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let mut stations = get_stations(&conn)?;
    get_tides(&conn, &mut stations).unwrap();
    //let json_string = serde_json::to_string_pretty(&payload).unwrap();
    //fs::write("./docs/data/stations.json", json_string).unwrap();
    Ok(())
}

fn get_tides(conn: &Connection, stations: &mut Vec<Station>) -> Result<()> {
    let mut sql = conn.prepare(
        "SELECT noaa_id, type, value, year, month, day, hour, min
            FROM predictions 
            WHERE noaa_id = ?1
            ORDER BY year, month, day, hour, min",
    )?;

    let bar = ProgressBar::new(stations.len().try_into().unwrap());
    for station in stations {
        let params = rusqlite::params![station.noaa_id];
        let rows_iter = sql.query_map(params, |row| {
            Ok(Tide::new(
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                station.lat,
                station.long,
                station.tz_offset,
            ))
        })?;
        for row_wrapped in rows_iter {
            station.tides.push(row_wrapped.unwrap());
        }
        if station.tides.len() > 0 {
            let json = serde_json::to_string(&station).unwrap();
            let path = PathBuf::from(format!("./docs/data/predictions/{}.json", station.noaa_id));
            fs::write(path, json).unwrap();
        }
        bar.inc(1);
    }
    bar.finish();

    Ok(())
}

fn get_stations(conn: &Connection) -> Result<Vec<Station>> {
    let mut stations = vec![];
    let mut sql = conn.prepare(
        "SELECT noaa_id, name, lat, long, state, tz_offset 
            FROM stations
            WHERE noaa_id IS NOT NULL",
    )?;
    let params = rusqlite::params![];
    let rows_iter = sql.query_map(params, |row| {
        Ok(Station {
            noaa_id: row.get(0)?,
            name: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
            state: row.get(4)?,
            tz_offset: row.get(5)?,
            tides: vec![],
        })
    })?;
    for station in rows_iter {
        stations.push(station.unwrap());
    }
    Ok(stations)
}
