use mise::{datastore, sqlite};
use rand::Rng;

use crate::{images_tests, recipes_tests, tags_tests, users_tests};

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
        let file_name: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let file_path = format!("/tmp/{}-mise-test.db", file_name);

        let (_, connections) = sqlite::datastore_handler(
            &file_path,
            &sqlite::DatastoreConfig {
                recipe_page_size: 2,
                recipe_dump_page_size: 2,
            },
        )
        .unwrap();
        TestPool {
            pool: datastore::Pool::new(connections),
            path: file_path,
        }
    }
}

images_tests!(crate::datastore::sqlite::SqliteCreator {});
recipes_tests!(crate::datastore::sqlite::SqliteCreator {});
tags_tests!(crate::datastore::sqlite::SqliteCreator {});
users_tests!(crate::datastore::sqlite::SqliteCreator {});
