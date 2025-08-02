use sqlx::{SqlitePool, Row};
use anyhow::Result;

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO users (username, email, password, role) VALUES (?, ?, ?, ?)"
    )
    .bind(username)
    .bind(email)
    .bind(password)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<(i64, String, String, String)>> {
    let row = sqlx::query(
        "SELECT id, username, password, role FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (
        r.get::<i64, _>("id"),
        r.get::<String, _>("username"),
        r.get::<String, _>("password"),
        r.get::<String, _>("role"),
    )))
}
