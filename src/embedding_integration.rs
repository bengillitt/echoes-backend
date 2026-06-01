use reqwest;

use serde::{Deserialize, Serialize};

use dotenv::dotenv;

use std::env;

#[derive(Serialize)]
struct OpenAIEmbedRequest {
    input: String,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIEmbedResponse {
    data: Vec<OpenAIEmbedData>,
}

#[derive(Deserialize)]
struct OpenAIEmbedData {
    embedding: Vec<f32>,
}

pub async fn get_embedding(prompt: String) -> Vec<f32> {
    dotenv().ok();

    let client = reqwest::Client::new();

    let res = client.post("https://api.openai.com/v1/embeddings")
    .bearer_auth(env::var("OPENAI_API_KEY").unwrap()).json(
        &OpenAIEmbedRequest {
            input: prompt,
            model: "text-embedding-3-small".to_string(),
        }
    ).send().await.unwrap().json::<OpenAIEmbedResponse>().await.unwrap();

    return res.data[0].embedding.clone();

    // let res = client.post("https://api.openai.com/v1/embeddings")
    // .bearer_auth(env::var("OPENAI_API_KEY").unwrap()).json(
    //     &OpenAIEmbedRequest {
    //         input: prompt,
    //         model: "text-embedding-3-small".to_string(),
    //     }
    // ).send().await.unwrap().text().await;

    // return res.unwrap();
}

fn calculate_dot_product(v1: &Vec<f32>, v2: &Vec<f32>) -> Result<f32, String> {
    if v1.len() != v2.len() {
        return Err("Vector lengths don't match".to_string());
    }

    let mut dot_product = 0.0;

    for i in 0..v1.len() {
        dot_product += v1[i] * v2[i];
    }

    return Ok(dot_product);
}

fn calculate_magnitude(v: &Vec<f32>,) -> f32 {
    let mut total_squares = 0.0;

    for i in 0..v.len() {
        total_squares = v[i] * v[i];
    }

    return total_squares.sqrt();
}

pub fn calculate_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = calculate_dot_product(v1, v2).unwrap();

    let magnitude1 = calculate_magnitude(v1);
    let magnitude2 = calculate_magnitude(v2);

    if magnitude1 * magnitude2 == 0.0 {
        return 0.0;
    } 

    return dot_product / (magnitude1 * magnitude2);
}