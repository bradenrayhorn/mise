use std::ops::Deref;

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct RegisteringUser {
    pub potential_id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct SessionKey(pub String);

impl Deref for SessionKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
