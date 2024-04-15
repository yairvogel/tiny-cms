
use axum::{Router, routing::get};
use tokio::runtime::Builder;

pub fn run_server(_content_dir: &str) {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(server_main());
}

async fn server_main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let addr: &str = "localhost:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("serving content at {}", addr);
    axum::serve(listener, app).await.unwrap();
}
