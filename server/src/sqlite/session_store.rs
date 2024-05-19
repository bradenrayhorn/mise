use std::{ops::Deref, thread};

use anyhow::anyhow;
use rusqlite::{params, Connection};
use tokio::sync::mpsc;

use crate::{
    domain::{Session, SessionKey},
    session_store::{Error, Message},
};

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::Other(anyhow!(value))
    }
}

const MIGRATION: &str = "
CREATE TABLE sessions (
    key TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    revalidate_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL
);
";

fn prepare_connection(conn: &Connection) -> Result<(), Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(())
}

pub fn session_store(path: String) -> Result<mpsc::Sender<Message>, Error> {
    let mut conn = Connection::open(&path)?;
    prepare_connection(&conn)?;

    // run migration
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let user_version: u32 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let desired_version: u32 = 1;
    if user_version < desired_version {
        tx.execute(MIGRATION, ())?;
        tx.pragma_update(None, "user_version", desired_version)?;
    }
    tx.commit()?;

    // create worker
    let (_, sender) = ThreadWorker::new(path)?;

    Ok(sender)
}

struct ThreadWorker {}

impl ThreadWorker {
    fn new(path: String) -> Result<(Self, mpsc::Sender<Message>), Error> {
        let (sender, mut receiver) = mpsc::channel(8);

        let conn = Connection::open(path)?;
        prepare_connection(&conn)?;

        thread::spawn(move || {
            while let Some(msg) = receiver.blocking_recv() {
                match msg {
                    Message::Get { respond_to, key } => {
                        let _ = respond_to.send(get(&conn, &key));
                    }
                    Message::Delete { key, respond_to } => {
                        let _ = respond_to.send(delete(&conn, &key));
                    }
                    Message::Set {
                        session,
                        respond_to,
                    } => {
                        let _ = respond_to.send(set(&conn, &session));
                    }
                }
            }
        });

        Ok((ThreadWorker {}, sender))
    }
}

fn set(conn: &Connection, session: &Session) -> Result<(), Error> {
    let q = "INSERT INTO sessions (key,user_id,refresh_token,revalidate_at,expires_at)
            VALUES (?1,?2,?3,?4,?5) ON CONFLICT (key) DO UPDATE
            SET user_id=?2, refresh_token=?3, revalidate_at=?4, expires_at=?5";

    let mut stmt = conn.prepare_cached(q)?;
    stmt.execute(params![
        session.key,
        session.user_id,
        session.refresh_token,
        session.revalidate_at,
        session.expires_at
    ])?;

    Ok(())
}

fn get(conn: &Connection, key: &SessionKey) -> Result<Session, Error> {
    let q = "SELECT * FROM sessions WHERE key = ?1 AND expires_at > datetime('now')";

    let mut stmt = conn.prepare_cached(q)?;
    let session = stmt.query_row(params![key.deref()], |row| {
        Ok(Session {
            key: row.get(0)?,
            user_id: row.get(1)?,
            refresh_token: row.get(2)?,
            revalidate_at: row.get(3)?,
            expires_at: row.get(4)?,
        })
    })?;

    Ok(session)
}

fn delete(conn: &Connection, key: &SessionKey) -> Result<(), Error> {
    let q = "DELETE FROM sessions WHERE key = ?1";

    let mut stmt = conn.prepare_cached(q)?;
    stmt.execute([key.deref()])?;

    Ok(())
}
