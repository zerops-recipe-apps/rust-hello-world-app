use std::env;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
        env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
        env::var("DB_PASS").unwrap_or_default(),
        env::var("DB_NAME").unwrap_or_else(|_| "db".to_string()),
    );

    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS greetings (
                id      INTEGER PRIMARY KEY,
                message TEXT    NOT NULL
            )",
            &[],
        )
        .await?;

    client
        .execute(
            "INSERT INTO greetings (id, message)
             VALUES (1, 'Hello from Zerops!')
             ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await?;

    println!("Migration completed successfully");
    Ok(())
}
