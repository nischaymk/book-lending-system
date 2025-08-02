use std::io::Write;
use sqlx::SqlitePool;
use serde_json::{json, Value};
use anyhow::Result;

pub async fn get_all_books(stream: &mut impl Write, pool: &SqlitePool) -> Result<()> {
    let books = sqlx::query!(
        "SELECT id, title, author, isbn, publication_year, genre, copies_available, status FROM books"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|b| {
        json!({
            "id": b.id,
            "title": b.title,
            "author": b.author,
            "isbn": b.isbn,
            "publication_year": b.publication_year,
            "genre": b.genre,
            "copies_available": b.copies_available,
            "status": b.status
        })
    })
    .collect::<Vec<Value>>();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&books)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

// Search books with query param like /api/admin/books?search=some_term
pub async fn search_books(stream: &mut impl Write, pool: &SqlitePool, query: &str) -> Result<()> {
    // Extract search term from query and build LIKE patterns
    let search_term = query.split('=').nth(1).unwrap_or("");
    let like_pattern = format!("%{}%", search_term);

    // Bind the pattern to variables to ensure proper lifetime
    let search_title = &like_pattern;
    let search_author = &like_pattern;
    let search_isbn = &like_pattern;

    let books = sqlx::query!(
        "SELECT id, title, author, isbn, publication_year, genre, copies_available, status
         FROM books
         WHERE title LIKE ? OR author LIKE ? OR isbn LIKE ?",
        search_title, search_author, search_isbn
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|b| {
        json!({
            "id": b.id,
            "title": b.title,
            "author": b.author,
            "isbn": b.isbn,
            "publication_year": b.publication_year,
            "genre": b.genre,
            "copies_available": b.copies_available,
            "status": b.status
        })
    })
    .collect::<Vec<Value>>();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&books)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

pub async fn get_all_users(stream: &mut impl Write, pool: &SqlitePool) -> Result<()> {
    let users = sqlx::query!(
        "SELECT id, username, email, role FROM users"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|u| {
        json!({
            "id": u.id,
            "username": u.username,
            "email": u.email,
            "role": u.role
        })
    })
    .collect::<Vec<Value>>();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&users)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

pub async fn delete_user(stream: &mut impl Write, request: &str, pool: &SqlitePool) -> Result<()> {
    let query = request.split('?').nth(1).unwrap_or("");
    let id = query
        .split('&')
        .find(|p| p.starts_with("id="))
        .and_then(|p| p.strip_prefix("id=").and_then(|id| id.parse::<i32>().ok()))
        .ok_or_else(|| anyhow::anyhow!("Invalid or missing user ID"))?;

    sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(pool)
        .await?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"deleted\"}";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

pub async fn get_all_borrowed_books(stream: &mut impl Write, pool: &SqlitePool) -> Result<()> {
    let records = sqlx::query!(
        r#"SELECT br.id, b.title, b.author, u.username, br.borrow_date, br.due_date
           FROM borrow_records br
           JOIN books b ON br.book_id = b.id
           JOIN users u ON br.user_id = u.id
           WHERE br.return_date IS NULL"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| {
        json!({
            "id": r.id,
            "title": r.title,
            "author": r.author,
            "username": r.username,
            "borrow_date": r.borrow_date,
            "due_date": r.due_date
        })
    })
    .collect::<Vec<Value>>();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&records)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

pub async fn get_all_overdue_books(stream: &mut impl Write, pool: &SqlitePool) -> Result<()> {
    let records = sqlx::query!(
        r#"SELECT br.id, b.title, b.author, u.username, br.borrow_date, br.due_date
           FROM borrow_records br
           JOIN books b ON br.book_id = b.id
           JOIN users u ON br.user_id = u.id
           WHERE br.return_date IS NULL AND br.due_date < date('now')"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| {
        json!({
            "id": r.id,
            "title": r.title,
            "author": r.author,
            "username": r.username,
            "borrow_date": r.borrow_date,
            "due_date": r.due_date
        })
    })
    .collect::<Vec<Value>>();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
        serde_json::to_string(&records)?
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}
