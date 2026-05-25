mod db_integration;
mod server_integration;

use futures::executor;
use tokio::task;

use tokio::sync::mpsc;

use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() {
    executor::block_on(async {
        let (server_tx, mut pool_rx) = mpsc::channel::<String>(32); // Creates a new channel
        let (pool_tx, mut server_rx) = mpsc::channel::<SqlitePool>(32);

        let server_handle =
            task::spawn(async move { server_integration::spawn_server(server_tx, server_rx).await });

        let pool_handle = task::spawn(async move { db_integration::get_pool(pool_tx).await });

        server_handle.await.unwrap();
        pool_handle.await.unwrap();
    })
}
