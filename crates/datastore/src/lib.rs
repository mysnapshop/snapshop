use mongodb::{
    bson::{oid::ObjectId, Document},
    error::Error,
    Client,
};
use serde::{Deserialize, Serialize};

pub trait Model: Serialize + for<'a> Deserialize<'a> {
    fn find_one(
        client: &Client,
        query: Document,
    ) -> impl std::future::Future<Output = Result<Option<Self>, Error>> + Send
    where
        Self: Sized;
    fn find_many(
        client: &Client,
        query: Document,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, Error>> + Send
    where
        Self: Sized;

    fn insert_one(
        client: &Client,
        data: &mut Self,
    ) -> impl std::future::Future<Output = Result<ObjectId, Error>> + Send
    where
        Self: Sized;
}

pub trait ModelExt<'a> {
    type Inner: Model;

    fn factory(client: Client, inner: &'a Self::Inner) -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct Datastore {
    pub client: Client,
}

impl Datastore {
    pub async fn new<'a>(uri: &'a str) -> Self {
        let client = Client::with_uri_str(uri)
            .await
            .expect("Error connecting to MongoDB");
        Datastore { client }
    }

    pub fn from(client: Client) -> Self {
        Datastore { client }
    }

    pub fn factory<'a, T>(&self, inner: &'a T::Inner) -> T
    where
        T: ModelExt<'a>,
    {
        T::factory(self.client.clone(), inner)
    }
}

// repository
impl Datastore {
    pub async fn find_one<M: Model>(&self, query: Document) -> Result<Option<M>, Error> {
        M::find_one(&self.client, query).await
    }

    pub async fn find_many<M: Model>(&self, query: Document) -> Result<Vec<M>, Error> {
        M::find_many(&self.client, query).await
    }

    pub async fn insert_one<M: Model>(&self, data: &mut M) -> Result<ObjectId, Error> {
        M::insert_one(&self.client, data).await
    }
}

#[cfg(test)]
mod tests {
    use mongodb::{
        bson::{doc, oid::ObjectId},
        error::Error,
        Client,
    };
    use serde::{Deserialize, Serialize};

    use crate::{Datastore, Model, ModelExt};

    #[derive(Serialize, Deserialize, Debug)]
    struct User {
        _id: Option<ObjectId>,
    }

    impl Model for User {
        async fn find_one(
            client: &Client,
            query: mongodb::bson::Document,
        ) -> Result<Option<Self>, Error>
        where
            Self: Sized,
        {
            Ok(Some(User { _id: None }))
        }

        async fn find_many(
            client: &Client,
            query: mongodb::bson::Document,
        ) -> Result<Vec<Self>, Error>
        where
            Self: Sized,
        {
            todo!()
        }

        async fn insert_one(client: &Client, query: &mut Self) -> Result<ObjectId, Error>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    struct UserExt<'a> {
        _client: Client,
        inner: &'a User,
    }

    impl<'a> UserExt<'a> {
        async fn lock_account(&self) {
            let id = self.inner._id.clone();
            // use id
        }
    }

    impl<'a> ModelExt<'a> for UserExt<'a> {
        type Inner = User;
        fn factory(_client: Client, inner: &'a Self::Inner) -> Self
        where
            Self: Sized,
        {
            UserExt { _client, inner }
        }
    }

    #[tokio::test]
    pub async fn test_find_one() {
        env::init("../../.env");
        let ds = Datastore::from(
            Client::with_uri_str(env::get("DATABASE_URL").unwrap())
                .await
                .unwrap(),
        );
        let u = match ds.find_one::<User>(doc! {"email": "acme@gmail.com"}).await {
            Ok(r) => r.unwrap(),
            Err(err) => panic!("{}", err),
        };
        let u_ext = ds.factory::<UserExt>(&u);
        u_ext.lock_account().await;
        assert_eq!(u._id, None);
    }
}
