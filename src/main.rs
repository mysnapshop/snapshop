use salvo::Service;
use tokio;

mod modules;
#[tokio::main]
async fn main() {
    let router = salvo::Router::new();
    let ds = datastore::Datastore::new(env::get("DATABASE_URL").unwrap().as_str()).await;
    // let router = mo::bind_http_route(router, &ds);
    // let svc = Service::new(router);
}
