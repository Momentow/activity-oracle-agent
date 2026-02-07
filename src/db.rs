use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("activity_log.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY,
            app_name TEXT,
            window_title TEXT,
            start_time TEXT,
            end_time TEXT
        )",
        [],
    )?;
    Ok(conn)
}