use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use maud::Markup;
use serde::Deserialize;

use crate::db::users;
use crate::paths;
use crate::routes::{AppError, AppState, Result};
use crate::sse;
use crate::views::{self, CRUD_ID};

#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    edit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct NameForm {
    name: String,
}

fn empty_to_none(name: &str) -> Option<&str> {
    (!name.is_empty()).then_some(name)
}

pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<IndexQuery>,
) -> Result<Markup> {
    let users = users::list(&state.pool).await?;

    Ok(views::root::page(&users, query.edit))
}

pub async fn create_user(
    State(state): State<AppState>,
    Form(form): Form<NameForm>,
) -> Result<Redirect> {
    let id = users::create(&state.pool, empty_to_none(&form.name)).await?;
    tracing::info!(id, "created user");

    Ok(Redirect::to(paths::INDEX))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<NameForm>,
) -> Result<Response> {
    if !users::update_name(&state.pool, id, empty_to_none(&form.name)).await? {
        return missing_user(&state, &headers, id).await;
    }
    tracing::info!(id, "updated user");

    if sse::is_datastar_request(&headers) {
        return crud_patch(&state).await;
    }
    Ok(Redirect::to(paths::INDEX).into_response())
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response> {
    if !users::delete(&state.pool, id).await? {
        return missing_user(&state, &headers, id).await;
    }
    tracing::info!(id, "deleted user");

    if sse::is_datastar_request(&headers) {
        return crud_patch(&state).await;
    }
    Ok(Redirect::to(paths::INDEX).into_response())
}

/// Missing row: a plain request gets a 404, a Datastar request gets the
/// re-rendered list so a row deleted in another tab disappears from the page.
async fn missing_user(state: &AppState, headers: &HeaderMap, id: i64) -> Result<Response> {
    if sse::is_datastar_request(headers) {
        tracing::info!(id, "datastar request for missing user; re-rendering list");
        return crud_patch(state).await;
    }

    Err(AppError::not_found(format!("user {id} not found")))
}

/// Re-renders the CRUD section for a Datastar patch after an edit or delete.
async fn crud_patch(state: &AppState) -> Result<Response> {
    let users = users::list(&state.pool).await?;
    let crud = views::root::crud(&users, None);

    Ok(sse::patch_response(
        &format!("#{CRUD_ID}"),
        &crud.into_string(),
    ))
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    fn datastar_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("datastar-request", HeaderValue::from_static("true"));
        headers
    }

    async fn body_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn datastar_update_for_a_missing_row_patches_the_list() {
        let state = AppState {
            pool: test_pool().await,
        };

        let response = update_user(
            State(state),
            Path(99),
            datastar_headers(),
            Form(NameForm { name: "Ada".into() }),
        )
        .await;
        let Ok(response) = response else {
            panic!("handler returned an error");
        };

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );
        let body = body_text(response).await;
        assert!(body.contains("event: datastar-patch-elements"));
        assert!(body.contains(&format!("selector #{CRUD_ID}")));
    }

    #[tokio::test]
    async fn datastar_delete_for_a_missing_row_patches_the_list() {
        let state = AppState {
            pool: test_pool().await,
        };

        let response = delete_user(State(state), Path(99), datastar_headers()).await;
        let Ok(response) = response else {
            panic!("handler returned an error");
        };

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );
        let body = body_text(response).await;
        assert!(body.contains("event: datastar-patch-elements"));
        assert!(body.contains(&format!("selector #{CRUD_ID}")));
    }

    #[tokio::test]
    async fn plain_update_for_a_missing_row_is_not_found() {
        let state = AppState {
            pool: test_pool().await,
        };

        let response = update_user(
            State(state),
            Path(99),
            HeaderMap::new(),
            Form(NameForm { name: "Ada".into() }),
        )
        .await;
        let Err(error) = response else {
            panic!("expected a 404");
        };

        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
}
