use anyhow::Result;
use mise::{datastore, domain::RegisteringUser};

#[macro_export]
macro_rules! users_tests {
    ($cd:expr) => {
        mod users {
            use crate::a_test;
            use crate::datastore::common::{users, CreatesDatastore, HoldsDatastore};
            use anyhow::Result;

            a_test!($cd, users, can_upsert_new_user);
            a_test!($cd, users, updates_existing_user_if_setup);
            a_test!($cd, users, cannot_get_non_existent_user);
            a_test!($cd, users, can_get_a_registered_user);
        }
    };
}

// upsert_user_by_oauth_id

pub async fn can_upsert_new_user(store: datastore::Pool) -> Result<()> {
    let registering = RegisteringUser {
        potential_id: "1234".into(),
        oauth_id: "11".into(),
        name: "Barley".into(),
    };

    let user = store.upsert_user_by_oauth_id(registering).await?;

    assert_eq!("1234", user.id);
    assert_eq!("11", user.oauth_id);
    assert_eq!("Barley", user.name);

    Ok(())
}

pub async fn updates_existing_user_if_setup(store: datastore::Pool) -> Result<()> {
    let registering = RegisteringUser {
        potential_id: "1234".into(),
        oauth_id: "11".into(),
        name: "Barley".into(),
    };

    store.upsert_user_by_oauth_id(registering).await?;

    let registering_again = RegisteringUser {
        potential_id: "12345678".into(),
        oauth_id: "11".into(),
        name: "Barley Bob".into(),
    };
    let user = store.upsert_user_by_oauth_id(registering_again).await?;

    assert_eq!("1234", user.id);
    assert_eq!("11", user.oauth_id);
    assert_eq!("Barley Bob", user.name);

    Ok(())
}

// get_user

pub async fn cannot_get_non_existent_user(store: datastore::Pool) -> Result<()> {
    let result = store.get_user("random_id".into()).await.unwrap_err();

    assert!(
        matches!(result, datastore::Error::NotFound),
        "wrong enum: {}",
        result
    );

    Ok(())
}

pub async fn can_get_a_registered_user(store: datastore::Pool) -> Result<()> {
    let registering = RegisteringUser {
        potential_id: "1234".into(),
        oauth_id: "11".into(),
        name: "Barley".into(),
    };

    store.upsert_user_by_oauth_id(registering).await?;

    let user = store.get_user("1234".into()).await?;

    assert_eq!("1234", user.id);
    assert_eq!("11", user.oauth_id);
    assert_eq!("Barley", user.name);

    Ok(())
}
