use std::io::Write;
use sqlx::{SqlitePool, Row};
use serde_json::{json, Value};
use chrono::{Duration, Utc};
use anyhow::Result;

/// Borrow a book for a user
pub async fn borrow_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let body_start = request.find("\r\n\r\n").ok_or_else(|| anyhow::anyhow!("No body found"))?;
    let body = &request[(body_start + 4)..];
    let data: Value = serde_json::from_str(body)?;

    let user_id = data["user_id"].as_i64().ok_or_else(|| anyhow::anyhow!("Missing user_id"))? as i32;
    let book_id = data["book_id"].as_i64().ok_or_else(|| anyhow::anyhow!("Missing book_id"))? as i32;

    let copies_available = sqlx::query("SELECT copies_available FROM books WHERE id = ?")
        .bind(book_id)
        .fetch_optional(pool).await?
        .map(|r| r.get::<i32, _>("copies_available"))
        .ok_or_else(|| anyhow::anyhow!("Book not found"))?;

    if copies_available <= 0 {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"No copies available\"}";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let borrow_date = Utc::now().to_rfc3339();
    let due_date = (Utc::now() + Duration::days(14)).to_rfc3339();

    sqlx::query("INSERT INTO borrow_records (user_id, book_id, borrow_date, due_date) VALUES (?, ?, ?, ?)")
        .bind(user_id)
        .bind(book_id)
        .bind(&borrow_date)
        .bind(&due_date)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE books SET copies_available = copies_available - 1 WHERE id = ?")
        .bind(book_id)
        .execute(pool)
        .await?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"borrowed\"}";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

/// Return a borrowed book
pub async fn return_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let body_start = request.find("\r\n\r\n").ok_or_else(|| anyhow::anyhow!("No body found"))?;
    let body = &request[(body_start + 4)..];
    let data: Value = serde_json::from_str(body)?;

    let record_id = data["record_id"].as_i64().ok_or_else(|| anyhow::anyhow!("Missing record_id"))? as i32;

    let record = sqlx::query("SELECT book_id FROM borrow_records WHERE id = ? AND return_date IS NULL")
        .bind(record_id)
        .fetch_optional(pool)
        .await?
        .map(|r| r.get::<i32, _>("book_id"))
        .ok_or_else(|| anyhow::anyhow!("Borrow record not found or already returned"))?;

    let return_date = Utc::now().to_rfc3339();

    sqlx::query("UPDATE borrow_records SET return_date = ? WHERE id = ?")
        .bind(&return_date)
        .bind(record_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE books SET copies_available = copies_available + 1 WHERE id = ?")
        .bind(record)
        .execute(pool)
        .await?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"returned\"}";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

/// Get borrowed books for a user by user_id query param
pub async fn get_borrowed_books(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let query = request.split('?').nth(1).unwrap_or("");
    let user_id = query
        .split('&')
        .find(|p| p.starts_with("user_id="))
        .and_then(|p| p.strip_prefix("user_id=").and_then(|id| id.parse::<i32>().ok()))
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing user_id"))?;

    let rows = sqlx::query(
        r#"SELECT br.id, b.title, b.author, br.borrow_date, br.due_date 
           FROM borrow_records br
           JOIN books b ON br.book_id = b.id
           WHERE br.user_id = ? AND br.return_date IS NULL"#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let records: Vec<_> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<i32, _>("id"),
            "title": row.get::<String, _>("title"),
            "author": row.get::<String, _>("author"),
            "borrow_date": row.get::<String, _>("borrow_date"),
            "due_date": row.get::<String, _>("due_date")
        })
    }).collect();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&records)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

/// Get overdue books for a user
pub async fn get_overdue_books(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let query = request.split('?').nth(1).unwrap_or("");
    let user_id = query
        .split('&')
        .find(|p| p.starts_with("user_id="))
        .and_then(|p| p.strip_prefix("user_id=").and_then(|id| id.parse::<i32>().ok()))
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing user_id"))?;

    let rows = sqlx::query(
        r#"SELECT br.id, b.title, b.author, br.borrow_date, br.due_date 
           FROM borrow_records br
           JOIN books b ON br.book_id = b.id
           WHERE br.user_id = ? AND br.return_date IS NULL AND br.due_date < date('now')"#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let records: Vec<_> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<i32, _>("id"),
            "title": row.get::<String, _>("title"),
            "author": row.get::<String, _>("author"),
            "borrow_date": row.get::<String, _>("borrow_date"),
            "due_date": row.get::<String, _>("due_date")
        })
    }).collect();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&records)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}
