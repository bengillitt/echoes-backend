use super::embedding_integration;
use super::llm_integration;
pub use sqlx::sqlite::SqlitePool;

use async_recursion::async_recursion;

use tokio::sync::mpsc;

use super::structs::{
    ContinuationChat, ID, Message, MessageReturnData, MessageWithScore, User, UserId, ChatReturnData, MessageResponse, ChatResponse, FeedbackData, UserResponse
};

use super::algorithms;

pub async fn get_pool(tx: mpsc::Sender<SqlitePool>) {
    let pool = SqlitePool::connect("sqlite:./data/db.sqlite")
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Database Ready");

    let _ = tx.try_send(pool);
}

pub async fn register_user(
    pool: &SqlitePool,
    username: String,
    email: String,
    hashed_password: String,
) -> Result<String, String> {
    if !email.contains("@") || !email.contains(".") {
        // Could change this so it scans for TLDs
        return Err("Invalid Email".to_string());
    }

    if username == "".to_string() {
        return Err("Invalid username. Cannot be empty".to_string());
    }

    let mut is_alphanumeric: bool = true;

    for c in username.chars() {
        if !c.is_alphanumeric() {
            is_alphanumeric = false;
            break;
        }
    }

    if !is_alphanumeric {
        return Err("Invalid Username, can't contain symbols".to_string());
    }

    let users = match get_user_from_username(pool, &username).await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if users.len() > 0 {
        return Err("Username already exists".to_string());
    }

    let users = match get_user_from_email(pool, &email).await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if users.len() > 0 {
        return Err("Email already exists".to_string());
    }

    return upload_user(pool, username, email, hashed_password).await;
}

async fn upload_user(
    pool: &SqlitePool,
    username: String,
    email: String,
    password: String,
) -> Result<String, String> {
    let password_pair = match algorithms::hash_password(password) {
        Ok(p) => p,
        Err(e) => return Err(format!("Couldn't hash password. failed with:\n{}", e)),
    };

    println!("Uploading User");
    let return_data = match sqlx::query_as::<_, ID>("INSERT INTO tblUsers (email, username, hashed_password, salt) VALUES ($1, $2, $3, $4) RETURNING id").bind(email).bind(username).bind(password_pair.hashed_password).bind(password_pair.salt).fetch_one(pool).await {
        Ok(id) => id.id.to_string(),
        Err(err) => match err {
            sqlx::Error::Database(err) => {
                return Err(format!("Database Error. Failed with {}", err.to_string()));
            },
            _ => return Err(format!("Unexpected Error. Failed with {}", err)),
        },
    };

    return Ok(return_data);
    // Handle NOT UNIQUE Errors
    // Handle errors better to avoid crashes (crashes would still allow the server to load, return a 200 OK, but nothing would occur)
}

pub async fn login_user(
    pool: &SqlitePool,
    username: String,
    email: String,
    password: String,
) -> Result<String, String> {
    if username == "".to_string() && email == "".to_string() {
        return Err("Must provide a username or email.".to_string());
    }

    if username != "".to_string() {
        return login_user_with_username(pool, username, password).await;
    } else {
        return login_user_with_email(pool, email, password).await;
    }
}

fn check_user_password(users: &Vec<User>, password: String) -> Result<String, String> {
    if users.len() == 0 {
        return Err(format!("Incorrect Credentials"));
    }

    if users.len() == 1 {
        let hashed_password =
            match algorithms::hash_password_with_salt(password, users[0].salt.clone()) {
                Ok(v) => v.hashed_password,
                Err(_) => return Err("Can't hash password".to_string()),
            };

        if users[0].hashed_password == hashed_password {
            return Ok(users[0].id.to_string());
        } else {
            return Err(format!("Incorrect Credentials"));
        }
    } else {
        return Err(
            "DB Error, more than 1 user with those credentials, contact support".to_string(),
        );
    }
}

async fn login_user_with_username(
    pool: &SqlitePool,
    username: String,
    hashed_password: String,
) -> Result<String, String> {
    let users = match get_user_from_username(pool, &username).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Database Error. Failed with: \n {}", e)),
    };

    return check_user_password(&users, hashed_password);
}

async fn login_user_with_email(
    pool: &SqlitePool,
    email: String,
    hashed_password: String,
) -> Result<String, String> {
    let users = match get_user_from_email(pool, &email).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to find user. Failed with: \n {}",
                e.to_string()
            ));
        }
    };

    return check_user_password(&users, hashed_password);
}

