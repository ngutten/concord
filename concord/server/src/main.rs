use std::sync::Arc;

use tracing::info;
use tracing_subscriber::EnvFilter;

use concord_server::db::pool::{create_pool, run_migrations};
use concord_server::engine::chat_engine::ChatEngine;
use concord_server::irc::listener::start_irc_listener;
use concord_server::web::router::build_router;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let web_addr = "0.0.0.0:8080";
    let irc_addr = "0.0.0.0:6667";
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:concord.db?mode=rwc".to_string());

    // Initialize database
    let pool = create_pool(&database_url)
        .await
        .expect("failed to connect to database");

    run_migrations(&pool)
        .await
        .expect("failed to run database migrations");

    // Create the shared chat engine with database
    let engine = Arc::new(ChatEngine::new(Some(pool)));

    // Load persisted channels into memory
    engine
        .load_channels_from_db()
        .await
        .expect("failed to load channels from database");

    // Start IRC listener (TCP port 6667)
    let irc_engine = engine.clone();
    let irc_addr_owned = irc_addr.to_string();
    tokio::spawn(async move {
        start_irc_listener(&irc_addr_owned, irc_engine).await;
    });

    // Build and start the HTTP/WebSocket server
    let app = build_router(engine);

    info!("Concord server starting — Web: {}, IRC: {}", web_addr, irc_addr);

    let listener = tokio::net::TcpListener::bind(web_addr)
        .await
        .expect("failed to bind web listener");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
