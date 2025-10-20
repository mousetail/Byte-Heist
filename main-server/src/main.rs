mod background_tasks;
mod controllers;
mod discord;
mod error;
mod filter_iterator_but_keep_context;
mod models;
mod strip_trailing_slashes;
mod tera_utils;
mod test_case_display;
mod test_solution;

use axum::{
    Extension, Router,
    routing::{get, post},
};
use background_tasks::{
    announce_ended_challenges::announce_ended_challenges_task, refresh_views::refresh_views_task,
    solution_invalidation::solution_invalidation_task,
};
use macros::OutputWrapperFactory;
use tera_utils::TeraHtmlRenderer;
use tower_sessions::session_store::ExpiredDeletion;

use anyhow::Context;
use controllers::{
    auth::{github_callback, github_login},
    challenges::{
        all_challenges, compose_challenge, get_homepage, new_challenge, post_comment,
        post_reaction, view_challenge,
    },
    global_leaderboard::{global_leaderboard, global_leaderboard_per_language},
    solution::{
        all_solutions, challenge_redirect, challenge_redirect_no_slug,
        challenge_redirect_with_slug, get_leaderboard, new_solution,
        post_mortem::{post_mortem_view, post_mortem_view_without_language},
    },
    user::get_user,
};
use discord::DiscordEventSender;
use discord_bot::Bot;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
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
        .with_name("byte_heist_session_store_id")
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(360),
        ));
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
    );

    let discord_bot = init_bot_from_env(&pool);

    let _invalidation_task = tokio::task::spawn(solution_invalidation_task(pool.clone()));
    let _announce_ended_challenges_task = tokio::task::spawn(announce_ended_challenges_task(
        pool.clone(),
        discord_bot.clone(),
    ));
    let _refresh_views_task = tokio::task::spawn(refresh_views_task(pool.clone()));

    let route_factory = OutputWrapperFactory {
        renderer: TeraHtmlRenderer,
    };

    let app = Router::new()
        .route(
            "/",
            get(route_factory.handler("home.html.jinja", get_homepage)),
        )
        .nest_service(
            "/ts/runner-lib.d.ts",
            ServeFile::new("scripts/build/runner-lib.d.ts"),
        )
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.svg"))
        .route(
            "/heists",
            get(route_factory.handler("heists.html.jinja", all_challenges)),
        )
        .route(
            "/leaderboard/{category}",
            get(route_factory.handler("global_leaderboard.html.jinja", global_leaderboard)),
        )
        .route(
            "/leaderboard/{category}/{language}",
            get(route_factory.handler(
                "global_leaderboard.html.jinja",
                global_leaderboard_per_language,
            )),
        )
        .route(
            "/challenge",
            get(route_factory.handler("submit_challenge/index.html.jinja", compose_challenge))
                .post(route_factory.handler("submit_challenge/index.html.jinja", new_challenge)),
        )
        .route("/challenge/{id}", get(challenge_redirect))
        .route(
            "/challenge/{id}/{slug}/edit",
            get(route_factory.handler("submit_challenge/index.html.jinja", compose_challenge))
                .post(route_factory.handler("submit_challenge/index.html.jinja", new_challenge)),
        )
        .route(
            "/challenge/{id}/{slug}/view",
            get(route_factory.handler("view_challenge.html.jinja", view_challenge))
                .post(route_factory.handler("view_challenge.html.jinja", post_comment)),
        )
        .route(
            "/challenge/{id}/{slug}/view/vote",
            post(route_factory.handler("view_challenge.html.jinja", post_reaction)),
        )
        .route(
            "/challenge/{id}/{slug}/solve",
            get(challenge_redirect_with_slug),
        )
        .route(
            "/challenge/{id}/{slug}/leaderboard/{language}",
            get(route_factory.handler("leaderboard.html.jinja", get_leaderboard)),
        )
        .route(
            "/challenge/{id}/{slug}/solve/{language}",
            get(route_factory.handler("solve_challenge/index.html.jinja", all_solutions))
                .post(route_factory.handler("solve_challenge/index.html.jinja", new_solution)),
        )
        .route(
            "/challenge/{id}/{slug}/solutions",
            get(post_mortem_view_without_language),
        )
        .route(
            "/challenge/{id}/{slug}/solutions/{language}",
            get(route_factory.handler("post_mortem_view.html.jinja", post_mortem_view)),
        )
        .route("/login/github", get(github_login))
        .route("/callback/github", get(github_callback))
        .route(
            "/user/{id}",
            get(route_factory.handler("user.html.jinja", get_user)),
        )
        .route("/{id}/{language}", get(challenge_redirect_no_slug))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(get(strip_trailing_slashes))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(Extension(pool))
        .layer(Extension(discord_bot))
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind(&format!(
        "{}:{}",
        env::var("BYTE_HEIST_HOST").expect("Expcted BYTE_HEIST_HOST var to be set"),
        env::var("BYTE_HEIST_PORT").expect("Excpected BYTE_HEIST_PORT var to be set")
    ))
    .await
    .unwrap();

    if let Ok(addr) = listener.local_addr() {
        eprintln!("Listening on http://{addr:?}");
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
