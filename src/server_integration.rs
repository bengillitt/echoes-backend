use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::response::{IntoResponse, Response};

use jsonwebtoken::{EncodingKey, Header, encode};

use axum::http::{StatusCode, header};

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use std::time::{SystemTime, UNIX_EPOCH};

use super::{db_integration, embedding_integration};

use super::structs::{
    AppState, ChatInteractionInput, Claims, ContinueChatInput, Prompt, UserInput,
};

use dotenv::dotenv;

use serde_json::json;

pub async fn spawn_server(mut rx: mpsc::Receiver<SqlitePool>) {
    let pool = rx.recv().await.unwrap();

    let state = AppState { pool: pool };

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/me",
            get(|| async {}).post(get_user),
        )
        .route(
            "/register", // Done
            get(|| async {}).post(register_user),
        )
        .route(
            "/login", // Done
            get(|| async {}).post(login_user),
        )
        .route(
            "/getSimilarChats", // Done
            get(|| async {}).post(get_similar_chats),
        )
        .route(
            "/createNewChat", // Done
            get(|| async {}).post(create_new_chat),
        )
        .route(
            "/continueChat", // Done
            get(|| async {}).post(continue_chat),
        )
        .route(
            "/getChat", // In Progress
            get(|| async {}).post(lookup_chat),
        )
        .route(
            "/chatInteraction", // In Progress
            get(|| async {}).post(chat_interaction),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register_user(
    State(pool_state): State<AppState>,
    Json(payload): Json<UserInput>,
) -> Response {
    dotenv().ok();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let user = match db_integration::register_user(
        &pool_state.pool,
        payload.username,
        payload.email,
        payload.password,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occured \n {}", e),
            )
                .into_response();
        }
    };

    let claims = Claims {
        sub: user,
        exp: (current_time + 60 * 60) as usize, // Token expires in 1 hour
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token generation failed \n {}", e),
            )
                .into_response();
        }
    };

    let cookie = format!("token={}; HttpOnly; Secure;", token);

    let body = Json(json!({
        "message": "User registered successfully",
    }));

    return (StatusCode::OK, [(header::SET_COOKIE, cookie)], body).into_response();
}

async fn login_user(
    State(pool_state): State<AppState>,
    Json(payload): Json<UserInput>,
) -> Response {
    dotenv().ok();
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let user_id = match db_integration::login_user(
        &pool_state.pool,
        payload.username,
        payload.email,
        payload.password,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occured Failed with: \n {}", e),
            )
                .into_response();
        }
    };

    let claims = Claims {
        sub: user_id.to_string(),
        exp: (current_time + 60 * 60) as usize, // Token expires in 1 hour
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token generation failed \n {}", e),
            )
                .into_response();
        }
    };

    let cookie = format!("token={}; HttpOnly; Secure;", token);

    let body = Json(json!({
        "message": "User logged in successfully",
    }));

    return (StatusCode::OK, [(header::SET_COOKIE, cookie)], body).into_response();
}

async fn get_similar_chats(
    State(pool_state): State<AppState>,
    Json(payload): Json<Prompt>,
) -> Response {
    let embedded_prompt = match embedding_integration::get_embedding(payload.prompt).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error occurred while fetching embedding: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response();
        }
    };

    let return_data =
        match db_integration::get_similar_messages(&pool_state.pool, embedded_prompt).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error occurred while fetching similar messages: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e })),
                )
                    .into_response();
            }
        };

    return (StatusCode::OK, Json(return_data)).into_response();
}

// async fn upload_embedding(Json(payload): Json<Prompt>) -> String {
//     return match embedding_integration::get_embedding(payload.prompt).await {
//         Ok(v) => format!("{:?}", v),
//         Err(e) => e,
//     };
// }

async fn create_new_chat(
    State(pool_state): State<AppState>,
    Json(payload): Json<Prompt>,
) -> Response {
    // Need to first figure out how tokens work to get and keep user data
    let body = match db_integration::upload_and_return_chat(
        &pool_state.pool,
        payload.prompt,
        payload.token,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": e
        }))).into_response(),
    };

    return (StatusCode::OK, Json(body)).into_response();
}

async fn continue_chat(
    State(pool_state): State<AppState>,
    Json(payload): Json<ContinueChatInput>,
) -> Response {
    let body = match db_integration::continue_chat(
        &pool_state.pool,
        payload.chat_id,
        payload.prompt,
        payload.token,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    };

    return (StatusCode::OK, Json(body)).into_response();
}

// async fn test_similarity(Json(payload): Json<SimilarityPrompts>) -> String {
//     let embedding1 = match embedding_integration::get_embedding(payload.prompt1).await {
//         Ok(v) => v,
//         Err(e) => return e,
//     };

//     let embedding2 = match embedding_integration::get_embedding(payload.prompt2).await {
//         Ok(v) => v,
//         Err(e) => return e,
//     };

//     println!("{:?}", embedding1);

//     return match embedding_integration::calculate_similarity(&embedding1, &embedding2) {
//         Ok(s) => s.to_string(),
//         Err(e) => e,
//     };
// }

async fn chat_interaction(
    State(pool_state): State<AppState>,
    Json(payload): Json<ChatInteractionInput>,
) -> Response {
    let body = match db_integration::chat_interaction(
        &pool_state.pool,
        payload.chat_id,
        payload.interaction,
        payload.token,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))).into_response(),
    };

    return (StatusCode::OK, Json(body)).into_response();
}

async fn lookup_chat(State(pool_state): State<AppState>) -> Response {
    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "In Progress!"}))).into_response();
}

async fn get_user(State(pool_state): State<AppState>) -> Response {
    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "In Progress!"}))).into_response();
}
