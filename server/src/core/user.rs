use uuid::Uuid;

use crate::{
    core::Error,
    datastore::{self, Pool},
    domain::User,
};

pub async fn get(datasource: &Pool, id: &str) -> Result<String, Error> {
    let user = datasource
        .get_user(id.to_owned())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("user {} does not exist", id)),
            _ => Error::Other(err.into()),
        })?;

    Ok(user.name)
}

pub async fn upsert(
    datasource: &Pool,
    oauth_src: &str,
    oauth_id: &str,
    name: &str,
) -> Result<(), Error> {
    let user = User {
        id: Uuid::new_v4().to_string(),
        oauth_id: format!("{}|{}", oauth_src, oauth_id),
        name: name.to_owned(),
    };

    datasource
        .upsert_user_by_oauth_id(user)
        .await
        .map_err(|err| Error::Other(err.into()))
}
