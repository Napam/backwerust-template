use std::collections::HashMap;
use std::env;
use std::path::Path;

use backwerust_template::{
    config::Config,
    db::{self, AppliedMigration},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = env::args().nth(1);
    match command.as_deref() {
        None | Some("status") => {}
        Some(other) => {
            anyhow::bail!("unknown command {other:?}: run `migrate` or `migrate status`")
        }
    }

    let config = Config::from_env()?;

    if command.as_deref() == Some("status") {
        let migrator = db::migrations();
        let migrations = embedded_migrations(&migrator);
        return print_status(&config.db_path, &migrations).await;
    }

    let pool = db::connect(&config.db_path).await?;
    db::migrate(&pool).await?;

    println!("migrations applied to {}", config.db_path.display());

    Ok(())
}

fn embedded_migrations(migrator: &sqlx::migrate::Migrator) -> Vec<(i64, &str, &[u8])> {
    migrator
        .iter()
        .map(|migration| {
            (
                migration.version,
                migration.description.as_ref(),
                migration.checksum.as_ref(),
            )
        })
        .collect()
}

/// Print the migration state without creating the file: an absent path is
/// reported as such instead of being initialized by the connection.
async fn print_status(path: &Path, migrations: &[(i64, &str, &[u8])]) -> anyhow::Result<()> {
    if !path.exists() {
        print!("{}", status_report(path, migrations, &[], false));
        return Ok(());
    }

    let pool = db::connect(path).await?;
    let applied = db::applied_migrations(&pool).await?;
    print!("{}", status_report(path, migrations, &applied, true));

    Ok(())
}

/// One line per embedded migration, marked applied, modified, failed or
/// pending, plus any recorded version whose migration file is gone. sqlx
/// refuses to run while a migration is modified, failed or missing.
fn status_report(
    path: &Path,
    migrations: &[(i64, &str, &[u8])],
    applied: &[AppliedMigration],
    exists: bool,
) -> String {
    let recorded: HashMap<i64, (&[u8], bool)> = applied
        .iter()
        .map(|migration| {
            (
                migration.version,
                (migration.checksum.as_slice(), migration.success),
            )
        })
        .collect();

    let mut report = if exists {
        format!("database: {}\n", path.display())
    } else {
        format!("database: {} (does not exist)\n", path.display())
    };
    let mut applied_count = 0;
    let mut modified_count = 0;
    let mut failed_count = 0;

    for (version, description, checksum) in migrations {
        let state = match recorded.get(version) {
            // a failed row blocks every later migration, so it outranks the
            // checksum comparison
            Some((_, false)) => {
                failed_count += 1;
                "failed"
            }
            Some((recorded, true)) if recorded == checksum => {
                applied_count += 1;
                "applied"
            }
            Some(_) => {
                modified_count += 1;
                "modified"
            }
            None => "pending",
        };
        report.push_str(&format!("  {state:<8} {version:04}_{description}\n"));
    }

    let missing: Vec<i64> = applied
        .iter()
        .map(|migration| migration.version)
        .filter(|version| {
            !migrations
                .iter()
                .any(|(embedded, _, _)| embedded == version)
        })
        .collect();
    for version in &missing {
        report.push_str(&format!(
            "  {:<8} {version:04} (no embedded migration)\n",
            "missing"
        ));
    }

    let pending_count = migrations.len() - applied_count - modified_count - failed_count;
    report.push_str(&format!(
        "\n{applied_count} applied, {modified_count} modified, {failed_count} failed, \
         {pending_count} pending, {} missing\n",
        missing.len()
    ));

    report
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{AppliedMigration, status_report};

    fn applied(version: i64, checksum: &[u8]) -> AppliedMigration {
        AppliedMigration {
            version,
            checksum: checksum.to_vec(),
            success: true,
        }
    }

    fn failed(version: i64, checksum: &[u8]) -> AppliedMigration {
        AppliedMigration {
            version,
            checksum: checksum.to_vec(),
            success: false,
        }
    }

    #[test]
    fn status_marks_applied_and_pending() {
        let migrations = [
            (1, "init", b"init".as_slice()),
            (20240101120000, "add_posts", b"posts"),
        ];
        let report = status_report(
            Path::new("./data/sqlite.db"),
            &migrations,
            &[applied(20240101120000, b"posts")],
            true,
        );

        assert_eq!(
            report,
            "database: ./data/sqlite.db\n  \
             pending  0001_init\n  \
             applied  20240101120000_add_posts\n\
             \n1 applied, 0 modified, 0 failed, 1 pending, 0 missing\n"
        );
    }

    #[test]
    fn status_reports_an_unmigrated_database() {
        let migrations = [(1, "init", b"init".as_slice())];
        let report = status_report(Path::new("./data/sqlite.db"), &migrations, &[], true);

        assert_eq!(
            report,
            "database: ./data/sqlite.db\n  pending  0001_init\n\
             \n0 applied, 0 modified, 0 failed, 1 pending, 0 missing\n"
        );
    }

    #[test]
    fn status_reports_a_missing_database_without_creating_it() {
        let migrations = [(1, "init", b"init".as_slice())];
        let report = status_report(Path::new("./data/typo.db"), &migrations, &[], false);

        assert_eq!(
            report,
            "database: ./data/typo.db (does not exist)\n  pending  0001_init\n\
             \n0 applied, 0 modified, 0 failed, 1 pending, 0 missing\n"
        );
    }

    #[test]
    fn status_marks_a_migration_edited_after_it_ran() {
        let migrations = [(1, "init", b"edited".as_slice())];
        let report = status_report(
            Path::new("./data/sqlite.db"),
            &migrations,
            &[applied(1, b"original")],
            true,
        );

        assert_eq!(
            report,
            "database: ./data/sqlite.db\n  modified 0001_init\n\
             \n0 applied, 1 modified, 0 failed, 0 pending, 0 missing\n"
        );
    }

    #[test]
    fn status_marks_a_failed_migration() {
        let migrations = [(1, "init", b"init".as_slice())];
        let report = status_report(
            Path::new("./data/sqlite.db"),
            &migrations,
            &[failed(1, b"init")],
            true,
        );

        assert_eq!(
            report,
            "database: ./data/sqlite.db\n  failed   0001_init\n\
             \n0 applied, 0 modified, 1 failed, 0 pending, 0 missing\n"
        );
    }

    #[test]
    fn status_lists_an_applied_migration_with_no_embedded_file() {
        let report = status_report(
            Path::new("./data/sqlite.db"),
            &[],
            &[applied(7, b"gone")],
            true,
        );

        assert_eq!(
            report,
            "database: ./data/sqlite.db\n  missing  0007 (no embedded migration)\n\
             \n0 applied, 0 modified, 0 failed, 0 pending, 1 missing\n"
        );
    }
}
