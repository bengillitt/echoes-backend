use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::response::{IntoResponse, Response};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use axum::http::{StatusCode, header};

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use std::time::{SystemTime, UNIX_EPOCH};

use super::{db_integration, embedding_integration, llm_integration};

use super::structs::{AppState, UserInput, Prompt, SimilarityPrompts, MessageWithScore, Claims};

use dotenv::dotenv;

use serde_json::json;

pub async fn spawn_server(mut rx: mpsc::Receiver<SqlitePool>) {
    let pool = rx.recv().await.unwrap();

    let state = AppState {pool: pool};

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/register", // Done
            get(|| async {}).post(register_user),
        ).route(
            "/login", // Done
            get(|| async {}).post(login_user),
        ).route(
            "/getSimilarChats", // Done
            get(|| async {}).post(get_similar_chats),
        ).route(
            "/createNewChat", // Done
            get(|| async {}).post(create_new_chat),
        ).route(
            "/continueChat", // In Progress
            get(|| async {}).post(continue_chat),
        ).route(
            "/getChat", // In Progress
            get(|| async {}).post(lookup_chats)
        ).route(
            "/chatInteraction", // In Progress
            get(|| async {}).post(chat_interaction)
        ).route(
            "/testSimilarity", // Will remove later
            get(|| async {}).post(test_similarity)
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register_user(State(pool_state): State<AppState>, Json(payload): Json<UserInput>) -> Response {
    dotenv().ok();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let user =  match db_integration::register_user(&pool_state.pool, payload.username, payload.email, payload.password).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("An error occured \n {}", e)).into_response(),
    };

    let claims = Claims {
        sub: user,
        exp: (current_time + 60 * 60) as usize, // Token expires in 1 hour
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Token generation failed \n {}", e)).into_response(),
    };

    let cookie = format!("token={}; HttpOnly; Secure;", token);

    let body = Json(json!({
        "message": "User registered successfully",
    }));

    return (StatusCode::OK, [(header::SET_COOKIE, cookie)], body).into_response();
}

async fn login_user(State(pool_state): State<AppState>, Json(payload): Json<UserInput>) -> Response {
    dotenv().ok();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let userId = match db_integration::login_user(&pool_state.pool, payload.username, payload.email, payload.password).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("An error occured Failed with: \n {}", e)).into_response(),
    };

    let claims = Claims {
        sub: userId.to_string(),
        exp: (current_time + 60 * 60) as usize, // Token expires in 1 hour
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Token generation failed \n {}", e)).into_response(),
    };

    let cookie = format!("token={}; HttpOnly; Secure;", token);

    let body = Json(json!({
        "message": "User logged in successfully",
    }));
    
    return (StatusCode::OK, [(header::SET_COOKIE, cookie)], body).into_response();
}

async fn get_similar_chats(State(pool_state): State<AppState>, Json(payload): Json<Prompt>) -> Response {
    let embedded_prompt = match embedding_integration::get_embedding(payload.prompt).await  {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error occurred while fetching embedding: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response()},
    };
    
    let return_data = match db_integration::get_similar_messages(&pool_state.pool, embedded_prompt).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error occurred while fetching similar messages: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))).into_response()},
     };

     return (StatusCode::OK, Json(return_data)).into_response();
}

// async fn upload_embedding(Json(payload): Json<Prompt>) -> String {
//     return match embedding_integration::get_embedding(payload.prompt).await {
//         Ok(v) => format!("{:?}", v),
//         Err(e) => e,
//     };
// }

async fn create_new_chat(State(pool_state): State<AppState>, Json(payload): Json<Prompt>) -> String { // Need to first figure out how tokens work to get and keep user data
    return match db_integration::upload_and_return_chat(&pool_state.pool, payload.prompt, payload.token).await {
        Ok(s) => s,
        Err(e) => format!("An error occurred: \n {}", e),
    };
}

async fn continue_chat(State(pool_state): State<AppState>, Json(payload): Json<Prompt>) -> String {
    return "In Progress".to_string();
}

async fn test_similarity(Json(payload): Json<SimilarityPrompts>) -> String {
    let embedding1 = match embedding_integration::get_embedding(payload.prompt1).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    
    let embedding2 = match embedding_integration::get_embedding(payload.prompt2).await {
        Ok(v) => v,
        Err(e) => return e,
    };

    println!("{:?}", embedding1);
    
    return match embedding_integration::calculate_similarity(&embedding1, &embedding2) {
        Ok(s) => s.to_string(),
        Err(e) => e,
    };
}

async fn lookup_chats(State(pool_state): State<AppState>) -> String {
    return "In Progress".to_string();
} 

async fn chat_interaction(State(pool_state): State<AppState>) -> String {
    return "In Progress".to_string();
}