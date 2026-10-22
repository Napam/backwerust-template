//! MiniJinja environment and template rendering.
//!
//! Debug builds load templates from `templates/` on disk and reload them when
//! they change, so editing a template needs no recompile. Release builds embed
//! the same directory with `minijinja-embed`, keeping the single-binary story.

use std::sync::OnceLock;

use minijinja::value::Serde;
use minijinja::{Environment, UndefinedBehavior};

/// Every template the app can render. Loading them at startup turns a syntax
/// error into a failed boot instead of a 500 on the first request.
const TEMPLATES: &[&str] = &[
    "base.html",
    "root.html",
    "partials/crud.html",
    "partials/users_table.html",
];

static ENV: OnceLock<Environment<'static>> = OnceLock::new();

fn environment() -> &'static Environment<'static> {
    ENV.get_or_init(|| {
        let mut env = Environment::new();

        #[cfg(debug_assertions)]
        {
            env.set_loader(minijinja::path_loader("templates"));
            env.set_auto_reload(true);
        }
        #[cfg(not(debug_assertions))]
        minijinja_embed::load_templates!(&mut env);

        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env
    })
}

/// Loads all known templates so syntax errors fail at startup.
pub fn init() -> Result<(), minijinja::Error> {
    let env = environment();
    for name in TEMPLATES {
        env.get_template(name)?;
    }
    Ok(())
}

/// Renders `name` with `ctx`. Errors carry the template name and line.
pub fn render(name: &str, ctx: impl serde::Serialize) -> Result<String, minijinja::Error> {
    environment().get_template(name)?.render(Serde(ctx))
}
