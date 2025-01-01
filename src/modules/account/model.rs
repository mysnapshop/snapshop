use std::collections::HashMap;

use datastore::Model;
use mongodb::{bson::oid::ObjectId, Client};
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
        client
            .database("snapshop")
            .collection::<Self>("users")
            .find_one(query)
            .await
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
        match client
            .database("snapshop")
            .collection::<Self>("users")
            .insert_one(data.clone())
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