async fn get_user_from_username(pool: &SqlitePool, username: &str) -> Result<Vec<User>, String> {
    let data: Vec<User> =
        match sqlx::query_as::<_, User>("SELECT id, email, username, hashed_password, salt, is_admin FROM tblUsers WHERE username = $1").bind(format!("{}", username))
            .fetch_all(pool)
            .await {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            };

    return Ok(data);
}

async fn get_user_from_email(pool: &SqlitePool, email: &str) -> Result<Vec<User>, String> {
    let data: Vec<User> = match sqlx::query_as::<_, User>("SELECT id, email, username, hashed_password, salt, is_admin FROM tblUsers WHERE email = $1")
        .bind(format!("{}", email)).fetch_all(pool).await {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

    return Ok(data);
}

async fn get_user_from_id(pool: &SqlitePool, id: i32) -> Result<Vec<User>, String> {
    let data: Vec<User> = match sqlx::query_as::<_, User>(
        "SELECT id, email, username, hashed_password, salt, is_admin FROM tblUsers WHERE id = $1",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    return Ok(data);
}

// async fn upload_embedding(pool: &SqlitePool, embedding: Vec<f32>) -> Result<String, String> {
//     println!("uploading embedding!");

//     let blob = vec_to_blob(&embedding);
//     let return_data = match sqlx::query("INSERT INTO tblEmbeddings (embedding) VALUES ($1);")
//         .bind(blob)
//         .execute(pool)
//         .await
//     {
//         Ok(_) => "Uploaded Embedding".to_string(),
//         Err(err) => {
//             return Err(format!(
//                 "Failed to upload embedding. Failed with: \n {}",
//                 err.to_string()
//             ));
//         }
//     };

//     return Ok(return_data);
// }

async fn create_new_chat_record(
    pool: &SqlitePool,
    user_id: i32,
    continue_chat_id: Option<i32>,
) -> Result<i32, String> {
    if continue_chat_id.is_some() {
        let new_chat_id = match sqlx::query_as::<_, ID>(
            "INSERT INTO tblChats (user_id, continuation_chat_id) VALUES ($1, $2) RETURNING id;",
        )
        .bind(user_id)
        .bind(continue_chat_id.unwrap())
        .fetch_one(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to create new chat record. Failed with: \n {}",
                    e.to_string()
                ));
            }
        }
        .id;

        return Ok(new_chat_id);
    }

    let new_chat_id =
        match sqlx::query_as::<_, ID>("INSERT INTO tblChats (user_id) VALUES ($1) RETURNING id;")
            .bind(user_id)
            .fetch_one(pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to create new chat record. Failed with: \n {}",
                    e.to_string()
                ));
            }
        }
        .id;

    return Ok(new_chat_id);
}

async fn create_new_message_with_embedding(
    pool: &SqlitePool,
    chat_id: i32,
    contents: String,
    position: i32,
    message_role: i32,
    embedding: Vec<f32>,
) -> Result<i32, String> {
    let blob = vec_to_blob(&embedding);
    let new_message_id = match sqlx::query_as::<_, ID>("INSERT INTO tblMessages (chat_id, contents, position, message_role, embedding) VALUES ($1, $2, $3, $4, $5) RETURNING id;").bind(chat_id).bind(contents).bind(position).bind(message_role).bind(blob).fetch_one(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to create new message record. Failed with: \n {}", e.to_string())),
    }.id;

    return Ok(new_message_id);
}

async fn create_new_message_without_embedding(
    pool: &SqlitePool,
    chat_id: i32,
    contents: String,
    position: i32,
    message_role: i32,
) -> Result<i32, String> {
    let new_message_id = match sqlx::query_as::<_, ID>("INSERT INTO tblMessages (chat_id, contents, position, message_role) VALUES ($1, $2, $3, $4) RETURNING id;").bind(chat_id).bind(contents).bind(position).bind(message_role).fetch_one(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to create new message record. Failed with: \n {}", e.to_string())),
    }.id;

    return Ok(new_message_id);
}

async fn create_new_message_record(
    pool: &SqlitePool,
    chat_id: i32,
    contents: String,
    position: i32,
    message_role: i32,
    embedding: Option<Vec<f32>>,
) -> Result<i32, String> {
    return Ok(match embedding {
        Some(v) => {
            create_new_message_with_embedding(pool, chat_id, contents, position, message_role, v)
                .await?
        }
        None => {
            create_new_message_without_embedding(pool, chat_id, contents, position, message_role)
                .await?
        }
    });
}

