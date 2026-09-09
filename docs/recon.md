# Directory starter reconnaissance

Status: Observed
Date: 2026-09-08

The developer named this repository as the destination for a reusable directory starter built with Suprnova. The purchased Laravel application supplies feature references. Its observed behavior is not automatically the contract for this application.

## Exists

| Finding | Evidence |
| --- | --- |
| The package is directory, described as a Suprnova directory starter. It requires Rust 1.94.0. Initially pinned to v1.3.5, it now pins v1.3.7 following developer direction. | `Cargo.toml:1`, `Cargo.toml:29`. |
| The frontend uses Vue 3, Inertia and Tailwind. Build performs Vue type checking before Vite; separate type-check and SSR build commands exist. | `frontend/package.json:6`. |
| The server registers configuration, shared bootstrap, HTTP bootstrap, routes and migrations. A separate console binary is declared. | `cmd/main.rs:8`, `Cargo.toml:25`. |
| Routes cover a home page, guest login/registration, authenticated dashboard and POST logout. No directory or moderation route is declared here. | `src/routes.rs:6`. |
| Registration checks duplicate email, creates a user, logs them in and redirects to the dashboard. The user model hashes passwords and declares fillable/hidden fields. | `src/controllers/auth.rs:115`, `src/models/user.rs:18`, `src/models/user.rs:59`. |
| The user model implements email-verification and password-reset traits. Those traits do not establish complete user flows: the current application route table has no corresponding endpoints. | `src/models/user.rs:124`, `src/models/user.rs:146`, `src/routes.rs:6`. |
| Registered migrations cover users, sessions, remember tokens and auth-flow tokens. No category or listing migration is registered. | `src/migrations/mod.rs:12`. |
| HTTP bootstrap wires logging, sessions, Vue Inertia, locale, CSRF, include-field parsing and shared locale data. | `src/bootstrap.rs:84`. |
| Home is a starter welcome page. Application pages discovered are Home, Dashboard, Login and Register. | `frontend/src/pages/Home.vue:8`, `frontend/src/pages/Dashboard.vue`, `frontend/src/pages/auth/Login.vue`, `frontend/src/pages/auth/Register.vue`. |

## Documented

| Finding | Evidence |
| --- | --- |
| Source comments explain authentication, model generation and middleware order. | `src/models/user.rs:1`, `src/bootstrap.rs:70`. |
| The ignore file documents tracking Cargo.lock and excluding runtime environment files and mail previews. | `.gitignore:6`, `.gitignore:21`, `.gitignore:128`. |
| No Markdown documentation or existing spec set was found before this recon. | Inventory record below. |

## Contradicted

| Finding | Both sides |
| --- | --- |
| The ignore-file commentary says the Dockerfile copies Cargo.lock, but neither file is present in this scaffold. This appears to be inherited scaffold documentation, not an established deployment procedure. | `.gitignore:6`; inventory record below. |

No Agreed requirements exist to assess contractual drift. No previous target recon existed to carry forward. Unresolved findings in the reference application's report remain there; this report does not close them.

## Unverified

| Finding | Evidence / next check |
| --- | --- |
| Runtime and compilation are unverified. No dependency lockfiles or installed frontend dependencies were found. | Inventory below; `Cargo.toml:29`, `frontend/package.json:6`. |
| Initial history was absent at first inspection; the developer subsequently committed and published the scaffold. Local HEAD and remote main now match. Only the new docs are untracked. | Follow-up Git record below. |
| No tests directory, scripts directory or .github directory was found. Inline tests and framework tests are not a substitute for verified directory behavior. | Inventory below. |
| No project-specific working agreement, production rules override or Cairn roadmap was found. | Inventory and wake result below. |
| Payment provider, entitlement rules, moderation lifecycle and starter customization contract remain undecided. | Developer requested a directory starter on 2026-09-08; proposed scope is in `docs/feature-map.md`. |

## Inspection record

Read-only inspection on 2026-09-08:

- Graph indexing and source inspection completed for this repository.
- `git status --short` showed the existing scaffold files as untracked. `git log -5 --oneline` reported that main has no commits yet.
- Markdown discovery returned no files before this report.
- Existence checks found no Cargo.lock, frontend/package-lock.json, frontend/bun.lock, frontend/node_modules, tests, scripts, .github, AGENTS.md, BEST_PRACTICES.md or docs/spec/overview.md. Root inventory contained no Dockerfile.
- `cairn wake` exited 3: not a Cairn repository, no docs/spec/roadmap.md.
- No application build, tests, migration, dependency install or service startup was attempted during this scope-definition pass. The spec lint script is absent.
- The existing .env was not read or changed.

Follow-up after the developer created the GitHub origin on 2026-09-08:

- Origin is `https://github.com/eas4ai/suprnova-directory-starter.git`.
- `git ls-remote --heads origin` succeeded and returned main at `a49cb8e841c6124381e12e7dea89ac544cee0916`.
- `git rev-parse HEAD` returned the same commit; `git log -3` names it `first commit`.
- `git status --short --branch` showed main tracking origin/main with only docs/ untracked. This supersedes the initial uncommitted-scaffold observation above.

## Todo

- Complete: inspect the target scaffold and feature gaps; verify cited source locations.
- Complete: record target recon and proposed feature scope.
- In progress: settle the first commitment's scope and acceptance checks with the developer.
- Pending: write requirements and their falsifiers, obtain agreement, then prepare the commitment and its mechanisms.
