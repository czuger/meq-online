//! Server entry point: configuration, logging, database, routes and graceful shutdown.

use std::error::Error;
use std::sync::Arc;

use meq_online::app_state::AppState;
use meq_online::config::Config;
use meq_online::db::pool;
use meq_online::game_data::GameData;
use meq_online::refresh::RefreshHub;
use meq_online::web::router::router;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let data = Arc::new(GameData::load(&config.data_dir)?);
    let db = pool::connect(&config.database_url).await?;
    let refresh = RefreshHub::new();
    let app = router(
        AppState {
            db,
            data,
            refresh: refresh.clone(),
        },
        &config.static_dir,
    );

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            refresh.shutdown();
        })
        .await?;
    tracing::info!("stopped");
    Ok(())
}

/// Waits for Ctrl-C or, on Unix, SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
