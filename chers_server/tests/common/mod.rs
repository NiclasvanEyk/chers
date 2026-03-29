use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use chers::Coordinate;
use chers_server::{AppState, MatchRepository};

pub struct TestServer {
    pub addr: SocketAddr,
    _shutdown: Option<oneshot::Sender<()>>,
}

impl TestServer {
    pub async fn start() -> Self {
        // Create fresh app state
        let state = Arc::new(AppState {
            matches: MatchRepository::new(),
        });

        let app = Router::new()
            .route(
                "/health",
                axum::routing::get(chers_server::handlers::health::check),
            )
            .route(
                "/matches/new",
                axum::routing::post(chers_server::handlers::matches::new::create_new_match),
            )
            .route(
                "/matches/{id}/play",
                axum::routing::get(chers_server::handlers::matches::play::websocket_handler),
            )
            .with_state(state);

        // Bind to random available port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });

        // Give server a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        Self {
            addr,
            _shutdown: Some(shutdown_tx),
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{}", self.addr, path)
    }

    pub async fn create_match(&self) -> String {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/matches/new", self.base_url()))
            .send()
            .await
            .expect("Failed to create match");

        assert!(
            resp.status().is_success(),
            "Failed to create match: {:?}",
            resp
        );

        let body: serde_json::Value = resp.json().await.unwrap();
        body["id"].as_str().unwrap().to_string()
    }

    pub fn stop(mut self) {
        if let Some(tx) = self._shutdown.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self._shutdown.take() {
            let _ = tx.send(());
        }
    }
}

/// Test client that wraps WebSocket connection
pub struct TestClient {
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    match_id: String,
}

impl TestClient {
    pub async fn connect(server: &TestServer, match_id: &str) -> Self {
        let url = server.ws_url(&format!("/matches/{}/play", match_id));

        let (ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect WebSocket");

        Self {
            ws,
            match_id: match_id.to_string(),
        }
    }

    pub async fn authenticate(&mut self, token: &str, name: &str) {
        use chers_server_api::ClientMessage;

        let msg = ClientMessage::Authenticate {
            token: token.to_string(),
            name: name.to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.ws
            .send(tokio_tungstenite::tungstenite::Message::Text(json))
            .await
            .expect("Failed to send authenticate");
    }

    pub async fn make_move(&mut self, from: &str, to: &str) {
        use chers_server_api::ClientMessage;

        // Parse coordinates (e.g., "e2" -> Coordinate { x: 4, y: 1 })
        let from_coord = parse_coordinate(from);
        let to_coord = parse_coordinate(to);

        let msg = ClientMessage::MakeMove {
            from: from_coord,
            to: to_coord,
            promotion: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.ws
            .send(tokio_tungstenite::tungstenite::Message::Text(json))
            .await
            .expect("Failed to send move");
    }

    pub async fn expect_message(
        &mut self,
        timeout_secs: u64,
    ) -> Option<chers_server_api::ServerMessage> {
        use tokio_tungstenite::tungstenite::Message;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                println!("Timeout waiting for message");
                return None;
            }

            match tokio::time::timeout(remaining, self.ws.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    return serde_json::from_str(&text).ok();
                }
                Ok(Some(Ok(Message::Close(_)))) => {
                    println!("WebSocket closed");
                    return None;
                }
                Ok(Some(Ok(Message::Ping(_)))) => {
                    // Skip ping messages and continue
                    continue;
                }
                Ok(Some(Ok(Message::Pong(_)))) => {
                    // Skip pong messages and continue
                    continue;
                }
                Ok(Some(Ok(Message::Binary(_)))) => {
                    // Skip binary messages and continue
                    continue;
                }
                Ok(Some(Ok(Message::Frame(_)))) => {
                    // Skip frame messages and continue
                    continue;
                }
                Ok(Some(Err(e))) => {
                    println!("WebSocket error: {:?}", e);
                    return None;
                }
                Ok(None) => {
                    println!("WebSocket stream ended");
                    return None;
                }
                Err(_) => {
                    println!("Timeout waiting for message");
                    return None;
                }
            }
        }
    }

    pub async fn close(mut self) {
        let _ = self.ws.close(None).await;
    }
}

fn parse_coordinate(s: &str) -> Coordinate {
    // Parse algebraic notation (e.g., "e2" -> x: 4, y: 6)
    // Files (columns): a=0, b=1, c=2, d=3, e=4, f=5, g=6, h=7
    // Ranks (rows): 1=7, 2=6, 3=5, 4=4, 5=3, 6=2, 7=1, 8=0 (inverted!)
    // This is because in chers, y=0 is the top (rank 8) and y=7 is the bottom (rank 1)
    let chars: Vec<char> = s.chars().collect();
    assert!(chars.len() == 2, "Invalid coordinate: {}", s);

    let file = chars[0];
    let rank = chars[1];

    let x = (file as u8 - b'a') as usize;
    let y = 8 - (rank.to_digit(10).expect("Invalid rank") as usize); // Inverted: rank 1 -> y=7, rank 8 -> y=0

    assert!(x < 8, "Invalid file: {}", file);
    assert!(y < 8, "Invalid rank: {}", rank);

    Coordinate { x, y }
}
