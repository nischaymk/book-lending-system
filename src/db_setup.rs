use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use anyhow::Result;

pub async fn initialize_db() -> Result<SqlitePool> {
    // Create pool to SQLite database file (ensure ./db folder exists)
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:db/database.db")
        .await?;

    // Schema and seed admin user
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            role TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            isbn TEXT NOT NULL UNIQUE,
            publication_year INTEGER NOT NULL,
            genre TEXT NOT NULL,
            copies_available INTEGER NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS borrow_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            book_id INTEGER NOT NULL,
            borrow_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            return_date TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (book_id) REFERENCES books(id)
        );
        INSERT OR IGNORE INTO users (username, email, password, role)
        VALUES ('admin', 'admin@example.com', '240be518fabd2724ddb6f04eeb1da5967448d7e831c08c8fa822809f74c720a9', 'admin');
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
