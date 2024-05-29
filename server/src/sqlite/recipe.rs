use rusqlite::{params, Connection};

use crate::{
    datastore::Error,
    domain::{self},
};

pub fn get(conn: &Connection, id: &str) -> Result<domain::RecipeDocument, Error> {
    let q = "SELECT id,title,document FROM recipes WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    stmt.query_row([id], |row| {
        Ok(domain::RecipeDocument {
            id: row.get(0)?,
            title: row.get(1)?,
            document: row.get(2)?,
        })
    })
    .map_err(|err| err.into())
}

pub fn insert(conn: &mut Connection, recipe: domain::RecipeDocument) -> Result<(), Error> {
    let q = "INSERT INTO recipes (id,title,document) VALUES (?1,?2,?3)";
    let mut stmt = conn.prepare_cached(q)?;
    stmt.insert(params![recipe.id, recipe.title, recipe.document])?;

    Ok(())
}

pub fn update(conn: &mut Connection, recipe: domain::RecipeDocument) -> Result<(), Error> {
    let q = "UPDATE recipes SET title=?2, document=?3 WHERE id=?1";
    let mut stmt = conn.prepare_cached(q)?;
    stmt.execute(params![recipe.id, recipe.title, recipe.document])?;

    Ok(())
}
