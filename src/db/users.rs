use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub name: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name FROM users ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &SqlitePool, name: Option<&str>) -> Result<i64, sqlx::Error> {
    let (id,): (i64,) = sqlx::query_as("INSERT INTO users (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(id)
}

/// Updates one user. Returns `false` when no user has that id.
pub async fn update_name(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET name = ? WHERE id = ?")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes one user. Returns `false` when no user has that id.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
