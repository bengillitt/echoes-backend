use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use serde::Deserialize;

use super::db_integration;

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<String>,
}

#[derive(Deserialize)]
struct Message {
    message: String,
}

pub async fn spawn_server(tx: mpsc::Sender<String>,mut rx: mpsc::Receiver<SqlitePool>) {
    let app_tx = tx.clone();

    let state = AppState { tx: app_tx};

    let pool = rx.recv().await.unwrap();

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/Send-to-db",
            get(|| async {}).post(
                |State(server_tx): State<AppState>, Json(payload): Json<Message>| async move {
                    db_integration::upload_user(&pool);

                    return "Message sent successfully";
                },
            ),
        )
        .with_state(state);

    tx.try_send("Cross Threaded COMMUNICATION!!!".to_string());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
