use reqwest;

use dotenv::dotenv;

use std::env;

pub fn get_embedding(prompt: String) -> Vec<f32> {
    let client = reqwest::Client::new();

    let res = client.post("https://api.openai.com/v1/embeddings").bearer_auth(env::var("OPENAI_API_KEY"));
} 