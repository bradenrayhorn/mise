use std::str::FromStr;

use askama::Template;
use rusqlite::{params, Connection};

use crate::{
    datastore::{Error, HashedRecipeDocument, RecipeDocument},
    domain,
};

pub fn get(conn: &Connection, id: &str) -> Result<domain::Recipe, Error> {
    let hashed_document = get_document(conn, id)?;
    let document = hashed_document.document;

    Ok(domain::Recipe {
        id: uuid::Uuid::from_str(id)?,
        hash: hashed_document.hash,
        title: document.title.try_into()?,
        ingredients: document.ingredients.try_into()?,
        instructions: document.instructions.try_into()?,
        notes: match document.notes {
            None => None,
            Some(s) => Some(s.try_into()?),
        },
        tags: get_tags_for_recipe(conn, document.tag_ids)?,
    })
}

pub fn insert(
    conn: &mut Connection,
    id: &str,
    user_id: &str,
    recipe: &RecipeDocument,
) -> Result<(), Error> {
    let serialized_document =
        postcard::to_allocvec(recipe).map_err(|err| Error::Unknown(err.into()))?;

    let tx = conn.transaction()?;

    {
        // create recipe
        let mut stmt =
            tx.prepare_cached("INSERT INTO recipes (id,title,document) VALUES (?1,?2,?3)")?;
        stmt.insert(params![id, recipe.title, serialized_document])?;

        // create revision
        let mut stmt =
            tx.prepare_cached("INSERT INTO recipe_revisions (recipe_id, revision, created_by_user_id) VALUES (?1,?2,?3)")?;
        stmt.execute(params![id, 0, user_id])?;

        // create tags
        update_tags_for_recipe(&tx, id, &recipe.tag_ids)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn update(
    conn: &mut Connection,
    id: &str,
    user_id: &str,
    recipe: &RecipeDocument,
    current_hash: &str,
) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    {
        let current_document = get_document(&tx, id)?;
        let current_serialized_document = postcard::to_allocvec(&current_document.document)
            .map_err(|err| Error::Unknown(err.into()))?;

        let new_serialized_document =
            postcard::to_allocvec(recipe).map_err(|err| Error::Unknown(err.into()))?;

        // validate current document has not changed
        if current_document.hash != current_hash {
            return Err(Error::Conflict);
        }

        // get current patch revision
        let mut stmt =
            tx.prepare_cached("SELECT COUNT(*) FROM recipe_revisions WHERE recipe_id = ?1")?;
        let patch_count: u64 = stmt.query_row(params![id], |row| row.get(0))?;

        // create patch to convert from new -> old
        let patch = diff(&new_serialized_document, &current_serialized_document)?;

        // save recipe change
        let mut stmt = tx.prepare_cached("UPDATE recipes SET title=?2, document=?3 WHERE id=?1")?;
        stmt.execute(params![id, &recipe.title, new_serialized_document])?;

        // save patch
        let mut stmt = tx.prepare_cached(
            "INSERT INTO recipe_revisions (recipe_id, revision, patch, created_by_user_id) VALUES (?1,?2,?3,?4)",
        )?;
        stmt.execute(params![id, patch_count, patch, user_id])?;

        // update tags
        update_tags_for_recipe(&tx, id, &recipe.tag_ids)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn get_revisions(
    conn: &Connection,
    recipe_id: &str,
) -> Result<Vec<domain::RecipeRevision>, Error> {
    let q = "SELECT revision FROM recipe_revisions WHERE recipe_id = ?1 ORDER BY revision DESC";

    let mut stmt = conn.prepare_cached(q)?;
    let result = stmt.query_and_then([recipe_id], |row| {
        Ok(domain::RecipeRevision {
            revision: row.get("revision")?,
        })
    })?;
    result.collect()
}

pub fn get_revision(
    conn: &Connection,
    recipe_id: &str,
    revision: usize,
) -> Result<domain::Recipe, Error> {
    // get current recipe
    let q = "SELECT document FROM recipes WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let current_document: Vec<u8> = stmt.query_row([recipe_id], |row| {
        Ok(row.get_ref(0)?.as_bytes()?.to_owned())
    })?;

    // get patches
    let q = "SELECT patch FROM recipe_revisions WHERE recipe_id = ?1 AND patch IS NOT NULL ORDER BY revision DESC";

    let mut stmt = conn.prepare_cached(q)?;
    let rows = stmt.query_map([recipe_id], |row| row.get(0))?;
    let patches: Vec<Vec<u8>> = rows
        .map(|row| row.map_err(|err| Error::Unknown(err.into())))
        .collect::<Result<Vec<Vec<u8>>, Error>>()?;
    let patch_count = patches.len(); // this excludes revision 0, so it is only patches

    // requested revision must exist
    if revision > patch_count {
        return Err(Error::NotFound);
    }

    // apply patches from newest to oldest until requested revision is found
    let mut serialized_document: Vec<u8> = current_document;
    for patch in patches.iter().take(patch_count - revision) {
        serialized_document = apply_patch(&serialized_document, patch)?;
    }

    // turn into a domain recipe
    let document: RecipeDocument =
        postcard::from_bytes(&serialized_document).map_err(|err| Error::Unknown(err.into()))?;
    let hash = sha256::digest(&serialized_document);

    Ok(domain::Recipe {
        id: uuid::Uuid::from_str(recipe_id)?,
        hash,
        title: document.title.try_into()?,
        ingredients: document.ingredients.try_into()?,
        instructions: document.instructions.try_into()?,
        notes: match document.notes {
            None => None,
            Some(s) => Some(s.try_into()?),
        },
        tags: get_tags_for_recipe(conn, document.tag_ids)?,
    })
}

fn get_document(conn: &Connection, id: &str) -> Result<HashedRecipeDocument, Error> {
    let q = "SELECT document FROM recipes WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let serialized_document: Vec<u8> = stmt.query_row([id], |row| row.get(0))?;

    let document: RecipeDocument =
        postcard::from_bytes(&serialized_document).map_err(|err| Error::Unknown(err.into()))?;
    let hash = sha256::digest(&serialized_document);

    Ok(HashedRecipeDocument { document, hash })
}

fn get_tags_for_recipe(
    conn: &Connection,
    tag_ids: Vec<i64>,
) -> Result<Vec<domain::tag::OnRecipe>, Error> {
    let query = query::GetTagsForRecipe { ids: &tag_ids }.render()?;
    let mut stmt = conn.prepare_cached(&query)?;

    let result = stmt.query_and_then(rusqlite::params_from_iter(tag_ids), |row| {
        Ok(domain::tag::OnRecipe {
            id: row.get("id")?,
            name: (row.get::<_, String>("name")?).try_into()?,
        })
    })?;

    result.collect()
}

fn update_tags_for_recipe(
    conn: &Connection,
    recipe_id: &str,
    tag_ids: &[i64],
) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached("DELETE FROM recipe_tags WHERE recipe_id = ?1")?;
    stmt.execute(params![recipe_id])?;

    for tag_id in tag_ids {
        let mut stmt =
            conn.prepare_cached("INSERT INTO recipe_tags (recipe_id, tag_id) VALUES (?1,?2)")?;
        stmt.execute(params![recipe_id, tag_id])?;
    }

    Ok(())
}

