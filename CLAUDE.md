# DeskAuth21

Free, private, cross-platform desktop authenticator (TOTP) living in the
menu bar / system tray. Tauri 2 (Rust core) + TypeScript frontend.

Global standards in `~/.claude/CLAUDE.md` apply. The limits are enforced by
`eslint.config.js` and `src-tauri/clippy.toml`; if you change a limit, change
it in all three places.

## Commands

- `npm run check` runs eslint, clippy (`-D warnings`), Rust tests and `tsc`. It must pass before a PR.
- `npm run tauri dev` runs the app.
- `npm run test:rust`, `npm run lint`, `npm run lint:rust` run the pieces separately.

## Architecture

Pure domain logic is kept apart from Tauri/OS/UI adapters.

- Rust, `src-tauri/src/`: one module per concern (`otp`, `import`, `vault`, then
  `tray`, `commands`, `state`). Domain modules never import `tauri`. Time and
  randomness are passed in, not read inside pure functions.
- `lib.rs` only wires modules together (builder, setup); no business logic.
- Frontend, `src/`: organised by feature (`features/vault`, `features/codes`,
  `features/import`), each with its own components, hooks and tests. Calls to
  Rust go through one typed `src/lib/api.ts` layer; components never call
  `invoke` directly.

## UI decisions

- Component library: **shadcn/ui** (requires React + Tailwind CSS; the
  vanilla-TS starter is migrated in its own branch).
- Theme: **black and white only** (neutral grayscale tokens, no accent hues;
  destructive/error states may use the single shadcn destructive token).
- Colour scheme follows the **system preference by default** (`prefers-color-scheme`),
  with an optional manual override of system / light / dark stored locally.
- Font: **Montserrat**, bundled locally (e.g. `@fontsource-variable/montserrat`),
  never loaded from a CDN: this app is privacy-first and works offline.
- Generated shadcn files live in `src/components/ui/` and are excluded from lint;
  do not hand-edit them beyond theming tokens.

## Security rules

- Secrets (TOTP seeds, vault key, passphrase) never reach logs, the DOM beyond
  the visible code, or disk unencrypted.
- No network calls and no telemetry unless a feature explicitly requires them
  (sync), and then only end-to-end encrypted.

## Git workflow

- One short-lived branch per feature, named `<type>/<kebab-case-summary>`
  (`feat/`, `fix/`, `refactor/`, `chore/`, `docs/`).
- Several focused commits per branch, each with a detailed message: a summary
  line, then what changed and why. Never commit to `main` directly; open a PR.
- No AI attribution in commits or PR text (see global CLAUDE.md).
