//! Typed HTML views built with maud.
//!
//! A view is a plain function returning [`Markup`]. The compiler checks the
//! fields and types used inside `html!`, so a renamed field or a wrong type
//! fails the build instead of a request.

use maud::{DOCTYPE, Markup, html};

use crate::db::users::User;

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

/// Landing page with the stack list and the CRUD example.
pub fn root(users: &[User], edit_id: i64) -> Markup {
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
pub fn crud(users: &[User], edit_id: i64) -> Markup {
    html! {
        section id="crud" class="flex flex-col gap-4 rounded-xs border border-gray-200 bg-white p-4 dark:border-neutral-700 dark:bg-neutral-800" {
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
            form method="post" action="/users" class="flex items-end gap-2" {
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

/// Rows of the user table, with an inline edit form for `edit_id`.
pub fn users_table(users: &[User], edit_id: i64) -> Markup {
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
                    tr {
                        td class="pt-2" { (user.id) }
                        @if edit_id == user.id {
                            td class="pt-2" {
                                form
                                    id=(format!("edit-{}", user.id))
                                    method="post"
                                    action=(format!("/users/{}/update", user.id))
                                    class="flex items-center gap-2"
                                    data-on:submit=(datastar_post(&format!("/users/{}/update", user.id)))
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
                            @if edit_id == user.id {
                                button type="submit" form=(format!("edit-{}", user.id)) class="text-primary-500 dark:text-primary-400 cursor-pointer text-xs" {
                                    "Save"
                                }
                                a href="/" class="cursor-pointer text-xs text-gray-400 hover:text-gray-600 dark:text-neutral-500 dark:hover:text-neutral-300" {
                                    "Cancel"
                                }
                            } @else {
                                a href=(format!("/?edit={}", user.id)) aria-label="Edit user" class="cursor-pointer" {
                                    edit-icon svgClass="dark:hover:text-primary-500 hover:text-primary-500 size-4 text-gray-400 dark:text-neutral-500" {}
                                }
                                form method="post" action=(format!("/users/{}/delete", user.id)) data-confirm="Delete this user?" {
                                    button
                                        type="submit"
                                        aria-label="Delete user"
                                        class="cursor-pointer"
                                        data-on:click__prevent=(format!("confirm('Delete this user?') && @post('/users/{}/delete', {{contentType: 'form'}})", user.id))
                                    {
                                        trash-icon svgClass="size-4 text-gray-400 hover:text-red-600 dark:text-neutral-500 dark:hover:text-red-600" {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A Datastar POST expression with form encoding.
fn datastar_post(path: &str) -> String {
    format!("@post('{path}', {{contentType: 'form'}})")
}