pub async fn upload_and_return_chat(
    pool: &SqlitePool,
    prompt: String,
    token: String,
) -> Result<String, String> {
    let user_id = match check_token(token) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to verify token. Failed with: \n {}", e)),
    };

    let user_data = get_user_from_id(pool, user_id).await?; // Check if user exists

    if user_data.len() < 1 && user_data.len() > 1 {
        return Err("User not found or multiple users found. Contact support".to_string());
    }

    let response = match llm_integration::upload_to_llm(prompt.clone(), None).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to upload to llm. Failed with: \n {}", e)),
    };

    let chat_id = match create_new_chat_record(pool, user_id, None).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to create new chat record. Failed with: \n {}",
                e
            ));
        }
    };

    let embedding = match embedding_integration::get_embedding(prompt.clone()).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to get embedding. Failed with: \n {}", e)),
    };

    match create_new_message_record(pool, chat_id, prompt, 0, 0, Some(embedding)).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to create new message record. Failed with: \n {}",
                e
            ));
        }
    };

    match create_new_message_record(pool, chat_id, response.clone(), 1, 1, None).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to create new message record. Failed with: \n {}",
                e
            ));
        }
    };

    return Ok(response);
}

async fn get_next_position(pool: &SqlitePool, chat_id: i32) -> Result<i32, String> {
    let data: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>("SELECT id, contents, chat_id, position, message_role, embedding FROM tblMessages WHERE chat_id = $1;").bind(chat_id).fetch_all(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to get next position. Failed with: \n {}", e)),
    };

    return Ok(data.len() as i32);
}

#[async_recursion]
async fn get_context(pool: &SqlitePool, chat_id: i32) -> Result<String, String> {
    let data: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>("SELECT id, contents, chat_id, position, message_role, embedding FROM tblMessages WHERE chat_id = $1 ORDER BY position ASC;").bind(chat_id).fetch_all(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to get context. Failed with: \n {}", e)),
    };

    let mut context = String::new();

    let chat_data = match sqlx::query_as::<_, ContinuationChat>(
        "SELECT continuation_chat_id FROM tblChats WHERE id = $1;",
    )
    .bind(chat_id)
    .fetch_one(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to get continuation chat. Failed with: \n {}",
                e
            ));
        }
    };

    if chat_data.continuation_chat_id.is_some() {
        let continuation_chat_id = chat_data.continuation_chat_id.unwrap();

        let continuation_context = match get_context(pool, continuation_chat_id).await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to get continuation context. Failed with: \n {}",
                    e
                ));
            }
        };

        context.push_str(format!("{}\n\n", continuation_context).as_str());
    }

    for message in data {
        context.push_str(format!("{}\n", message.contents).as_str());
    }

    return Ok(context);
}

pub async fn continue_chat(
    pool: &SqlitePool,
    chat_id: i32,
    prompt: String,
    token: String,
) -> Result<i32, String> {
    let user_id = match check_token(token) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to verify token. Failed with: \n {}", e)),
    };

    let user_data = get_user_from_id(pool, user_id).await?; // Check if user exists

    if user_data.len() < 1 && user_data.len() > 1 {
        return Err("User not found or multiple users found. Contact support".to_string());
    }

    let mut context = String::new();

    context.push_str("Following this is context from the previous chat. Please use this context to answer the following prompt. \n\n");

    let context_str = match get_context(pool, chat_id).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to get context. Failed with: \n {}", e)),
    };

    context.push_str(&context_str);
    context.push_str("\n\nEnd of context. Please answer the following prompt: \n\n");

    let response = match llm_integration::upload_to_llm(prompt.clone(), Some(context)).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to upload to llm. Failed with: \n {}", e)),
    };

    let embedding =
        match embedding_integration::get_embedding(format!("{}{}", context_str, prompt)).await {
            // add previous prompt to context for embedding
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to get embedding. Failed with: \n {}", e)),
        };

    let is_owner =
        match sqlx::query_as::<_, UserId>("SELECT id, user_id FROM tblChats WHERE id = $1;")
            .bind(chat_id)
            .fetch_one(pool)
            .await
        {
            Ok(v) => v.user_id == user_id,
            Err(e) => {
                return Err(format!(
                    "Failed to verify chat ownership. Failed with: \n {}",
                    e
                ));
            }
        };

    let new_chat_id: i32;

    if !is_owner {
        new_chat_id = match create_new_chat_record(pool, user_id, Some(chat_id)).await {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to create new chat record. Failed with: \n {}",
                    e
                ));
            }
        };
    } else {
        new_chat_id = chat_id;
    }

    let position = match get_next_position(pool, chat_id).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to get next position. Failed with: \n {}",
                e
            ));
        }
    };

    match create_new_message_record(pool, new_chat_id, prompt, position, 0, Some(embedding)).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to create new message record. Failed with: \n {}",
                e
            ));
        }
    };

    match create_new_message_record(pool, new_chat_id, response.clone(), position + 1, 1, None)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to create new message record. Failed with: \n {}",
                e
            ));
        }
    };

    return Ok(new_chat_id);
}

