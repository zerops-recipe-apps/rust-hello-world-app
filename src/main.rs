use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Build connection string from environment variables
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_pass = env::var("DB_PASS").unwrap_or_else(|_| "postgres".to_string());
    let db_name = env::var("DB_NAME").unwrap_or_else(|_| "postgres".to_string());

    let conn_str = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_pass, db_host, db_port, db_name
    );

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_str)
        .await
        .expect("Failed to connect to database");

    // Create table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS entries (id SERIAL PRIMARY KEY, data TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("Failed to ensure table exists");

    // Define routes
    let app = Router::new()
        .route("/", get(add_entry))
        .route("/status", get(status_check))
        .with_state(pool);

    // Listen on 0.0.0.0:3000 (required for Zerops)
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Server running on http://localhost:{}", port);

    // Start server
    axum::serve(listener, app).await.unwrap();
}

async fn add_entry(State(pool): State<PgPool>) -> (StatusCode, Json<Value>) {
    // Generate random UUID
    let random_data = uuid::Uuid::new_v4().to_string();

    // Insert entry into database
    if let Err(e) = sqlx::query("INSERT INTO entries(data) VALUES ($1)")
        .bind(&random_data)
        .execute(&pool)
        .await
    {
        error!("Failed to insert entry: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to insert entry"})),
        );
    }

    // Count entries
    let count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM entries")
        .fetch_one(&pool)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to count entries: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to count entries"})),
            );
        }
    };

    // Log entry
    info!(data = %random_data, total = count, "entry added");
    warn!(data = %random_data, total = count, "entry added");
    error!(data = %random_data, total = count, "entry added");

    let response = json!({
        "message": "This is a simple, basic Rust application running on Zerops.io, each request adds an entry to the PostgreSQL database and returns a count. See the source repository (https://github.com/zeropsio/recipe-rust) for more information.",
        "newEntry": random_data,
        "count": count
    });

    (StatusCode::CREATED, Json(response))
}

async fn status_check() -> Json<Value> {
    Json(json!({"status": "UP"}))
}