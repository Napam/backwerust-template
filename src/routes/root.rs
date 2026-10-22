use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

use crate::db::users::{self, User};
use crate::routes::{AppError, AppState, Result};
use crate::sse;
use crate::templates;

const CRUD_SELECTOR: &str = "#crud";

#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    edit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NameForm {
    name: String,
}

#[derive(Serialize)]
pub struct Context {
    pub users: Vec<User>,
    pub edit_id: i64,
}

fn optional_name(name: &str) -> Option<&str> {
    (!name.is_empty()).then_some(name)
}

pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<IndexQuery>,
) -> Result<Html<String>> {
    let users = users::list(&state.pool).await?;
    let edit_id = query.edit.and_then(|id| id.parse().ok()).unwrap_or(0);

    Ok(Html(render("root.html", Context { users, edit_id })?))
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
    let crud = render("partials/crud.html", Context { users, edit_id: 0 })?;

    Ok(sse::patch_response(CRUD_SELECTOR, &crud))
}

/// Renders a template, logging the template name and line on failure before
/// the error becomes a plain 500.
fn render(name: &str, ctx: impl Serialize) -> Result<String> {
    templates::render(name, ctx).map_err(|error| {
        tracing::error!(
            template = error.name().unwrap_or(name),
            line = ?error.line(),
            error = %error,
            "rendering template failed"
        );
        AppError::from(error)
    })
}
