use std::io::Write;
use sqlx::SqlitePool;
use bcrypt::{hash, verify};
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use anyhow::Result;
use crate::db;

/// Sanitizes the request body to extract valid JSON
fn sanitize_json_body(raw_body: &str) -> &str {
    let body = raw_body.trim();
    if let Some(start) = body.find('{') {
        let mut brace_count = 1;
        let mut end = start + 1;
        while end < body.len() && brace_count > 0 {
            match body.chars().nth(end) {
                Some('{') => brace_count += 1,
                Some('}') => brace_count -= 1,
                _ => {}
            }
            end += 1;
        }
        if brace_count == 0 && end <= body.len() {
            return &body[start..end];
        }
    }
    body
}

/// Handles user registration (only lenders allowed)
pub async fn handle_register(
    stream: &mut impl Write,
    request: &str,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(body_start) = request.find("\r\n\r\n") {
        let raw_body = &request[(body_start + 4)..];
        let body = sanitize_json_body(raw_body);

        let data: Value = serde_json::from_str(body).map_err(|e| {
            eprintln!("JSON parse error in register: {}", e);
            e
        })?;

        let username = data["username"].as_str().unwrap_or("").trim();
        let email = data["email"].as_str().unwrap_or("").trim();
        let password = data["password"].as_str().unwrap_or("").trim();
        let role = data["role"].as_str().unwrap_or("lender").trim();

        if username.is_empty() || email.is_empty() || password.is_empty() {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Username, email, and password are required\"}";
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
        if role != "lender" {
            let response = "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Only lenders can register via this endpoint\"}";
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }

        let hashed = hash(password, 10).map_err(|e| {
            eprintln!("Password hashing error: {}", e);
            e
        })?;

        match db::create_user(pool, username, email, &hashed, role).await {
            Ok(_) => {
                let json = json!({ "status": "registered", "role": role });
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}", json);
                stream.write_all(response.as_bytes())?;
            }
            Err(e) => {
                eprintln!("DB error during registration: {}", e);
                let response = format!("HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{{\"error\":\"{}\"}}", e);
                stream.write_all(response.as_bytes())?;
            }
        }
    } else {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"No body found\"}";
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}

/// Handles user login, supports admin (sha256) and lender (bcrypt)
pub async fn handle_login(
    stream: &mut impl Write,
    request: &str,
    pool: &SqlitePool,
) -> Result<()> {
    if let Some(body_start) = request.find("\r\n\r\n") {
        let raw_body = &request[(body_start + 4)..];
        let body = sanitize_json_body(raw_body);

        let data: Value = if request.contains("Content-Type: application/json") {
            serde_json::from_str(body).map_err(|e| {
                eprintln!("JSON parse error in login: {}", e);
                e
            })?
        } else if request.contains("Content-Type: application/x-www-form-urlencoded") {
            let mut data = json!({});
            for pair in body.split('&') {
                let parts: Vec<&str> = pair.splitn(2, '=').collect();
                if parts.len() == 2 {
                    data[parts[0]] = Value::String(urlencoding::decode(parts[1])?.into_owned());
                }
            }
            data
        } else {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Unsupported content type\"}";
            stream.write_all(response.as_bytes())?;
            return Ok(());
        };

        let username = data["username"].as_str().unwrap_or("").trim();
        let password = data["password"].as_str().unwrap_or("").trim();
        let role = data["role"].as_str().unwrap_or("").trim();

        if username.is_empty() || password.is_empty() || role.is_empty() {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Missing credentials\"}";
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }

        match db::get_user_by_username(pool, username).await {
            Ok(Some((id, _uname, stored_password, stored_role))) => {
                if stored_role != role {
                    let response = "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Role mismatch\"}";
                    stream.write_all(response.as_bytes())?;
                    return Ok(());
                }

                let is_valid = if stored_role == "admin" {
                    let sha = format!("{:x}", Sha256::digest(password.as_bytes()));
                    sha == stored_password
                } else {
                    verify(password, &stored_password).unwrap_or(false)
                };

                if is_valid {
                    let json = json!({
                        "status": "success",
                        "username": username,
                        "role": stored_role,
                        "user_id": id
                    });
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nSet-Cookie: username={}; Path=/; HttpOnly\r\nSet-Cookie: user_id={}; Path=/; HttpOnly\r\n\r\n{}",
                        username, id, json
                    );
                    eprintln!("Login successful for user: {}", username);
                    stream.write_all(response.as_bytes())?;
                } else {
                    let response = "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Invalid credentials\"}";
                    stream.write_all(response.as_bytes())?;
                }
            }
            Ok(None) => {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\":\"User not found\"}";
                stream.write_all(response.as_bytes())?;
            }
            Err(e) => {
                eprintln!("DB error during login: {}", e);
                let response = format!("HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n{{\"error\":\"Database error: {}\"}}", e);
                stream.write_all(response.as_bytes())?;
            }
        }
    } else {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\r\n{\"error\":\"No body found\"}";
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}
