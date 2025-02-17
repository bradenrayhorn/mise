use rusqlite::{params, Connection};

use crate::datastore::Error;

pub fn insert(conn: &Connection, id: &str) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached("INSERT INTO images (id) VALUES (?1)")?;
    stmt.insert(params![id])?;

    Ok(())
}

pub fn get_image(conn: &Connection, id: &str) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached("SELECT id FROM images WHERE id = ?1")?;
    stmt.query_row(params![id], |_| Ok(()))?;

    Ok(())
}
