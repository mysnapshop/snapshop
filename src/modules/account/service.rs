use super::model::User;
use crate::modules::utils::{validate_email, validate_passowrd};
use datastore::Datastore;
use mongodb::bson::doc;
pub mod error {
    use std::fmt::Display;

    use json_response::ErrorCode;
    use serde::Serialize;

    #[derive(Debug, PartialEq, ErrorCode, Serialize)]
    pub enum AccountError {
        UserAlreadyExist,
        PasswordInvalid,
        EmailInvalid,
        InternalServerError(String),
    }

    impl Display for AccountError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::UserAlreadyExist => f.write_str("UserAlreadyExist"),
                Self::PasswordInvalid => f.write_str("InvalidPassword"),
                Self::EmailInvalid => f.write_str("EmailInvalid"),
                Self::InternalServerError(_) => f.write_str("InternalServerError"),
            }
        }
    }
}

#[derive(Clone)]
pub struct AccountService {
    store: Datastore,
}

impl AccountService {
    pub fn new(store: Datastore) -> Self {
        AccountService { store }
    }
}

impl AccountService {
    pub(crate) async fn register(
        &self,
        email: String,
        password: String,
    ) -> Result<(), error::AccountError> {
        match self
            .store
            .find_one::<User>(doc! {"email": email.clone()})
            .await
        {
            Ok(u) => {
                if u.is_some() {
                    return Err(error::AccountError::UserAlreadyExist);
                }
            }
            Err(err) => return Err(error::AccountError::InternalServerError(err.to_string())),
        };

        // validate password
        if !validate_passowrd(password.as_str()) {
            return Err(error::AccountError::PasswordInvalid);
        }

        if !validate_email(email.as_str()) {
            return Err(error::AccountError::EmailInvalid);
        }

        // Hash password using Bcrypt
        let password = match crypto::hash::make(password.as_str()) {
            Ok(s) => s,
            Err(_) => return Err(error::AccountError::PasswordInvalid),
        };

        let mut u = User::new(email).with_password(password);
        match self.store.insert_one(&mut u).await {
            Ok(_) => Ok(()),
            Err(err) => return Err(error::AccountError::InternalServerError(err.to_string())),
        }
    }

    pub(crate) async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(), error::AccountError> {
        let r = self.store.find_one::<User>(doc! {"email": email}).await;
        todo!()
    }

    pub(crate) async fn verify_email(&self, email: String) -> Result<(), error::AccountError> {
        todo!()
    }

    pub(crate) async fn verify_phone(&self, phone: String) -> Result<(), error::AccountError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::modules::account::service::{error::AccountError, AccountService};

    #[tokio::test]
    async fn test_register_failed_password_invalid() {
        let store = datastore::Datastore::new(env::get("DATABASE_URL").unwrap().as_str()).await;
        let svc = AccountService::new(store);
        let r = svc
            .register("acme@gmail.com".to_string(), "".to_string())
            .await;
        match r {
            Ok(ok) => assert_eq!(ok, ()),
            Err(err) => assert_eq!(err, AccountError::PasswordInvalid),
        };
    }

    #[tokio::test]
    async fn test_register_failed_email_invalid() {
        let store = datastore::Datastore::new(env::get("DATABASE_URL").unwrap().as_str()).await;
        let svc = AccountService::new(store);
        let r = svc
            .register("acme".to_string(), "password".to_string())
            .await;
        match r {
            Ok(ok) => assert_eq!(ok, ()),
            Err(err) => assert_eq!(err, AccountError::EmailInvalid),
        };
    }
}
