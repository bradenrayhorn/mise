use crate::{core::Error, datastore::Pool, domain};

fn validation_to_other(err: domain::ValidationError) -> Error {
    Error::Other(err.into())
}

pub async fn create(
    datastore: &Pool,
    user: domain::user::Authenticated,
    tag: domain::tag::Creating,
) -> Result<i64, Error> {
    let id = datastore
        .create_tag(user.id, tag.name.into())
        .await
        .map_err(|err| Error::Other(err.into()))?;

    Ok(id)
}

pub async fn get_all(datastore: &Pool) -> Result<Vec<domain::tag::Tag>, Error> {
    let result = datastore
        .get_tags()
        .await
        .map_err(|err| Error::Other(err.into()))?;

    result
        .into_iter()
        .map(|tag| {
            Ok(domain::tag::Tag {
                id: tag.id,
                name: tag.name.try_into().map_err(validation_to_other)?,
            })
        })
        .collect()
}
