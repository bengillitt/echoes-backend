pub use sqlx::sqlite::SqlitePool;

use tokio::sync::mpsc;

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

pub async fn upload_user(pool: &SqlitePool, username: String, email: String, hashed_password: String) -> String {
    println!("Uploading User");
    let return_data = match sqlx::query("INSERT INTO tblUsers (email, username, hashed_password) VALUES ($1, $2, $3)").bind(email).bind(username).bind(hashed_password).execute(pool).await {
        Ok(_) => "User Uploaded".to_string(),
        Err(err) => match err {
            sqlx::Error::Database(err) => {
                println!("{}", err);
                handle_db_error(&*err.code().unwrap()).to_string()
            },
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
    is_admin: bool,
}

pub async fn get_user(pool: &SqlitePool, username: String) -> String {
    let data: Vec<User> =
        sqlx::query_as::<_, User>("SELECT id, email, username, hashed_password, is_admin FROM tblUsers WHERE username LIKE $1").bind(format!("{}%", username))
            .fetch_all(pool)
            .await
            .unwrap();
    
    let mut return_data = String::new();

    for i in data {
        return_data.push_str(&(format!("UserID: {} \n email: {} \n username: {} \n hashed_password: {}\n is_admin: {}\n\n", i.id, i.email, i.username, i.hashed_password, i.is_admin)));
    }

    println!("{}", return_data);
    return return_data;
}

fn handle_db_error(err_code: &str) -> String {
    match err_code {
        "2067" => "DB element not unique".to_string(),
        _ => panic!("db error: Err Code: {}", err_code),
    }
}
