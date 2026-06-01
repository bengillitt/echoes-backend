use serde::{Serialize, Deserialize};

use reqwest;

use dotenv::dotenv;

#[derive(Serialize)]
struct OpenAIChatRequest {
    input: String,
    model: String,
}

pub async fn upload_to_llm(prompt: String) -> Result<String, String> {
    dotenv().ok();

    let client = reqwest::Client::new();

    let mut input = String::new();

    input.push_str(&std::env::var("LLM_INSTRUCTIONS").unwrap());
    input.push_str(&prompt);

    let res = client.post("https://api.openai.com/v1/responses")
    .bearer_auth(std::env::var("OPENAI_API_KEY").unwrap())
    .json(&OpenAIChatRequest {
        input,
        model: "gpt-5.4-mini".to_string(),
    }).send().await.unwrap().text().await.unwrap();

    return Ok(res);
}