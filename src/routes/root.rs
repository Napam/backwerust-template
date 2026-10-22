use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use maud::Markup;
use serde::Deserialize;

use crate::db::users;
use crate::routes::{AppState, Result};
use crate::sse;
use crate::views;

const CRUD_SELECTOR: &str = "#crud";

#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    edit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NameForm {
    name: String,
}

fn optional_name(name: &str) -> Option<&str> {
    (!name.is_empty()).then_some(name)
}

pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<IndexQuery>,
) -> Result<Markup> {
    let users = users::list(&state.pool).await?;
    let edit_id = query.edit.and_then(|id| id.parse().ok()).unwrap_or(0);

    Ok(views::root(&users, edit_id))
}

pub async fn create(State(state): State<AppState>, Form(form): Form<NameForm>) -> Result<Redirect> {
    let id = users::create(&state.pool, optional_name(&form.name)).await?;
    tracing::info!(id, "created user");

    Ok(Redirect::to("/"))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<NameForm>,
) -> Result<Response> {
    users::update_name(&state.pool, id, optional_name(&form.name)).await?;
    tracing::info!(id, "updated user");

    if sse::is_datastar_request(&headers) {
        return crud_patch(&state).await;
    }
    Ok(Redirect::to("/").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response> {
    users::delete(&state.pool, id).await?;
    tracing::info!(id, "deleted user");

    if sse::is_datastar_request(&headers) {
        return crud_patch(&state).await;
    }
    Ok(Redirect::to("/").into_response())
}

/// Re-renders the CRUD section for a Datastar patch after an edit or delete.
async fn crud_patch(state: &AppState) -> Result<Response> {
    let users = users::list(&state.pool).await?;
    let crud = views::crud(&users, 0);

    Ok(sse::patch_response(CRUD_SELECTOR, &crud.into_string()))
}
