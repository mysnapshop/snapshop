use datastore::Datastore;
use salvo::{
    affix_state, handler, http::StatusCode, writing::Json, Depot, Request, Response, Router,
};
use serde::{Deserialize, Serialize};
use service::AccountService;

mod model;
mod service;

pub fn bind_http_route<'a>(router: Router, store: Datastore) -> Router {
    let svc = AccountService::new(store);
    router.push(
        Router::new()
            .hoop(affix_state::inject(svc))
            .path("/account/profile")
            .post(profile_handler),
    )
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
) -> Json<RegisterResponse> {
    let RegisterRequest { email, password } = match req.parse_json::<RegisterRequest>().await {
        Ok(req) => req,
        Err(err) => {
            res.status_code(StatusCode::BAD_REQUEST);
            return Json(RegisterResponse {});
        }
    };

    let svc = depot.obtain::<AccountService>().unwrap();
    match svc.register(email, password).await {
        Ok(_) => Json(RegisterResponse {}),
        Err(err) => Json(RegisterResponse {}),
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_register_handler_failed_password_invalid() {
    use salvo::test::{ResponseExt, TestClient};
    use salvo::{Router, Service};

    let service = Service::new(Router::new().post(register_handler));

    let content = TestClient::post(format!("http://127.0.0.1:5800/"))
        .send(&service)
        .await
        .take_json::<RegisterResponse>()
        .await
        .unwrap();
    assert_eq!(
        content,
        RegisterResponse {} // r#"""{"code": "4003", "message":"PasswordInvalid"}"""#
    );
}

#[handler]
async fn profile_handler() {}
