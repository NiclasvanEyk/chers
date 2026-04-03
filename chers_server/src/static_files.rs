use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;
use tracing::{debug, error, info, warn};

#[derive(RustEmbed)]
#[folder = "../chers_web/dist/client/"]
struct StaticFiles;

/// The SPA shell HTML file that serves as the entry point for client-side routing
const SHELL_FILE: &str = "_shell.html";

/// Serves the SPA shell HTML for root path and /_shell.html
pub async fn serve_shell() -> impl IntoResponse {
    debug!("Serving shell file ({}) for SPA route", SHELL_FILE);
    match StaticFiles::get(SHELL_FILE) {
        Some(content) => {
            debug!("Successfully serving {} ({} bytes)", SHELL_FILE, content.data.len());
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            error!("{} not found in embedded static files", SHELL_FILE);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Static files not found"))
                .unwrap()
        }
    }
}

pub async fn handler(path: Option<Path<String>>) -> impl IntoResponse {
    // Extract the path string, default to empty if not provided
    let path_str = path.map(|p| p.0).unwrap_or_default();
    
    debug!("Static file request: path='{}'", path_str);
    
    // If path is empty or doesn't have an extension, serve the shell (SPA routing)
    let is_spa_route = path_str.is_empty() || !path_str.contains('.');
    
    let file_path = if is_spa_route {
        debug!("Path '{}' identified as SPA route, serving {}", path_str, SHELL_FILE);
        SHELL_FILE
    } else {
        debug!("Path '{}' identified as static asset", path_str);
        &path_str
    };

    match StaticFiles::get(file_path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(file_path)
                .first_or_octet_stream()
                .to_string();
            
            debug!("Serving static file: {} ({} bytes, mime: {})", file_path, content.data.len(), mime_type);
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            warn!("Static file not found: {} (SPA route: {})", file_path, is_spa_route);
            // For SPA routes, always fall back to the shell
            if is_spa_route || !path_str.contains('.') {
                debug!("Falling back to {} for SPA route", SHELL_FILE);
                match StaticFiles::get(SHELL_FILE) {
                    Some(content) => Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(content.data))
                        .unwrap(),
                    None => {
                        error!("{} not found in embedded static files", SHELL_FILE);
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Static files not found"))
                            .unwrap()
                    }
                }
            } else {
                debug!("Returning 404 for missing asset: {}", file_path);
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not found"))
                    .unwrap()
            }
        }
    }
}

/// Verify at startup that we have the frontend files embedded
pub fn verify_embedded_files() {
    if StaticFiles::get(SHELL_FILE).is_none() {
        panic!(
            "Static frontend files not found!\n\
             The 'bundle-frontend' feature is enabled but chers_web/dist/client/ appears to be empty.\n\
             Please build the frontend first: just chers-static"
        );
    }
}
