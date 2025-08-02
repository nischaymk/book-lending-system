use std::io::Write;
use sqlx::{SqlitePool, Row};
use serde_json::{Value, json};
use anyhow::Result;

pub async fn create_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let body_start = request.find("\r\n\r\n").unwrap_or(request.len());
    let body = &request[(body_start + 4)..];
    let data: Value = serde_json::from_str(body)?;

    let title = data["title"].as_str().unwrap_or("").trim();
    let author = data["author"].as_str().unwrap_or("").trim();
    let isbn = data["isbn"].as_str().unwrap_or("").trim();
    let year = data["publication_year"].as_i64().unwrap_or(0);
    let genre = data["genre"].as_str().unwrap_or("").trim();
    let copies = data["copies_available"].as_i64().unwrap_or(1);

    if title.is_empty() || author.is_empty() || isbn.is_empty() || year == 0 || genre.is_empty() {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Missing or invalid book fields\"}";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO books (title, author, isbn, publication_year, genre, copies_available, status) VALUES (?, ?, ?, ?, ?, ?, 'available')"
    )
    .bind(title)
    .bind(author)
    .bind(isbn)
    .bind(year)
    .bind(genre)
    .bind(copies)
    .execute(pool)
    .await?;

    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"book added\"}";
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

pub async fn get_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let query = request.split('?').nth(1).unwrap_or("");
    let id = query.split('&')
        .find(|p| p.starts_with("id="))
        .and_then(|p| p.strip_prefix("id=").and_then(|id| id.parse::<i64>().ok()));

    if id.is_none() {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Missing or invalid book id\"}";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }
    let id = id.unwrap();

    if let Some(row) = sqlx::query("SELECT * FROM books WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await? {
        let book = json!({
            "id": row.get::<i64, _>("id"),
            "title": row.get::<String, _>("title"),
            "author": row.get::<String, _>("author"),
            "isbn": row.get::<String, _>("isbn"),
            "publication_year": row.get::<i64, _>("publication_year"),
            "genre": row.get::<String, _>("genre"),
            "copies_available": row.get::<i64, _>("copies_available"),
            "status": row.get::<String, _>("status"),
        });

        let json = serde_json::to_string(&book)?;
        let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}", json);
        stream.write_all(resp.as_bytes())?;
    } else {
        let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Book not found\"}";
        stream.write_all(resp.as_bytes())?;
    }
    Ok(())
}

pub async fn update_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let body_start = request.find("\r\n\r\n").unwrap_or(request.len());
    let body = &request[(body_start + 4)..];
    let data: Value = serde_json::from_str(body)?;

    let id = data["id"].as_i64().unwrap_or(-1);
    if id < 1 {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Invalid book id\"}";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    let title = data["title"].as_str().unwrap_or("").trim();
    let author = data["author"].as_str().unwrap_or("").trim();
    let isbn = data["isbn"].as_str().unwrap_or("").trim();
    let year = data["publication_year"].as_i64().unwrap_or(0);
    let genre = data["genre"].as_str().unwrap_or("").trim();
    let copies = data["copies_available"].as_i64().unwrap_or(1);
    let status = data["status"].as_str().unwrap_or("available").trim();

    if title.is_empty() || author.is_empty() || isbn.is_empty() || year == 0 || genre.is_empty() {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Missing or invalid book fields\"}";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    sqlx::query(
        "UPDATE books SET title = ?, author = ?, isbn = ?, publication_year = ?, genre = ?, copies_available = ?, status = ? WHERE id = ?"
    )
    .bind(title)
    .bind(author)
    .bind(isbn)
    .bind(year)
    .bind(genre)
    .bind(copies)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;

    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"book updated\"}";
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

pub async fn delete_book(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let body_start = request.find("\r\n\r\n").unwrap_or(request.len());
    let body = &request[(body_start + 4)..];
    let data: Value = serde_json::from_str(body)?;
    let id = data["id"].as_i64().unwrap_or(-1);

    if id < 1 {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Invalid book id\"}";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"book deleted\"}";
    stream.write_all(resp.as_bytes())?;
    Ok(())
}
