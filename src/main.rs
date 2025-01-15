use modules::account;
use salvo::{conn::TcpListener, Listener, Server};
use tokio;

mod modules;
#[tokio::main]
async fn main() {
    let router = salvo::Router::new();
    let store = datastore::Datastore::new(env::get("DATABASE_URL").unwrap().as_str()).await;
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    let router = account::bind_http_route(router, store);

    println!("{:#?}", &router.routers);
    println!(
        "Server started on: http://{}",
        &acceptor.local_addr().unwrap()
    );
    Server::new(acceptor).serve(router).await;
}
