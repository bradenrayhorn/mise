use std::{sync::mpsc, thread};

use anyhow::anyhow;
use rusqlite::Connection;

use crate::{
    datastore::{DatastoreMessage, Error},
    domain::{RegisteringUser, User},
};

use super::recipe;

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound,
            _ => Error::Unknown(value.into()),
        }
    }
}

const MIGRATION: [&'static str; 2] = [
    "
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    oauth_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);",
    "
CREATE TABLE recipes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    document TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);",
];

fn prepare_connection(conn: &Connection) -> Result<(), Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

pub struct WorkerPool {}

pub fn worker_pool(
    path: String,
) -> Result<(WorkerPool, Vec<mpsc::Sender<DatastoreMessage>>), Error> {
    let mut conn = Connection::open(path.clone())?;
    prepare_connection(&conn)?;

    // run migrations
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let user_version: usize = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let desired_version = MIGRATION.len();
    if user_version < desired_version {
        // need to run migrations
        for query in &MIGRATION[user_version..] {
            tx.execute(query, ())?;
        }
        tx.pragma_update(None, "user_version", desired_version)?;
    } else {
        return Err(Error::Unknown(anyhow!(
            "Database at unknown migration: {}",
            user_version
        )));
    }
    tx.commit()?;

    let mut senders: Vec<mpsc::Sender<DatastoreMessage>> = Vec::new();

    for _ in 0..5 {
        let (_, sender) = ThreadWorker::new(path.clone())?;
        senders.push(sender);
    }

    Ok((WorkerPool {}, senders))
}

struct ThreadWorker {}

impl ThreadWorker {
    fn new(path: String) -> Result<(Self, mpsc::Sender<DatastoreMessage>), Error> {
        let (sender, receiver) = mpsc::channel();

        let mut conn = Connection::open(path)?;
        prepare_connection(&conn)?;

        thread::spawn(move || {
            while let Ok(msg) = receiver.recv() {
                match msg {
                    DatastoreMessage::Health { respond_to } => {
                        // TODO - check health
                        let _ = respond_to.send(Ok(()));
                    }
                    DatastoreMessage::GetUser { respond_to, id } => {
                        let _ = respond_to.send(get_user(&conn, &id));
                    }
                    DatastoreMessage::UpsertUserByOauthId { respond_to, user } => {
                        let _ = respond_to.send(upsert_user_by_oauth_id(&conn, &user));
                    }
                    DatastoreMessage::GetRecipe { id, respond_to } => {
                        let _ = respond_to.send(recipe::get(&conn, &id));
                    }
                    DatastoreMessage::CreateRecipe { recipe, respond_to } => {
                        let _ = respond_to.send(recipe::insert(&mut conn, recipe));
                    }
                    DatastoreMessage::UpdateRecipe { recipe, respond_to } => {
                        let _ = respond_to.send(recipe::update(&mut conn, recipe));
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
