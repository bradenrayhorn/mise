use anyhow::Context;
use rusqlite::{params, Connection};

use crate::{
    datastore::{Error, HashedRecipeDocument, RecipeDocument, VersionedRecipeDocument},
    domain::{self, ListedRecipe},
};

pub fn get(conn: &Connection, id: &str) -> Result<domain::Recipe, Error> {
    let hashed_document = get_document(conn, id)?;
    let document = hashed_document.document;

    Ok(domain::Recipe {
        id: id.try_into()?,
        hash: hashed_document.hash,
        title: document.title.try_into()?,
        image_id: document.image_id,
        ingredients: document
            .ingredients
            .into_iter()
            .map(domain::recipe::IngredientBlock::try_from)
            .collect::<Result<Vec<domain::recipe::IngredientBlock>, domain::ValidationError>>()?,
        instructions: document
            .instructions
            .into_iter()
            .map(domain::recipe::InstructionBlock::try_from)
            .collect::<Result<Vec<domain::recipe::InstructionBlock>, domain::ValidationError>>()?,
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
    recipe: RecipeDocument,
) -> Result<(), Error> {
    let versioned_recipe = VersionedRecipeDocument::from(recipe);

    let serialized_document =
        postcard::to_allocvec(&versioned_recipe).map_err(|err| Error::Unknown(err.into()))?;

    let recipe = RecipeDocument::from(versioned_recipe);

    let tx = conn.transaction()?;

    {
        // create recipe
        let mut stmt = tx.prepare_cached(
            "INSERT INTO recipes (id,title,image_id,document) VALUES (?1,?2,?3,?4)",
        )?;
        stmt.insert(params![
            id,
            recipe.title,
            recipe.image_id.as_ref().map(String::from),
            serialized_document
        ])?;

        // create revision
        let mut stmt =
            tx.prepare_cached("INSERT INTO recipe_revisions (recipe_id, revision, created_by_user_id) VALUES (?1,?2,?3)")?;
        stmt.execute(params![id, 0, user_id])?;

        // create tags
        update_tags_for_recipe(&tx, id, recipe.tag_ids)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn update(
    conn: &mut Connection,
    id: &str,
    user_id: &str,
    recipe: RecipeDocument,
    current_hash: &str,
) -> Result<(), Error> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    {
        let current_document = get_document(&tx, id)?;
        let current_serialized_document =
            postcard::to_allocvec(&VersionedRecipeDocument::from(current_document.document))
                .map_err(|err| Error::Unknown(err.into()))?;

        let versioned_recipe = VersionedRecipeDocument::from(recipe);
        let new_serialized_document =
            postcard::to_allocvec(&versioned_recipe).map_err(|err| Error::Unknown(err.into()))?;
        let recipe = RecipeDocument::from(versioned_recipe);

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
        let mut stmt =
            tx.prepare_cached("UPDATE recipes SET title=?2, document=?3, image_id=?4 WHERE id=?1")?;
        stmt.execute(params![
            id,
            &recipe.title,
            new_serialized_document,
            recipe.image_id.as_ref().map(String::from),
        ])?;

        // save patch
        let mut stmt = tx.prepare_cached(
            "INSERT INTO recipe_revisions (recipe_id, revision, patch, created_by_user_id) VALUES (?1,?2,?3,?4)",
        )?;
        stmt.execute(params![id, patch_count, patch, user_id])?;

        // update tags
        update_tags_for_recipe(&tx, id, recipe.tag_ids)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn dump_recipes_for_index(
    conn: &Connection,
    page_size: u64,
    cursor: Option<domain::page::cursor::DumpedIndexableRecipe>,
) -> Result<domain::page::DumpedIndexableRecipe, Error> {
    struct QueryResult {
        id: domain::recipe::Id,
        document: Vec<u8>,
    }

    let query =
        format!("SELECT id,document FROM recipes WHERE id > ? ORDER BY id ASC LIMIT {page_size}");

    let mut stmt = conn.prepare_cached(&query)?;
    let result = stmt.query_and_then(params![cursor.map_or(String::new(), |c| c.id)], |row| {
        Ok(QueryResult {
            id: (row.get::<_, String>("id")?.as_str()).try_into()?,
            document: row.get("document")?,
        })
    })?;

    let results: Result<Vec<QueryResult>, Error> = result.collect();
    let results = results?;

    let recipes: Result<Vec<domain::DumpedIndexableRecipe>, Error> = results
        .into_iter()
        .map(|result| -> Result<domain::DumpedIndexableRecipe, Error> {
            let versioned_document: VersionedRecipeDocument =
                postcard::from_bytes(&result.document).map_err(|err| Error::Unknown(err.into()))?;
            let document: RecipeDocument = versioned_document.into();

            Ok(RecipeDocument::to_dumped_indexable_recipe(
                result.id, document,
            )?)
        })
        .collect();
    let recipes = recipes?;

    let last =
        if recipes.len() == usize::try_from(page_size).map_err(|err| Error::Unknown(err.into()))? {
            recipes.last()
        } else {
            None
        };

    Ok(domain::page::DumpedIndexableRecipe {
        next: last.map(|last| domain::page::cursor::DumpedIndexableRecipe {
            id: last.id.to_string(),
        }),
        items: recipes,
    })
}

pub fn list_recipes(
    conn: &Connection,
    page_size: u64,
    filter: &domain::filter::Recipe,
    cursor: Option<domain::page::cursor::Recipe>,
) -> Result<domain::page::Recipe, Error> {
    let tag_count =
        u64::try_from(filter.tag_ids.len()).context("could not convert vec len to u64")?;

    let mut params = query::Params::new();

    // WHERE
    let mut wheres: Vec<String> = vec![];

    if let Some(cursor) = cursor {
        params.push(query::Param::String(cursor.name.clone()));
        params.push(query::Param::String(cursor.name.clone()));
        params.push(query::Param::String(cursor.id));
        wheres.push("(recipes.title > ? OR (recipes.title = ? AND recipes.id > ?))".into());
    }

    if !filter.tag_ids.is_empty() {
        for tag_id in &filter.tag_ids {
            let tag_id = String::from(tag_id);
            params.push(query::Param::String(tag_id));
        }

        let param_string = query::param_string(filter.tag_ids.len());
        wheres.push(format!("recipe_tags.tag_id IN ({param_string})"));
    }

    let where_clause = if wheres.is_empty() {
        ""
    } else {
        &format!("WHERE {}", wheres.join(" AND "))
    };

    // HAVING
    let having_clause = if tag_count > 0 {
        params.push(query::Param::U64(tag_count));

        "HAVING count(recipes.id) = ?"
    } else {
        ""
    };

    let query = format!(
        "
        SELECT recipes.id, recipes.title, recipes.image_id
        FROM recipes
        LEFT JOIN recipe_tags ON recipes.id = recipe_tags.recipe_id
        {where_clause}
        GROUP BY recipes.id, recipes.title
        {having_clause}
        ORDER BY recipes.title ASC, recipes.id ASC
        LIMIT {page_size}
        ;",
    );

    let mut stmt = conn.prepare_cached(&query)?;
    let result = stmt.query_and_then(&*params.to_params(), |row| {
        Ok(domain::ListedRecipe {
            id: (row.get::<_, String>("id")?.as_str()).try_into()?,
            title: (row.get::<_, String>("title")?).try_into()?,
            image_id: match row.get::<_, Option<String>>("image_id")? {
                None => None,
                Some(id) => Some(id.as_str().try_into()?),
            },
        })
    })?;

    let recipes: Result<Vec<ListedRecipe>, Error> = result.collect();
    let recipes = recipes?;
    let last =
        if recipes.len() == usize::try_from(page_size).map_err(|err| Error::Unknown(err.into()))? {
            recipes.last()
        } else {
            None
        };

    Ok(domain::page::Recipe {
        next: last.map(|last| domain::page::cursor::Recipe {
            id: last.id.to_string(),
            name: last.title.clone().into(),
        }),
        items: recipes,
    })
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
    let versioned_document: VersionedRecipeDocument =
        postcard::from_bytes(&serialized_document).map_err(|err| Error::Unknown(err.into()))?;
    let document: RecipeDocument = versioned_document.into();
    let hash = sha256::digest(&serialized_document);

    Ok(domain::Recipe {
        id: recipe_id.try_into()?,
        hash,
        title: document.title.try_into()?,
        image_id: document.image_id,
        ingredients: document
            .ingredients
            .into_iter()
            .map(domain::recipe::IngredientBlock::try_from)
            .collect::<Result<Vec<domain::recipe::IngredientBlock>, domain::ValidationError>>()?,
        instructions: document
            .instructions
            .into_iter()
            .map(domain::recipe::InstructionBlock::try_from)
            .collect::<Result<Vec<domain::recipe::InstructionBlock>, domain::ValidationError>>()?,
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

    let versioned_document: VersionedRecipeDocument =
        postcard::from_bytes(&serialized_document).map_err(|err| Error::Unknown(err.into()))?;
    let document: RecipeDocument = versioned_document.into();
    let hash = sha256::digest(&serialized_document);

    Ok(HashedRecipeDocument { document, hash })
}

fn get_tags_for_recipe(
    conn: &Connection,
    tag_ids: Vec<domain::tag::Id>,
) -> Result<Vec<domain::tag::OnRecipe>, Error> {
    let tag_ids: Vec<String> = tag_ids.into_iter().map(Into::<String>::into).collect();

    let query = format!(
        "SELECT id, name FROM tags WHERE id IN ({}) ORDER BY name ASC",
        query::param_string(tag_ids.len())
    );

    let mut stmt = conn.prepare_cached(&query)?;

    let result = stmt.query_and_then(rusqlite::params_from_iter(tag_ids), |row| {
        let id: String = row.get("id")?;
        let name: String = row.get("name")?;
        Ok(domain::tag::OnRecipe {
            id: id.as_str().try_into()?,
            name: name.try_into()?,
        })
    })?;

    result.collect()
}

fn update_tags_for_recipe(
    conn: &Connection,
    recipe_id: &str,
    tag_ids: Vec<domain::tag::Id>,
) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached("DELETE FROM recipe_tags WHERE recipe_id = ?1")?;
    stmt.execute(params![recipe_id])?;

    let tag_ids: Vec<String> = tag_ids.into_iter().map(Into::<String>::into).collect();
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
    use rusqlite::ToSql;

    pub struct Params(Vec<Param>);

    #[derive(Clone, Debug)]
    pub enum Param {
        U64(u64),
        String(String),
    }

    impl Params {
        pub fn new() -> Params {
            Params(vec![])
        }

        pub fn push(&mut self, param: Param) {
            self.0.push(param);
        }

        pub fn to_params(&self) -> Vec<&dyn ToSql> {
            self.0
                .iter()
                .map(|value| {
                    let v: &dyn ToSql = value;
                    v
                })
                .collect()
        }
    }

    pub fn param_string(count: usize) -> String {
        let mut param_string = "?,".repeat(count);
        param_string.pop();
        param_string
    }

    impl ToSql for Param {
        fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
            match &self {
                Param::U64(value) => value.to_sql(),
                Param::String(value) => value.to_sql(),
            }
        }
    }
}
