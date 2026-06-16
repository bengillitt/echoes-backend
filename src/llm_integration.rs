use serde::{Serialize, Deserialize};

use reqwest;

use dotenv::dotenv;

use super::structs::{OpenAIRequest, LLMResponse};

pub async fn upload_to_llm(prompt: String) -> Result<String, String> {
    dotenv().ok();

    let client = reqwest::Client::new();

    let mut input = String::new();

    input.push_str(&std::env::var("LLM_INSTRUCTIONS").unwrap());
    input.push_str(&prompt);

    let res = client.post("https://api.openai.com/v1/responses")
    .bearer_auth(std::env::var("OPENAI_API_KEY").unwrap())
    .json(&OpenAIRequest {
        input,
        model: "gpt-5.4-mini".to_string(),
    }).send().await.unwrap();

    // println!("{}", res.text().await.unwrap());

    // return Ok("test".to_string());

    let json_data = res.json::<LLMResponse>().await.unwrap();

    return Ok(json_data.output[0].content[0].text.clone());
}