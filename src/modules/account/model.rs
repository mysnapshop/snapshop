use std::collections::HashMap;

use datastore::Model;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// User model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthProvider {
    Password,
    Google,
}

impl AuthProvider {
    pub fn is_password(&self) -> bool {
        match self {
            Self::Password => true,
            _ => false,
        }
    }

    pub fn is_google(&self) -> bool {
        match self {
            Self::Google => true,
            _ => false,
        }
    }
}

pub type MetaData = HashMap<String, serde_json::Value>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub email: String,
    pub providers: Vec<AuthProvider>,
    pub meta: MetaData,

    #[serde(skip_serializing)]
    credentials: MetaData,
    pub profiles: HashMap<ProfileType, Profile>,
}

impl User {
    pub fn new(email: String) -> Self {
        User {
            _id: None,
            email,
            providers: vec![],
            meta: HashMap::default(),
            profiles: HashMap::default(),
            credentials: HashMap::default(),
        }
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.credentials
            .insert("password".to_string(), serde_json::Value::String(password));
        self
    }
}

impl Model for User {
    async fn find_one(
        client: &mongodb::Client,
        query: mongodb::bson::Document,
    ) -> Result<Option<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        match client
            .database("snapshop")
            .collection::<UserForDB>("users")
            .find_one(query)
            .await
        {
            Ok(ok) => match ok {
                Some(ok) => Ok(Some(ok.into())),
                None => Ok(None),
            },
            Err(err) => Err(err),
        }
    }

    async fn find_many(
        client: &mongodb::Client,
        query: mongodb::bson::Document,
    ) -> Result<Vec<Self>, mongodb::error::Error>
    where
        Self: Sized,
    {
        todo!()
    }

    async fn insert_one(
        client: &mongodb::Client,
        data: &mut Self,
    ) -> Result<ObjectId, mongodb::error::Error>
    where
        Self: Sized,
    {
        let user_for_db: UserForDB = data.clone().into();
        match client
            .database("snapshop")
            .collection::<UserForDB>("users")
            .insert_one(user_for_db)
            .await
        {
            Ok(result) => {
                let id = result.inserted_id.as_object_id().unwrap();
                data._id = Some(id.clone());
                Ok(id)
            }
            Err(err) => return Err(err),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserForDB {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    email: String,
    providers: Vec<AuthProvider>,
    meta: MetaData,
    credentials: MetaData,
    profiles: HashMap<ProfileType, Profile>,
}

impl From<User> for UserForDB {
    fn from(user: User) -> Self {
        Self {
            _id: user._id,
            email: user.email,
            providers: user.providers,
            meta: user.meta,
            credentials: user.credentials,
            profiles: user.profiles,
        }
    }
}

impl From<UserForDB> for User {
    fn from(user_db: UserForDB) -> Self {
        Self {
            _id: user_db._id,
            email: user_db.email,
            providers: user_db.providers,
            meta: user_db.meta,
            credentials: user_db.credentials,
            profiles: user_db.profiles,
        }
    }
}

// Profile model
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Phone {
    country_code: u32,
    phone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProfileType {
    Buyer,
    Seller,
}

impl ProfileType {
    pub fn is_buyer(&self) -> bool {
        match self {
            Self::Buyer => true,
            _ => false,
        }
    }

    pub fn is_seller(&self) -> bool {
        match self {
            Self::Seller => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Profile {
    lname: String,
    fname: String,
    meta: MetaData,
    phone: Phone,
}
