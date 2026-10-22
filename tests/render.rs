//! Renders the real views through the same functions the handlers call.
//!
//! The compiler checks field names and types. These tests cover what it
//! cannot: the URLs, Datastar attributes and escaping the browser sees.

use backwerust_template::db::users::User;
use backwerust_template::views;

fn user(id: i64, name: Option<&str>) -> User {
    User {
        id,
        name: name.map(str::to_owned),
    }
}

#[test]
fn root_renders_list_and_inline_edit_form() {
    let users = vec![
        user(1, Some("Ada")),
        user(2, None),
        user(3, Some("<script>alert(1)</script>")),
    ];
    let html = views::root(&users, 1).into_string();

    assert!(html.starts_with("<!DOCTYPE html>"));
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
    let users = vec![user(1, Some("Ada")), user(2, None)];
    let html = views::crud(&users, 0).into_string();

    assert!(html.contains("<td class=\"pt-2\">Ada</td>"));
    assert!(html.contains("<td class=\"pt-2\"></td>"));
    assert!(!html.contains("None"));
    assert!(!html.contains("/update"));
}

#[test]
fn user_text_is_html_escaped() {
    let users = vec![user(7, Some("<script>alert(1)</script>"))];
    let html = views::crud(&users, 0).into_string();

    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
}

#[test]
fn user_text_is_escaped_inside_attributes() {
    let users = vec![user(7, Some("a\"b"))];
    let html = views::crud(&users, 7).into_string();

    assert!(html.contains("value=\"a&quot;b\""));
}
