//! Typed HTML views built with maud.
//!
//! A view is a plain function returning [`Markup`]. The compiler checks the
//! fields and types used inside `html!`, so a renamed field or a wrong type
//! fails the build instead of a request.

// The root view sits next to its page JS so a page's markup and browser code
// live in one directory. `#[path]` keeps the module identity unchanged.
#[path = "../../web/pages/root.rs"]
pub mod root;

use axum::http::StatusCode;
use maud::{DOCTYPE, Markup, html};

use crate::paths;

/// Page shell shared by every route.
pub fn layout(scripts: Markup, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" class="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Backwerust Template" }
                link rel="stylesheet" id="tailwindcss" href="/static/tailwind.css";
                script src="/static/bundle.js" {}
                script type="module" defer src="/static/assets/vendor/datastar.js" {}
                (scripts)
            }
            body class="min-h-screen bg-gray-50 font-sans text-gray-700 dark:bg-neutral-900 dark:text-neutral-300" {
                (content)
            }
        }
    }
}

/// Full page for an error response.
pub fn error(status: StatusCode) -> Markup {
    let reason = status.canonical_reason().unwrap_or("Error");

    layout(
        Markup::default(),
        html! {
            main class="mx-auto max-w-3xl px-6 py-16" {
                h1 class="mb-2 text-2xl font-bold tracking-tight text-gray-700 dark:text-neutral-200" {
                    (status.as_u16()) " " (reason)
                }
                p class="mb-6 text-gray-500 dark:text-neutral-400" {
                    "The request could not be completed."
                }
                a class="text-primary-500 hover:underline dark:text-primary-400" href=(paths::INDEX) {
                    "Back to the start page"
                }
            }
        },
    )
}

/// A Datastar POST expression with form encoding.
fn datastar_post(path: &str) -> String {
    format!("@post('{path}', {{contentType: 'form'}})")
}
