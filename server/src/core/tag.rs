use crate::{core::Error, datastore::Pool, domain};

pub async fn create(
    datastore: &Pool,
    user: domain::user::Authenticated,
    tag: domain::tag::Creating,
) -> Result<domain::tag::Id, Error> {
    let id = datastore
        .create_tag(user.id, tag.name.into())
        .await
        .map_err(|err| Error::Other(err.into()))?;

    Ok(id)
}

pub async fn get_all(datastore: &Pool) -> Result<Vec<domain::tag::Tag>, Error> {
    datastore
        .get_tags()
        .await
        .map_err(|err| Error::Other(err.into()))
}

pub async fn get_all_with_stats(datastore: &Pool) -> Result<Vec<domain::tag::WithStats>, Error> {
    datastore
        .get_tags_with_stats()
        .await
        .map_err(|err| Error::Other(err.into()))
}
