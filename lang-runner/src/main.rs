mod cachemap;
mod error;
mod parse_output;
mod run;

use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use cachemap::CacheMap;
use common::RunLangOutput;
use error::RunLangError;
use run::{get_lang_versions, process_message};
use serde::{Deserialize, Serialize};
use tokio::signal;

#[derive(Serialize, Debug, Deserialize)]
pub struct Message {
    lang: String,
    version: String,
    code: String,
    judge: String,
}

#[tokio::main]
async fn main() {
    println!("Starting server");
    // initialize tracing
    tracing_subscriber::fmt::init();

    let lang_versions = get_lang_versions().await;

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root).post(handle_message))
        .route("/lang-versions", get(lang_versions_endpoint))
        .with_state(Arc::new(lang_versions));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("Server Clean Exit");
}

async fn root() -> &'static str {
    "Server is working properly"
}

async fn lang_versions_endpoint(
    State(lang_versions): State<Arc<CacheMap<String, CacheMap<String, ()>>>>,
) -> Json<impl Serialize> {
    Json(serde_json::to_value(&*lang_versions).unwrap())
}

#[axum::debug_handler]
async fn handle_message(
    lang_versions: State<Arc<CacheMap<String, CacheMap<String, ()>>>>,
    message: Json<Message>,
) -> Result<Json<RunLangOutput>, RunLangError> {
    let result = process_message(message.0, &lang_versions.0).await?;
    Ok(Json(result))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
