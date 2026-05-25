use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use serde::Deserialize;

use super::db_integration;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Deserialize)]
struct NewUser {
    username: String,
    email: String,
    hashed_password: String,
}

#[derive(Deserialize)]
struct UserSearch {
    username: String,
}

pub async fn spawn_server(mut rx: mpsc::Receiver<SqlitePool>) {
    let pool = rx.recv().await.unwrap();

    let state = AppState {pool: pool};

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/createUser",
            get(|| async {}).post(create_user),
        ).route(
            "/searchUser",
            get(search_user),
        ).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_user(State(pool_state): State<AppState>, Json(payload): Json<NewUser>) -> String {
    return db_integration::upload_user(&pool_state.pool, payload.username, payload.email, payload.hashed_password).await;
}

async fn search_user(State(pool_state): State<AppState>, Json(payload): Json<UserSearch>) -> String {
    return db_integration::get_user(&pool_state.pool, payload.username).await;
}
