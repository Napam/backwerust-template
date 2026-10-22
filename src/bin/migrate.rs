use backwerust_template::{config::Config, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let pool = db::connect(&config.db_path).await?;
    db::migrate(&pool).await?;

    println!("migrations applied to {}", config.db_path.display());

    Ok(())
}
