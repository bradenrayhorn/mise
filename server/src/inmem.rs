use std::collections::HashMap;

use chrono::Utc;
use tokio::sync::mpsc;

use crate::cache::{CacheMessage, Error};

pub fn cache() -> Vec<mpsc::Sender<CacheMessage>> {
    let mut senders: Vec<mpsc::Sender<CacheMessage>> = Vec::new();

    let (tx, rx) = mpsc::channel(1);

    let actor = CacheActor {
        rx,
        cache: HashMap::new(),
    };
    tokio::spawn(run_actor(actor));

    senders.push(tx);

    senders
}

struct CacheItem {
    value: String,
    expires_at: chrono::DateTime<Utc>,
}

struct CacheActor {
    rx: mpsc::Receiver<CacheMessage>,
    cache: HashMap<String, CacheItem>,
}

impl CacheActor {
    fn process(&mut self, msg: CacheMessage) {
        match msg {
            CacheMessage::Health { respond_to } => {
                let _ = respond_to.send(Ok(()));
            }
            CacheMessage::Get { key, respond_to } => {
                let item = self.cache.get(&key);
                let response = match item {
                    Some(value) => match value.expires_at >= Utc::now() {
                        true => Ok(value.value.to_owned()),
                        false => Err(Error::NoMatchingValue),
                    },
                    None => Err(Error::NoMatchingValue),
                };

                let _ = respond_to.send(response);
            }
            CacheMessage::Set {
                key,
                value,
                expires_at,
                respond_to,
            } => {
                self.cache.insert(key, CacheItem { value, expires_at });
                let _ = respond_to.send(Ok(()));
            }
            CacheMessage::Remove { key, respond_to } => {
                let _ = self.cache.remove(&key);
                let _ = respond_to.send(Ok(()));
            }
        }
    }
}

async fn run_actor(mut actor: CacheActor) {
    // TODO - periodically clean expired items

    while let Some(msg) = actor.rx.recv().await {
        actor.process(msg);
    }
}
