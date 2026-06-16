pub use sqlx::{sqlite::SqlitePool};
use super::embedding_integration;

use tokio::sync::mpsc;

use super::structs::{User, Message, MessageWithScore, MessageReturnData, PasswordPair};

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

pub async fn register_user(pool: &SqlitePool, username: String, email: String, hashed_password: String) -> Result<String, String> {
    if !email.contains("@") || !email.contains(".") { // Could change this so it scans for TLDs
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

async fn upload_user(pool: &SqlitePool, username: String, email: String, password: String) -> Result<String, String> {
    let password_pair = match algorithms::hash_password(password) {
        Ok(p) => p,
        Err(e) => return Err(format!("Couldn't hash password. failed with:\n{}", e)),
    };
    
    println!("Uploading User");
    let return_data = match sqlx::query("INSERT INTO tblUsers (email, username, hashed_password, salt) VALUES ($1, $2, $3, $4)").bind(email).bind(username).bind(password_pair.hashed_password).bind(password_pair.salt).execute(pool).await {
        Ok(_) => "User Uploaded".to_string(),
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

pub async fn login_user(pool: &SqlitePool, username: String, email: String, password: String) -> Result<String, String> {
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
        let hashed_password = match algorithms::hash_password_with_salt(password, users[0].salt.clone()) {
            Ok(v) => v.hashed_password,
            Err(_) => return Err("Can't hash password".to_string()),
        };

        if users[0].hashed_password ==  hashed_password{
            return Ok("Login Successful".to_string());
        } else {
            return Err(format!("Incorrect Credentials"));
        }
    } else {
        return Err("DB Error, more than 1 user with those credentials, contact support".to_string());
    }
}

async fn login_user_with_username(pool: &SqlitePool, username: String, hashed_password: String) -> Result<String, String> {
    let users = match get_user_from_username(pool, &username).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Database Error. Failed with: \n {}", e)),
    };

    return check_user_password(&users, hashed_password);
}

async fn login_user_with_email(pool: &SqlitePool, email: String, hashed_password: String) -> Result<String, String> {
    let users = match get_user_from_email(pool, &email).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to find user. Failed with: \n {}", e.to_string())),
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
    let data: Vec<User> = 
        match sqlx::query_as::<_, User>("SELECT id, email, username, hashed_password, salt, is_admin FROM tblUsers WHERE email = $1")
        .bind(format!("{}", email)).fetch_all(pool).await {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

    return Ok(data);
}

async fn upload_embedding(pool: &SqlitePool, embedding: Vec<f32>) -> Result<String, String> {
    println!("uploading embedding!");

    let blob = vec_to_blob(&embedding);
    let return_data = match sqlx::query("INSERT INTO tblEmbeddings (embedding) VALUES ($1);").bind(blob).execute(pool).await {
        Ok(_) => "Uploaded Embedding".to_string(),
        Err(err) => return Err(format!("Failed to upload embedding. Failed with: \n {}", err.to_string())),
    };

    return Ok(return_data);
}

pub async fn get_similar_messages(pool: &SqlitePool, embedded_prompt: Vec<f32>) -> Result<Vec<MessageWithScore>, String> {
    let messages = match get_messages(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to fetch embeddings from db. Failed with: \n{}", e)),
    };

    let mut similar_messages: Vec<MessageWithScore> = Vec::new();

    for message in messages {
        let score = embedding_integration::calculate_similarity(&message.embedding, &embedded_prompt)?;

        if score > 0.6 {
            similar_messages.push(MessageWithScore {
                id: message.id,
                contents: message.contents,
                chat_id: message.chat_id,
                position: message.position,
                embedding: message.embedding,
                score,
            });
        }
    }

    // Create a sorting algorithm
    let sorted_messages = match algorithms::sort_messages_by_similarity(similar_messages) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to sort messages by similarity. Failed with: \n {}", e)),
    };
    

    return Ok(sorted_messages);
}



async fn get_messages(pool: &SqlitePool) -> Result<Vec<Message>, String> {
    let data: Vec<MessageReturnData> = match sqlx::query_as::<_, MessageReturnData>("SELECT id, contents, chat_id, position, embedding FROM tblEmbeddings;").fetch_all(pool).await {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to get embeddings from db. Failed with: \n {}", e)),
    };

    let mut return_data = Vec::new();

    for message in data {
        return_data.push(Message {
            id: message.id,
            contents: message.contents,
            chat_id: message.chat_id,
            position: message.position,
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

fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4).map(|x| f32::from_le_bytes(x.try_into().unwrap())).collect()
}
