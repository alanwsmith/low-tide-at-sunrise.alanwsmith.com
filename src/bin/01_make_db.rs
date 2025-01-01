use rusqlite::Connection;
use rusqlite::Result;

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let create_table = "
        CREATE TABLE IF NOT EXISTS stations (
            db_id INTEGER PRIMARY KEY, 
            noaa_id TEXT NOT NULL UNIQUE,
            name TEXT,
            lat INTEGER,
            long INTEGER,
            state TEXT,
            tz_offset INTEGER
        )";
    conn.execute(create_table, ())?;
    Ok(())
}
