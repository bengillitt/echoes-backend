use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use serde::Deserialize;

use super::{db_integration, embedding_integration};

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
        ).route(
            "/createEmbedding",
            get(create_embedding)
        ).route(
            "/getEmbeddings",
            get(get_embeddings)
        ).route(
            "/uploadEmbedding",
            get(|| async {}).post(upload_embedding),
        ).route(
            "/calculateSimilarity",
            get(|| async {}).post(test_similarity),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_user(State(pool_state): State<AppState>, Json(payload): Json<NewUser>) -> String {
    return db_integration::upload_user(&pool_state.pool, payload.username, payload.email, payload.hashed_password).await;
}

async fn search_user(State(pool_state): State<AppState>, Json(payload): Json<UserSearch>) -> String {
    return db_integration::get_user(&pool_state.pool, payload.username).await;
}

async fn create_embedding(State(pool_state): State<AppState>) -> String {
    return db_integration::upload_embedding(&pool_state.pool, vec![1.1, 4.8]).await;
}

async fn get_embeddings(State(pool_state): State<AppState>) -> String {
     println!("{:?}", db_integration::get_embeddings(&pool_state.pool).await);
     return "Success".to_string();
}

#[derive(Deserialize)]
struct EmbeddingPrompt {
    prompt: String
}

async fn upload_embedding(Json(payload): Json<EmbeddingPrompt>) -> String {
    println!("{:?}", embedding_integration::get_embedding(payload.prompt).await);
    return "Success".to_string();
}

#[derive(Deserialize)]
struct SimilarityPrompts {
    prompt1: String,
    prompt2: String,
}

async fn test_similarity(Json(payload): Json<SimilarityPrompts>) -> String {
    let embedding1 = embedding_integration::get_embedding(payload.prompt1).await;
    let embedding2 = embedding_integration::get_embedding(payload.prompt2).await;
    
    return embedding_integration::calculate_similarity(&embedding1, &embedding2).to_string();
}