use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub struct Error<'a> {
    pub message: &'a str,
}
