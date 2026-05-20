use axum::{Router, routing::get};

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<String>,
}

#[derive(Deserialize)]
struct Message {
    message: String,
}

pub async fn spawn_server(tx: mpsc::Sender<String>,rx: mpsc::Receiver<String>) {
    let app_tx = tx.clone();

    let state = AppState { tx: app_tx};

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/Send-to-db",
            get(|| async {}).post(
                |State(server_tx): State<AppState>, Json(payload): Json<Message>| async move {
                    server_tx
                        .tx
                        .try_send(format!("{}", payload.message).to_string());

                    return "Message sent successfully";
                },
            ),
        )
        .with_state(state);

    tx.try_send("Cross Threaded COMMUNICATION!!!".to_string());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