pub async fn get_similar_messages(
    pool: &SqlitePool,
    embedded_prompt: Vec<f32>,
) -> Result<Vec<MessageWithScore>, String> {
    let messages = match get_messages(pool).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to fetch embeddings from db. Failed with: \n{}",
                e
            ));
        }
    };

    let mut similar_messages: Vec<MessageWithScore> = Vec::new();

    for message in messages {
        let score =
            match embedding_integration::calculate_similarity(&message.embedding, &embedded_prompt)
            {
                Ok(v) => v,
                Err(_) => continue,
            };

        if score > 0.6 {
            similar_messages.push(MessageWithScore {
                id: message.id,
                contents: message.contents,
                chat_id: message.chat_id,
                position: message.position,
                message_role: message.message_role,
                embedding: message.embedding,
                score,
            });
        }
    }

    // Create a sorting algorithm
    let sorted_messages = match algorithms::sort_messages_by_similarity(similar_messages) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to sort messages by similarity. Failed with: \n {}",
                e
            ));
        }
    };

    return Ok(sorted_messages);
}

async fn get_messages(pool: &SqlitePool) -> Result<Vec<Message>, String> {
    let data: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>(
        "SELECT id, contents, message_role, chat_id, position, embedding FROM tblMessages;",
    )
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to get embeddings from db. Failed with: \n {}",
                e
            ));
        }
    };

    let mut return_data = Vec::new();

    for message in data {
        return_data.push(Message {
            id: message.id,
            contents: message.contents,
            chat_id: message.chat_id,
            position: message.position,
            message_role: message.message_role,
            embedding: blob_to_vec(&message.embedding),
        });
    }

    return Ok(return_data);
}

// fn handle_db_error(err_code: &str) -> String {
//     match err_code {
//         "2067" => "DB element not unique".to_string(),
//         _ => panic!("db error: Err Code: {}", err_code),
//     }
// }

async fn check_previous_interaction(
    pool: &SqlitePool,
    chat_id: i32,
    user_id: i32,
) -> Result<bool, String> {
    let data: Vec<UserId> = match sqlx::query_as::<_, UserId>(
        "SELECT user_id FROM tblFeedback WHERE chat_id = $1 AND user_id = $2;",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to check previous interaction. Failed with: \n {}",
                e
            ));
        }
    };

    if data.len() > 0 {
        return Ok(true);
    } else {
        return Ok(false);
    }
}

pub async fn chat_interaction(
    pool: &SqlitePool,
    chat_id: i32,
    interaction: i32,
    token: String,
) -> Result<String, String> {
    let user_id = match check_token(token) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to verify token. Failed with: \n {}", e)),
    };

    let user_data = get_user_from_id(pool, user_id).await?; // Check if user exists

    if user_data.len() < 1 && user_data.len() > 1 {
        return Err("User not found or multiple users found. Contact support".to_string());
    }

    let previous_interaction = match check_previous_interaction(pool, chat_id, user_id).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Failed to check previous interaction. Failed with: \n {}",
                e
            ));
        }
    };

    let return_data;

    if previous_interaction {
        return_data = match sqlx::query(
            "UPDATE tblFeedback SET vote_type = $1 WHERE chat_id = $2 AND user_id = $3;",
        )
        .bind(interaction)
        .bind(chat_id)
        .bind(user_id)
        .execute(pool)
        .await
        {
            Ok(_) => Ok("Interaction updated successfully".to_string()),
            Err(e) => return Err(format!("Failed to update feedback. Failed with: \n {}", e)),
        };
    } else {
        return_data = match sqlx::query(
            "INSERT INTO tblFeedback (chat_id, user_id, vote_type) VALUES ($1, $2, $3);",
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(interaction)
        .execute(pool)
        .await
        {
            Ok(_) => Ok("Interaction inserted successfully".to_string()),
            Err(e) => return Err(format!("Failed to insert feedback. Failed with: \n {}", e)),
        };
    }

    return return_data;
}

