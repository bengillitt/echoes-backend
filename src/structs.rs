use sqlx::{sqlite::SqlitePool, FromRow};
use serde::{Deserialize, Serialize};

// -------------
// DB Structs
// -------------

#[derive(FromRow)]
pub struct EmbeddingReturnData {
    pub id: i32,
    pub embedding: Vec<u8>,
}

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
    pub email: String,
    pub username: String,
    pub hashed_password: String,
    pub salt: Vec<u8>,
    pub is_admin: bool,
}

#[derive(FromRow)]
pub struct ID {
    pub id: i32,
}

// ----------------
// Server Structs
// ----------------
#[derive(Deserialize)]
pub struct Prompt {
    pub prompt: String,
    pub token: String,
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
pub struct UserSearch {
    pub username: String,
}

#[derive(Deserialize)]
pub struct SimilarityPrompts {
    pub prompt1: String,
    pub prompt2: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

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