use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;
use tracing::error;

#[derive(RustEmbed)]
#[folder = "../chers_web/dist/"]
struct StaticFiles;

pub async fn handler(Path(path): Path<String>) -> impl IntoResponse {
    // If path is empty or doesn't have an extension, serve index.html (SPA routing)
    let is_spa_route = path.is_empty() || !path.contains('.');
    
    let file_path = if is_spa_route {
        "index.html"
    } else {
        &path
    };

    match StaticFiles::get(file_path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(file_path)
                .first_or_octet_stream()
                .to_string();
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            if is_spa_route {
                // For SPA routes, always fall back to index.html
                match StaticFiles::get("index.html") {
                    Some(content) => Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(content.data))
                        .unwrap(),
                    None => {
                        error!("index.html not found in embedded static files");
                        panic!("index.html not found in embedded static files - was the frontend built?");
                    }
                }
            } else {
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
    if StaticFiles::get("index.html").is_none() {
        panic!(
            "Static frontend files not found!\n\
             The 'bundle-frontend' feature is enabled but chers_web/dist/ appears to be empty.\n\
             Please build the frontend first: cd chers_web && BUILD_STATIC=true pnpm run build"
        );
    }
}