fn diff(old: &[u8], new: &[u8]) -> Result<Vec<u8>, Error> {
    let diff_params = bidiff::DiffParams::new(2, None).unwrap();
    let mut patch = vec![];
    bidiff::simple_diff_with_params(old, new, &mut patch, &diff_params)
        .map_err(|err| Error::Unknown(err.into()))?;

    zstd::bulk::compress(&patch, 0).map_err(|err| Error::Unknown(err.into()))
}

fn apply_patch(current: &[u8], compressed_patch: &[u8]) -> Result<Vec<u8>, Error> {
    let mut patch = vec![];
    zstd::stream::copy_decode(std::io::Cursor::new(compressed_patch), &mut patch)
        .map_err(|err| Error::Unknown(err.into()))?;
    let mut reader =
        bipatch::Reader::new(std::io::Cursor::new(patch), std::io::Cursor::new(current))
            .map_err(|err| Error::Unknown(err.into()))?;
    let mut patched_text = vec![];
    std::io::copy(&mut reader, &mut patched_text).map_err(|err| Error::Unknown(err.into()))?;

    Ok(patched_text)
}

mod query {
    use askama::Template;

    #[derive(Template)]
    #[template(
        ext = "txt",
        source = "SELECT id,name FROM tags WHERE id IN (
            {% for id in ids %}?{% if !loop.last %},{% endif %}{% endfor %}
            ) ORDER BY name ASC"
    )]
    pub struct GetTagsForRecipe<'a> {
        pub ids: &'a [i64],
    }
}
