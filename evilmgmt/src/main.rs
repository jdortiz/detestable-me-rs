use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    const SERVER_ADDR: &str = "127.0.0.1:8080";
    let router = Router::new();

    println!("Launching evilmgmt: http://{SERVER_ADDR}");
    let listener = TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Unable to create listener");
    axum::serve(listener, router).await.unwrap();
}
