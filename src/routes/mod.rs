pub mod root;

use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sqlx::SqlitePool;

use crate::assets;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Wraps handler errors so `?` works in handlers and responses stay plain.
pub struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.0, "request failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
    }
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(root::index))
        .route("/users", post(root::create))
        .route("/users/{id}/update", post(root::update))
        .route("/users/{id}/delete", post(root::delete))
        .route("/static/{*path}", get(assets::serve))
        .with_state(AppState { pool })
}
