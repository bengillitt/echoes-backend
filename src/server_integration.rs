use axum::{Router, routing::get};

use sqlx::sqlite::SqlitePool;

use axum::extract::{Json, State};

use tokio::sync::mpsc;

use super::{db_integration, embedding_integration, llm_integration};

use super::structs::{AppState, NewUser, Prompt, SimilarityPrompts};

pub async fn spawn_server(mut rx: mpsc::Receiver<SqlitePool>) {
    let pool = rx.recv().await.unwrap();

    let state = AppState {pool: pool};

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Get Echoes" }).post(|| async { "Post Echoes" }),
        )
        .route(
            "/register",
            get(|| async {}).post(register_user),
        ).route(
            "/login",
            get(|| async {}).post(login_user),
        ).route(
            "/getSimilarChats",
            get(get_similar_chats)
        ).route(
            "/createNewChat",
            get(|| async {}).post(create_new_chat),
        ).route(
            "/continueChat",
            get(|| async {}).post(continue_chat),
        ).route(
            "/lookupChats",
            get(|| async {}).post(lookup_chats)
        ).route(
            "/chatInteraction",
            get(|| async {}).post(chat_interaction)
        ).route(
            "/testSimilarity",
            get(|| async {}).post(test_similarity)
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register_user(State(pool_state): State<AppState>, Json(payload): Json<NewUser>) -> String {
    return match db_integration::register_user(&pool_state.pool, payload.username, payload.email, payload.hashed_password).await {
        Ok(s) => s,
        Err(e) => format!("An error occured \n {}", e),
    };
}

async fn login_user(State(pool_state): State<AppState>, Json(payload): Json<NewUser>) -> String {
    return match db_integration::login_user(&pool_state.pool, payload.username, payload.email, payload.hashed_password).await {
        Ok(s) => s,
        Err(e) => format!("An error occured Failed with: \n {}", e),
    };
}

async fn get_similar_chats(State(pool_state): State<AppState>, Json(payload): Json<Prompt>) -> String {
    let embedded_prompt = match embedding_integration::get_embedding(payload.prompt).await  {
        Ok(v) => v,
        Err(e) => return e,
    };
    
    let return_data = match db_integration::get_similar_messages(&pool_state.pool, embedded_prompt).await {
        Ok(s) => s,
        Err(e) => return e,
     };

     return "in progress".to_string();
}

// async fn upload_embedding(Json(payload): Json<Prompt>) -> String {
//     return match embedding_integration::get_embedding(payload.prompt).await {
//         Ok(v) => format!("{:?}", v),
//         Err(e) => e,
//     };
// }

async fn create_new_chat(Json(payload): Json<Prompt>) -> String {
    return llm_integration::upload_to_llm(payload.prompt).await.unwrap();
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
    
    return embedding_integration::calculate_similarity(&embedding1, &embedding2).to_string();
}

async fn lookup_chats(State(pool_state): State<AppState>) -> String {
    return "In Progress".to_string();
} 

async fn chat_interaction(State(pool_state): State<AppState>) -> String {
    return "In Progress".to_string();
}

