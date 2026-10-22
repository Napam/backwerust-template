- Use the `tmp/` directory for temporary files when testing or experimenting
  (scratch scripts, test output, etc). It is gitignored.

- Keep yourself updated with what is in Taskfile.yml in order to ensure that you
  run correct commands.

- After doing changes, run static and build checks:
  - Rust code, run `task check.rust`
  - web code, run `task check.web`
  - if both, run `task check` to check everything
  - `check.web` runs prettier, eslint, tsc, lit-analyzer, the bundle and Tailwind
    builds, the page-guard tests, and the static-ref check; `check.rust`
    runs fmt, clippy, and tests
  - to auto-fix formatting/linting, run `task fix` (or `task fix.rust` / `task fix.web`)

- Never put inline SVGs in views (`src/views/`). Icons are Lit web
  components in `web/lib/web-components/icons`, so they inherit `currentColor`,
  size with font-size, and can be reused in shadow DOM.

- JS bundling (see `web/build.ts`):
  - `web/lib/**/*.ts` → single shared `static/bundle.js`, loaded synchronously
    in `<head>` (registers all web components before the body parses).
  - `.ts` files under `web/pages/` → per-file page bundles under
    `static/page-files/pages/`, loaded per page with `<script defer>`.
    `pages/` is an allowlist: other top-level dirs are not bundled.
  - Page files: avoid importing from `lib/`. Small utility imports are fine,
    but each duplicates code into the page bundle (iife has no code
    splitting). Don't import from `lib/web-components/` outside `base/`:
    `bundle.js` already registers every component under that directory, and the
    page build fails on the import. `lib/web-components/base/` holds the
    support classes that are safe for pages.
  - `web/assets/**` is served as-is under `static/assets/**` (hardlinked, not
    copied). Prebuilt third-party scripts go there; normal packages belong in
    `web/lib` or a page file and come in through `bun add`.
  - Static URLs in views are plain strings. `task check.web` runs
    `web/scripts/check-static-refs.ts` after the build and fails when a
    `/static/...` URL in `src/views/**/*.rs` has no file in the output.

- Tailwind: `web/tailwind.css` uses `@import 'tailwindcss' source(none)` plus
  explicit `@source` lines for `src/**/*.rs`, `web/lib/**/*.ts`, and
  `web/pages/**/*.ts`. Automatic detection is off, so a source directory with
  class names needs its own `@source` line.

- Views are maud functions under `src/views/`, one module per page, and return
  `Markup`. `html!` checks field names and types at compile time, so a renamed
  field or a wrong type fails the build instead of a request. A view edit is a
  Rust change: bacon rebuilds and restarts the server.
  - A route handler in `src/routes/<page>.rs` calls the view; register it in
    `src/routes/mod.rs`. `src/views/root.rs` builds the page shell with `layout`
    and calls `crud`; small partials are plain functions in the same module
    (`users_table`, `user_row`, `delete_form`).
  - Route patterns and link builders live in `src/paths.rs`. Use them in the
    router, views, redirects and tests instead of repeating URL literals.
  - Handlers return `routes::Result` with `AppError`. A missing row is
    `AppError::not_found(...)`; handler errors render `views::error` with their
    status. A Datastar request for a missing row re-renders the CRUD section
    instead of returning 404, so a row deleted in another tab disappears.
  - `tests/render.rs` renders the real views and asserts URLs, data attributes,
    and escaping.
  - `html!` bodies are not reflowed by rustfmt, so keep the markup layout tidy
    by hand.

- Datastar covers progressive enhancement:
  - Forms work as plain POST + 303 redirect for clients without JS.
  - Request handlers that have a Datastar caller answer it with SSE
    `datastar-patch-elements` instead of a redirect; the wire helpers live in
    `src/sse.rs`. Use `views::root::CRUD_ID` for the patch target instead of a
    second literal.
  - Keep page JS for behaviour that Datastar expressions can't express, and put
    it in `web/pages/`, not in a `<script>` tag emitted by a view.

- Database: schema changes go through sqlx migrations in `db/migrations/`
  (`task db.new -- <name>`, then edit the file, then `task db.migrate`).
  Queries are plain `sqlx` calls in `src/db/`, runtime-checked, so the crate
  builds without a live database. Don't add a `query!` macro without also
  committing the `.sqlx` query cache and a `DATABASE_URL` story for CI.

- `web/static/` is build output. Don't edit it and don't commit it; run
  `task build.assets` to regenerate.
