use std::{sync::mpsc, thread};

use anyhow::anyhow;
use rusqlite::Connection;

use crate::{
    datastore::{DatastoreMessage, Error},
    domain::User,
};

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

const MIGRATION: &str = "
CREATE TABLE users (
    id CHAR(27) PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
";

fn prepare_connection(conn: &Connection) -> Result<(), Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

pub struct WorkerPool {}

pub fn worker_pool() -> Result<(WorkerPool, Vec<mpsc::Sender<DatastoreMessage>>), Error> {
    let path = "mise.db";
    let mut conn = Connection::open(path)?;
    prepare_connection(&conn)?;

    // run migrations
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let user_version: u32 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let desired_version: u32 = 1;
    // TODO - be able to run multiple migrations
    // TODO - handle if migrations have some version problems
    if user_version < desired_version {
        // need to run migrations
        tx.execute(MIGRATION, ())?;
        tx.pragma_update(None, "user_version", desired_version)?;
    }
    tx.commit()?;

    let mut senders: Vec<mpsc::Sender<DatastoreMessage>> = Vec::new();

    for _ in 0..5 {
        let (_, sender) = ThreadWorker::new()?;
        senders.push(sender);
    }

    Ok((WorkerPool {}, senders))
}

struct ThreadWorker {}

impl ThreadWorker {
    fn new() -> Result<(Self, mpsc::Sender<DatastoreMessage>), Error> {
        let (sender, receiver) = mpsc::channel();

        let path = "mise.db";
        let conn = Connection::open(path)?;
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
                    DatastoreMessage::InsertUser {
                        respond_to,
                        id,
                        username,
                    } => {
                        let _ = respond_to.send(insert_user(&conn, &id, &username));
                    }
                }
            }
        });

        Ok((ThreadWorker {}, sender))
    }
}

fn insert_user(conn: &Connection, id: &str, username: &str) -> Result<(), Error> {
    let q = "INSERT INTO users (id,username) VALUES (?1,?2)";

    let mut stmt = conn.prepare_cached(q)?;
    stmt.execute([id, username])?;
    Ok(())
}

fn get_user(conn: &Connection, id: &str) -> Result<User, Error> {
    let q = "SELECT * FROM users WHERE id = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    let user = stmt.query_row([id], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(user)
}