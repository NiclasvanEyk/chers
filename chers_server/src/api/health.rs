use std::net::SocketAddr;

use axum::{extract::ConnectInfo, response::Html};

pub async fn check(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Html<String> {
    Html(format!("Perfectly fine, thanks for asking {}", addr))
}
