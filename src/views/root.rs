//! The landing page: stack list and CRUD example.

use maud::{Markup, html};

use super::{datastar_post, layout};
use crate::db::users::User;
use crate::paths;

/// Id of the CRUD section that Datastar replaces after an edit or delete.
pub const CRUD_ID: &str = "crud";

/// Question asked before deleting a user, shared by the plain and Datastar paths.
const DELETE_CONFIRM: &str = "Delete this user?";

/// Landing page with the stack list and the CRUD example.
pub fn page(users: &[User], edit_id: Option<i64>) -> Markup {
    let scripts = html! {
        script defer src="/static/page-files/pages/root.js" {}
    };

    let content = html! {
        header class="sticky top-0 z-10 border-b border-gray-200 bg-white/80 backdrop-blur-sm dark:border-neutral-800 dark:bg-neutral-900/80" {
            div class="mx-auto flex max-w-3xl items-center justify-between px-6 py-4" {
                h1 class="text-2xl font-bold tracking-tight text-gray-700 dark:text-neutral-200" {
                    "Backwerust Template"
                }
                theme-button {}
            }
        }
        main class="mx-auto max-w-3xl px-6 py-10" {
            section class="mb-12" {
                div class="mb-8 flex items-center justify-between gap-4" {
                    p class="text-xl leading-relaxed text-gray-600 dark:text-neutral-400" {
                        "A starter template for building fullstack SSR-first web applications"
                    }
                    img class="h-16 w-auto shrink-0 sm:h-24" alt="" src="/static/assets/ferris.svg";
                }
                dl class="grid grid-cols-2 gap-x-4 gap-y-6 text-sm text-gray-600 dark:text-neutral-400" {
                    div {
                        dt class="mb-1" {
                            a class="text-primary-500 dark:text-primary-400 font-bold hover:underline dark:font-medium" href="https://www.rust-lang.org" {
                                "Rust"
                            }
                            " - backend language"
                        }
                        dd class="text-gray-500 dark:text-neutral-500" {
                            a class="hover:underline" href="https://github.com/tokio-rs/axum" { "axum" }
                            " - HTTP router"
                            br;
                            a class="hover:underline" href="https://github.com/launchbadge/sqlx" { "sqlx" }
                            " - SQLite driver and migrations"
                            br;
                            a class="hover:underline" href="https://github.com/lambda-fairy/maud" { "maud" }
                            " - compile-time HTML templates"
                        }
                    }
                    div {
                        dt class="mb-1" {
                            a class="text-primary-500 dark:text-primary-400 font-bold hover:underline dark:font-medium" href="https://data-star.dev" {
                                "Datastar"
                            }
                            " - hypermedia"
                        }
                        dd class="text-gray-500 dark:text-neutral-500" {
                            "SSE-driven DOM patches - the server keeps rendering HTML, the browser swaps it in"
                        }
                    }
                    div {
                        dt class="mb-1" {
                            a class="text-primary-500 dark:text-primary-400 font-bold hover:underline dark:font-medium" href="https://tailwindcss.com/" {
                                "TailwindCSS"
                            }
                            " - CSS utility classes"
                        }
                        dd class="text-gray-500 dark:text-neutral-500" {
                            "No custom CSS files - responsive, dark mode, and design system out of the box"
                        }
                    }
                    div {
                        dt class="mb-1" {
                            a class="text-primary-500 dark:text-primary-400 font-bold hover:underline dark:font-medium" href="https://bun.com/" {
                                "Bun"
                            }
                            " - web asset bundling"
                        }
                        dd class="text-gray-500 dark:text-neutral-500" {
                            "Fast all-in-one JS runtime - replaces webpack, rollup, and esbuild"
                        }
                    }
                    div {
                        dt class="mb-1" {
                            a class="text-primary-500 dark:text-primary-400 font-bold hover:underline dark:font-medium" href="https://lit.dev/" {
                                "Lit"
                            }
                            " - web components"
                        }
                        dd class="text-gray-500 dark:text-neutral-500" {
                            "Custom " code class="text-xs" { "StyledLitElement" } " base class enables shadow DOM components to share your Tailwind stylesheet"
                        }
                    }
                }
            }
            (crud(users, edit_id))
        }
    };

    layout(scripts, content)
}

