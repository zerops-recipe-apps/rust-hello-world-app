use axum::{routing::get, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 1. Definisci le rotte
    let app = Router::new().route("/", get(handler));

    // 2. Ascolta su 0.0.0.0:3000 (Obbligatorio per Zerops)
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on port 3000");

    // 3. Avvia il server
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Hello from Zerops! 🚀 Rust (Axum) is running successfully."
}