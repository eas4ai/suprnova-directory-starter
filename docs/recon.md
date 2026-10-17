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


## Cairn mechanism archive — 2026-10-17

Documented: these declarations are copied verbatim from commit `3d568ec154d1f6bcd1015225e9546450249d66ba` before migration. They preserve the commands, declared inputs and requirement mapping; they are not fresh Sudus evidence.

### admin-overview

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/admin-overview`.

```yaml
command: node scripts/verify-complete-starter.mjs overview
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - OVR-001
  - OVR-002
  - OVR-003
```

### delegated-administration

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/delegated-administration`.

```yaml
command: node scripts/verify-complete-starter.mjs administration
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - ADM-001
  - ADM-002
  - ADM-003
```

### directory-workflows

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/directory-workflows`.

```yaml
command: node scripts/verify-paid-directory.mjs directory
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - DIR-001
  - DIR-002
  - DIR-003
  - DIR-004
  - DIR-005
  - DIR-006
  - DIR-007
```

### editorial-publication

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/editorial-publication`.

```yaml
command: node scripts/verify-complete-starter.mjs editorial
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - CNT-001
  - CNT-002
  - CNT-003
  - CNT-004
  - CNT-005
  - CNT-006
```

### foundation-accounts

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-accounts`.

```yaml
command: node scripts/verify-foundation.mjs accounts
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - FND-003
```

### foundation-build

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-build`.

```yaml
command: node scripts/verify-foundation.mjs build
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-001
reviewed:
  - FND-001 sha256:cab1d772d0a6216adaec8e1b1fa93bc3e8c9d2199843736579fda8969d488ab2
  - FND-001 sha256:6b895667a44f2b9e4492727a51f67211445bb32d91c43b6f3dfd914b5478a67c
  - FND-001 sha256:736d84250da2c2b6d70a8aadcc8038ee6fdcc1be05f1d4ecf77f62413cbf3c43
```

### foundation-database

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-database`.

```yaml
command: node scripts/verify-foundation.mjs database
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-002
```

### foundation-setup

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-setup`.

```yaml
command: node scripts/verify-foundation.mjs setup
results: per-requirement
inputs:
  - .gitignore
  - handoff.md
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - .env.example
requirements:
  - FND-005
```

### foundation-ui

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/foundation-ui`.

```yaml
command: node scripts/verify-foundation.mjs ui
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - FND-004
```

### paid-lifecycle

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/paid-lifecycle`.

```yaml
command: node scripts/verify-paid-directory.mjs payments
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - DIR-008
  - PAY-002
  - PAY-003
  - PAY-009
  - PAY-010
  - PAY-011
  - PAY-012
  - PAY-013
  - PAY-014
  - PAY-015
  - PAY-016
```

### provider-administration

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/provider-administration`.

```yaml
command: node scripts/verify-provider-administration.mjs
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - PAY-001
  - PAY-004
  - PAY-005
  - PAY-006
  - PAY-007
  - PAY-008
reviewed:
  - PAY-001 sha256:88cb0a671a89e6443adcb96e5edf94ec2351a9cdf32a2536dec94a3925e302c3
  - PAY-001 sha256:1af61b53911c61f25abcfe20021aa6c54ac13c2dc77e580db1e54d0a2d404d19
  - PAY-001 sha256:47a350feb1d4eef45f36d3a14e93696635fef8dcb4b62909edcb9088ba9e51e4
```

### seo-controls

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/seo-controls`.

```yaml
command: node scripts/verify-complete-starter.mjs seo
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - SEO-001
  - SEO-002
  - SEO-003
  - SEO-004
  - SEO-005
  - SEO-006
reviewed:
  - SEO-001 sha256:7805abeb592222e448f738704c52b7305c14bbae1918ea2a8ce9f81f2baa182a
```

### site-appearance

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/site-appearance`.

```yaml
command: node scripts/verify-appearance.mjs
results: per-requirement
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
  - README.md
requirements:
  - UI-001
  - UI-002
```

### starter-adoption

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/starter-adoption`.

```yaml
command: node scripts/verify-complete-starter.mjs adoption
results: per-requirement
inputs:
  - .gitignore
  - Cargo.toml
  - Cargo.lock
  - lang/
  - README.md
  - handoff.md
  - LICENSE
  - THIRD_PARTY_NOTICES.md
  - src/
  - cmd/
  - frontend/
  - scripts/
  - tests/
  - .env.example
requirements:
  - KIT-001
  - KIT-002
  - KIT-003
  - KIT-004
  - KIT-005
```

### upgrade-guidance

Source: `3d568ec154d1f6bcd1015225e9546450249d66ba:.cairn/mechanisms/upgrade-guidance`.

```yaml
command: node .cairn/checks/verify-upgrade-guide.mjs
results: per-requirement
inputs:
  - UPGRADE.md
  - Cargo.toml
  - Cargo.lock
  - frontend/package.json
  - README.md
  - .cairn/checks/verify-upgrade-guide.mjs
requirements:
  - UPG-001
```
