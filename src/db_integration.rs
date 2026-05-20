pub use sqlx::sqlite::SqlitePool;

use tokio::sync::mpsc;

pub async fn get_pool(tx: mpsc::Sender<String>, mut rx: mpsc::Receiver<String>) {
    let pool = SqlitePool::connect("sqlite:./data/db.sqlite")
        .await
        .expect("Failed to connect to db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Database Ready");

    loop {
        let receivedData = rx.recv().await.unwrap();
        println!("{receivedData}");

        let return_data = match &receivedData[..] {
            "upload" => upload_user(&pool).await,
            "get" => get_user(&pool).await,
            _ => "".to_string(),
        };

        tx.try_send(return_data);
    }
}

async fn upload_user(pool: &SqlitePool) -> String {
    println!("Uploading User");
    let return_data = match sqlx::query("INSERT INTO users (email, username, hashed_password) VALUES ('test@test1.com', 'test1', 'password')").execute(pool).await {
        Ok(_) => "User Uploaded".to_string(),
        Err(err) => match err {
            sqlx::Error::Database(err) => handle_db_error(&*err.code().unwrap()).to_string(),
            _ => panic!("Unexpected error"),
        },
    };

    return return_data;
    // Handle NOT UNIQUE Errors
    // Handle errors better to avoid crashes (crashes would still allow the server to load, return a 200 OK, but nothing would occur)
}

#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i32,
    email: String,
    username: String,
    hashed_password: String,
}

async fn get_user(pool: &SqlitePool) -> String {
    let data: Vec<User> =
        (sqlx::query_as::<_, User>("SELECT id, email, username, hashed_password FROM users")
            .fetch_all(pool)
            .await
            .unwrap());
    
    let return_data =  format!("UserID: {} \n email: {} \n username: {} \n hashed_password: {}", data[0].id, data[0].email, data[0].username, data[0].hashed_password).to_string();
    println!("{}", return_data);
    return return_data;
}

fn handle_db_error(err_code: &str) -> String {
    match err_code {
        "2067" => "DB element not unique".to_string(),
        _ => panic!("db error"),
    }
}
