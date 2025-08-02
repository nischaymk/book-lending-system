use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use sqlx::SqlitePool;
use crate::utils::respond_404;
mod db;
mod auth;
mod book;
mod borrow;
mod admin;
mod db_setup;
mod utils;

fn parse_request(request: &str) -> (String, String) {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return (String::new(), String::new());
    }
    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 2 {
        return (String::new(), String::new());
    }
    (first_line[0].to_string(), first_line[1].to_string())
}

async fn handle_client(mut stream: TcpStream, pool: SqlitePool) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) | Err(_) => return,
        Ok(n) => n,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    println!("Request: {}", request); // Debug

    let (method, path) = parse_request(&request);

    // Serve static files
    if path.starts_with("/static/") {
        if let Err(e) = utils::serve_static(&mut stream, &request) {
            eprintln!("Error serving static: {}", e);
        }
        return;
    }

    // Serve HTML templates from ./templates
    let template_path = if path == "/" {
        "templates/login.html".to_string()
    } else {
        format!("templates{}", path)
    };
    if method == "GET" && Path::new(&template_path).exists() {
        match fs::read_to_string(&template_path) {
            Ok(content) => {
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", content);
                if let Err(e) = stream.write_all(response.as_bytes()) {
                    eprintln!("Error writing response: {}", e);
                }
                return;
            }
            Err(e) => {
                eprintln!("Failed to read template {}: {}", template_path, e);
                let _ = respond_404(&mut stream);
                return;
            }
        }
    }

    // API routing
    match (method.as_str(), path.as_str()) {
        ("POST", "/api/auth/register") => {
            if let Err(e) = auth::handle_register(&mut stream, &request, &pool).await {
                eprintln!("Error in handle_register: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("POST", "/api/auth/login") => {
            if let Err(e) = auth::handle_login(&mut stream, &request, &pool).await {
                eprintln!("Error in handle_login: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/admin") => {
            if let Err(e) = admin::get_all_books(&mut stream, &pool).await {
                eprintln!("Error in get_all_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/admin/books") => {
            let query = path.split('?').nth(1).unwrap_or("");
            if let Err(e) = admin::search_books(&mut stream, &pool, query).await {
                eprintln!("Error in search_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/book") => {
            if let Err(e) = book::get_book(&mut stream, &request, &pool).await {
                eprintln!("Error in get_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("POST", "/api/book") => {
            if let Err(e) = book::create_book(&mut stream, &request, &pool).await {
                eprintln!("Error in create_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("PUT", "/api/book") => {
            if let Err(e) = book::update_book(&mut stream, &request, &pool).await {
                eprintln!("Error in update_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("DELETE", "/api/book") => {
            if let Err(e) = book::delete_book(&mut stream, &request, &pool).await {
                eprintln!("Error in delete_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("POST", "/api/borrow") => {
            if let Err(e) = borrow::borrow_book(&mut stream, &request, &pool).await {
                eprintln!("Error in borrow_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("PUT", "/api/borrow") => {
            if let Err(e) = borrow::return_book(&mut stream, &request, &pool).await {
                eprintln!("Error in return_book: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/borrow") => {
            if let Err(e) = borrow::get_borrowed_books(&mut stream, &request, &pool).await {
                eprintln!("Error in get_borrowed_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/borrow/overdue") => {
            if let Err(e) = borrow::get_overdue_books(&mut stream, &request, &pool).await {
                eprintln!("Error in get_overdue_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/admin/users") => {
            if let Err(e) = admin::get_all_users(&mut stream, &pool).await {
                eprintln!("Error in get_all_users: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("DELETE", "/api/admin/users") => {
            if let Err(e) = admin::delete_user(&mut stream, &request, &pool).await {
                eprintln!("Error in delete_user: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/admin/borrowed") => {
            if let Err(e) = admin::get_all_borrowed_books(&mut stream, &pool).await {
                eprintln!("Error in get_all_borrowed_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        ("GET", "/api/admin/overdue") => {
            if let Err(e) = admin::get_all_overdue_books(&mut stream, &pool).await {
                eprintln!("Error in get_all_overdue_books: {}", e);
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Internal server error\"}";
                let _ = stream.write_all(response.as_bytes());
            }
        }
        _ => {
            if method == "GET" && path.contains("?") {
                eprintln!("Unhandled GET with query parameters: {}", path);
            }
            let _ = respond_404(&mut stream);
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("🚀 Server running at http://127.0.0.1:8080");

    let pool = db_setup::initialize_db().await.expect("Failed to initialize database");
    println!("Database connected successfully");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool = pool.clone();
                tokio::spawn(async move {
                    handle_client(stream, pool).await;
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e)
        }
    }
}
