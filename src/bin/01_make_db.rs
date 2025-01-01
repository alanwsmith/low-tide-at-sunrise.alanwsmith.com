use rusqlite::Connection;
use rusqlite::Result;

fn main() -> Result<()> {
    let conn = Connection::open("./data/data.sqlite")?;
    let create_table = "
        CREATE TABLE IF NOT EXISTS storage (name TEXT, color TEXT)";
    conn.execute(create_table, ())?;
    Ok(())
}
