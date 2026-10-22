use std::io::IsTerminal;

use anyhow::Context;
use backwerust_template::{config::Config, db, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cwd = std::env::current_dir().context("reading working directory")?;
    tracing::info!(cwd = %cwd.display(), "starting backwerust-template");

    let config = Config::from_env()?;
    let pool = db::connect(&config.db_path).await?;
    if config.db_migrate_on_start {
        db::migrate(&pool).await?;
        tracing::info!("migrations up to date");
    }

    let app = routes::router(pool);

    #[cfg(debug_assertions)]
    let app = {
        tracing::info!("live reload is active");
        app.layer(backwerust_template::dev::layer())
    };

    let address = config.address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("binding {address}"))?;
    tracing::info!(address = %listener.local_addr()?, "server running");

    let server = axum::serve(listener, app);

    // Live reload holds an SSE stream open for every browser tab, so a
    // graceful drain would wait for those clients. Debug builds keep the
    // default signal disposition and terminate immediately instead.
    #[cfg(debug_assertions)]
    {
        server.await.context("serving requests")
    }

    #[cfg(not(debug_assertions))]
    {
        server
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("serving requests")
    }
}

/// Resolves on Ctrl+C or SIGTERM. The binary runs as PID 1 in the container,
/// where default signal dispositions are ignored, so it installs its own.
#[cfg(not(debug_assertions))]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("installing ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("installing SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(false)
        .compact()
        .init();
}
