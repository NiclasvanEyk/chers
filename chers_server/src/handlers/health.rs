use axum::response::Html;

pub async fn check() -> Html<String> {
    Html("Perfectly fine, thanks for asking".to_string())
}