/// The CRUD section. Datastar replaces this whole section after an edit.
pub fn crud(users: &[User], edit_id: Option<i64>) -> Markup {
    html! {
        section id=(CRUD_ID) class="flex flex-col gap-4 rounded-xs border border-gray-200 bg-white p-4 dark:border-neutral-700 dark:bg-neutral-800" {
            div class="flex items-baseline justify-between" {
                h2 class="text-sm text-gray-500 dark:text-neutral-400" {
                    "Example CRUD - User registry"
                }
            }
            p class="text-xs text-gray-400 dark:text-neutral-500" {
                "Native "
                code class="rounded bg-gray-100 px-1 py-0.5 dark:bg-neutral-900" { "<input>" }
                " elements inside forms, with "
                code class="rounded bg-gray-100 px-1 py-0.5 dark:bg-neutral-900" { "<button-component>" }
                " for buttons. Update and delete forms answer both: plain requests redirect back, Datastar requests patch this section in place."
            }
            form method="post" action=(paths::USERS) class="flex items-end gap-2" {
                input
                    name="name"
                    placeholder="New user name"
                    aria-label="New user name"
                    class="grow rounded-xs bg-gray-100 px-2 py-1 text-sm text-gray-600 placeholder:font-extralight placeholder:text-gray-400 dark:bg-neutral-700 dark:text-neutral-200 dark:placeholder:text-neutral-400";
                button-component type="submit" buttonClass="font-normal" { "Add +" }
            }
            (users_table(users, edit_id))
        }
    }
}

/// The user table, with an inline edit form for `edit_id`.
fn users_table(users: &[User], edit_id: Option<i64>) -> Markup {
    html! {
        table class="w-full text-sm" {
            thead class="border-b border-gray-200 dark:border-neutral-700" {
                tr class="text-gray-500 dark:text-neutral-500" {
                    th class="py-1 pr-4 text-left font-normal" { "ID" }
                    th class="py-1 text-left font-normal" { "Name" }
                    th {}
                }
            }
            tbody class="font-light text-gray-500 dark:text-neutral-400" {
                @for user in users {
                    (user_row(user, edit_id))
                }
            }
        }
    }
}

/// One row. The update and delete forms are built from one path each, so the
/// plain and Datastar requests cannot drift apart.
fn user_row(user: &User, edit_id: Option<i64>) -> Markup {
    let editing = edit_id == Some(user.id);
    let form_id = format!("edit-{}", user.id);
    let update_action = paths::user_update(user.id);
    let delete_action = paths::user_delete(user.id);

    html! {
        tr {
            td class="pt-2" { (user.id) }
            @if editing {
                td class="pt-2" {
                    form
                        id=(form_id.as_str())
                        method="post"
                        action=(update_action)
                        class="flex items-center gap-2"
                        data-on:submit=(datastar_post(&update_action))
                    {
                        input
                            name="name"
                            value=[user.name.as_deref()]
                            aria-label="User name"
                            class="w-full border-b border-gray-300 bg-transparent text-sm text-gray-700 outline-none dark:border-neutral-500 dark:text-neutral-300";
                    }
                }
            } @else {
                td class="pt-2" { (user.name.as_deref().unwrap_or("")) }
            }
            td class="flex justify-end gap-2 pt-2" {
                @if editing {
                    button type="submit" form=(form_id.as_str()) class="text-primary-500 dark:text-primary-400 cursor-pointer text-xs" {
                        "Save"
                    }
                    a href=(paths::INDEX) class="cursor-pointer text-xs text-gray-400 hover:text-gray-600 dark:text-neutral-500 dark:hover:text-neutral-300" {
                        "Cancel"
                    }
                } @else {
                    a href=(paths::index_edit(user.id)) aria-label="Edit user" class="cursor-pointer" {
                        edit-icon svgClass="dark:hover:text-primary-500 hover:text-primary-500 size-4 text-gray-400 dark:text-neutral-500" {}
                    }
                    (delete_form(&delete_action))
                }
            }
        }
    }
}

/// The delete form. `data-confirm` guards the plain submit; the Datastar
/// click handler asks the same question and posts the same path.
fn delete_form(action: &str) -> Markup {
    html! {
        form method="post" action=(action) data-confirm=(DELETE_CONFIRM) {
            button
                type="submit"
                aria-label="Delete user"
                class="cursor-pointer"
                data-on:click__prevent=(format!("confirm('{DELETE_CONFIRM}') && {}", datastar_post(action)))
            {
                trash-icon svgClass="size-4 text-gray-400 hover:text-red-600 dark:text-neutral-500 dark:hover:text-red-600" {}
            }
        }
    }
}
