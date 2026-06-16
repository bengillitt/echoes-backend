use reqwest;

use dotenv::dotenv;

use std::env;

use super::structs::{OpenAIEmbedResponse, OpenAIRequest};

pub async fn get_embedding(prompt: String) -> Result<Vec<f32>, String> {
    dotenv().ok();

    let client = reqwest::Client::new();

    let res = match client
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(env::var("OPENAI_API_KEY").unwrap())
        .json(&OpenAIRequest {
            input: prompt,
            model: "text-embedding-3-large".to_string(), // model can be changes if needed
        })
        .send()
        .await
    {
        Ok(d) => d,
        Err(e) => return Err(format!("Failed to fetch embedding. Failed with {}", e)),
    };

    let json_data = match res.json::<OpenAIEmbedResponse>().await {
        Ok(d) => d,
        Err(e) => {
            return Err(format!(
                "Failed to convert embedding into json. Failed with {}",
                e
            ));
        }
    };

    return Ok(json_data.data[0].embedding.clone());

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

fn calculate_magnitude(v: &Vec<f32>) -> f32 {
    let mut total_squares = 0.0;

    for i in 0..v.len() {
        total_squares += v[i] * v[i];
    }

    return total_squares.sqrt();
}

pub fn calculate_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> Result<f32, String> {
    let dot_product = calculate_dot_product(v1, v2)?;

    let magnitude1 = calculate_magnitude(v1);
    let magnitude2 = calculate_magnitude(v2);

    if magnitude1 * magnitude2 == 0.0 {
        return Err("One or both vectors have zero magnitude".to_string());
    }

    return Ok(dot_product / (magnitude1 * magnitude2));
}
