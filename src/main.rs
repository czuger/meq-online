//! Server entry point: configuration, logging, database, routes and graceful shutdown.

use std::error::Error;
use std::sync::Arc;

use meq_online::app_state::AppState;
use meq_online::board_versions::BoardVersions;
use meq_online::config::Config;
use meq_online::db::pool;
use meq_online::game_data::GameData;
use meq_online::logging;
use meq_online::web::router::router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_env()?;
    // Flushes the debug log file when dropped, at the end of `main`.
    let _log_guard = logging::init(&config)?;
    tracing::debug!(?config, "configuration");
    let data = Arc::new(GameData::load(&config.data_dir)?);
    tracing::debug!(folder = %config.data_dir.display(), "game data loaded");
    let db = pool::connect(&config.database_url).await?;
    tracing::debug!(url = %config.database_url, "database ready");
    let app = router(
        AppState {
            db,
            data,
            versions: BoardVersions::new(),
        },
        &config.static_dir,
    );

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
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
