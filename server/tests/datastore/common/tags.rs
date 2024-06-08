use anyhow::Result;
use mise::{
    datastore::{self},
    domain::{RegisteringUser, User},
};

#[macro_export]
macro_rules! tags_tests {
    ($cd:expr) => {
        mod tags {
            use crate::a_test;
            use crate::datastore::common::{tags, CreatesDatastore, HoldsDatastore};
            use anyhow::Result;

            a_test!($cd, tags, can_create);
            a_test!($cd, tags, cannot_create_duplicate_name);
            a_test!($cd, tags, can_get_all_tags);
        }
    };
}

async fn user(store: &datastore::Pool) -> Result<User> {
    Ok(store
        .upsert_user_by_oauth_id(RegisteringUser {
            potential_id: "user-id".into(),
            oauth_id: "custom|user-1".into(),
            name: "user".into(),
        })
        .await?)
}

pub async fn can_create(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    store.create_tag(user.id, "Main Dish".into()).await?;

    Ok(())
}

pub async fn cannot_create_duplicate_name(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    store
        .create_tag(user.id.clone(), "Main Dish".into())
        .await?;
    let result = store.create_tag(user.id.clone(), "Main Dish".into()).await;
    if let Ok(_) = result {
        panic!("result is Ok, expected error.");
    }

    Ok(())
}

pub async fn can_get_all_tags(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    let main_id = store
        .create_tag(user.id.clone(), "Main Dish".into())
        .await?;
    let side_id = store
        .create_tag(user.id.clone(), "Side Dish".into())
        .await?;

    let result = store.get_tags().await?;

    assert_eq!(2, result.len());

    assert_eq!(main_id, result[0].id);
    assert_eq!("Main Dish", result[0].name);

    assert_eq!(side_id, result[1].id);
    assert_eq!("Side Dish", result[1].name);

    Ok(())
}
