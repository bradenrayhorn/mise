use rusqlite::{Connection, params};

use crate::{datastore::Error, domain};

pub fn get_all(conn: &Connection) -> Result<Vec<domain::tag::Tag>, Error> {
    let q = "SELECT id,name FROM tags ORDER BY name ASC";

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        Ok(domain::tag::Tag {
            id: id.as_str().try_into()?,
            name: name.try_into()?,
        })
    })?;
    result.collect()
}

pub fn insert(conn: &Connection, user_id: &str, name: &str) -> Result<domain::tag::Id, Error> {
    let id = domain::tag::Id::new();
    let id_string: String = id.clone().into();
    let mut stmt =
        conn.prepare_cached("INSERT INTO tags (id,name,created_by_user_id) VALUES (?1,?2,?3)")?;
    stmt.insert(params![id_string, name, user_id])?;

    Ok(id)
}
