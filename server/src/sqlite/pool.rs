use std::{cmp::Ordering, sync::mpsc, thread};

use anyhow::anyhow;
use rusqlite::Connection;

use crate::{
    datastore::{Error, Message},
    domain::{RegisteringUser, User},
};

use super::{image, recipe, tag};

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound,
            _ => Error::Unknown(value.into()),
        }
    }
}

const MIGRATION: [&str; 8] = [
    // -- Add users table.
    "
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    oauth_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);",
    // -- Add images table.
    "
CREATE TABLE images (
    id TEXT PRIMARY KEY
);",
    // -- Add recipes tables.
    "
CREATE TABLE recipes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    image_id INTEGER,
    document BLOB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (image_id) REFERENCES images (id) ON DELETE RESTRICT
);",
    "
CREATE TABLE recipe_revisions (
    id INTEGER PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    patch BLOB,
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (recipe_id, revision),
    FOREIGN KEY (created_by_user_id) REFERENCES users (id) ON DELETE RESTRICT,
    FOREIGN KEY (recipe_id) REFERENCES recipes (id) ON DELETE CASCADE
);",
    // -- Add tags tables.
    "
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_by_user_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name)
);",
    "
CREATE TABLE recipe_tags (
    id INTEGER PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    UNIQUE (recipe_id, tag_id),
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes (id) ON DELETE CASCADE
);",
    // -- Add tag groups tables.
    "
CREATE TABLE tag_groups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_multi_select BOOL NOT NULL,
    is_required BOOL NOT NULL,
    color TEXT NOT NULL,
    auto_classifier TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);",
    // -- Add description and group columns to tags.
    "
    ALTER TABLE tags RENAME TO tags_old;
    CREATE TABLE tags (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        group_id TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (group_id) REFERENCES tag_groups (id) ON DELETE SET NULL,
    );
    INSERT INTO tags (id, name, created_by_user_id, created_at)
    SELECT id, name, created_at FROM tags_old;
    DROP TABLE tags_old;
",
];

fn prepare_connection(conn: &Connection) -> Result<(), Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

pub struct DatastoreHandler {}

#[derive(Debug, Clone)]
pub struct DatastoreConfig {
    pub recipe_page_size: u64,
    pub recipe_dump_page_size: u64,
}

pub fn datastore_handler(
    path: &str,
    config: &DatastoreConfig,
) -> Result<(DatastoreHandler, Vec<mpsc::Sender<Message>>), Error> {
    let mut conn = Connection::open(path)?;
    prepare_connection(&conn)?;

    // run migrations
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let user_version: usize = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let desired_version = MIGRATION.len();
    match user_version.cmp(&desired_version) {
        Ordering::Less => {
            // need to run migrations
            for query in &MIGRATION[user_version..] {
                tx.execute(query, ())?;
            }
            tx.pragma_update(None, "user_version", desired_version)?;
        }
        Ordering::Equal => {}
        Ordering::Greater => {
            return Err(Error::Unknown(anyhow!(
                "Database at unknown migration: {}",
                user_version
            )));
        }
    }

    tx.commit()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    let mut senders: Vec<mpsc::Sender<Message>> = Vec::new();

    for _ in 0..5 {
        let (_, sender) = ThreadWorker::new(path, config)?;
        senders.push(sender);
    }

    Ok((DatastoreHandler {}, senders))
}

struct ThreadWorker {}

impl ThreadWorker {
    fn new(path: &str, config: &DatastoreConfig) -> Result<(Self, mpsc::Sender<Message>), Error> {
        let (sender, receiver) = mpsc::channel();

        let recipe_page_size = config.recipe_page_size;
        let recipe_dump_page_size = config.recipe_dump_page_size;

        let mut conn = Connection::open(path)?;
        prepare_connection(&conn)?;

        thread::spawn(move || {
            while let Ok(msg) = receiver.recv() {
                match msg {
                    Message::Health { respond_to } => {
                        // TODO - check health
                        let _ = respond_to.send(Ok(()));
                    }
                    Message::GetUser { respond_to, id } => {
                        let _ = respond_to.send(get_user(&conn, &id));
                    }
                    Message::UpsertUserByOauthId { respond_to, user } => {
                        let _ = respond_to.send(upsert_user_by_oauth_id(&conn, &user));
                    }
                    Message::GetRecipe { id, respond_to } => {
                        let _ = respond_to.send(recipe::get(&conn, &id));
                    }
                    Message::ListRecipes {
                        filter,
                        cursor,
                        respond_to,
                    } => {
                        let _ = respond_to.send(recipe::list_recipes(
                            &conn,
                            recipe_page_size,
                            &filter,
                            cursor,
                        ));
                    }
                    Message::CreateRecipe {
                        id,
                        user_id,
                        recipe,
                        respond_to,
                    } => {
                        let _ = respond_to.send(recipe::insert(&mut conn, &id, &user_id, recipe));
                    }
                    Message::UpdateRecipe {
                        id,
                        user_id,
                        recipe,
                        current_hash,
                        respond_to,
                    } => {
                        let _ = respond_to.send(recipe::update(
                            &mut conn,
                            &id,
                            &user_id,
                            recipe,
                            &current_hash,
                        ));
                    }
                    Message::GetRevisions {
                        recipe_id,
                        respond_to,
                    } => {
                        let _ = respond_to.send(recipe::get_revisions(&conn, &recipe_id));
                    }
                    Message::GetRevision {
                        recipe_id,
                        revision,
                        respond_to,
                    } => {
                        let _ = respond_to.send(recipe::get_revision(&conn, &recipe_id, revision));
                    }
                    Message::DumpRecipesForIndex { cursor, respond_to } => {
                        let _ = respond_to.send(recipe::dump_recipes_for_index(
                            &conn,
                            recipe_dump_page_size,
                            cursor,
                        ));
                    }
                    Message::GetTags { respond_to } => {
                        let _ = respond_to.send(tag::get_all(&conn));
                    }
                    Message::GetTagsWithStats { respond_to } => {
                        let _ = respond_to.send(tag::get_all_with_stats(&conn));
                    }
                    Message::CreateTag {
                        user_id,
                        name,
                        respond_to,
                    } => {
                        let _ = respond_to.send(tag::insert(&conn, &user_id, &name));
                    }
                    Message::CreateImage { id, respond_to } => {
                        let _ = respond_to.send(image::insert(&conn, &id));
                    }
                    Message::GetImage { id, respond_to } => {
                        let _ = respond_to.send(image::get_image(&conn, &id));
                    }
                }
            }
        });

        Ok((ThreadWorker {}, sender))
    }
}

fn upsert_user_by_oauth_id(
    conn: &Connection,
    registering: &RegisteringUser,
) -> Result<User, Error> {
    let q = "INSERT INTO users (id,oauth_id,name) VALUES (?1,?2,?3) ON CONFLICT (oauth_id) DO UPDATE SET name = ?3";

    let mut stmt = conn.prepare_cached(q)?;
    stmt.execute([
        &registering.potential_id,
        &registering.oauth_id,
        &registering.name,
    ])?;

    // fetch user back from the database so that the id is known
    let mut stmt = conn.prepare_cached("SELECT * FROM users WHERE oauth_id = ?1")?;
    let user = stmt.query_row([&registering.oauth_id], |row| {
        Ok(User {
            id: row.get(0)?,
            oauth_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?;
    Ok(user)
}

fn get_user(conn: &Connection, id: &str) -> Result<User, Error> {
    let q = "SELECT * FROM users WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let user = stmt.query_row([id], |row| {
        Ok(User {
            id: row.get(0)?,
            oauth_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?;
    Ok(user)
}
