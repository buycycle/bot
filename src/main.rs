mod assistant;
use assistant::{assistant_chat_handler, DB};
use assistant::{create_assistant, create_files};
use axum::{extract::Extension, routing::post, Router};
use dotenv::dotenv;
use sqlx::SqlitePool;
// Define a function to create the Axum app with the database pool and assistant.
// Define a function to create the Axum app with the database pool and assistant.
async fn app(db_pool: SqlitePool, assistant_id: String) -> Router {
    Router::new()
        .route("/assistant", post(assistant_chat_handler)) // Updated route
        .layer(Extension(db_pool))
        .layer(Extension(assistant_id)) // Add the assistant ID as a layer
}
#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();
    // Create the files for the assistant.
    let files = match create_files("context", Vec::new()).await {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Failed to create files: {:?}", e);
            return;
        }
    };
    // Create an assistant outside of the main function.
    let assistant = match create_assistant(
        "My Assistant",
        "gpt-4-turbo-preview",
        "On buycycle.com, users can buy and sell per-owned bicycles.
        Help the users with how the website works, use the faq.html for refeeral links.
        ",
        &files.file_ids,
    )
    .await
    {
        Ok(assistant) => assistant,
        Err(e) => {
            eprintln!("Failed to create assistant: {:?}", e);
            return;
        }
    };
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = DB::create_db_pool(&database_url)
        .await
        .expect("Failed to create database pool");
    let db_pool = db.pool; // Extract the SqlitePool from the DB struct
                           // Run database migrations here if necessary
                           // sqlx::migrate!("./migrations").run(&db_pool).await.expect("Failed to run database migrations");
                           // Bind the server to an address and start it.
    let server = tokio::net::TcpListener::bind(&"0.0.0.0:3000")
        .await
        .unwrap();
    let router = app(db_pool, assistant.id).await; // Pass the assistant ID to the app
    axum::serve(server, router.into_make_service())
        .await
        .unwrap();
}
