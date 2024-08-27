use anyhow::Result;
use chrono::{TimeDelta, Utc};
use mise::{domain, session_store::BackgroundResultMessage};
use rand::{distributions::Alphanumeric, Rng};
use rusqlite::{params, Connection};
use tokio::sync::{mpsc, oneshot};

pub struct TestPool {
    path: String,
}

impl Drop for TestPool {
    fn drop(&mut self) {
        // TODO - shutdown pool
        let _ = std::fs::remove_file(&self.path);
    }
}

fn new() -> (TestPool, mpsc::Sender<mise::session_store::Message>) {
    let file_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let file_path = format!("/tmp/{}-mise-test.db", file_name);

    let sender = mise::sqlite::session_store(&file_path).unwrap();

    (TestPool { path: file_path }, sender)
}

#[tokio::test]
async fn prunes_expired_sessions() -> Result<()> {
    let (pool, sender) = new();

    // make two sessions. Session 2 is expired.
    let (tx, rx) = oneshot::channel();

    let session = domain::Session {
        key: "1".into(),
        user_id: "1".into(),
        refresh_token: "token".into(),
        revalidate_at: Utc::now()
            .checked_add_signed(TimeDelta::seconds(20))
            .unwrap(),
        expires_at: Utc::now()
            .checked_add_signed(TimeDelta::seconds(20))
            .unwrap(),
    };

    let _ = sender
        .send(mise::session_store::Message::Set {
            session,
            respond_to: tx,
        })
        .await;
    rx.await.unwrap().unwrap();

    let (tx, rx) = oneshot::channel();
    let session = domain::Session {
        key: "2".into(),
        user_id: "1".into(),
        refresh_token: "token".into(),
        revalidate_at: Utc::now()
            .checked_add_signed(TimeDelta::seconds(20))
            .unwrap(),
        expires_at: Utc::now()
            .checked_add_signed(TimeDelta::seconds(-20))
            .unwrap(),
    };

    let _ = sender
        .send(mise::session_store::Message::Set {
            session,
            respond_to: tx,
        })
        .await;
    rx.await.unwrap().unwrap();

    // verify two sessions exist
    let conn = Connection::open(&pool.path)?;
    let count: usize = conn
        .query_row("SELECT count(*) FROM sessions", [], |x| x.get(0))
        .unwrap();
    assert_eq!(2, count);

    // startup session store
    let (background_result_sender, mut receiver) = tokio::sync::mpsc::channel(8);
    let store = mise::session_store::SessionStore::new(sender, background_result_sender);

    // wait for session prune
    while let Some(msg) = receiver.recv().await {
        if let BackgroundResultMessage::PruneExpiredSessions { result } = msg {
            assert_eq!(true, result.is_ok());
            receiver.close();
            break;
        }
    }

    // verify only one session exists
    let conn = Connection::open(&pool.path)?;
    let count: usize = conn
        .query_row("SELECT count(*) FROM sessions", [], |x| x.get(0))
        .unwrap();
    assert_eq!(1, count);

    // try to get session one
    let session = store.get(domain::SessionKey("1".into())).await.unwrap();
    assert_eq!("1", &session.key);

    Ok(())
}

#[tokio::test]
async fn prunes_refresh_locks() -> Result<()> {
    let (pool, sender) = new();

    // make two locks, lock 2 is expired.
    let conn = Connection::open(&pool.path)?;
    let mut stmt = conn.prepare_cached(
        "INSERT INTO refresh_locks VALUES (?1, datetime('now', ?2 || ' seconds'))",
    )?;

    stmt.execute(params!["1", "-10"])?;
    stmt.execute(params!["2", "10"])?;

    // verify two locks exist
    let count: usize = conn.query_row("SELECT count(*) FROM refresh_locks", [], |x| x.get(0))?;
    assert_eq!(2, count);

    // startup session store
    let (background_result_sender, mut receiver) = tokio::sync::mpsc::channel(8);
    let _ = mise::session_store::SessionStore::new(sender, background_result_sender);

    // wait for session prune
    while let Some(msg) = receiver.recv().await {
        if let BackgroundResultMessage::PruneExpiredRefreshLocks { result } = msg {
            assert_eq!(true, result.is_ok());
            receiver.close();
            break;
        }
    }

    // verify only one lock exists
    let count: usize = conn.query_row("SELECT count(*) FROM refresh_locks", [], |x| x.get(0))?;
    assert_eq!(1, count);

    // lock two should still exist
    let key: String = conn.query_row("SELECT key FROM refresh_locks LIMIT 1", [], |x| x.get(0))?;
    assert_eq!("2", key);

    Ok(())
}
