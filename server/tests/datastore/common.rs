use mise::datastore;

pub mod users;

pub trait CreatesDatastore {
    fn new(&self) -> impl HoldsDatastore;
}

pub trait HoldsDatastore {
    fn get(&self) -> datastore::Pool;
}
