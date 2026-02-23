use actix_web::{web, App, HttpResponse, HttpServer};
use serde_json::json;
use std::env;
use tokio_postgres::NoTls;

async fn health_check() -> HttpResponse {
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_pass = env::var("DB_PASS").unwrap_or_default();
    let db_name = env::var("DB_NAME").unwrap_or_else(|_| "db".to_string());

    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        db_host, db_port, db_user, db_pass, db_name
    );

    match tokio_postgres::connect(&conn_str, NoTls).await {
        Ok((client, connection)) => {
            // Drive the connection in the background; it runs until dropped
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("DB connection error: {}", e);
                }
            });

            // Query the migrated data — proves both connectivity and that migrations ran
            match client
                .query_one("SELECT message FROM greetings LIMIT 1", &[])
                .await
            {
                Ok(row) => {
                    let greeting: String = row.get(0);
                    HttpResponse::Ok().json(json!({
                        "type": "rust",
                        "greeting": greeting,
                        "status": {
                            "database": "OK"
                        }
                    }))
                }
                Err(e) => HttpResponse::ServiceUnavailable().json(json!({
                    "type": "rust",
                    "greeting": null,
                    "status": {
                        "database": format!("ERROR: {}", e)
                    }
                })),
            }
        }
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({
            "type": "rust",
            "greeting": null,
            "status": {
                "database": format!("ERROR: {}", e)
            }
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server on {}", addr);

    HttpServer::new(|| App::new().route("/", web::get().to(health_check)))
        .bind(&addr)?
        .run()
        .await
}
