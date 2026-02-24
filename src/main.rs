use actix_web::{web, App, HttpResponse, HttpServer};
use serde_json::json;
use std::env;
use tokio_postgres::NoTls;

async fn health_check() -> HttpResponse {
    match check_database().await {
        Ok(greeting) => HttpResponse::Ok().json(json!({
            "type": "rust",
            "greeting": greeting,
            "status": {
                "database": "OK"
            }
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({
            "type": "rust",
            "greeting": serde_json::Value::Null,
            "status": {
                "database": format!("ERROR: {}", e)
            }
        })),
    }
}

async fn check_database() -> Result<String, Box<dyn std::error::Error>> {
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()),
        env::var("DB_PORT").unwrap_or_else(|_| "5432".into()),
        env::var("DB_USER")?,
        env::var("DB_PASS")?,
        env::var("DB_NAME")?,
    );

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("database connection error: {}", e);
        }
    });

    let row = client
        .query_one("SELECT message FROM greetings LIMIT 1", &[])
        .await?;
    let greeting: String = row.get(0);
    Ok(greeting)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on port 8080");
    HttpServer::new(|| App::new().route("/", web::get().to(health_check)))
        .bind("[::]:8080")?
        .run()
        .await
}
