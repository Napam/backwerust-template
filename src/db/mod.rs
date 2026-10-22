pub mod users;

use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

pub async fn connect(db_path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating database directory {}", parent.display()))?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("opening database at {}", db_path.display()))
}

/// The migrations embedded at compile time, in filename order.
pub fn migrations() -> sqlx::migrate::Migrator {
    sqlx::migrate!("./db/migrations")
}

pub async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    migrations().run(pool).await.context("running migrations")
}

/// A migration sqlx recorded, with the checksum it stored. `success` is false
/// when the migration did not finish; sqlx then refuses to run any migration
/// until the database is reset.
pub struct AppliedMigration {
    pub version: i64,
    pub checksum: Vec<u8>,
    pub success: bool,
}

/// Migrations recorded in sqlx's bookkeeping table. Empty when the database
/// has never been migrated.
pub async fn applied_migrations(pool: &SqlitePool) -> anyhow::Result<Vec<AppliedMigration>> {
    let table: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(pool)
    .await
    .context("looking for the sqlx migrations table")?;

    if table.is_none() {
        return Ok(Vec::new());
    }

    let rows: Vec<(i64, Vec<u8>, bool)> =
        sqlx::query_as("SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version")
            .fetch_all(pool)
            .await
            .context("reading applied migrations")?;

    Ok(rows
        .into_iter()
        .map(|(version, checksum, success)| AppliedMigration {
            version,
            checksum,
            success,
        })
        .collect())
}
