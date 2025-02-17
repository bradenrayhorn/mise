use anyhow::Result;
use mise::{datastore, domain};

#[macro_export]
macro_rules! images_tests {
    ($cd:expr) => {
        mod images {
            use crate::a_test;
            use crate::datastore::common::{images, CreatesDatastore, HoldsDatastore};
            use anyhow::Result;

            a_test!($cd, images, can_create);
            a_test!($cd, images, cannot_create_duplicate);
            a_test!($cd, images, can_get_existing_image);
            a_test!($cd, images, returns_failure_if_image_does_not_exist);
        }
    };
}
pub async fn can_create(store: datastore::Pool) -> Result<()> {
    let id = domain::image::Id::new();
    store.create_image(&id).await?;

    Ok(())
}

pub async fn cannot_create_duplicate(store: datastore::Pool) -> Result<()> {
    let id = domain::image::Id::new();
    store.create_image(&id).await?;

    let result = store.create_image(&id).await;
    if let Ok(_) = result {
        panic!("result is Ok, expected error.");
    }

    Ok(())
}

pub async fn can_get_existing_image(store: datastore::Pool) -> Result<()> {
    let id = domain::image::Id::new();
    store.create_image(&id).await?;

    let _ = store.get_image(&id).await?;

    Ok(())
}

pub async fn returns_failure_if_image_does_not_exist(store: datastore::Pool) -> Result<()> {
    let id = domain::image::Id::new();

    let result = store.get_image(&id).await;
    if let Ok(_) = result {
        panic!("result is Ok, expected error.");
    }

    Ok(())
}
