mod auto_output_format;
mod controllers;
mod discord;
mod error;
mod markdown;
mod models;
mod slug;
mod solution_invalidation;
mod strip_trailing_slashes;
mod test_case_display;
mod test_solution;

use axum::{routing::get, Extension, Router};
use tower_sessions::session_store::ExpiredDeletion;

use anyhow::Context;
use controllers::{
    auth::{github_callback, github_login},
    challenges::{all_challenges, compose_challenge, new_challenge, view_challenge},
    global_leaderboard::global_leaderboard,
    solution::{
        all_solutions, challenge_redirect, challenge_redirect_no_slug,
        challenge_redirect_with_slug, get_leaderboard, new_solution,
        post_mortem::{post_mortem_view, post_mortem_view_without_language},
    },
    user::get_user,
};
use discord::DiscordEventSender;
use discord_bot::Bot;
use solution_invalidation::solution_invalidation_task;
use sqlx::{postgres::PgPoolOptions, query, PgPool};
use std::{env, time::Duration};
use strip_trailing_slashes::strip_trailing_slashes;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_file_store::FileSessionStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup .env
    #[cfg(debug_assertions)]
    {
        dotenvy::from_filename(".env.local")?;
        dotenvy::dotenv()?;
    }

    // Setup Tracking Subscriber
    tracing_subscriber::fmt()
        .log_internal_errors(true)
        // .with_span_events(FmtSpan::FULL)
        .init();

    // Setup SQLX
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL").expect("Missing .env var: DATABASE_URL"))
        .await
        .context("could not connect to database_url")?;

    // Setup Sessions
    let session_store = FileSessionStorage::new();
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_name("yq_session_store_id")
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(360),
        ));
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
    );

    let _invalidation_task = tokio::task::spawn(solution_invalidation_task(pool.clone()));

    // Bot
    let bot = init_bot_from_env(&pool);

    start_task_to_refresh_views(pool.clone());

    let app = Router::new()
        .route("/", get(all_challenges))
        .nest_service(
            "/ts/runner-lib.d.ts",
            ServeFile::new("scripts/build/runner-lib.d.ts"),
        )
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.svg"))
        .route("/leaderboard/{category}", get(global_leaderboard))
        .route("/challenge", get(compose_challenge).post(new_challenge))
        .route("/challenge/{id}", get(challenge_redirect))
        .route(
            "/challenge/{id}/{slug}/edit",
            get(compose_challenge).post(new_challenge),
        )
        .route("/challenge/:id/:slug/view", get(view_challenge))
        .route(
            "/challenge/{id}/{slug}/solve",
            get(challenge_redirect_with_slug),
        )
        .route(
            "/challenge/{id}/{slug}/leaderboard/:language",
            get(get_leaderboard),
        )
        .route(
            "/challenge/{id}/{slug}/solve/:language",
            get(all_solutions).post(new_solution),
        )
        .route(
            "/challenge/{id}/{slug}/solutions",
            get(post_mortem_view_without_language),
        )
        .route(
            "/challenge/{id}/{slug}/solutions/:language",
            get(post_mortem_view),
        )
        .route("/login/github", get(github_login))
        .route("/callback/github", get(github_callback))
        .route("/user/{id}", get(get_user))
        .route("/{id}/{language}", get(challenge_redirect_no_slug))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(get(strip_trailing_slashes))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(Extension(pool))
        .layer(Extension(bot))
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind(&format!(
        "{}:{}",
        env::var("YQ_HOST").expect("Expcted YQ_HOST var to be set"),
        env::var("YQ_PORT").expect("Excpected YQ_PORT var to be set")
    ))
    .await
    .unwrap();

    if let Ok(addr) = listener.local_addr() {
        println!("Listening on http://{addr:?}");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

fn init_bot_from_env(pool: &PgPool) -> DiscordEventSender {
    let bot = if let Some((token, channel_id)) = std::env::var("DISCORD_TOKEN")
        .ok()
        .zip(std::env::var("DISCORD_CHANNEL_ID").ok())
    {
        Some(Bot::new(pool.clone(), token, channel_id.parse().unwrap()))
    } else {
        None
    };
    DiscordEventSender::new(pool.clone(), bot)
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

fn start_task_to_refresh_views(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            let statement = query!("REFRESH MATERIALIZED VIEW scores")
                .execute(&pool)
                .await;
            match statement {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error refreshing scores: {e:?}");
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
