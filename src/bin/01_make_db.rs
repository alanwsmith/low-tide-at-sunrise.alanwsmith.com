use rusqlite::Connection;
use rusqlite::Result;

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let create_stations_table = "
        CREATE TABLE IF NOT EXISTS stations (
            db_id INTEGER PRIMARY KEY, 
            noaa_id TEXT NOT NULL UNIQUE,
            name TEXT,
            lat INTEGER,
            long INTEGER,
            state TEXT,
            tz_name TEXT,
            get_for_dev INTEGER
        )";
    conn.execute(create_stations_table, ())?;
    let create_predictions_table = "
        CREATE TABLE IF NOT EXISTS predictions (
            noaa_id TEXT NOT NULL,
            year INTEGER,
            month INTEGER,
            day INTEGER,
            hour INTEGER,
            min INTEGER,
            value DECIMAL(10,3),
            type TEXT NOT NULL,
            UNIQUE(noaa_id, year, month, day, hour, min) ON CONFLICT REPLACE
        )";
    conn.execute(create_predictions_table, ())?;
    let _ = conn.close();
    Ok(())
}
