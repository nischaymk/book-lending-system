use std::fs;
use std::io::Write;
use std::path::Path;

/// Returns the proper Content-Type based on file extension
fn get_content_type(path: &str) -> &str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

/// Serves static files from ./static/ folder
pub fn serve_static(stream: &mut impl Write, request: &str) -> std::io::Result<()> {
    if let Some(path_start) = request.find("GET ") {
        if let Some(path_end) = request[path_start..].find(" HTTP/") {
            let request_path = &request[path_start + 4..path_start + path_end];
            let filepath = format!(".{}", request_path);

            if Path::new(&filepath).exists() {
                let content = fs::read(&filepath)?;
                let content_type = get_content_type(&filepath);

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    content_type,
                    content.len()
                );
                stream.write_all(response.as_bytes())?;
                stream.write_all(&content)?;
                return Ok(());
            }
        }
    }
    respond_404(stream)
}

/// Sends a basic 404 Not Found response
pub fn respond_404(stream: &mut impl Write) -> std::io::Result<()> {
    let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\n\r\n<h1>404 Not Found</h1>";
    stream.write_all(response.as_bytes())?;
    stream.flush()
}
