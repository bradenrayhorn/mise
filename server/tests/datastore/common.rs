use mise::datastore;

pub mod recipes;
pub mod users;

pub trait CreatesDatastore {
    fn new(&self) -> impl HoldsDatastore;
}

pub trait HoldsDatastore {
    fn get(&self) -> datastore::Pool;
}

#[macro_export]
macro_rules! a_test {
    ($($type:expr,$module:ident,$fn:ident)*) => {
    $(

        #[tokio::test]
        async fn $fn() -> Result<()> {
            let store = $type.new();
            $module::$fn(store.get()).await
        }

     )*
    };
}
