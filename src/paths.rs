//! Route patterns and the links built from them, shared by the router,
//! handlers, views and tests.

pub const INDEX: &str = "/";
pub const USERS: &str = "/users";
pub const USER_UPDATE: &str = "/users/{id}/update";
pub const USER_DELETE: &str = "/users/{id}/delete";

/// The start page with one row open for editing.
pub fn index_edit(id: i64) -> String {
    format!("{INDEX}?edit={id}")
}

pub fn user_update(id: i64) -> String {
    with_id(USER_UPDATE, id)
}

pub fn user_delete(id: i64) -> String {
    with_id(USER_DELETE, id)
}

fn with_id(pattern: &str, id: i64) -> String {
    pattern.replace("{id}", &id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_links_the_views_emit() {
        assert_eq!(user_update(7), "/users/7/update");
        assert_eq!(user_delete(7), "/users/7/delete");
        assert_eq!(index_edit(7), "/?edit=7");
    }
}
