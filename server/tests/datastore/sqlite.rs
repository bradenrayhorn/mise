use mise::{datastore, sqlite};
use rand::{distributions::Alphanumeric, Rng};

use crate::users_tests;

use super::common::{CreatesDatastore, HoldsDatastore};

pub struct TestPool {
    pool: datastore::Pool,
    path: String,
}

impl Drop for TestPool {
    fn drop(&mut self) {
        // TODO - shutdown pool
        let _ = std::fs::remove_file(&self.path);
    }
}

impl HoldsDatastore for TestPool {
    fn get(&self) -> datastore::Pool {
        return self.pool.clone();
    }
}

pub struct SqliteCreator {}

impl CreatesDatastore for SqliteCreator {
    fn new(&self) -> impl HoldsDatastore {
        let file_name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let file_path = format!("/tmp/{}-mise-test.db", file_name);

        let (_, connections) = sqlite::worker_pool(file_path.clone()).unwrap();
        TestPool {
            pool: datastore::Pool::new(connections),
            path: file_path,
        }
    }
}

users_tests!(crate::datastore::sqlite::SqliteCreator {});
