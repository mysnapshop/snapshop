use datastore::Datastore;
use json_response::{ApiResponse, RequestError};
use salvo::{affix_state, handler, Depot, Request, Response, Router};
use serde::{Deserialize, Serialize};
use service::{error::AccountError, AccountService};

use super::utils::{validate_email, validate_passowrd};

mod model;
mod service;

pub fn bind_http_route<'a>(router: Router, store: Datastore) -> Router {
    let svc = AccountService::new(store);
    router
        .hoop(affix_state::inject(svc))
        .push(Router::new().path("/account/profile").post(profile_handler))
        .push(Router::new().path("/auth/register").post(register_handler))
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct RegisterResponse {}

#[handler]
async fn register_handler(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> ApiResponse<RegisterResponse, RequestError<AccountError>> {
    let (email, password) = match req.parse_json::<RegisterRequest>().await {
        Ok(req) => {
            let RegisterRequest { email, password } = req;
            // validate password
            if !validate_passowrd(password.as_str()) {
                return ApiResponse::error(RequestError::BadRequest("invalid_password".into()));
            }

            if !validate_email(email.as_str()) {
                return ApiResponse::error(RequestError::BadRequest("invalid_email".into()));
            }
            (email, password)
        }
        Err(_) => {
            return ApiResponse::error(RequestError::BadRequest("invalid_request_body".into()));
        }
    };

    let svc = depot.obtain::<AccountService>().unwrap();
    match svc.register(email, password).await {
        Ok(_) => ApiResponse::success(RegisterResponse {}),
        Err(err) => ApiResponse::error(RequestError::ServiceError(err)),
    }
}

#[cfg(test)]
mod register_tests {
    use super::{register_handler, RegisterRequest};

    #[tokio::test]
    async fn test_register_handler_failed_bad_request() {
        let _ = tracing_subscriber::fmt::try_init();
        use salvo::http::header;
        use salvo::test::{ResponseExt, TestClient};
        use salvo::{Router, Service};

        let service = Service::new(Router::new().post(register_handler));

        let req = TestClient::post(format!("http://127.0.0.1:5800/")).add_header(
            header::CONTENT_TYPE,
            "application/json",
            true,
        );
        let content = req.send(&service).await.take_string().await.unwrap();
        assert_eq!(
            content,
            "{\"status\":\"failed\",\"error\":{\"code\":400,\"message\":\"BadRequest:invalid_request_body\"}}"
        );
    }

    #[tokio::test]
    async fn test_register_handler_failed_password_invalid() {
        let _ = tracing_subscriber::fmt::try_init();
        use salvo::http::header;
        use salvo::test::{ResponseExt, TestClient};
        use salvo::{Router, Service};

        let service = Service::new(Router::new().post(register_handler));

        let req = TestClient::post(format!("http://127.0.0.1:5800/"))
            .add_header(header::CONTENT_TYPE, "application/json", true)
            .raw_json(r#"{"email":"acme@gmail.com", "password":""}"#);
        let content = req.send(&service).await.take_string().await.unwrap();
        println!("{}", &content);
        assert_eq!(
            content,
            r#"{"status":"failed","error":{"code":400,"message":"BadRequest:invalid_password"}}"#
        );
    }
}

#[handler]
async fn profile_handler() {}
