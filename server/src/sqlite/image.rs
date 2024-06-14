use rusqlite::{params, Connection};

use crate::datastore::Error;

pub fn insert(conn: &Connection, id: &str) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached("INSERT INTO images (id) VALUES (?1)")?;
    stmt.insert(params![id])?;

    Ok(())
}
