use rusqlite::{params, Connection};

use crate::{
    datastore::Error,
    domain::{self},
};

pub fn get(conn: &Connection, id: &str) -> Result<domain::HashedRecipeDocument, Error> {
    let q = "SELECT document FROM recipes WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let serialized_document: Vec<u8> = stmt.query_row([id], |row| Ok(row.get(0)?))?;

    Ok(domain::HashedRecipeDocument {
        document: postcard::from_bytes(&serialized_document)
            .map_err(|err| Error::Unknown(err.into()))?,
        hash: sha256::digest(&serialized_document),
    })
}

pub fn insert(
    conn: &mut Connection,
    id: &str,
    recipe: &domain::RecipeDocument,
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
            tx.prepare_cached("INSERT INTO recipe_revisions (recipe_id, revision) VALUES (?1,?2)")?;
        stmt.execute(params![id, 0])?;
    }

    tx.commit()?;

    Ok(())
}

pub fn update(
    conn: &mut Connection,
    id: &str,
    recipe: &domain::RecipeDocument,
    current_hash: &str,
) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    {
        let current_document = get(&tx, id)?;
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
        let patch_count: u64 = stmt.query_row(params![id], |row| Ok(row.get(0)?))?;

        // create patch to convert from new -> old
        let patch = diff(&new_serialized_document, &current_serialized_document)?;

        // save recipe change
        let mut stmt = tx.prepare_cached("UPDATE recipes SET title=?2, document=?3 WHERE id=?1")?;
        stmt.execute(params![id, &recipe.title, new_serialized_document])?;

        // save patch
        let mut stmt = tx.prepare_cached(
            "INSERT INTO recipe_revisions (recipe_id, revision, patch) VALUES (?1,?2,?3)",
        )?;
        stmt.execute(params![id, patch_count, patch])?;
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
) -> Result<domain::HashedRecipeDocument, Error> {
    // get current recipe
    let q = "SELECT document FROM recipes WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let current_document: Vec<u8> = stmt.query_row([recipe_id], |row| {
        Ok(row.get_ref(0)?.as_bytes()?.to_owned())
    })?;

    // get patches
    let q = "SELECT patch FROM recipe_revisions WHERE recipe_id = ?1 AND patch IS NOT NULL ORDER BY revision DESC";

    let mut stmt = conn.prepare_cached(q)?;
    let rows = stmt.query_map([recipe_id], |row| Ok(row.get(0)?))?;
    let patches: Vec<Vec<u8>> = rows
        .map(|row| row.map_err(|err| Error::Unknown(err.into())))
        .collect::<Result<Vec<Vec<u8>>, Error>>()?;
    let patch_count = patches.len(); // this excludes revision 0, so it is only patches

    // requested revision must exist
    if revision > patch_count {
        return Err(Error::NotFound);
    }

    // apply patches from newest to oldest until requested revision is found
    let mut document: Vec<u8> = current_document;
    for patch in patches.iter().take(patch_count - revision) {
        document = apply_patch(&document, patch)?;
    }

    Ok(domain::HashedRecipeDocument {
        document: postcard::from_bytes(&document).map_err(|err| Error::Unknown(err.into()))?,
        hash: sha256::digest(&document),
    })
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
