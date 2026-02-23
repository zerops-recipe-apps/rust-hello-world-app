use actix_web::{web, App, HttpResponse, HttpServer};
use serde_json::json;
use std::env;
use tokio_postgres::NoTls;

async fn health_check() -> HttpResponse {
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
        env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
        env::var("DB_PASS").unwrap_or_default(),
        env::var("DB_NAME").unwrap_or_else(|_| "db".to_string()),
    );

    match tokio_postgres::connect(&conn_str, NoTls).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            match client
                .query_one("SELECT message FROM greetings LIMIT 1", &[])
                .await
            {
                Ok(row) => {
                    let greeting: String = row.get(0);
                    HttpResponse::Ok().json(json!({
                        "type": "rust",
                        "greeting": greeting,
                        "status": { "database": "OK" }
                    }))
                }
                Err(e) => HttpResponse::ServiceUnavailable().json(json!({
                    "type": "rust",
                    "greeting": null,
                    "status": { "database": format!("ERROR: {}", e) }
                })),
            }
        }
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({
            "type": "rust",
            "greeting": null,
            "status": { "database": format!("ERROR: {}", e) }
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    println!("Starting server on port {}", port);

    HttpServer::new(|| App::new().route("/", web::get().to(health_check)))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