#[async_recursion]
async fn get_chat_messages(pool: &SqlitePool, id: i32, position: Option<i32>) -> Result<Vec<MessageResponse>, String> {
    let chat = match sqlx::query_as::<_, ChatReturnData>("SELECT id, user_id, continuation_chat_id FROM tblChats WHERE id = $1").bind(id).fetch_one(pool).await {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to fetch chat. Failed with: {}", e)),
    };

    let mut messages: Vec<MessageResponse> = Vec::new();

    if position.is_none() {
        let raw_messages: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>(
            "SELECT id, contents, message_role, chat_id, position, embedding FROM tblMessages WHERE chat_id = $1;",
        ).bind(id)
        .fetch_all(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to get embeddings from db. Failed with: \n {}",
                    e
                ));
            }
        };

        messages.extend(raw_messages.into_iter().map(|x| MessageResponse {
            id: x.id,
            contents: x.contents,
            message_role: x.message_role,
            position: x.position,
        }));
    } else {
        let raw_messages: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>(
            "SELECT id, contents, message_role, chat_id, position, embedding FROM tblMessages WHERE chat_id = $1 AND position < $2;",
        ).bind(id).bind(position.unwrap())
        .fetch_all(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Failed to get embeddings from db. Failed with: \n {}",
                    e
                ));
            }
        };

        messages.extend(raw_messages.into_iter().map(|x| MessageResponse {
            id: x.id,
            contents: x.contents,
            message_role: x.message_role,
            position: x.position,
        }));
    }

    let mut min_position: Option<i32>;

    if messages.len() != 0 {
        min_position = Some(messages[0].position);
    } else {
        min_position = None;
    }

    if min_position.is_some() {
        for m in &messages {
            if min_position.unwrap() > m.position {
                min_position = Some(m.position);
            }
        }
    }

    if chat.continuation_chat_id != None {
        let continuation_messages = match get_chat_messages(pool, chat.continuation_chat_id.unwrap(), min_position).await {
            Ok(v) => v,
            Err(e) => return Err(format!("An error occured. Failed with: {}", e)),
        };

        messages.extend(continuation_messages);
    }

    return Ok(messages);
}

#[async_recursion]
async fn get_feedback(pool: &SqlitePool, id: i32) -> Result<i32, String>{
    let chat = match sqlx::query_as::<_, ChatReturnData>("SELECT id, user_id, continuation_chat_id FROM tblChats WHERE id = $1").bind(id).fetch_one(pool).await {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to fetch chat. Failed with: {}", e)),
    };

    let mut feedback: i32 = 0;   

    if chat.continuation_chat_id != None {
        feedback += match get_feedback(pool, chat.continuation_chat_id.unwrap()).await {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
    }

    let raw_feedback_data: Vec<FeedbackData> = match sqlx::query_as::<_, FeedbackData>("SELECT user_id, chat_id, vote_type FROM tblFeedback WHERE chat_id = $1;").bind(id).fetch_all(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("An error occured fetching feedback. Failed with: {}", e)),
    };

    for f in raw_feedback_data {
        if f.vote_type == 1 {
            feedback += 1;
        } else {
            feedback -= 1;
        }
    }

    return Ok(feedback);
}

pub async fn get_chat(pool: &SqlitePool, id: i32) -> Result<ChatResponse, String> {
    let messages = match get_chat_messages(pool, id, None).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Error fetching messages: {}", e)),
    };

    let database_response: ChatReturnData = match sqlx::query_as::<_, ChatReturnData>("SELECT id, user_id, continuation_chat_id FROM tblChats WHERE id = $1").bind(id).fetch_one(pool).await {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to fetch chat. Failed with: {}", e)),
    };

    let feedback_score = match get_feedback(pool, id).await {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let messages = match algorithms::sort_messages_by_position(messages) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let response = ChatResponse { 
        id: id,
        user_id: database_response.user_id,
        messages: messages,
        feedback: feedback_score,
     };

    return Ok(response);
}

pub async fn get_user_data_from_token(pool: &SqlitePool, token: String) -> Result<UserResponse, String> {
    let user_id = match check_token(token) {
        Ok(i) => i,
        Err(e) => return Err(e),
    };

    let user_data: UserResponse = match sqlx::query_as::<_, UserResponse>("SELECT email, username FROM tblUsers WHERE id = $1;").bind(user_id).fetch_one(pool).await {
        Ok(u) => u,
        Err(e) => return Err(format!("Couldn't fetch user data. Failed with: {}", e)),
    };

    return Ok(user_data);
}

fn check_token(token: String) -> Result<i32, String> {
    return match algorithms::verify_token(&token[..]) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("Token verification failed. Failed with: \n {}", e)),
    };
}

fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|x| f32::from_le_bytes(x.try_into().unwrap()))
        .collect()
}
