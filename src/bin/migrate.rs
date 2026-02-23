use std::env;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_pass = env::var("DB_PASS").unwrap_or_default();
    let db_name = env::var("DB_NAME").unwrap_or_else(|_| "db".to_string());

    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        db_host, db_port, db_user, db_pass, db_name
    );

    println!("Connecting to database...");
    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
        .await
        .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {}", e);
        }
    });

    println!("Running migration...");

    // Idempotent schema: safe to run on every deploy as defense-in-depth,
    // though zsc execOnce in zerops.yaml ensures it only runs once per version
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS greetings (
                id      INTEGER PRIMARY KEY,
                message TEXT NOT NULL
            )",
            &[],
        )
        .await
        .expect("Failed to create greetings table");

    // ON CONFLICT DO NOTHING makes this safe even if zsc execOnce somehow
    // runs twice — the row is never duplicated
    client
        .execute(
            "INSERT INTO greetings (id, message)
             VALUES (1, 'Hello from Zerops!')
             ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await
        .expect("Failed to seed greetings table");

    println!("Migration completed successfully.");
}
