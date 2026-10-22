pub mod root;

use std::fmt::Display;

use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sqlx::SqlitePool;
use tracing::Instrument;

use crate::assets;
use crate::paths;
use crate::views;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Wraps handler errors so `?` works in handlers. Each error carries the
/// status its response uses.
pub struct AppError {
    status: StatusCode,
    error: anyhow::Error,
}

impl AppError {
    /// A 404 for a request that names a row that does not exist.
    pub fn not_found(message: impl Display) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: anyhow::anyhow!("{message}"),
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(error: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status;
        if status.is_server_error() {
            tracing::error!(error = %self.error, "request failed");
        } else {
            tracing::info!(error = %self.error, "request rejected");
        }
        (status, views::error(status)).into_response()
    }
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(paths::INDEX, get(root::index))
        .route(paths::USERS, post(root::create_user))
        .route(paths::USER_UPDATE, post(root::update_user))
        .route(paths::USER_DELETE, post(root::delete_user))
        .route("/static/{*path}", get(assets::serve))
        .layer(middleware::from_fn(request_span))
        .with_state(AppState { pool })
}

/// A span per request so logs emitted by a handler carry the method and URI.
async fn request_span(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    next.run(request)
        .instrument(tracing::info_span!("request", %method, %uri))
        .await
}

#[cfg(test)]
mod tests {
    use axum::http::header;

    use super::*;

    #[test]
    fn not_found_renders_the_error_page() {
        let response = AppError::not_found("user 9 not found").into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn other_errors_are_internal_server_errors() {
        let response = AppError::from(anyhow::anyhow!("database is gone")).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }
}
