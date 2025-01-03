use super::model::User;
use crate::modules::utils::{validate_email, validate_passowrd};
use datastore::Datastore;
use error::AccountError;
use json_response::Error;
use mongodb::bson::doc;
pub mod error {
    use std::fmt::Display;

    use json_response::{Error, ErrorLogger};
    use serde::Serialize;

    #[derive(Debug, PartialEq, Error, Serialize)]
    pub enum AccountError {
        #[error_code(4001)]
        UserAlreadyExist,
        InternalServerError(String),
    }

    impl ErrorLogger for AccountError {
        fn log_error(&self, req: &mut salvo::Request) {
            ()
        }
    }

    impl Display for AccountError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::UserAlreadyExist => f.write_str("UserAlreadyExist"),
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
    ) -> Result<(), AccountError> {
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

        // Hash password using Bcrypt
        let password = match crypto::hash::make(password.as_str()) {
            Ok(s) => s,
            Err(err) => return Err(error::AccountError::InternalServerError(err.to_string())),
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
    async fn test_register_failed_email_exist() {
        let _ = tracing_subscriber::fmt::try_init();
        let (_server, connection_string) = crate::modules::utils::setup_test_db().await;
        let store = datastore::Datastore::new(connection_string.as_str()).await;
        let svc = AccountService::new(store);
        let r = svc
            .register("acme@gmail.com".into(), "password".into())
            .await;
        match r {
            Ok(ok) => assert_eq!(ok, ()),
            Err(err) => match err {
                AccountError::InternalServerError(err) => panic!("{err}"),
                _ => panic!("{err}"),
            },
        };
    }
}
