- Use the `tmp/` directory for temporary files when testing or experimenting
  (scratch scripts, test output, etc). It is gitignored.

- Keep yourself updated with what is in Taskfile.yml in order to ensure that you
  run correct commands.

- After doing changes, run static and build checks:
  - Rust code, run `task check.rust`
  - web code, run `task check.web`
  - if both, run `task check` to check everything
  - to auto-fix formatting/linting, run `task fix` (or `task fix.rust` / `task fix.web`)

- Never put inline SVGs in `templates/`. Icons are Lit web components in
  `web/lib/web-components/icons`, so they inherit `currentColor`, size with
  font-size, and can be reused in shadow DOM.

- JS bundling (see `web/build.ts`):
  - `web/lib/**/*.ts` → single shared `static/bundle.js`, loaded synchronously
    in `<head>` (registers all web components before the body parses).
  - `.ts` files in any other top-level dir (e.g. `web/pages/root.ts`, next to
    its template) → per-file page bundles in `static/page-files/<dir>/`,
    loaded per page with `<script defer>`.
  - Page files: avoid importing from `lib/`. Small utility imports are fine,
    but each duplicates code into the page bundle (iife has no code
    splitting). Don't import web components from `lib/`: re-registering them
    throws.
  - `web/assets/**` is served as-is under `static/assets/**` (hardlinked, not
    copied). Prebuilt third-party scripts go there; normal packages belong in
    `web/lib` or a page file and come in through `bun add`.

- Templates are Jinja files in `templates/`, rendered with minijinja from
  `src/templates.rs`. Debug builds load them from disk with auto reload:
  editing one shows up on the next request and the live reload watcher
  refreshes the browser, no recompile or restart. Release builds embed them
  with minijinja-embed, so a template change needs a rebuild there.
- There is no compile-time field checking: handlers pass `#[derive(Serialize)]`
  context structs. Every variable must exist at render time
  (`UndefinedBehavior::Strict`), so use `{{ user.name or '' }}` for nullable
  columns. `tests/render.rs` renders the real templates as the guardrail.
  `djlint --reformat templates` formats them; they are plain Jinja.

- Datastar covers progressive enhancement:
  - Forms work as plain POST + 303 redirect for clients without JS.
  - Request handlers that have a Datastar caller answer it with SSE
    `datastar-patch-elements` instead of a redirect; the wire helpers live in
    `src/sse.rs`.
  - Keep page JS for behaviour that Datastar expressions can't express, and
    put it in a page file, not in `<script>` tags in templates.

- Database: schema changes go through sqlx migrations in `db/migrations/`
  (`task db.new -- <name>`, then edit the file, then `task db.migrate`).
  Queries are plain `sqlx` calls in `src/db/`, runtime-checked, so the crate
  builds without a live database. Don't add a `query!` macro without also
  committing the `.sqlx` query cache and a `DATABASE_URL` story for CI.

- `web/static/` is build output. Don't edit it and don't commit it; run
  `task build.assets` to regenerate.
