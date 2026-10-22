# Backwerust Template

Opinionated template for SSR-first Rust apps: axum for HTTP, maud for
compile-time checked HTML, tailwindcss for styling, Lit for web components,
sqlite for data, sqlx for migrations and queries. Everything compiles into a
single binary. All tooling is managed by mise. Live reload development loop is
set up and ready to go.

It's a base for hypermedia-driven applications, and pairs well with frameworks
like [Datastar](https://data-star.dev/) (wired up in the example) or
[htmx](https://htmx.org/).

Requirements:

- [mise](https://mise.jdx.dev) for development environment setup - mise must be [activated](https://mise.jdx.dev/getting-started.html#activate-mise).
- Docker, only if you want to build the container image.

## Quickstart

1. Grab the repo via GitHub's "Use this template" button
   (gives you a fresh repo with no history).

   <details>
   <summary>Or you can git clone and reset the git state</summary>

   ```sh
   git clone --depth 1 https://github.com/Napam/backwerust-template my-app
   cd my-app
   rm -rf .git
   git init && git add -A && git commit -m "initial"
   ```

   </details>

2. Planning to use it for your own project? Do the [renaming](#renaming) steps
   first.

3. Run `task dev`. It installs the tools and JS dependencies, sets up the git
   pre-push hook, and starts the dev server with live reload.

4. Open `localhost:6900`. There is no proxy port: the server listens on `PORT`
   (default `6900`) and the live reload script is injected into the page.

5. Start making changes. The application entrypoint is `src/main.rs`; the
   router is `src/routes/mod.rs`; handlers are in `src/routes/root.rs`; the
   page view is `src/views/root.rs`; URL patterns are in `src/paths.rs`; Lit
   components are in `web/lib/web-components`; page JS is `web/pages/root.ts`.

For production build: `task build` (output in `bin/app`).

Docker:

```sh
task build.docker       # build image
task build.docker.run   # build and run
```

## Renaming

There is no script for this. Rename by hand:

1. `Cargo.toml`: change `[package] name`, `default-run`, and the `[[bin]]`
   names if you don't want `backwerust` and `migrate`. `src/bin/migrate.rs` is
   the migration binary; if you rename the `migrate` bin, also change
   `cargo run --quiet --bin migrate` in the `db.migrate` task.
2. `bacon.toml`: change the `--bin` value in the `command` list if you renamed
   the server binary.
3. `Taskfile.yml`: change `cp target/release/backwerust ./bin/app`, and the two
   `backwerust:latest` image references in the `build.docker` tasks.
4. Brand strings: the `layout` title in `src/views/mod.rs`, the page heading
   in `src/views/root.rs`, the theme storage key in `web/lib/theme.ts`,
   `README.md`, and the startup log line in `src/main.rs`.

The Dockerfile needs no change: it copies `bin/app`, whatever the binary is
called.

References inside the crate use `crate::`, but the lib target is named after the
package (`my-app` becomes `my_app`). A package rename means updating the
`backwerust_template::` imports in `src/main.rs`, `src/bin/migrate.rs`, and
`tests/render.rs`.

## How it works

The server renders HTML with maud. A view is a Rust function under
`src/views/`, one module per page, that returns `Markup`, and `html!` checks
field names and types at compile time, so a renamed field or a wrong type fails
the build instead of a request. Forms POST to the server, which processes and
redirects back with a `303`. No client-side JS is required. Datastar and web
components enhance where needed.

- The route -> view path is short: a handler in `src/routes/root.rs` calls
  `views::root::page`, which builds the page shell with `layout` and renders the
  CRUD example from `views::root::crud`. A Datastar patch re-renders
  `views::root::crud` into the section named by `views::root::CRUD_ID`. Lit components
  live in `web/lib/web-components` and are registered by the shared bundle.
- Route patterns and the links built from them live in `src/paths.rs`, so the
  router, views, redirects and tests cannot drift apart.
- View edits are Rust changes: bacon rebuilds and restarts the server, and the
  browser reconnects. Changes under `web/static` trigger tower-livereload
  without a restart.
- `tests/render.rs` renders the real view functions and asserts the URLs,
  data attributes and escaping the browser sees.
- Handlers return `routes::Result`; `AppError` carries the status, and missing
  rows and server failures render through `views::error`. A Datastar request
  for a missing row re-renders the CRUD section instead, so a row deleted in
  another tab disappears from the page.
- Debug builds serve `web/static` from disk with `no-store`, and
  tower-livereload injects a reload script. Release builds embed the same
  directory with rust-embed and serve it with ETag-based cache headers
  (details under [Web assets](#web-assets)).
- Migrations are embedded with `sqlx::migrate!`, so the binary has no runtime
  file dependency besides the sqlite database itself.
- Web components use shadow DOM and shared TailwindCSS, with working dark mode
  toggling.
- Dockerfile builds a slim glibc image, not scratch: sqlx's sqlite driver links
  C, so the builder needs a C toolchain and the runtime needs a libc. The image
  still carries one self-contained binary.
- rustfmt, clippy, eslint, prettier, tsc and lit-analyzer are preconfigured.
  `task check` runs a full backend+frontend static, lint, and test check.

Tech stack:

- [axum](https://github.com/tokio-rs/axum) - HTTP router and server
- [maud](https://github.com/lambda-fairy/maud) - compile-time HTML templates in Rust
- [tailwindcss](https://tailwindcss.com/) - CSS styling
- [Lit](https://lit.dev/) - web components
- [Datastar](https://data-star.dev/) - hypermedia over SSE
- [sqlx](https://github.com/launchbadge/sqlx) - database driver, queries, migrations
- [rust-embed](https://github.com/pyrossh/rust-embed) - embeds static assets in release builds
- [tower-livereload](https://github.com/leotaku/tower-livereload) - live reload in debug builds
- [bacon](https://dystroy.org/bacon/) - rebuild and restart on change
- [tracing](https://github.com/tokio-rs/tracing) - structured logs

## Web assets

`web/build.ts` (run by `task dev` with `--watch`, and by `task build.web`)
produces two kinds of bundles:

- **Shared bundle:** everything in `web/lib/**/*.ts` becomes a single
  `static/bundle.js`. This is where web components live, and it is loaded
  synchronously in `<head>` so all custom elements are registered before the
  body parses.
- **Page bundles:** a `.ts` file under `web/pages/` becomes its own bundle
  under `static/page-files/pages/`, keeping the relative path.
  `web/pages/root.ts` pairs with `views::root::page` in `src/views/root.rs`.
  Load it on just that page. `layout` already loads `bundle.js` and Datastar,
  so the page only passes its own script to `layout`:

  ```rust
  let scripts = html! {
      script defer src="/static/page-files/pages/root.js" {}
  };
  ```

Page files are plain top-level browser code, no exports needed. `pages/` is an
allowlist: files in other top-level directories are not bundled, so scripts and
tests cannot be served by accident. Avoid importing from `lib/`: the iife
bundle has no code splitting, so each import duplicates code into the page
bundle. Small utility imports are fine if you accept that. Don't import from
`lib/web-components/` outside `base/`: `bundle.js` already registers every
component under that directory, and the page build fails on the import.
`web/lib/web-components/base/` holds the support classes that are safe for
pages.

Tailwind scans the paths declared in `web/tailwind.css`: `source(none)` turns
automatic detection off, and the explicit `@source` lines cover
`src/**/*.rs`, `web/lib/**/*.ts`, and `web/pages/**/*.ts`. Class names in a new
source directory need their own `@source` line.

Datastar is vendored at `web/assets/vendor/datastar.js` and served from
`/static/assets/vendor/datastar.js` as a module. Load it before page scripts
when a page uses it.

Images and other files served as-is live in `web/assets/`, committed to git;
subdirectories welcome. `web/build.ts` hardlinks every file there into
`web/static/assets/`, mirroring the directory structure (a hardlink, not a
copy, so the repo keeps a single on-disk copy). The `task dev` watcher picks up
asset edits (images, svgs, fonts, favicons) and re-runs the build
automatically.

`web/assets/` is excluded from prettier, eslint, and tsc, so prebuilt
third-party scripts (a copied library, an analytics snippet) can be dropped in
as-is without being reformatted or linted. If it's a normal package, prefer
`bun add` and import it from `lib/` or a page bundle instead.

Reference assets with the plain static path inside `html!`:

```rust
img src="/static/assets/logo.svg" alt="Logo";
```

- Production builds embed `web/static` into the binary with rust-embed and
  serve it with an ETag per file plus `must-revalidate`, so a browser
  revalidates instead of refetching. Dev mode serves assets straight from disk
  with `no-store`.
- Don't put files in `web/static/` by hand: it is build output, gitignored and
  wiped on every `task build`.
- For icons that should scale or restyle with the theme, use the Lit SVG icon
  components in `web/lib/web-components/icons` (see AGENTS.md);
  `web/assets/` is for images and other static files.

The `task dev` loop runs three watchers in parallel: `build.ts --watch` for
bundles and assets, the tailwind CLI in watch mode for CSS, and bacon for the
Rust server. The app is always served on `localhost:6900`; open that, not a
proxy port. Ctrl+C stops all three.

## Database

Migrations live in `db/migrations/` (sqlx naming: `<version>_<description>.sql`,
for example `0001_init.sql`). To change the schema:

1. `task db.new -- add_posts` creates an empty, UTC-timestamped migration
2. Edit the generated file
3. `task db.migrate` applies it to your local `data/sqlite.db`

`sqlx::migrate!` embeds the whole directory in the server and migration
binaries, so new files are picked up on the next build.

Migrations also run on server startup when `DB_MIGRATE_ON_START=true` (set by
the Taskfile and the Docker image; the binary defaults to false). The database
path comes from `DB_PATH` (Taskfile default `./data/sqlite.db`).

Queries live in `src/db/`, for example `src/db/users.rs`. They use the
runtime-checked `sqlx::query_as` API on purpose, so a fresh clone builds
without a database or a `DATABASE_URL`. If you want compile-time checked SQL,
replace those calls with the `query!` / `query_as!` macros, set
`DATABASE_URL=sqlite:./data/sqlite.db`, apply the migrations, and run
`cargo sqlx prepare` to commit the `.sqlx` query cache so builds keep working
offline. That adds a `cargo sqlx` tool requirement to the project.

## Linting and checks

`task check` runs two groups in parallel:

- `task check.rust`: `cargo fmt --check`, `cargo clippy --all-targets --locked
  -- -D warnings`, and `cargo test --all-targets --locked`. There is no
  separate `cargo build`: clippy checks every target and the test harnesses
  link, so an extra build only duplicates work.
- `task check.web`: prettier, eslint, `tsc --noEmit`, `lit-analyzer` on
  `web/lib/**/*.ts`, a bundle build, a Tailwind build into
  `web/tmp/check-static` (so a running dev session isn't clobbered), and the
  page-guard tests (`bun test ./scripts`). After the build,
  `web/scripts/check-static-refs.ts` asserts that every `/static/...` URL in
  `src/views/**/*.rs` exists in the output, so a renamed page file or asset
  fails the check instead of 404ing in production.

`tsc` ignores the `ts-lit-plugin` setting in `web/tsconfig.json` because that
is editor-only, so `lit-analyzer` is the check that covers Lit templates in
`web/lib/`.

`task fix` auto-fixes the formatting and lint issues it can
(`cargo fmt`, `clippy --fix`, `prettier --write`, `eslint --fix`) and rebuilds
to catch fixes that left the tree uncompilable. The git pre-push hook
(enabled by `task init`) runs `task check` as well.

Maud views are Rust, so editor support comes from rust-analyzer and
`cargo fmt --check` in `task check.rust` covers the crate. The markup layout
inside `html!` is not reflowed by rustfmt, so keep it tidy by hand.

## What next

You may want to:

- **Add a page:** add a handler in `src/routes/<name>.rs` and register it in
  `src/routes/mod.rs`, add `src/views/<name>.rs` with the page function, and
  pair it with `web/pages/<name>.ts` when it needs page JS. `src/views/root.rs`
  and `web/pages/root.ts` are the example to copy.
- **Add more hypermedia:** the example uses Datastar for update and delete
  (create is a plain POST + `303`). htmx or Alpine AJAX work too; handlers
  already answer plain form posts, so progressive enhancement stays easy.
- **Swap the database:** change the sqlx feature and `SqlitePool` in
  `src/db/mod.rs` to Postgres or MySQL, update `DB_PATH` to a
  `DATABASE_URL`-style key, and point the migration binary at the new database.
- **Switch to JSON logging:** enable the `json` feature on `tracing-subscriber`,
  then replace the fmt layer in `src/main.rs` with
  `tracing_subscriber::fmt().json()` for production-ready JSON logs.

## Project structure

```
.
├── bacon.toml           dev runner config (cargo run, restart on change)
│
├── bin                  build output, gitignored inside
│
├── Cargo.toml           crate manifest, server + migrate binaries
│
├── data                 runtime sqlite data (gitignored inside)
│
├── db
│   └── migrations       sqlx migrations, embedded in the binary
│
├── Dockerfile           production container image
│
├── .hooks               git hooks, enabled by task init
│
├── mise.toml            dev environment config
│
├── rust-toolchain.toml  rust channel and components
│
├── src
│   ├── main.rs          server entrypoint
│   ├── lib.rs           library crate root: modules below
│   ├── assets.rs        static assets: disk in dev, embedded in release
│   ├── config.rs        env config (HOST, PORT, DB_PATH, ...)
│   ├── dev.rs           live reload wiring for debug builds
│   ├── paths.rs         route patterns and link builders
│   ├── sse.rs           Datastar SSE helpers
│   ├── db/              pool, queries, migration runner
│   ├── routes/          axum handlers, error type, router
│   ├── views/           maud views: layout, error, one module per page
│   └── bin/migrate.rs     standalone migration binary
│
├── Taskfile.yml         taskfile with all tasks
│
├── tests
│   └── render.rs        view output tests: URLs, data attrs, escaping
│
├── tmp                  temp files, gitignored inside
│
└── web
    ├── assets           committed static assets (→ static/assets/)
    ├── build.ts         bundles lib/ and pages/, hardlinks assets
    ├── lib              shared ts libraries and Lit components (→ static/bundle.js)
    │   └── web-components/base  page-safe support classes
    ├── pages            page-specific entries (→ static/page-files/pages/)
    ├── scripts          build/check scripts (static refs, page guard)
    ├── static           build artifacts, gitignored
    └── tailwind.css     tailwind entry, source list, and theme
```

## Who this is for

I've tried to keep the template nice to use for anybody, but I've mainly
optimized it for my own tools and workflows:

1. Neovim 0.12+ with my custom config
2. Being very CLI-first
3. UNIX first, I have no plans of supporting Windows. It may work out of the box
   anyways, I haven't tested.

## License

[MIT](LICENSE)
