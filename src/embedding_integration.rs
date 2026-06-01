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