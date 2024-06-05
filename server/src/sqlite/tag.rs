use rusqlite::{params, Connection};

use crate::datastore::{Error, Tag};

pub fn get_all(conn: &Connection) -> Result<Vec<Tag>, Error> {
    let q = "SELECT id,name FROM tags ORDER BY name ASC";

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    result.collect()
}

pub fn insert(conn: &Connection, user_id: &str, name: &str) -> Result<i64, Error> {
    let mut stmt =
        conn.prepare_cached("INSERT INTO tags (name,created_by_user_id) VALUES (?1,?2)")?;
    stmt.insert(params![name, user_id])?;

    Ok(conn.last_insert_rowid())
}
