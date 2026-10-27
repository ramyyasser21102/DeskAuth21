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
- Frontend, `src/`: React + TypeScript organised by feature (`features/vault`,
  `features/codes`, `features/theme`, later `features/import`), each with its own
  components, hooks and tests. Keep pure logic in plain `.ts` files (e.g.
  `theme-preference.ts`) and keep components thin, with state in `use-*` hooks.
  Calls to Rust go through one typed `src/lib/api.ts` layer; components never
  call `invoke` directly. `src/components/ui/` holds generated shadcn files only.

## UI decisions

- Component library: **shadcn/ui** (radix base, `radix-vega` style) on React 19
  and Tailwind CSS v4. Config is in `components.json`; the `@/` alias maps to `src/`.
- Adding shadcn components: do **not** run `shadcn add` directly. This CLI
  version resolves the utils import to a package named `cn` and tries to
  `npm install cn`, which is an unrelated npm package. Instead run
  `npx shadcn@latest add <name> --view` (dry run, writes nothing), save the
  printed files under `src/components/ui/`, and change `from "cn"` to
  `from "@/lib/utils"`. Install any new dependencies by hand with `npm install`.
- Theme: **black and white only** (neutral grayscale tokens, no accent hues;
  destructive/error states may use the single shadcn destructive token).
- Colour scheme follows the **system preference by default** (`prefers-color-scheme`),
  with an optional manual override of system / light / dark stored locally.
- Font: **Montserrat**, bundled locally (e.g. `@fontsource-variable/montserrat`),
  never loaded from a CDN: this app is privacy-first and works offline.
- Newer shadcn output styles Base UI attributes (`data-checked`, `data-open`, ...). This project
  uses Radix, so `src/index.css` maps those variants onto Radix's `data-state`; keep the
  generated files untouched and extend that mapping if a new component needs another state.
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
