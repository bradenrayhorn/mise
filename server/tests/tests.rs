mod datastore {
    mod common;
    pub mod sqlite;
}

mod http {
    mod auth;
    mod image;
    mod recipe;
    mod requests;
    mod responses;
    mod setup;
    mod tag;
}
