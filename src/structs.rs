use sqlx::{sqlite::SqlitePool, FromRow};
use serde::Deserialize;

// DB Structs
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
    pub embedding: Vec<u8>,
}

pub struct Message {
    pub id: i32,
    pub contents: String,
    pub chat_id: i32,
    pub position: i32,
    pub embedding: Vec<f32>,
}

pub struct MessageWithScore {
    pub id: i32,
    pub contents: String,
    pub chat_id: i32,
    pub position: i32,
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

// Server Structs
#[derive(Deserialize)]
pub struct Prompt {
    pub prompt: String
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct NewUser {
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
}

// Algorithms
pub struct PasswordPair {
    pub hashed_password: String,
    pub salt: Vec<u8>,
}