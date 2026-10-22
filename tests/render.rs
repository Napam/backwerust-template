//! Renders the real templates through the app's environment and context types.
//! Strict undefined plus these tests are the guardrail for handler fields.

use backwerust_template::db::users::User;
use backwerust_template::routes::root::Context;
use backwerust_template::templates;

fn render(name: &str, ctx: &Context) -> String {
    templates::render(name, ctx).unwrap()
}

#[test]
fn root_renders_list_and_inline_edit_form() {
    let users = vec![
        User {
            id: 1,
            name: Some("Ada".to_owned()),
        },
        User { id: 2, name: None },
        User {
            id: 3,
            name: Some("<script>alert(1)</script>".to_owned()),
        },
    ];
    let html = render("root.html", &Context { users, edit_id: 1 });

    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("id=\"crud\""));
    assert!(html.contains("action=\"/users\""));
    assert!(html.contains("id=\"edit-1\""));
    assert!(html.contains("action=\"/users/1/update\""));
    assert!(html.contains("value=\"Ada\""));
    assert!(html.contains("href=\"/?edit=2\""));
    assert!(html.contains("data-confirm=\"Delete this user?\""));
    assert!(html.contains("action=\"/users/2/delete\""));

    // Static URLs the web build and the shadow-DOM components rely on.
    assert!(html.contains("id=\"tailwindcss\" href=\"/static/tailwind.css\""));
    assert!(html.contains("src=\"/static/bundle.js\""));
    assert!(html.contains("src=\"/static/page-files/pages/root.js\""));
    assert!(html.contains("src=\"/static/assets/vendor/datastar.js\""));
}

#[test]
fn null_name_renders_as_empty_text_and_no_edit_form() {
    let users = vec![
        User {
            id: 1,
            name: Some("Ada".to_owned()),
        },
        User { id: 2, name: None },
    ];
    let html = render("partials/crud.html", &Context { users, edit_id: 0 });

    assert!(html.contains("<td class=\"pt-2\">Ada</td>"));
    assert!(html.contains("<td class=\"pt-2\"></td>"));
    assert!(!html.contains("None"));
    assert!(!html.contains("/update"));
}

#[test]
fn user_text_is_html_escaped() {
    let users = vec![User {
        id: 7,
        name: Some("<script>alert(1)</script>".to_owned()),
    }];
    let html = render("partials/crud.html", &Context { users, edit_id: 0 });

    assert!(html.contains("&lt;script&gt;alert(1)&lt;&#x2f;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
}

#[test]
fn missing_template_is_an_error() {
    let context = Context {
        users: Vec::new(),
        edit_id: 0,
    };

    assert!(templates::render("partials/does-not-exist.html", &context).is_err());
}

#[test]
fn strict_undefined_fails_instead_of_rendering_blank() {
    // Root needs `users`; lenient undefined would render an empty list.
    assert!(templates::render("root.html", ()).is_err());
}
