use indicatif::ProgressBar;
use rusqlite::Connection;
use rusqlite::Result;
// use rusqlite::Transaction;
use serde::Deserialize;
// use std::{thread, time};

#[derive(Debug, Deserialize)]
struct Data {
    predictions: Vec<Prediction>,
}

#[derive(Debug, Deserialize)]
struct Prediction {
    t: String,
    v: String,
    r#type: String,
}

fn main() {
    println!("get station tide data");
    match get_station_data() {
        Ok(()) => println!("got it"),
        Err(e) => {
            dbg!(e);
            ()
        }
    }
}

fn get_station_data() -> Result<()> {
    let mut station_ids: Vec<String> = vec![];
    let conn = Connection::open("./data/data.sqlite")?;
    let debug = false;
    if debug == true {
        let mut get_data = conn.prepare("SELECT noaa_id FROM stations WHERE get_for_dev = ?1 ")?;
        let response = get_data.query_map([1], |row| {
            Ok((row.get::<usize, String>(0), row.get::<usize, String>(1)))
        })?;
        for item in response {
            station_ids.push(item.unwrap().0.unwrap().clone());
        }
    } else {
        let mut get_data = conn.prepare("SELECT noaa_id FROM stations")?;
        let response = get_data.query_map([], |row| {
            Ok((row.get::<usize, String>(0), row.get::<usize, String>(1)))
        })?;
        for item in response {
            station_ids.push(item.unwrap().0.unwrap().clone());
        }
    }
     //let sleep_time = time::Duration::from_millis(300);
    let _ = conn.close();

    let cnt = station_ids.len() * 10;
    let bar = ProgressBar::new(cnt.try_into().unwrap());

            // let url = format!("https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?begin_date={}0101&end_date={}1231&station={}&product=predictions&datum=STND&time_zone=gmt&units=english&format=json&interval=hilo",
            //     2025, 2025, 
            //     "8720218",
            // );
            // let _ = get_json(&url, "8720218");

    station_ids.iter().for_each(|station_id| {
        for year in 2025..2035 {
            let url = format!("https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?begin_date={}0101&end_date={}1231&station={}&product=predictions&datum=STND&time_zone=gmt&units=english&format=json&interval=hilo",
                year, 
                year,
                &station_id, 
            );
            let _ = get_json(&url, &station_id);
            // thread::sleep(sleep_time);
            bar.inc(1);
        }
    });

    bar.finish();
    Ok(())
}

fn get_json(url: &str, noaa_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::open("./data/data.sqlite")?;
    let tx = conn.transaction()?;
    // println!("Loading: {}", &noaa_id);
    let insert_data = "
        INSERT INTO
            predictions(noaa_id, year, month, day, hour, min, value, type) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
    let res = reqwest::blocking::get(url)?;
    if res.status() == 200 {
        let body = res.text()?;
        let data: Data = serde_json::from_str(&body)?;
        data.predictions.iter().for_each(|prediction| {
            let string_parts = &mut prediction.t.split(" ");
            let mut date_parts = string_parts.next().expect("doing split").split("-");
            let year: u32 = date_parts.next().expect("parsing").parse().unwrap();
            let month: u32 = date_parts.next().expect("parsing").parse().unwrap();
            let day: u32 = date_parts.next().expect("parsing").parse().unwrap();
            let mut time_parts = string_parts.next().expect("doing split").split(":");
            let hour: u32 = time_parts.next().expect("parsing").parse().unwrap();
            let min: u32 = time_parts.next().expect("parsing").parse().unwrap();
            let value: f64 = prediction.v.parse().unwrap();
            let r#type: String = prediction.r#type.clone();
            match tx.execute(
                insert_data,
                (noaa_id, year, month, day, hour, min, value, r#type),
            ) {
                Ok(_) => (),
                Err(e) => {
                    dbg!(e);
                    ()
                }
            };
        });
    };
    tx.commit()?;
    let _ = conn.close();
    Ok(())
}
