use std::env;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS greetings (
                id INTEGER PRIMARY KEY,
                message TEXT NOT NULL
            );
            INSERT INTO greetings (id, message) VALUES (1, 'Hello from Zerops!')
                ON CONFLICT (id) DO NOTHING;",
        )
        .await?;

    println!("Migration completed successfully");
    Ok(())
}
