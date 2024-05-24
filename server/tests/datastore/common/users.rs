#[macro_export]
macro_rules! users_tests {
    ($($type:expr)*) => {
    $(
        use anyhow::Result;
        use crate::datastore::common::users::upsert_user;
        use crate::datastore::common::users::get_user;

        // upsert_user

        #[tokio::test]
        async fn can_upsert_new_user() -> Result<()> {
            let store = $type.new();
            upsert_user::can_upsert_new_user(store.get()).await
        }

        #[tokio::test]
        async fn updates_existing_user_if_setup() -> Result<()> {
            let store = $type.new();
            upsert_user::updates_existing_user_if_setup(store.get()).await
        }

        // get_user

        #[tokio::test]
        async fn cannot_get_non_existent_user() -> Result<()> {
            let store = $type.new();
            get_user::cannot_get_non_existent_user(store.get()).await
        }

        #[tokio::test]
        async fn can_get_a_registered_user() -> Result<()> {
            let store = $type.new();
            get_user::can_get_a_registered_user(store.get()).await
        }
     )*
    };
}

pub mod upsert_user {
    use anyhow::Result;
    use mise::{datastore, domain::RegisteringUser};

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
}

pub mod get_user {
    use anyhow::Result;
    use mise::{datastore, domain::RegisteringUser};

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
}
