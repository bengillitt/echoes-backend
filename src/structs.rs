use serde::{Deserialize, Serialize};
use sqlx::{FromRow, sqlite::SqlitePool};

// -------------
// DB Structs
// -------------

// #[derive(FromRow)]
// pub struct EmbeddingReturnData {
//     pub id: i32,
//     pub embedding: Vec<u8>,
// }

#[derive(FromRow)]
pub struct MessageReturnData {
    pub id: i32,
    pub contents: String,
    pub chat_id: i32,
    pub position: i32,
    pub message_role: i32,
    pub embedding: Vec<u8>,
}

pub struct Message {
    pub id: i32,
    pub contents: String,
    pub chat_id: i32,
    pub position: i32,
    pub message_role: i32,
    pub embedding: Vec<f32>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct MessageResponse {
    pub id: i32,
    pub contents: String,
    pub message_role: i32,
    pub position: i32,
}

#[derive(FromRow)]
pub struct ChatReturnData {
    pub user_id: i32,
    pub continuation_chat_id: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct ChatResponse {
    pub id: i32,
    pub user_id: i32,
    pub messages: Vec<MessageResponse>,
    pub feedback: i32,
}

#[derive(Clone, Serialize, Debug)]
pub struct MessageWithScore {
    pub id: i32,
    pub contents: String,
    pub chat_id: i32,
    pub position: i32,
    pub message_role: i32,
    pub embedding: Vec<f32>,
    pub score: f32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub hashed_password: String,
    pub salt: Vec<u8>,
}

#[derive(FromRow, Serialize)]
pub struct UserResponse {
    pub email: String,
    pub username: String,
}

#[derive(FromRow, Deserialize)]
pub struct ID {
    pub id: i32,
}

#[derive(FromRow)]
pub struct UserId {
    pub user_id: i32,
}

#[derive(FromRow)]
pub struct ContinuationChat {
    pub continuation_chat_id: Option<i32>,
}

#[derive(FromRow)]
pub struct FeedbackData {
    pub vote_type: i32,
}

// ----------------
// Server Structs
// ----------------
#[derive(Deserialize)]
pub struct Prompt {
    pub prompt: String,
    // pub token: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct UserInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ContinueChatInput {
    pub chat_id: i32,
    pub prompt: String,
    // pub token: String,
}

#[derive(Deserialize)]
pub struct ChatInteractionInput {
    pub chat_id: i32,
    pub interaction: i32, // 1 = like, 0 = dislike
    // pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

// #[derive(Deserialize)]
// pub struct Token {
//     pub token: String,
// }

// ---------------
// Algorithms
// ---------------
pub struct PasswordPair {
    pub hashed_password: String,
    pub salt: Vec<u8>,
}

// -------------
// LLM Structs
// -------------

#[derive(Deserialize)]
pub struct LLMResponse {
    pub output: Vec<LLMOutput>,
}

#[derive(Deserialize)]
pub struct LLMOutput {
    pub content: Vec<LLMContent>,
}

#[derive(Deserialize)]
pub struct LLMContent {
    pub text: String,
}

// -------------
// Embedding Structs
// -------------

#[derive(Serialize)]
pub struct OpenAIRequest {
    pub input: String,
    pub model: String,
}

#[derive(Deserialize)]
pub struct OpenAIEmbedResponse {
    pub data: Vec<OpenAIEmbedData>,
}

#[derive(Deserialize)]
pub struct OpenAIEmbedData {
    pub embedding: Vec<f32>,
}
