# Suprnova 4.6.1 upgrade verification

Suprnova Directory Starter 1.1.0 selects framework and Stripe/Paddle adapter tag
`v4.6.1`, commit `5112862367b76a2ea3875d0135ca4ca1ec602c32`. Markdown selects
`v0.5.5`, commit `1d2650a315a1f424931f2362355fc8ad9dd28b45`. There is one
Suprnova dependency identity. Both application version entries and the Cargo
lock's application entry are 1.1.0. Bun's generated root workspace entry has
no version field.

The frontend is Vue. The matching templates live at
`suprnova-cli/src/templates/files/frontend/vue`. The Inertia core, Vue and Vite
dependencies use the current kit's `^3.8.0` ranges, with 3.8.0 resolved in Bun's
lock. The kit also adds `webmcp-types` 0.1.10. No OAuth GitHub plugin is installed
in this application, so its separate release needs no dependency change.

The original upgrade commits remain on `upgrade/suprnova-4.6.0`; that branch name
is retained as requested. The final target is 4.6.1. No push, tag, merge, main
update, framework-checkout edit, Sudus command, skill or AI sub-thread was used.

## Commits

- `3297933`: initial release pins and Vue kit dependency alignment.
- `a527103`: Vue kit runtime adaptation while preserving directory flows.
- `d11bc57`: starter Clippy warning fixes.
- `563f33d`: earlier 1.1.0 release preparation, retained in branch history.
- `99808b7 build: pin Suprnova 4.6.1 and refresh vulnerable dependencies`.
- `4e08adf chore: remove the remaining migration borrow warning`.

The final commit is `release: Suprnova Directory Starter 1.1.0 on Suprnova 4.6.1`.
It updates the guide, notices and this renamed verification report. Its own SHA
is recorded by Git rather than embedded in the file it commits.

## Environment and isolation

Starter checks use Rust 1.95.0, Node 24.22.1 and Bun 1.4.4. The pinned upstream
adapter verifier uses its committed archive's Rust 1.98.1 toolchain. Every Cargo
invocation runs serially through a local wrapper using `taskset -c 12-24:2`:

```sh
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 RUST_MIN_STACK=33554432
export TMPDIR=/home/shawn/workspace2/scratchpads/tmp/claude-1000/-home-shawn-workspace2-suprnova/59af0312-be91-4d54-9a2d-50788f39d099/scratchpad/starters/suprnova-directory-starter/tmp
export CARGO_PROFILE_DEV_DEBUG=0
```

Repository verifiers copy only application inputs to disposable workspaces under
`.upgrade-work/workspaces`. They exclude operator environment and data. The final
whole suite uses built client and SSR assets and one local SSR worker. A test
runner gives each binary a fresh SQLite file and media directory under the
supplied `TMPDIR`, with the origin its existing assertions expect. No operator
environment file or credential is read or printed.

The provider-key test and paid-browser server are contextual fixtures ignored
by the ordinary whole suite. Their owning verifiers run them with their required
fixture configuration. Documentation examples remain ignored Rust doctests.

## Dependency refresh and audit

The release pin refresh was limited to the four requested packages:

```sh
cargo update -p suprnova -p suprnova-payments-stripe -p suprnova-payments-paddle -p suprnova-markdown
```

It exits zero and moves all eight Suprnova Git packages to the new tags, without
an unrestricted update. The expanded security allowance then permits these
individual updates, each exiting zero:

```sh
cargo update -p ammonia
cargo update -p quinn-proto
cargo update -p rustls
cargo update -p rustls-webpki
cargo update -p webpki-root-certs
cargo update -p webpki-roots@1.0.9
cargo update -p basic-toml
cargo update -p bincode
cargo update -p paste
cargo update -p ttf-parser
cargo update -p yaml-rust
```

| Package | Previous | Final | Cleared advisories |
| --- | --- | --- | --- |
| ammonia | 4.1.4 | 4.2.4 | RUSTSEC-2026-0373, RUSTSEC-2021-0158, RUSTSEC-2021-0157 |
| quinn-proto | 0.11.17 | 0.11.19 | RUSTSEC-2026-0343, RUSTSEC-2026-0338, RUSTSEC-2026-0340 |
| rustls | 0.23.44 | 0.23.46 | RUSTSEC-2026-0349, RUSTSEC-2026-0346, RUSTSEC-2026-0285, RUSTSEC-2026-0348, RUSTSEC-2026-0347 |
| rustls-webpki | 0.103.15 | 0.103.16 | RUSTSEC-2026-0382 |
| webpki-root-certs | 1.0.9 | 1.0.10 | RUSTSEC-2026-0383 |
| webpki-roots | 1.0.9 | 1.0.10 | RUSTSEC-2026-0381 |

The five maintenance-warning package updates select no newer compatible version.
The warnings remain visible and are not ignored:

| Advisory | Package | Dependency path and reason retained |
| --- | --- | --- |
| RUSTSEC-2026-0374 | basic-toml 0.1.10 | Askama derive 0.16.0 uses it; no compatible update resolves this unmaintained notice. |
| RUSTSEC-2025-0141 | bincode 1.3.3 | Comrak 0.52.0 uses Syntect 5.3.0, which requires bincode 1.x. Updating bincode alone cannot replace that dependency API. |
| RUSTSEC-2024-0436 | paste 1.0.15 | Tokenizers 0.23.2 uses it; no newer compatible paste release exists in this resolution. |
| RUSTSEC-2026-0192 | ttf-parser 0.25.1 | Live charts use charts-rs 1.0.0 and fontdue 0.9.4; its compatible ttf-parser resolution remains unchanged. |
| RUSTSEC-2024-0320 | yaml-rust 0.4.5 | Comrak/Syntect requires yaml-rust 0.4; a different parser requires an upstream integration change. |

The framework's `.cargo/audit.toml` documents RUSTSEC-2026-0235 (rkyv) and
RUSTSEC-2023-0071 (rsa). Neither crate is present in this application's lockfile.
No copied exception or application audit ignore file is needed. The final audit
has zero vulnerabilities and five allowed maintenance warnings.

`cargo tree --locked -i suprnova` exits zero:

```text
    Blocking waiting for file lock on package cache
suprnova v4.6.1 (https://github.com/eas4ai/suprnova.git?tag=v4.6.1#51128623)
├── directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter)
├── suprnova-markdown v0.5.5 (https://github.com/eas4ai/suprnova-markdown.git?tag=v0.5.5#1d2650a3)
│   └── directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter)
├── suprnova-payments-paddle v4.6.1 (https://github.com/eas4ai/suprnova.git?tag=v4.6.1#51128623)
│   └── directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter)
└── suprnova-payments-stripe v4.6.1 (https://github.com/eas4ai/suprnova.git?tag=v4.6.1#51128623)
    └── directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter)
```

## Final verification commands

The whole Rust suite and `cargo clippy --locked --all-targets` each ran once
after the repository acceptance tasks. The whole suite passed. Clippy exited
zero with one remaining migration borrow warning, whose suggestion was applied
after that run. Formatting, all-target compilation, both migration baselines,
payment wire checks and both payment modes then ran on the last source. The whole
suite and Clippy were not repeated, respecting the requested once-only limits;
they therefore precede that mechanical one-line correction. No warning-free
Clippy rerun is claimed. `-D warnings` was never used. The following table records
every 4.6.1 invocation through the evidence logger, including dependency updates
and the preliminary audit.

| Invocation | Exact command | Exit |
| --- | --- | --- |
| `461-update-pins` | `cargo update -p suprnova -p suprnova-payments-stripe -p suprnova-payments-paddle -p suprnova-markdown` | 0 |
| `461-audit-baseline` | `cargo audit` | 1 |
| `461-security-update-ammonia` | `cargo update -p ammonia` | 0 |
| `461-security-update-quinn-proto` | `cargo update -p quinn-proto` | 0 |
| `461-security-update-rustls` | `cargo update -p rustls` | 0 |
| `461-security-update-rustls-webpki` | `cargo update -p rustls-webpki` | 0 |
| `461-security-update-webpki-root-certs` | `cargo update -p webpki-root-certs` | 0 |
| `461-security-update-webpki-roots@1.0.9` | `cargo update -p webpki-roots@1.0.9` | 0 |
| `461-security-update-basic-toml` | `cargo update -p basic-toml` | 0 |
| `461-security-update-bincode` | `cargo update -p bincode` | 0 |
| `461-security-update-paste` | `cargo update -p paste` | 0 |
| `461-security-update-ttf-parser` | `cargo update -p ttf-parser` | 0 |
| `461-security-update-yaml-rust` | `cargo update -p yaml-rust` | 0 |
| `461-audit-remediated` | `cargo audit` | 0 |
| `461-dependency-tree` | `cargo tree --locked -i suprnova` | 0 |
| `461-upgrade-guide` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `461-syntax-verify-foundation` | `node --check scripts/verify-foundation.mjs` | 0 |
| `461-syntax-verify-framework-upgrade` | `node --check scripts/verify-framework-upgrade.mjs` | 0 |
| `461-syntax-verify-pinned-checkout` | `node --check scripts/verify-pinned-checkout.mjs` | 0 |
| `461-syntax-verify-upgrade-guide` | `node --check scripts/verify-upgrade-guide.mjs` | 0 |
| `461-generate-types` | `/home/shawn/workspace2/scratchpads/targets/suprnova/main/debug/suprnova generate-types` | 0 |
| `461-framework-build` | `node scripts/verify-framework-upgrade.mjs build` | 0 |
| `461-ripwire-test-gate` | `ripwire . --test-gate=cmd/main.rs,frontend/src/lib/agents/core.ts,frontend/src/lib/agents/index.ts,frontend/src/lib/agents/inertia.ts,frontend/src/lib/static/core.ts,frontend/src/lib/static/index.ts,frontend/src/lib/stream/core.ts,frontend/src/lib/stream/index.ts,frontend/src/main.ts,frontend/src/resolve.ts,frontend/tests/directory-workflows.mjs,frontend/vite.config.ts,scripts/verification-workspace.mjs,scripts/verify-appearance.mjs,scripts/verify-complete-starter.mjs,scripts/verify-foundation.mjs,scripts/verify-framework-upgrade.mjs,scripts/verify-paid-directory.mjs,scripts/verify-pinned-checkout.mjs,scripts/verify-provider-administration.mjs,scripts/verify-upgrade-guide.mjs,src/accounts/provider.rs,src/agents.rs,src/articles/workflow.rs,src/billing/events.rs,src/bootstrap.rs,src/controllers/account_links.rs,src/controllers/auth.rs,src/email_address.rs,src/lib.rs,src/listings/media.rs,src/listings/workflow.rs,src/live/mod.rs,src/middleware/authenticate.rs,src/migrations/m20260909_000007_create_publishing.rs,src/migrations/m20260909_000008_create_articles.rs,src/migrations/mod.rs,src/models/user.rs,src/public_pages.rs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs --legend=compact` | 4 |
| `461-ripwire-edits` | `ripwire . --edit-check=verifyPin --legend=compact` | 0 |
| `461-ripwire-quality` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 0 |
| `461-framework-migration` | `node scripts/verify-framework-upgrade.mjs migration` | 0 |
| `461-framework-auth` | `node scripts/verify-framework-upgrade.mjs auth` | 0 |
| `461-framework-http` | `node scripts/verify-framework-upgrade.mjs http` | 0 |
| `461-framework-console` | `node scripts/verify-framework-upgrade.mjs console` | 0 |
| `461-framework-regression` | `node scripts/verify-framework-upgrade.mjs regression` | 0 |
| `461-final-frontend-install` | `bun install --frozen-lockfile --cwd frontend` | 0 |
| `461-final-frontend-check` | `bun run --cwd frontend check` | 0 |
| `461-final-frontend-build` | `bun run --cwd frontend build` | 0 |
| `461-final-frontend-ssr` | `bun run --cwd frontend build:ssr` | 0 |
| `461-final-migration-4.2` | `cargo test --locked --test framework_upgrade_migration` | 0 |
| `461-final-check` | `cargo check --locked --all-targets` | 0 |
| `461-final-format` | `cargo fmt --check` | 0 |
| `461-final-test` | `node scripts/with-ssr.mjs cargo test --locked` | 0 |
| `461-final-clippy` | `cargo clippy --locked --all-targets` | 0 |
| `461-final-audit` | `cargo audit` | 0 |
| `461-post-clippy-format` | `cargo fmt --check` | 0 |
| `461-post-clippy-check` | `cargo check --locked --all-targets` | 0 |
| `461-post-clippy-migration-payments` | `node scripts/with-ssr.mjs cargo test --locked --test framework_upgrade_migration --test paid_lifecycle --test payment_sdk_wire` | 0 |
| `461-post-clippy-migration-4.2` | `cargo test --locked --test framework_upgrade_migration` | 0 |
| `461-post-clippy-paid-live` | `node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle` | 0 |
| `461-ripwire-quality-complete` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 0 |
| `461-ripwire-migration-edits` | `ripwire . --edit-check=src/migrations/m20260909_000007_create_publishing.rs:up --legend=compact` | 0 |
| `461-ripwire-test-gate-complete` | `ripwire . --test-gate=cmd/main.rs,frontend/src/lib/agents/core.ts,frontend/src/lib/agents/index.ts,frontend/src/lib/agents/inertia.ts,frontend/src/lib/static/core.ts,frontend/src/lib/static/index.ts,frontend/src/lib/stream/core.ts,frontend/src/lib/stream/index.ts,frontend/src/main.ts,frontend/src/resolve.ts,frontend/tests/directory-workflows.mjs,frontend/vite.config.ts,scripts/verification-workspace.mjs,scripts/verify-appearance.mjs,scripts/verify-complete-starter.mjs,scripts/verify-foundation.mjs,scripts/verify-framework-upgrade.mjs,scripts/verify-paid-directory.mjs,scripts/verify-pinned-checkout.mjs,scripts/verify-provider-administration.mjs,scripts/verify-upgrade-guide.mjs,src/accounts/provider.rs,src/agents.rs,src/articles/workflow.rs,src/billing/events.rs,src/bootstrap.rs,src/controllers/account_links.rs,src/controllers/auth.rs,src/email_address.rs,src/lib.rs,src/listings/media.rs,src/listings/workflow.rs,src/live/mod.rs,src/middleware/authenticate.rs,src/migrations/m20260909_000007_create_publishing.rs,src/migrations/m20260909_000008_create_articles.rs,src/migrations/mod.rs,src/models/user.rs,src/public_pages.rs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs --legend=compact` | 4 |

Additional read-only verification: the supplied CLI's `--version` reports
`suprnova 4.6.1`; its regenerated Inertia props, language keys and tool schemas
match the checked-in files byte for byte. Pin/lock inspection confirms one
Suprnova identity and no rsa/rkyv package. `git diff --check` verifies whitespace.
The release check also verifies package versions, report results, target-version
references, and the unchanged main ref.

```sh
/home/shawn/workspace2/scratchpads/targets/suprnova/main/debug/suprnova --version
git diff --check
git diff --cached --check
git diff --check main..HEAD
python3 .upgrade-work/verify-report-461.py
git status --short --branch
git log -7 --format='%h %s'
git rev-parse main
```

The six repository tasks cover the build, empty and populated 3.2.1/4.2.0
migration paths with repeated migration, accounts, auth races/revocation, HTTP
cache/asset/SSR behavior, console exit codes and all 55 inherited requirements.
Regression runs the foundation build/database/accounts/UI/setup checks, provider
administration, paid-directory directory/payments, complete-starter editorial/
administration/adoption/overview/SEO, appearance and upgrade-guide verifiers.
The paid verifier exercises the committed 4.6.1 Stripe/Paddle SDK fixture tests;
it does not patch the Cargo cache or contact real payment providers.

### Final Rust suite output

```text
     Running unittests src/lib.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/directory-3c44c7a9e7163cbb)
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.59s
     Running unittests src/bin/console.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/console-c58bae15e4fb3a90)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests cmd/main.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/directory-1649c3c70073a431)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/administration_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/administration_workflows-fc85d1417d089357)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.17s
     Running tests/adoption_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/adoption_workflows-86f3ab96748c9172)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.39s
     Running tests/demo_seed.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/demo_seed-1051a74a538b0fa4)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.42s
     Running tests/directory_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/directory_workflows-44788c0480baedc9)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.13s
     Running tests/editorial_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/editorial_workflows-01fc75bc6a1c0cf0)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.23s
     Running tests/foundation_accounts.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/foundation_accounts-5a1e828037de237e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.70s
     Running tests/foundation_ui.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/foundation_ui-41e4664a906512ca)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.30s
     Running tests/framework_upgrade_auth.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/framework_upgrade_auth-049f7ca41b4f9d88)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.29s
     Running tests/framework_upgrade_cache.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/framework_upgrade_cache-0b6d7c19da8d27c4)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
     Running tests/framework_upgrade_migration.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/framework_upgrade_migration-93baef7e9a644d54)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
     Running tests/overview_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/overview_workflows-684a4d8753854d2c)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.05s
     Running tests/page_budget.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/page_budget-c396b93874923fbd)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/paid_lifecycle.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/paid_lifecycle-5c4bf5be044d6801)
test browser::serve_paid_browser_fixture ... ignored, Started only by the paid browser verifier
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.96s
     Running tests/payment_sdk_wire.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/payment_sdk_wire-5de25c3944a98d19)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/provider_administration.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/provider_administration-dfabb57d3d5a0755)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s
     Running tests/provider_key_policy.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/provider_key_policy-132e4ea25396ba3b)
test deployment_key_policy ... ignored, Started with persisted credentials by verify-provider-administration.mjs
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/seo_workflows.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/seo_workflows-736b14063f020995)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.21s
test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Final Clippy output

```text
    Checking directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-final-461-mRPmQ7)
warning: the borrowed expression implements the required traits
   --> src/migrations/m20260909_000007_create_publishing.rs:205:25
    |
205 | /                         &mut ColumnDef::new(Alias::new("raw_body"))
206 | |                             .binary()
207 | |                             .not_null()
208 | |                             .to_owned(),
    | |_______________________________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#needless_borrows_for_generic_args
    = note: `#[warn(clippy::needless_borrows_for_generic_args)]` on by default
help: change this to
    |
205 ~                         ColumnDef::new(Alias::new("raw_body"))
206 +                             .binary()
207 +                             .not_null()
208 ~                             .to_owned(),
    |

warning: `directory` (lib test) generated 1 warning (1 duplicate)
warning: `directory` (lib) generated 1 warning (run `cargo clippy --fix --lib -p directory -- ` to apply 1 suggestion)
    Finished `dev` profile [unoptimized] target(s) in 13.19s
```

### Final cargo audit output

```text
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1349 security advisories (from /home/shawn/.cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (818 crate dependencies)
Crate:     basic-toml
Version:   0.1.10
Warning:   unmaintained
Title:     `basic-toml` is unmaintained
Date:      2026-09-26
ID:        RUSTSEC-2026-0374
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0374
Dependency tree:
basic-toml 0.1.10
└── askama_derive 0.16.0
    └── askama_macros 0.16.0
        └── askama 0.16.0
            ├── suprnova-live 4.6.1
            │   └── suprnova 4.6.1
            │       ├── suprnova-payments-stripe 4.6.1
            │       │   └── directory 1.1.0
            │       ├── suprnova-payments-paddle 4.6.1
            │       │   └── directory 1.1.0
            │       ├── suprnova-markdown 0.5.5
            │       │   └── directory 1.1.0
            │       └── directory 1.1.0
            └── suprnova 4.6.1

Crate:     bincode
Version:   1.3.3
Warning:   unmaintained
Title:     Bincode is unmaintained
Date:      2025-12-16
ID:        RUSTSEC-2025-0141
URL:       https://rustsec.org/advisories/RUSTSEC-2025-0141
Dependency tree:
bincode 1.3.3
└── syntect 5.3.0
    └── comrak 0.52.0
        └── suprnova 4.6.1
            ├── suprnova-payments-stripe 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-payments-paddle 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-markdown 0.5.5
            │   └── directory 1.1.0
            └── directory 1.1.0

Crate:     paste
Version:   1.0.15
Warning:   unmaintained
Title:     paste - no longer maintained
Date:      2024-10-07
ID:        RUSTSEC-2024-0436
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
Dependency tree:
paste 1.0.15
└── tokenizers 0.23.2
    └── suprnova 4.6.1
        ├── suprnova-payments-stripe 4.6.1
        │   └── directory 1.1.0
        ├── suprnova-payments-paddle 4.6.1
        │   └── directory 1.1.0
        ├── suprnova-markdown 0.5.5
        │   └── directory 1.1.0
        └── directory 1.1.0

Crate:     ttf-parser
Version:   0.25.1
Warning:   unmaintained
Title:     `ttf-parser` is unmaintained
Date:      2026-06-28
ID:        RUSTSEC-2026-0192
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0192
Dependency tree:
ttf-parser 0.25.1
└── fontdue 0.9.4
    └── charts-rs 1.0.0
        └── suprnova-live 4.6.1
            └── suprnova 4.6.1
                ├── suprnova-payments-stripe 4.6.1
                │   └── directory 1.1.0
                ├── suprnova-payments-paddle 4.6.1
                │   └── directory 1.1.0
                ├── suprnova-markdown 0.5.5
                │   └── directory 1.1.0
                └── directory 1.1.0

Crate:     yaml-rust
Version:   0.4.5
Warning:   unmaintained
Title:     yaml-rust is unmaintained.
Date:      2024-03-20
ID:        RUSTSEC-2024-0320
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0320
Dependency tree:
yaml-rust 0.4.5
└── syntect 5.3.0
    └── comrak 0.52.0
        └── suprnova 4.6.1
            ├── suprnova-payments-stripe 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-payments-paddle 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-markdown 0.5.5
            │   └── directory 1.1.0
            └── directory 1.1.0

warning: 5 allowed warnings found
```

### Public page budget output

```text
/: 6 requests, 476003 bytes
/listings: 6 requests, 475607 bytes
/articles: 6 requests, 475045 bytes
/login: 6 requests, 469263 bytes
/register: 6 requests, 469269 bytes
/forgot-password: 6 requests, 469282 bytes
/reset-password: 6 requests, 469291 bytes
test budget_public_pages ... ok
test budget_public_pages ... ok
```

### All-target compilation after the borrow correction

```text
    Checking directory v1.1.0 (/home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-final-461-mRPmQ7)
    Finished `dev` profile [unoptimized] target(s) in 12.14s
```

### Migration and payment follow-up results

```text
     Running tests/framework_upgrade_migration.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/framework_upgrade_migration-93baef7e9a644d54)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s
     Running tests/paid_lifecycle.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/paid_lifecycle-5c4bf5be044d6801)
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.15s
     Running tests/payment_sdk_wire.rs (/home/shawn/workspace2/suprnova-directory-starter/target/debug/deps/payment_sdk_wire-5de25c3944a98d19)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Populated 4.2 migration follow-up result

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

### Live checkout-mode follow-up result

```text
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.50s
```

## Failures and warnings

`461-audit-baseline` runs after the release pin change but before the authorized
security refresh. It exits 1 with 14 vulnerabilities. Its complete output follows;
the targeted updates above clear every vulnerability. Subsequent successful
checks do not erase this failed baseline.

```text
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1349 security advisories (from /home/shawn/.cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (819 crate dependencies)
Crate:     ammonia
Version:   4.1.4
Title:     Missing case-insensitive matching for CSS url function
Date:      2026-10-16
ID:        RUSTSEC-2026-0373
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0373
Solution:  Upgrade to >=4.2.4 OR >=4.1.8, <4.2.0
Dependency tree:
ammonia 4.1.4
└── suprnova 4.6.1
    ├── suprnova-payments-stripe 4.6.1
    │   └── directory 1.1.0
    ├── suprnova-payments-paddle 4.6.1
    │   └── directory 1.1.0
    ├── suprnova-markdown 0.5.5
    │   └── directory 1.1.0
    └── directory 1.1.0

Crate:     ammonia
Version:   4.1.4
Title:     XSS in ammonia via SVG `animateMotion`, `animateTransform`, and/or `animateColor` tags
Date:      2021-10-10
ID:        RUSTSEC-2021-0158
URL:       https://rustsec.org/advisories/RUSTSEC-2021-0158
Solution:  Upgrade to >=4.2.3 OR >=4.1.7, <4.2.0

Crate:     ammonia
Version:   4.1.4
Title:     Incorrect escaping for `url("")` functions in CSS style attributes
Date:      2021-10-10
ID:        RUSTSEC-2021-0157
URL:       https://rustsec.org/advisories/RUSTSEC-2021-0157
Solution:  Upgrade to >=4.2.3 OR >=4.1.7, <4.2.0

Crate:     quinn-proto
Version:   0.11.17
Title:     Outgoing datagram buffer accounting can panic the sender in quinn-proto
Date:      2026-09-14
ID:        RUSTSEC-2026-0343
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0343
Severity:  7.5 (high)
Solution:  Upgrade to >=0.11.18
Dependency tree:
quinn-proto 0.11.17
└── quinn 0.11.11
    ├── reqwest 0.13.5
    │   ├── suprnova-web-push 4.6.1
    │   │   └── suprnova 4.6.1
    │   │       ├── suprnova-payments-stripe 4.6.1
    │   │       │   └── directory 1.1.0
    │   │       ├── suprnova-payments-paddle 4.6.1
    │   │       │   └── directory 1.1.0
    │   │       ├── suprnova-markdown 0.5.5
    │   │       │   └── directory 1.1.0
    │   │       └── directory 1.1.0
    │   ├── suprnova-magnetar 4.6.1
    │   │   └── suprnova 4.6.1
    │   ├── suprnova 4.6.1
    │   ├── sentry 0.49.2
    │   │   └── suprnova 4.6.1
    │   └── qdrant-client 1.19.0
    │       └── suprnova 4.6.1
    └── reqwest 0.12.28
        ├── paddle-rust-sdk 0.18.0
        │   ├── suprnova-payments-paddle 4.6.1
        │   └── directory 1.1.0
        └── apple 0.3.1
            └── suprnova-magnetar 4.6.1

Crate:     quinn-proto
Version:   0.11.17
Title:     Remote peer can panic quinn-proto with zero-length STREAM frames during unordered reads
Date:      2026-09-14
ID:        RUSTSEC-2026-0338
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0338
Solution:  Upgrade to >=0.11.18

Crate:     quinn-proto
Version:   0.11.17
Title:     Peer minimum ACK delay can panic quinn-proto when ACK_FREQUENCY transmission is enabled
Date:      2026-09-14
ID:        RUSTSEC-2026-0340
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0340
Severity:  7.5 (high)
Solution:  Upgrade to >=0.11.18

Crate:     rustls
Version:   0.23.44
Title:     Unauthenticated ServerHello can downgrade later connections to classical key exchange
Date:      2026-10-14
ID:        RUSTSEC-2026-0349
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0349
Severity:  3.7 (low)
Solution:  Upgrade to >=0.23.46
Dependency tree:
rustls 0.23.44
├── ureq 3.4.2
│   └── sentry 0.49.2
│       └── suprnova 4.6.1
│           ├── suprnova-payments-stripe 4.6.1
│           │   └── directory 1.1.0
│           ├── suprnova-payments-paddle 4.6.1
│           │   └── directory 1.1.0
│           ├── suprnova-markdown 0.5.5
│           │   └── directory 1.1.0
│           └── directory 1.1.0
├── tokio-rustls 0.26.5
│   ├── tonic 0.14.6
│   │   ├── tonic-prost 0.14.6
│   │   │   └── qdrant-client 1.19.0
│   │   │       └── suprnova 4.6.1
│   │   └── qdrant-client 1.19.0
│   ├── reqwest 0.13.5
│   │   ├── suprnova-web-push 4.6.1
│   │   │   └── suprnova 4.6.1
│   │   ├── suprnova-magnetar 4.6.1
│   │   │   └── suprnova 4.6.1
│   │   ├── suprnova 4.6.1
│   │   ├── sentry 0.49.2
│   │   └── qdrant-client 1.19.0
│   ├── reqwest 0.12.28
│   │   ├── paddle-rust-sdk 0.18.0
│   │   │   ├── suprnova-payments-paddle 4.6.1
│   │   │   └── directory 1.1.0
│   │   └── apple 0.3.1
│   │       └── suprnova-magnetar 4.6.1
│   ├── redis 1.7.0
│   │   └── suprnova 4.6.1
│   ├── lettre 0.11.23
│   │   └── suprnova 4.6.1
│   └── hyper-rustls 0.27.9
│       ├── reqwest 0.13.5
│       ├── reqwest 0.12.28
│       └── async-stripe 1.0.0-rc.5
│           ├── suprnova-payments-stripe 4.6.1
│           └── directory 1.1.0
├── suprnova 4.6.1
├── sqlx-core 0.9.0
│   ├── sqlx-sqlite 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   │   └── sqlx-macros 0.9.0
│   │   │       └── sqlx 0.9.0
│   │   │           ├── suprnova 4.6.1
│   │   │           ├── sea-schema 0.18.1
│   │   │           │   ├── sea-orm-migration 2.0.2
│   │   │           │   │   ├── suprnova 4.6.1
│   │   │           │   │   └── directory 1.1.0
│   │   │           │   ├── sea-orm-cli 2.0.2
│   │   │           │   │   └── sea-orm-migration 2.0.2
│   │   │           │   └── sea-orm 2.0.2
│   │   │           │       ├── suprnova-magnetar 4.6.1
│   │   │           │       ├── suprnova 4.6.1
│   │   │           │       ├── sea-orm-migration 2.0.2
│   │   │           │       └── directory 1.1.0
│   │   │           ├── sea-query-sqlx 0.9.1
│   │   │           │   ├── sea-schema 0.18.1
│   │   │           │   └── sea-orm 2.0.2
│   │   │           ├── sea-orm-cli 2.0.2
│   │   │           └── sea-orm 2.0.2
│   │   └── sqlx 0.9.0
│   ├── sqlx-postgres 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   └── sqlx 0.9.0
│   ├── sqlx-mysql 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   └── sqlx 0.9.0
│   ├── sqlx-macros-core 0.9.0
│   ├── sqlx-macros 0.9.0
│   ├── sqlx 0.9.0
│   └── sea-orm 2.0.2
├── sentry 0.49.2
├── rustls-platform-verifier 0.7.0
│   └── reqwest 0.13.5
├── reqwest 0.13.5
├── reqwest 0.12.28
├── redis 1.7.0
├── quinn-proto 0.11.17
│   └── quinn 0.11.11
│       ├── reqwest 0.13.5
│       └── reqwest 0.12.28
├── quinn 0.11.11
├── lettre 0.11.23
├── hyper-rustls 0.27.9
└── directory 1.1.0

Crate:     rustls
Version:   0.23.44
Title:     `Debug` impls for client session values expose TLS secrets
Date:      2026-10-14
ID:        RUSTSEC-2026-0346
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0346
Severity:  4 (medium)
Solution:  Upgrade to >=0.23.46

Crate:     rustls
Version:   0.23.44
Title:     TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries
Date:      2026-09-14
ID:        RUSTSEC-2026-0285
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0285
Severity:  5.3 (medium)
Solution:  Upgrade to >=0.23.45

Crate:     rustls
Version:   0.23.44
Title:     Brotli certificate decompression can allocate up to 1 GiB before authentication
Date:      2026-10-14
ID:        RUSTSEC-2026-0348
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0348
Severity:  3.7 (low)
Solution:  Upgrade to >=0.23.46

Crate:     rustls
Version:   0.23.44
Title:     Buffered application data sent to outer server after ECH rejection
Date:      2026-10-14
ID:        RUSTSEC-2026-0347
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0347
Severity:  2.3 (low)
Solution:  Upgrade to >=0.23.46

Crate:     rustls-webpki
Version:   0.103.15
Title:     Certificate verification may accept distrusted intermediates
Date:      2026-10-20
ID:        RUSTSEC-2026-0382
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0382
Solution:  Upgrade to >=0.103.16, <0.104.0-alpha.1 OR >=0.104.0-alpha.8
Dependency tree:
rustls-webpki 0.103.15
├── rustls-platform-verifier 0.7.0
│   └── reqwest 0.13.5
│       ├── suprnova-web-push 4.6.1
│       │   └── suprnova 4.6.1
│       │       ├── suprnova-payments-stripe 4.6.1
│       │       │   └── directory 1.1.0
│       │       ├── suprnova-payments-paddle 4.6.1
│       │       │   └── directory 1.1.0
│       │       ├── suprnova-markdown 0.5.5
│       │       │   └── directory 1.1.0
│       │       └── directory 1.1.0
│       ├── suprnova-magnetar 4.6.1
│       │   └── suprnova 4.6.1
│       ├── suprnova 4.6.1
│       ├── sentry 0.49.2
│       │   └── suprnova 4.6.1
│       └── qdrant-client 1.19.0
│           └── suprnova 4.6.1
└── rustls 0.23.44
    ├── ureq 3.4.2
    │   └── sentry 0.49.2
    ├── tokio-rustls 0.26.5
    │   ├── tonic 0.14.6
    │   │   ├── tonic-prost 0.14.6
    │   │   │   └── qdrant-client 1.19.0
    │   │   └── qdrant-client 1.19.0
    │   ├── reqwest 0.13.5
    │   ├── reqwest 0.12.28
    │   │   ├── paddle-rust-sdk 0.18.0
    │   │   │   ├── suprnova-payments-paddle 4.6.1
    │   │   │   └── directory 1.1.0
    │   │   └── apple 0.3.1
    │   │       └── suprnova-magnetar 4.6.1
    │   ├── redis 1.7.0
    │   │   └── suprnova 4.6.1
    │   ├── lettre 0.11.23
    │   │   └── suprnova 4.6.1
    │   └── hyper-rustls 0.27.9
    │       ├── reqwest 0.13.5
    │       ├── reqwest 0.12.28
    │       └── async-stripe 1.0.0-rc.5
    │           ├── suprnova-payments-stripe 4.6.1
    │           └── directory 1.1.0
    ├── suprnova 4.6.1
    ├── sqlx-core 0.9.0
    │   ├── sqlx-sqlite 0.9.0
    │   │   ├── sqlx-macros-core 0.9.0
    │   │   │   └── sqlx-macros 0.9.0
    │   │   │       └── sqlx 0.9.0
    │   │   │           ├── suprnova 4.6.1
    │   │   │           ├── sea-schema 0.18.1
    │   │   │           │   ├── sea-orm-migration 2.0.2
    │   │   │           │   │   ├── suprnova 4.6.1
    │   │   │           │   │   └── directory 1.1.0
    │   │   │           │   ├── sea-orm-cli 2.0.2
    │   │   │           │   │   └── sea-orm-migration 2.0.2
    │   │   │           │   └── sea-orm 2.0.2
    │   │   │           │       ├── suprnova-magnetar 4.6.1
    │   │   │           │       ├── suprnova 4.6.1
    │   │   │           │       ├── sea-orm-migration 2.0.2
    │   │   │           │       └── directory 1.1.0
    │   │   │           ├── sea-query-sqlx 0.9.1
    │   │   │           │   ├── sea-schema 0.18.1
    │   │   │           │   └── sea-orm 2.0.2
    │   │   │           ├── sea-orm-cli 2.0.2
    │   │   │           └── sea-orm 2.0.2
    │   │   └── sqlx 0.9.0
    │   ├── sqlx-postgres 0.9.0
    │   │   ├── sqlx-macros-core 0.9.0
    │   │   └── sqlx 0.9.0
    │   ├── sqlx-mysql 0.9.0
    │   │   ├── sqlx-macros-core 0.9.0
    │   │   └── sqlx 0.9.0
    │   ├── sqlx-macros-core 0.9.0
    │   ├── sqlx-macros 0.9.0
    │   ├── sqlx 0.9.0
    │   └── sea-orm 2.0.2
    ├── sentry 0.49.2
    ├── rustls-platform-verifier 0.7.0
    ├── reqwest 0.13.5
    ├── reqwest 0.12.28
    ├── redis 1.7.0
    ├── quinn-proto 0.11.17
    │   └── quinn 0.11.11
    │       ├── reqwest 0.13.5
    │       └── reqwest 0.12.28
    ├── quinn 0.11.11
    ├── lettre 0.11.23
    ├── hyper-rustls 0.27.9
    └── directory 1.1.0

Crate:     webpki-root-certs
Version:   1.0.9
Title:     Bundled root certificates allow trust in distrusted emSign intermediates
Date:      2026-10-19
ID:        RUSTSEC-2026-0383
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0383
Solution:  Upgrade to >=1.0.10
Dependency tree:
webpki-root-certs 1.0.9
└── rustls-platform-verifier 0.7.0
    └── reqwest 0.13.5
        ├── suprnova-web-push 4.6.1
        │   └── suprnova 4.6.1
        │       ├── suprnova-payments-stripe 4.6.1
        │       │   └── directory 1.1.0
        │       ├── suprnova-payments-paddle 4.6.1
        │       │   └── directory 1.1.0
        │       ├── suprnova-markdown 0.5.5
        │       │   └── directory 1.1.0
        │       └── directory 1.1.0
        ├── suprnova-magnetar 4.6.1
        │   └── suprnova 4.6.1
        ├── suprnova 4.6.1
        ├── sentry 0.49.2
        │   └── suprnova 4.6.1
        └── qdrant-client 1.19.0
            └── suprnova 4.6.1

Crate:     webpki-roots
Version:   1.0.9
Title:     Bundled root certificates allow trust in distrusted emSign intermediates
Date:      2026-10-19
ID:        RUSTSEC-2026-0381
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0381
Solution:  Upgrade to >=1.0.10
Dependency tree:
webpki-roots 1.0.9
├── ureq 3.4.2
│   └── sentry 0.49.2
│       └── suprnova 4.6.1
│           ├── suprnova-payments-stripe 4.6.1
│           │   └── directory 1.1.0
│           ├── suprnova-payments-paddle 4.6.1
│           │   └── directory 1.1.0
│           ├── suprnova-markdown 0.5.5
│           │   └── directory 1.1.0
│           └── directory 1.1.0
├── sqlx-core 0.9.0
│   ├── sqlx-sqlite 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   │   └── sqlx-macros 0.9.0
│   │   │       └── sqlx 0.9.0
│   │   │           ├── suprnova 4.6.1
│   │   │           ├── sea-schema 0.18.1
│   │   │           │   ├── sea-orm-migration 2.0.2
│   │   │           │   │   ├── suprnova 4.6.1
│   │   │           │   │   └── directory 1.1.0
│   │   │           │   ├── sea-orm-cli 2.0.2
│   │   │           │   │   └── sea-orm-migration 2.0.2
│   │   │           │   └── sea-orm 2.0.2
│   │   │           │       ├── suprnova-magnetar 4.6.1
│   │   │           │       │   └── suprnova 4.6.1
│   │   │           │       ├── suprnova 4.6.1
│   │   │           │       ├── sea-orm-migration 2.0.2
│   │   │           │       └── directory 1.1.0
│   │   │           ├── sea-query-sqlx 0.9.1
│   │   │           │   ├── sea-schema 0.18.1
│   │   │           │   └── sea-orm 2.0.2
│   │   │           ├── sea-orm-cli 2.0.2
│   │   │           └── sea-orm 2.0.2
│   │   └── sqlx 0.9.0
│   ├── sqlx-postgres 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   └── sqlx 0.9.0
│   ├── sqlx-mysql 0.9.0
│   │   ├── sqlx-macros-core 0.9.0
│   │   └── sqlx 0.9.0
│   ├── sqlx-macros-core 0.9.0
│   ├── sqlx-macros 0.9.0
│   ├── sqlx 0.9.0
│   └── sea-orm 2.0.2
├── reqwest 0.12.28
│   ├── paddle-rust-sdk 0.18.0
│   │   ├── suprnova-payments-paddle 4.6.1
│   │   └── directory 1.1.0
│   └── apple 0.3.1
│       └── suprnova-magnetar 4.6.1
├── lettre 0.11.23
│   └── suprnova 4.6.1
└── hyper-rustls 0.27.9
    ├── reqwest 0.13.5
    │   ├── suprnova-web-push 4.6.1
    │   │   └── suprnova 4.6.1
    │   ├── suprnova-magnetar 4.6.1
    │   ├── suprnova 4.6.1
    │   ├── sentry 0.49.2
    │   └── qdrant-client 1.19.0
    │       └── suprnova 4.6.1
    ├── reqwest 0.12.28
    └── async-stripe 1.0.0-rc.5
        ├── suprnova-payments-stripe 4.6.1
        └── directory 1.1.0

Crate:     basic-toml
Version:   0.1.10
Warning:   unmaintained
Title:     `basic-toml` is unmaintained
Date:      2026-09-26
ID:        RUSTSEC-2026-0374
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0374
Dependency tree:
basic-toml 0.1.10
└── askama_derive 0.16.0
    └── askama_macros 0.16.0
        └── askama 0.16.0
            ├── suprnova-live 4.6.1
            │   └── suprnova 4.6.1
            │       ├── suprnova-payments-stripe 4.6.1
            │       │   └── directory 1.1.0
            │       ├── suprnova-payments-paddle 4.6.1
            │       │   └── directory 1.1.0
            │       ├── suprnova-markdown 0.5.5
            │       │   └── directory 1.1.0
            │       └── directory 1.1.0
            └── suprnova 4.6.1

Crate:     bincode
Version:   1.3.3
Warning:   unmaintained
Title:     Bincode is unmaintained
Date:      2025-12-16
ID:        RUSTSEC-2025-0141
URL:       https://rustsec.org/advisories/RUSTSEC-2025-0141
Dependency tree:
bincode 1.3.3
└── syntect 5.3.0
    └── comrak 0.52.0
        └── suprnova 4.6.1
            ├── suprnova-payments-stripe 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-payments-paddle 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-markdown 0.5.5
            │   └── directory 1.1.0
            └── directory 1.1.0

Crate:     paste
Version:   1.0.15
Warning:   unmaintained
Title:     paste - no longer maintained
Date:      2024-10-07
ID:        RUSTSEC-2024-0436
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
Dependency tree:
paste 1.0.15
└── tokenizers 0.23.2
    └── suprnova 4.6.1
        ├── suprnova-payments-stripe 4.6.1
        │   └── directory 1.1.0
        ├── suprnova-payments-paddle 4.6.1
        │   └── directory 1.1.0
        ├── suprnova-markdown 0.5.5
        │   └── directory 1.1.0
        └── directory 1.1.0

Crate:     ttf-parser
Version:   0.25.1
Warning:   unmaintained
Title:     `ttf-parser` is unmaintained
Date:      2026-06-28
ID:        RUSTSEC-2026-0192
URL:       https://rustsec.org/advisories/RUSTSEC-2026-0192
Dependency tree:
ttf-parser 0.25.1
└── fontdue 0.9.4
    └── charts-rs 1.0.0
        └── suprnova-live 4.6.1
            └── suprnova 4.6.1
                ├── suprnova-payments-stripe 4.6.1
                │   └── directory 1.1.0
                ├── suprnova-payments-paddle 4.6.1
                │   └── directory 1.1.0
                ├── suprnova-markdown 0.5.5
                │   └── directory 1.1.0
                └── directory 1.1.0

Crate:     yaml-rust
Version:   0.4.5
Warning:   unmaintained
Title:     yaml-rust is unmaintained.
Date:      2024-03-20
ID:        RUSTSEC-2024-0320
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0320
Dependency tree:
yaml-rust 0.4.5
└── syntect 5.3.0
    └── comrak 0.52.0
        └── suprnova 4.6.1
            ├── suprnova-payments-stripe 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-payments-paddle 4.6.1
            │   └── directory 1.1.0
            ├── suprnova-markdown 0.5.5
            │   └── directory 1.1.0
            └── directory 1.1.0

error: 14 vulnerabilities found!
warning: 5 allowed warnings found
```

### Report consistency check: initial failure

The first `python3 .upgrade-work/verify-report-461.py` invocation exits 1:

```text
  File "/home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/verify-report-461.py", line 26, in <module>
    assert 'error:' not in log, name
           ^^^^^^^^^^^^^^^^^^^
AssertionError: 461-final-test
```

The initial heuristic matched `tcp connect error:` inside the documented
private-form SSR fallback diagnostics. The suite itself exited zero. The
consistency check now looks for compiler error lines at the start of a line and
still requires the recorded test exit code to be zero. Its corrected invocation
passes; no application test or acceptance criterion was changed for this check.

### `461-ripwire-test-gate`: exit 4

```text
<!-- ripwire test-gate schema=ripwire.test-gate/v1: tests <t p= changed= partner= hops= run=> + untested blast radius <u sym= p= l= ccx=>; exit 4 while either exists. window: total= has_more= next_offset= offset= limit= (capped=1 cut; next_offset= pastes as offset=). tests_capped=/untested_capped=: 1 = cut. counts_floor=1: every count is a FLOOR, never a total. graph_ambiguous=/graph_unresolved=: resolver gauge. graph_unindexed=N: N files no grammar could read (the map header's unindexed=); their calls raise neither gauge. declined_calls=K: K call sites left unbound (no evidence chose one def), in no count or row. run_unknown=1: no runner derivable (a guess would be worse). <g n= p=a,b,c>: n runner-less rows with equal attrs as ONE row, paths verbatim; a path holding ',' is never grouped, so p= splits into exactly n=; shown=/total= over these rows counts test FILES. at=: commit+dirty+shallow. root=: p= relative to it. next=: the one pasteable follow-up. impacted=N: symbols that transitively call the change (changed symbols excluded). shown_tests=/shown_untested=: t rows and u rows printed, two independent counts. script_gates_unmodelled=N: test/*.sh runners in the corpus, a path count; not call-graph modelled. script_gates_registered=N: shell gates test/regression.sh registers as suite members. script_gates_mapped=N: registered gates with exact dependency evidence (literal paths or RIPWIRE_TEST_DEPS). script_gates_unresolved_dynamic=N: registered gates with no mappable deps; they may cover the change unlisted. ccx_bar=N: the cognitive-complexity bar a u row's ccx= is read against. untested_modscope=N: <file-scope> owners excluded from untested= (#324, uncallable); still in impacted=. tests=/untested=: tests to run (t total) / impacted symbols no test reaches (u total). --><test-gate schema="ripwire.test-gate/v1" changed="49" impacted="175" tests="21" untested="84" untested_modscope="6" shown_tests="21" tests_capped="0" shown_untested="25" untested_capped="1" script_gates_unmodelled="0" script_gates_registered="0" script_gates_mapped="0" script_gates_unresolved_dynamic="0" ccx_bar="15" graph_ambiguous="130" graph_unresolved="443" graph_unindexed="53" counts_floor="1" total="84" has_more="1" next_offset="25" offset="0" limit="0" at="99808b766+dirty" root="." declined_calls="172" next="--callers=save"><g changed="1" n="11" p="frontend/tests/directory-workflows.mjs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs" run_unknown="1"/><g hops="1" n="10" p="tests/administration_workflows.rs,tests/adoption_workflows.rs,tests/common/paid_browser.rs,tests/directory_workflows.rs,tests/editorial_workflows.rs,tests/fixtures/m20260909_000009_create_account_access_3_2_1.rs,tests/overview_workflows.rs,tests/paid_lifecycle.rs,tests/provider_administration.rs,tests/seo_workflows.rs" run_unknown="1"/><u sym="save" p="src/articles/taxonomy.rs" l="99" ccx="34"/><u sym="offers" p="src/billing/checkout.rs" l="132" ccx="26"/><u sym="load" p="src/seo/report.rs" l="58" ccx="23"/><u sym="save" p="src/seo/redirects.rs" l="127" ccx="13"/><u sym="remove" p="src/articles/taxonomy.rs" l="277" ccx="12"/><u sym="up" p="src/migrations/m20260909_000011_create_owner_notifications.rs" l="10" ccx="11"/><u sym="save" p="src/accounts/mod.rs" l="114" ccx="9"/><u sym="run" p="src/commands/billing_reconcile.rs" l="34" ccx="7"/><u sym="request" p="src/billing/cancellation.rs" l="7" ccx="6"/><u sym="index" p="src/controllers/admin.rs" l="73" ccx="6"/><u sym="update" p="src/controllers/accounts.rs" l="53" ccx="4"/><u sym="search" p="src/accounts/queries.rs" l="127" ccx="3"/><u sym="upload" p="src/controllers/articles.rs" l="203" ccx="3"/><u sym="upload" p="src/controllers/listings.rs" l="245" ccx="3"/><u sym="up" p="src/migrations/m20260909_000006_create_directory.rs" l="13" ccx="3"/><u sym="admin_search" p="src/articles/queries.rs" l="229" ccx="2"/><u sym="approved_listing" p="src/billing/checkout.rs" l="99" ccx="2"/><u sym="webhook" p="src/controllers/publishing.rs" l="126" ccx="2"/><u sym="remove" p="src/seo/redirects.rs" l="216" ccx="2"/><u sym="list" p="src/articles/taxonomy.rs" l="50" ccx="1"/><u sym="main" p="src/bin/console.rs" l="26" ccx="1"/><u sym="public_media" p="src/controllers/articles.rs" l="247" ccx="1"/><u sym="show" p="src/controllers/articles.rs" l="97" ccx="1"/><u sym="private_media" p="src/controllers/listings.rs" l="297" ccx="1"/><u sym="public_media" p="src/controllers/listings.rs" l="313" ccx="1"/></test-gate>
```

### `461-ripwire-test-gate-complete`: exit 4

```text
<!-- ripwire test-gate schema=ripwire.test-gate/v1: tests <t p= changed= partner= hops= run=> + untested blast radius <u sym= p= l= ccx=>; exit 4 while either exists. window: total= has_more= next_offset= offset= limit= (capped=1 cut; next_offset= pastes as offset=). tests_capped=/untested_capped=: 1 = cut. counts_floor=1: every count is a FLOOR, never a total. graph_ambiguous=/graph_unresolved=: resolver gauge. graph_unindexed=N: N files no grammar could read (the map header's unindexed=); their calls raise neither gauge. declined_calls=K: K call sites left unbound (no evidence chose one def), in no count or row. run_unknown=1: no runner derivable (a guess would be worse). <g n= p=a,b,c>: n runner-less rows with equal attrs as ONE row, paths verbatim; a path holding ',' is never grouped, so p= splits into exactly n=; shown=/total= over these rows counts test FILES. at=: commit+dirty+shallow. root=: p= relative to it. next=: the one pasteable follow-up. impacted=N: symbols that transitively call the change (changed symbols excluded). shown_tests=/shown_untested=: t rows and u rows printed, two independent counts. script_gates_unmodelled=N: test/*.sh runners in the corpus, a path count; not call-graph modelled. script_gates_registered=N: shell gates test/regression.sh registers as suite members. script_gates_mapped=N: registered gates with exact dependency evidence (literal paths or RIPWIRE_TEST_DEPS). script_gates_unresolved_dynamic=N: registered gates with no mappable deps; they may cover the change unlisted. ccx_bar=N: the cognitive-complexity bar a u row's ccx= is read against. untested_modscope=N: <file-scope> owners excluded from untested= (#324, uncallable); still in impacted=. tests=/untested=: tests to run (t total) / impacted symbols no test reaches (u total). --><test-gate schema="ripwire.test-gate/v1" changed="49" impacted="175" tests="21" untested="84" untested_modscope="6" shown_tests="21" tests_capped="0" shown_untested="25" untested_capped="1" script_gates_unmodelled="0" script_gates_registered="0" script_gates_mapped="0" script_gates_unresolved_dynamic="0" ccx_bar="15" graph_ambiguous="130" graph_unresolved="443" graph_unindexed="53" counts_floor="1" total="84" has_more="1" next_offset="25" offset="0" limit="0" at="4e08adf11+dirty" root="." declined_calls="172" next="--callers=save"><g changed="1" n="11" p="frontend/tests/directory-workflows.mjs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs" run_unknown="1"/><g hops="1" n="10" p="tests/administration_workflows.rs,tests/adoption_workflows.rs,tests/common/paid_browser.rs,tests/directory_workflows.rs,tests/editorial_workflows.rs,tests/fixtures/m20260909_000009_create_account_access_3_2_1.rs,tests/overview_workflows.rs,tests/paid_lifecycle.rs,tests/provider_administration.rs,tests/seo_workflows.rs" run_unknown="1"/><u sym="save" p="src/articles/taxonomy.rs" l="99" ccx="34"/><u sym="offers" p="src/billing/checkout.rs" l="132" ccx="26"/><u sym="load" p="src/seo/report.rs" l="58" ccx="23"/><u sym="save" p="src/seo/redirects.rs" l="127" ccx="13"/><u sym="remove" p="src/articles/taxonomy.rs" l="277" ccx="12"/><u sym="up" p="src/migrations/m20260909_000011_create_owner_notifications.rs" l="10" ccx="11"/><u sym="save" p="src/accounts/mod.rs" l="114" ccx="9"/><u sym="run" p="src/commands/billing_reconcile.rs" l="34" ccx="7"/><u sym="request" p="src/billing/cancellation.rs" l="7" ccx="6"/><u sym="index" p="src/controllers/admin.rs" l="73" ccx="6"/><u sym="update" p="src/controllers/accounts.rs" l="53" ccx="4"/><u sym="search" p="src/accounts/queries.rs" l="127" ccx="3"/><u sym="upload" p="src/controllers/articles.rs" l="203" ccx="3"/><u sym="upload" p="src/controllers/listings.rs" l="245" ccx="3"/><u sym="up" p="src/migrations/m20260909_000006_create_directory.rs" l="13" ccx="3"/><u sym="admin_search" p="src/articles/queries.rs" l="229" ccx="2"/><u sym="approved_listing" p="src/billing/checkout.rs" l="99" ccx="2"/><u sym="webhook" p="src/controllers/publishing.rs" l="126" ccx="2"/><u sym="remove" p="src/seo/redirects.rs" l="216" ccx="2"/><u sym="list" p="src/articles/taxonomy.rs" l="50" ccx="1"/><u sym="main" p="src/bin/console.rs" l="26" ccx="1"/><u sym="public_media" p="src/controllers/articles.rs" l="247" ccx="1"/><u sym="show" p="src/controllers/articles.rs" l="97" ccx="1"/><u sym="private_media" p="src/controllers/listings.rs" l="297" ccx="1"/><u sym="public_media" p="src/controllers/listings.rs" l="313" ccx="1"/></test-gate>
```

The type generator emits the same three metadata diagnostics for `Mode`,
`CheckoutView` and `Provider`: those enum/external shapes become `unknown` in
generated metadata. Existing explicit frontend types retain their precise
application contracts, and the final frontend check passes. Some private-form
SSR fallback diagnostics name the fixture's default port; public SSR content
is served by the configured worker and verified without JavaScript.

Vite reports that its configured `public/assets` output is outside the frontend
root and will not be emptied automatically. This is the existing application
layout; verification workspaces start without generated assets and final builds
complete successfully. No stale asset was treated as a new build artifact.

Clippy's one starter warning was `needless_borrows_for_generic_args` at
`src/migrations/m20260909_000007_create_publishing.rs:205`. The publishing-events
`raw_body` column now passes its owned `ColumnDef` directly to `.col(...)`.
This follows the exact suggestion without changing the column definition or
adding an allow attribute. The follow-up commands above verify compilation,
migration preservation and payment ingestion after the correction.

Ripwire's test gate is a structural test-selection report. Exit 4 means it names
tests or unresolved reach, rather than executing tests. Runtime evidence comes
from the Rust, browser and SDK checks above. The quality delta reports existing
symbol regressions separately; new-symbol notes are not hidden by acknowledgments.

## Upgrade chapter decisions

The 4.4, 4.5 and 4.6 chapters were read in order, with the 4.6.1 additions read
again for the final target. `docs/spec/framework-upgrade.md` was read without
editing its agreed text. The user's requested release pins supersede its old
4.3 release pins; its migration, account, payment, permission and operational
acceptance boundaries remain covered by the repository's six mechanisms.

- Framework default features remain enabled, so 4.4's agents and search-media
  feature-list edits do not apply. No custom default-feature list is maintained.
- OAuth, agent accounts, outside issuers, consent pages, WebAssembly tools and
  sample notes tools remain opt-in. The MCP wrapper exposes no tools or identity
  resolver. No new signing key or application OAuth/agent migrations are needed.
  The reserved notes topic keeps its owner-only rule in `live::authorize()`,
  called by both bootstrap and the page budget test.
- No constructed lifecycle-event structs, exhaustive changed-enum matches,
  custom Magnetar engine, TokenStore or graph/search provider need conversion.
  The account wrapper forwards the new atomic credential-rehash hook.
- Magnetar's address-digest column and default-feature auth tables migrate at
  framework boot. MySQL/MariaDB table widening is not exercised by SQLite checks;
  the guide describes the single-instance quiet window. No PostgreSQL service
  was used in local verification, so PostgreSQL behavior is not claimed verified.
- Framework search, embeddings and recommendations are not used. Directory and
  article queries use SQL. No framework search generation needs reindexing and
  no recommendation digest or list needs migration. Stream helper fixes are
  included; the kit's demo notes hit-cap change has no corresponding handler.
- No application route declares an Inertia cache policy. The ordinary opaque
  share provider is valid at boot. The guide explains named shares for cacheable
  forks, including per-request auth, locale, errors and flash values.
- Image quality 85 is accepted; no pipeline intentionally depended on 70. Stored
  uploads are not rewritten. Media fixtures exercise new encodes. No application
  after-commit callback or Live refresh timing assertion needs conversion.
- Static publication, event sourcing and CDN deployment remain optional and
  unused. CloudFront stays enabled by the framework defaults with no provisioned
  publication or distribution.
- The 4.6.1 kit router and shared notes gate fixes are already included in this
  starter, and generated Rust passes formatting. The Live action debug-stack fix
  comes from the pinned framework; it requires no application code change or
  added migration. The application budget audits all seven public entry pages
  through the same Live/RenderCache wrappers as the served router.
- The earlier restriction on security dependency updates is superseded by the
  explicit 4.6.1 allowance. All 14 vulnerabilities are fixed without ignore entries.
- Live charges, real SMTP, Redis persistence/outage recovery, cloud storage and
  production TLS were not tested. Disposable SQLite, provider fixtures, actual
  browser requests, SSR and pinned upstream SDK tests are the local evidence.

## Earlier upgrade attempts

These failures occurred while implementing the earlier framework target. Their
corrections are included in the final tree. The following prior-stage record is
historical; it does not describe the final 4.6.1 audit or Clippy result.
Every unsuccessful build or verification attempt is retained here, including
test-fixture and tool-invocation mistakes. Later successful results do not erase
these failures.

- Intermediate `bun run --cwd frontend build`, exit 1: Vite reported
  `Invalid type: Expected Function but received Object`, followed by
  `TypeError: manualChunks is not a function`. The chunk option was corrected to
  the function form, then the type check and both builds passed.
- `framework-auth`, exit 1: `upgrade_preserves_authentication_boundaries` failed
  with `left: 500`, `right: 302` during the new registration exercise. The suite
  had discarded the RAII fake-mail guard. It now holds that guard until completion.
- `framework-auth-retry`, exit 1: `left: 302`, `right: 409`. The new test expected
  Inertia's status on an ordinary HTTP request. It now asserts the 302 destination
  for ordinary requests and the 409 `X-Inertia-Location` for Inertia requests.
- `framework-auth-final`, exit 1: `left: 422`, `right: 302`. The new validation
  assertions mixed direct handler errors with FormRequest redirects. They now use
  the application's Inertia form protocol consistently and assert both 303
  destinations and the exact field errors. `framework-auth-validated` passed.
- `framework-http`, exit 1: `expect(locator).toBeVisible() failed`, for
  `[data-shell="admin"]`. Lazy admin CSS was requested at `/assets/Overview-...css`
  while built files lived under `/assets/assets/`. The kit's `base: './'` fixes
  chunk URLs. The repeated HTTP task and browser navigation passed.
- `framework-regression`, exit 1: the directory browser test expected
  `/listings?q=Pagination&category=software` but received
  `/listings?category=software&q=Pagination`. Both filtering values and results
  were correct. The assertion now checks the origin, path, parameter count and
  values, following the earlier assertion in that same test.
- Initial Ripwire quality delta, exit 2: it reported two existing-test regressions,
  migration-test complexity `was=9 now=20`, and cache-test size `was=50 now=68`.
  Named assertion helpers now keep those concerns separate. The reviewed range
  check exited zero with no existing-symbol regressions.
- `ripwire-quality-release`, exit 2: the tuple-based request cleanup made the
  existing CSRF and signature test wrappers a 40-token clone group. The CSRF
  helper now reuses the existing explicit-header helper. All-target compilation,
  auth and SDK wire checks passed again; `ripwire-quality-complete` exits zero.
- A preliminary report-completeness assertion failed with `AssertionError` because
  it rejected the words `not run` anywhere, including the deliberate explanation
  of the four-package restriction. The corrected check inspects result-table cells;
  no verification result was missing.
- Initial Ripwire test-gate invocation, exit 1:
  `'Cargo.lock' matches no indexed file`. The retry supplied only indexed source
  and test paths.

The filtered Ripwire test gate exits 4 by design when it names tests or unresolved
reach. The final range named 21 test files and 84 structural reach entries. Those entries
include macro-registered handlers, webhook wrappers, console main and a registered
unit test. The actual Rust, HTTP, browser and SDK verifiers exercise these paths;
the structural tool does not consume their execution receipts. This exit is
reported, not presented as a test pass. Its graph counts are lower bounds.

The reviewed quality delta exits zero with no existing-symbol regressions. It
also reports new-symbol notes, mostly the copied kit algorithms, exported helpers
and indirect trait callbacks. No quality acknowledgments or suppressions were
added to hide those notes. The source and license of the kit copies are recorded
in `THIRD_PARTY_NOTICES.md`.

The 4.6 type generator exited zero and emitted three diagnostics: `Mode`,
`CheckoutView` and `Provider` become `unknown` in its generated metadata because
it does not resolve those enum/external shapes. The application retains its
existing explicit frontend types for them, and frontend type checks pass.
Private-form SSR fallback messages target port 13714; public SSR is explicitly
configured to the test worker and verified without JavaScript.


## Earlier recorded commands

This historical invocation list preserves commands run before the target
changed. The 4.6.1 table above is the final verification record.

Exit codes below include expected nonzero structural guidance and earlier failed
attempts. Commands run inside the verifiers are listed in the following appendix.
Read-only source discovery is not treated as a build or test invocation.

| Name | Command | Exit |
| --- | --- | --- |
| `frontend-check` | `bun run --cwd frontend check` | 0 |
| `frontend-frozen` | `bun install --frozen-lockfile --cwd frontend` | 0 |
| `format-source` | `cargo fmt` | 0 |
| `upgrade-guide` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `frontend-version-lock` | `bun install --cwd frontend` | 0 |
| `generate-types` | `/home/shawn/workspace2/scratchpads/targets/suprnova/main/debug/suprnova generate-types` | 0 |
| `frontend-generated-check` | `bun run --cwd frontend check` | 0 |
| `upgrade-guide-after-types` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `framework-build` | `node scripts/verify-framework-upgrade.mjs build` | 0 |
| `format-source-upgrade` | `cargo fmt` | 0 |
| `framework-migration` | `node scripts/verify-framework-upgrade.mjs migration` | 0 |
| `upgrade-guide-final-source` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `framework-auth` | `node scripts/verify-framework-upgrade.mjs auth` | 1 |
| `framework-auth-retry` | `node scripts/verify-framework-upgrade.mjs auth` | 1 |
| `format-auth-fixture` | `cargo fmt` | 0 |
| `framework-auth-final` | `node scripts/verify-framework-upgrade.mjs auth` | 1 |
| `framework-auth-validated` | `node scripts/verify-framework-upgrade.mjs auth` | 0 |
| `framework-http` | `node scripts/verify-framework-upgrade.mjs http` | 1 |
| `framework-http-relative-assets` | `node scripts/verify-framework-upgrade.mjs http` | 0 |
| `framework-console` | `node scripts/verify-framework-upgrade.mjs console` | 0 |
| `upgrade-guide-reviewed` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `ripwire-quality` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 2 |
| `framework-regression` | `node scripts/verify-framework-upgrade.mjs regression` | 1 |
| `format-test-assertions` | `cargo fmt` | 0 |
| `ripwire-quality-reviewed` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 0 |
| `ripwire-test-gate-reviewed` | `ripwire . --test-gate=cmd/main.rs,frontend/src/lib/agents/core.ts,frontend/src/lib/agents/index.ts,frontend/src/lib/agents/inertia.ts,frontend/src/lib/static/core.ts,frontend/src/lib/static/index.ts,frontend/src/lib/stream/core.ts,frontend/src/lib/stream/index.ts,frontend/src/main.ts,frontend/src/resolve.ts,frontend/vite.config.ts,scripts/verification-workspace.mjs,scripts/verify-appearance.mjs,scripts/verify-complete-starter.mjs,scripts/verify-foundation.mjs,scripts/verify-framework-upgrade.mjs,scripts/verify-paid-directory.mjs,scripts/verify-pinned-checkout.mjs,scripts/verify-provider-administration.mjs,scripts/verify-upgrade-guide.mjs,src/accounts/provider.rs,src/agents.rs,src/bootstrap.rs,src/controllers/account_links.rs,src/controllers/auth.rs,src/email_address.rs,src/lib.rs,src/live/mod.rs,src/middleware/authenticate.rs,src/migrations/mod.rs,src/models/user.rs,src/public_pages.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/provider_key_policy.rs --legend=compact` | 4 |
| `edit-check-provider` | `ripwire . --edit-check=AccountUserProvider --legend=compact` | 0 |
| `edit-check-resolver` | `ripwire . --edit-check=resolvePage --legend=compact` | 0 |
| `ripwire-quality-final-source` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 0 |
| `reviewed-source-check` | `cargo check --locked --all-targets` | 0 |
| `framework-regression-query-parameters` | `node scripts/verify-framework-upgrade.mjs regression` | 0 |
| `final-frontend-install` | `bun install --frozen-lockfile --cwd frontend` | 0 |
| `final-frontend-check` | `bun run --cwd frontend check` | 0 |
| `final-frontend-build` | `bun run --cwd frontend build` | 0 |
| `final-frontend-ssr` | `bun run --cwd frontend build:ssr` | 0 |
| `final-migration-4.2` | `cargo test --locked --test framework_upgrade_migration` | 0 |
| `final-format` | `cargo fmt --check` | 0 |
| `final-test` | `node scripts/with-ssr.mjs cargo test --locked` | 0 |
| `final-clippy` | `cargo clippy --locked --all-targets` | 0 |
| `final-audit` | `cargo audit` | 1 |
| `format-clippy-fixes` | `cargo fmt` | 0 |
| `check-clippy-fixes` | `cargo check --locked --all-targets` | 0 |
| `post-clippy-format` | `cargo fmt --check` | 0 |
| `post-clippy-check` | `cargo check --locked --all-targets` | 0 |
| `upgrade-guide-audit-status` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `post-clippy-affected-tests` | `node scripts/with-ssr.mjs cargo test --locked --lib --test framework_upgrade_migration --test directory_workflows --test editorial_workflows --test paid_lifecycle --test payment_sdk_wire --test page_budget --test framework_upgrade_auth` | 0 |
| `post-clippy-migration-4.2` | `cargo test --locked --test framework_upgrade_migration` | 0 |
| `post-clippy-paid-live` | `node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle` | 0 |
| `ripwire-quality-release` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 2 |
| `ripwire-test-gate-release` | `ripwire . --test-gate=cmd/main.rs,frontend/src/lib/agents/core.ts,frontend/src/lib/agents/index.ts,frontend/src/lib/agents/inertia.ts,frontend/src/lib/static/core.ts,frontend/src/lib/static/index.ts,frontend/src/lib/stream/core.ts,frontend/src/lib/stream/index.ts,frontend/src/main.ts,frontend/src/resolve.ts,frontend/tests/directory-workflows.mjs,frontend/vite.config.ts,scripts/verification-workspace.mjs,scripts/verify-appearance.mjs,scripts/verify-complete-starter.mjs,scripts/verify-foundation.mjs,scripts/verify-framework-upgrade.mjs,scripts/verify-paid-directory.mjs,scripts/verify-pinned-checkout.mjs,scripts/verify-provider-administration.mjs,scripts/verify-upgrade-guide.mjs,src/accounts/provider.rs,src/agents.rs,src/articles/workflow.rs,src/billing/events.rs,src/bootstrap.rs,src/controllers/account_links.rs,src/controllers/auth.rs,src/email_address.rs,src/lib.rs,src/listings/media.rs,src/listings/workflow.rs,src/live/mod.rs,src/middleware/authenticate.rs,src/migrations/m20260909_000007_create_publishing.rs,src/migrations/m20260909_000008_create_articles.rs,src/migrations/mod.rs,src/models/user.rs,src/public_pages.rs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs --legend=compact` | 4 |
| `format-shared-header` | `cargo fmt` | 0 |
| `post-shared-header-check` | `cargo check --locked --all-targets` | 0 |
| `post-shared-header-auth` | `node scripts/with-ssr.mjs cargo test --locked --test framework_upgrade_auth --test payment_sdk_wire` | 0 |
| `ripwire-quality-complete` | `ripwire . --quality-delta=main..HEAD --legend=compact` | 0 |
| `ripwire-test-gate-complete` | `ripwire . --test-gate=cmd/main.rs,frontend/src/lib/agents/core.ts,frontend/src/lib/agents/index.ts,frontend/src/lib/agents/inertia.ts,frontend/src/lib/static/core.ts,frontend/src/lib/static/index.ts,frontend/src/lib/stream/core.ts,frontend/src/lib/stream/index.ts,frontend/src/main.ts,frontend/src/resolve.ts,frontend/tests/directory-workflows.mjs,frontend/vite.config.ts,scripts/verification-workspace.mjs,scripts/verify-appearance.mjs,scripts/verify-complete-starter.mjs,scripts/verify-foundation.mjs,scripts/verify-framework-upgrade.mjs,scripts/verify-paid-directory.mjs,scripts/verify-pinned-checkout.mjs,scripts/verify-provider-administration.mjs,scripts/verify-upgrade-guide.mjs,src/accounts/provider.rs,src/agents.rs,src/articles/workflow.rs,src/billing/events.rs,src/bootstrap.rs,src/controllers/account_links.rs,src/controllers/auth.rs,src/email_address.rs,src/lib.rs,src/listings/media.rs,src/listings/workflow.rs,src/live/mod.rs,src/middleware/authenticate.rs,src/migrations/m20260909_000007_create_publishing.rs,src/migrations/m20260909_000008_create_articles.rs,src/migrations/mod.rs,src/models/user.rs,src/public_pages.rs,tests/common/mod.rs,tests/demo_seed.rs,tests/foundation_accounts.rs,tests/foundation_ui.rs,tests/framework_upgrade_auth.rs,tests/framework_upgrade_cache.rs,tests/framework_upgrade_migration.rs,tests/page_budget.rs,tests/payment_sdk_wire.rs,tests/provider_key_policy.rs --legend=compact` | 4 |
| `release-format` | `cargo fmt --check` | 0 |
| `release-upgrade-guide` | `node scripts/verify-upgrade-guide.mjs` | 0 |
| `report-release-consistency` | `python3 .upgrade-work/verify-report.py` | 0 |

Additional syntax/whitespace checks ran successfully:

```sh
for file in scripts/verify-*.mjs scripts/verification-workspace.mjs; do node --check "$file" || exit; done
node --check frontend/tests/directory-workflows.mjs
git diff --check
```

## Commands launched by repository verifiers

The following is the ordered command transcript, filtered to command lines only.
A zero parent result means its assertions accepted each child result, including
intentional console refusals (exit 1) and the `compat:exit` fixture (exit 23).
Opaque paths below are disposable test paths, not operator data.

```text
[framework-build]
[framework-build] node scripts/verify-foundation.mjs build
[build] cargo metadata --locked --format-version 1
[build] cargo build --locked --bins
[build] bun install --frozen-lockfile
[build] bun run build
[build] bun run build:ssr
[framework-migration]
[framework-migration] cargo test --locked --test framework_upgrade_migration -- --nocapture
[framework-migration] cargo test --locked --test framework_upgrade_migration -- --nocapture
[framework-migration] node scripts/verify-foundation.mjs database
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-jzOpml/database.db --seed-probe
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-jzOpml/database.db
[framework-auth-validated]
[framework-auth] cargo test --locked --test framework_upgrade_auth -- --nocapture
[framework-auth] node scripts/verify-foundation.mjs accounts
[accounts] cargo test --locked --test foundation_accounts -- --nocapture
[framework-http-relative-assets]
[framework-http] node scripts/verify-foundation.mjs ui
[ui] bun install --frozen-lockfile
[ui] bun run build
[ui] bun run build:ssr
[ui] cargo test --locked --test foundation_ui -- --nocapture
[ui] node scripts/with-ssr.mjs node frontend/tests/foundation-ui.mjs
[framework-http] bun install --frozen-lockfile --cwd frontend
[framework-http] bun run --cwd frontend build
[framework-http] bun run --cwd frontend build:ssr
[framework-http] cargo build --locked --bins
[framework-http] cargo test --locked --test framework_upgrade_cache -- --nocapture
[framework-http] node scripts/with-ssr.mjs cargo test --locked --test page_budget -- --nocapture
[framework-http] node scripts/with-ssr.mjs node frontend/tests/framework-upgrade.mjs
[framework-console]
[framework-console] cargo build --locked --bin console
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console --help
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console --version
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console not-a-real-command
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access --not-a-real-option
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console compat:exit
[framework-regression-query-parameters]
[framework-regression] node scripts/verify-foundation.mjs build
[build] cargo metadata --locked --format-version 1
[build] cargo build --locked --bins
[build] bun install --frozen-lockfile
[build] bun run build
[build] bun run build:ssr
[framework-regression] node scripts/verify-foundation.mjs database
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-mzQTgv/database.db --seed-probe
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-mzQTgv/database.db
[framework-regression] node scripts/verify-foundation.mjs accounts
[accounts] cargo test --locked --test foundation_accounts -- --nocapture
[framework-regression] node scripts/verify-foundation.mjs ui
[ui] bun install --frozen-lockfile
[ui] bun run build
[ui] bun run build:ssr
[ui] cargo test --locked --test foundation_ui -- --nocapture
[ui] node scripts/with-ssr.mjs node frontend/tests/foundation-ui.mjs
[framework-regression] node scripts/verify-foundation.mjs setup
[setup] node frontend/tests/foundation-setup.mjs
[framework-regression] node scripts/verify-provider-administration.mjs
[providers] format: cargo fmt --check
[providers] build: cargo build --locked --bins
[providers] HTTP and persistence contract: cargo test --locked --test provider_administration -- --nocapture
[providers] first administrator grant: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 2
[providers] idempotent administrator grant: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 2
[providers] unknown administrator rejected: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 9223372036854775807
[providers] restart with the original key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] missing deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] default deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] wrong deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] constructor failure and corrected case: cargo test --locked --lib billing::settings::tests::constructor_failure_preserves_configuration -- --nocapture
[providers] frontend install: bun install --frozen-lockfile --cwd frontend
[providers] frontend build: bun run --cwd frontend build
[providers] frontend SSR build: bun run --cwd frontend build:ssr
[providers] browser administration journeys: node scripts/with-ssr.mjs node frontend/tests/provider-administration.mjs
[framework-regression] node scripts/verify-paid-directory.mjs directory
[directory] cargo fmt --check
[directory] cargo build --locked --bins
[directory] bun install --frozen-lockfile --cwd frontend
[directory] bun run --cwd frontend build
[directory] bun run --cwd frontend build:ssr
[directory] cargo test --locked --lib listings::media::tests -- --nocapture
[directory] node scripts/with-ssr.mjs cargo test --locked --test directory_workflows -- --nocapture
[directory] node scripts/with-ssr.mjs node frontend/tests/directory-workflows.mjs
[framework-regression] node scripts/verify-paid-directory.mjs payments
[payments] cargo fmt --check
[payments] cargo build --locked --bins
[payments] bun install --frozen-lockfile --cwd frontend
[payments] bun run --cwd frontend build
[payments] bun run --cwd frontend build:ssr
[payments] cargo test --locked --lib billing:: -- --nocapture
[payments] cargo test --locked --test payment_sdk_wire -- --nocapture
[payments] node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle -- --nocapture
[payments] node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle -- --nocapture
[payments] node scripts/verify-pinned-checkout.mjs
[pinned-checkout] git -C /home/shawn/.cargo/git/checkouts/suprnova-efd29c0c4437ef3f/cdf9532 archive --format=tar --output=/home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-q7vL0P/source.tar cdf953217bfdae4d4a90d845ca78d14d02b54d53
[pinned-checkout] tar -xf /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-q7vL0P/source.tar -C /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-q7vL0P/repository
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib -- --list --format terse
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib checkout::tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib idempotency_tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib -- --list --format terse
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib checkout::tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib idempotency_tests:: -- --nocapture --test-threads=1
[payments] node scripts/with-ssr.mjs node frontend/tests/paid-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs editorial
[editorial] cargo fmt --check
[editorial] cargo build --locked --bins
[editorial] bun install --frozen-lockfile --cwd frontend
[editorial] bun run --cwd frontend build
[editorial] bun run --cwd frontend build:ssr
[editorial] node scripts/with-ssr.mjs cargo test --locked --test editorial_workflows -- --nocapture
[editorial] node scripts/with-ssr.mjs node frontend/tests/editorial-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs administration
[administration] cargo fmt --check
[administration] cargo build --locked --bins
[administration] bun install --frozen-lockfile --cwd frontend
[administration] bun run --cwd frontend build
[administration] bun run --cwd frontend build:ssr
[administration] node scripts/with-ssr.mjs cargo test --locked --test administration_workflows -- --nocapture
[administration] node scripts/with-ssr.mjs node frontend/tests/administration-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs adoption
[adoption] cargo fmt --check
[adoption] cargo build --locked --bins
[adoption] bun install --frozen-lockfile --cwd frontend
[adoption] bun run --cwd frontend build
[adoption] bun run --cwd frontend build:ssr
[adoption] cargo test --locked --test demo_seed -- --nocapture
[adoption] node scripts/verify-adoption-install.mjs
[adoption install] cargo run
[adoption install] python3 -c
[adoption install] cargo run
[adoption install] python3 -c
[adoption install] bash -euc
[adoption install] python3 -c
[adoption install] bash -euc
[adoption install] python3 -c
[adoption install] python3 -c
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] bash -euc
[adoption install] python3 -c
[adoption install] bash -euc
[adoption install] bash -euc
[adoption install] python3 -c
[adoption install] cargo run
[adoption install] cargo run
[adoption install] bash -euc
[adoption] node scripts/with-ssr.mjs cargo test --locked --test adoption_workflows -- --nocapture
[adoption] node scripts/with-ssr.mjs node frontend/tests/adoption-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs overview
[overview] cargo fmt --check
[overview] cargo build --locked --bins
[overview] bun install --frozen-lockfile --cwd frontend
[overview] bun run --cwd frontend build
[overview] bun run --cwd frontend build:ssr
[overview] node scripts/with-ssr.mjs cargo test --locked --test overview_workflows -- --nocapture
[overview] node scripts/with-ssr.mjs node frontend/tests/overview-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs seo
[seo] cargo fmt --check
[seo] cargo build --locked --bins
[seo] bun install --frozen-lockfile --cwd frontend
[seo] bun run --cwd frontend build
[seo] bun run --cwd frontend build:ssr
[seo] node scripts/with-ssr.mjs cargo test --locked --test seo_workflows -- --nocapture
[seo] node scripts/with-ssr.mjs node frontend/tests/seo-workflows.mjs
[framework-regression] node scripts/verify-appearance.mjs
[appearance] cargo fmt --check
[appearance] cargo build --locked --bins
[appearance] bun install --frozen-lockfile --cwd frontend
[appearance] bun run --cwd frontend build
[appearance] bun run --cwd frontend build:ssr
[appearance] cargo test --locked --lib appearance_tests
[appearance] node scripts/with-ssr.mjs cargo test --locked --test adoption_workflows adoption_workflows_contract -- --exact --nocapture
[appearance] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console directory:demo
[appearance] node scripts/with-ssr.mjs node frontend/tests/appearance.mjs
[framework-regression] node scripts/verify-upgrade-guide.mjs
```

Self-audit: source changes, tests, docs, notices and release versions were reviewed
against the imported production rules. The audit and structural-tool limits above
are explicit; no check is described as passing unless it ran successfully.

## Commands inside the final repository tasks

The following lines are the actual nested command trace from the six final
repository verifiers. Repeated entries represent repeated executions.

```text
[framework-build] node scripts/verify-foundation.mjs build
[build] cargo metadata --locked --format-version 1
[build] cargo build --locked --bins
[build] bun install --frozen-lockfile
[build] bun run build
[build] bun run build:ssr
[framework-migration] cargo test --locked --test framework_upgrade_migration -- --nocapture
[framework-migration] cargo test --locked --test framework_upgrade_migration -- --nocapture
[framework-migration] node scripts/verify-foundation.mjs database
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-qW1iGX/database.db --seed-probe
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-qW1iGX/database.db
[framework-auth] cargo test --locked --test framework_upgrade_auth -- --nocapture
[framework-auth] node scripts/verify-foundation.mjs accounts
[accounts] cargo test --locked --test foundation_accounts -- --nocapture
[framework-http] node scripts/verify-foundation.mjs ui
[ui] bun install --frozen-lockfile
[ui] bun run build
[ui] bun run build:ssr
[ui] cargo test --locked --test foundation_ui -- --nocapture
[ui] node scripts/with-ssr.mjs node frontend/tests/foundation-ui.mjs
[framework-http] bun install --frozen-lockfile --cwd frontend
[framework-http] bun run --cwd frontend build
[framework-http] bun run --cwd frontend build:ssr
[framework-http] cargo build --locked --bins
[framework-http] cargo test --locked --test framework_upgrade_cache -- --nocapture
[framework-http] node scripts/with-ssr.mjs cargo test --locked --test page_budget -- --nocapture
[framework-http] node scripts/with-ssr.mjs node frontend/tests/framework-upgrade.mjs
[framework-console] cargo build --locked --bin console
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console --help
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console --version
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console not-a-real-command
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access --not-a-real-option
[framework-console] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console compat:exit
[framework-regression] node scripts/verify-foundation.mjs build
[build] cargo metadata --locked --format-version 1
[build] cargo build --locked --bins
[build] bun install --frozen-lockfile
[build] bun run build
[build] bun run build:ssr
[framework-regression] node scripts/verify-foundation.mjs database
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-G2qU8U/database.db --seed-probe
[database] cargo run --locked --bin directory -- migrate
[database] bun scripts/check-schema.mjs /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/directory-foundation-G2qU8U/database.db
[framework-regression] node scripts/verify-foundation.mjs accounts
[accounts] cargo test --locked --test foundation_accounts -- --nocapture
[framework-regression] node scripts/verify-foundation.mjs ui
[ui] bun install --frozen-lockfile
[ui] bun run build
[ui] bun run build:ssr
[ui] cargo test --locked --test foundation_ui -- --nocapture
[ui] node scripts/with-ssr.mjs node frontend/tests/foundation-ui.mjs
[framework-regression] node scripts/verify-foundation.mjs setup
[setup] node frontend/tests/foundation-setup.mjs
[framework-regression] node scripts/verify-provider-administration.mjs
[providers] format: cargo fmt --check
[providers] build: cargo build --locked --bins
[providers] HTTP and persistence contract: cargo test --locked --test provider_administration -- --nocapture
[providers] first administrator grant: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 2
[providers] idempotent administrator grant: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 2
[providers] unknown administrator rejected: /home/shawn/workspace2/suprnova-directory-starter/target/debug/console admin:access grant --user-id 9223372036854775807
[providers] restart with the original key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] missing deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] default deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] wrong deployment key: cargo test --locked --test provider_key_policy -- --ignored --nocapture
[providers] constructor failure and corrected case: cargo test --locked --lib billing::settings::tests::constructor_failure_preserves_configuration -- --nocapture
[providers] frontend install: bun install --frozen-lockfile --cwd frontend
[providers] frontend build: bun run --cwd frontend build
[providers] frontend SSR build: bun run --cwd frontend build:ssr
[providers] browser administration journeys: node scripts/with-ssr.mjs node frontend/tests/provider-administration.mjs
[framework-regression] node scripts/verify-paid-directory.mjs directory
[directory] cargo fmt --check
[directory] cargo build --locked --bins
[directory] bun install --frozen-lockfile --cwd frontend
[directory] bun run --cwd frontend build
[directory] bun run --cwd frontend build:ssr
[directory] cargo test --locked --lib listings::media::tests -- --nocapture
[directory] node scripts/with-ssr.mjs cargo test --locked --test directory_workflows -- --nocapture
[directory] node scripts/with-ssr.mjs node frontend/tests/directory-workflows.mjs
[framework-regression] node scripts/verify-paid-directory.mjs payments
[payments] cargo fmt --check
[payments] cargo build --locked --bins
[payments] bun install --frozen-lockfile --cwd frontend
[payments] bun run --cwd frontend build
[payments] bun run --cwd frontend build:ssr
[payments] cargo test --locked --lib billing:: -- --nocapture
[payments] cargo test --locked --test payment_sdk_wire -- --nocapture
[payments] node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle -- --nocapture
[payments] node scripts/with-ssr.mjs cargo test --locked --test paid_lifecycle -- --nocapture
[payments] node scripts/verify-pinned-checkout.mjs
[pinned-checkout] git -C /home/shawn/.cargo/git/checkouts/suprnova-efd29c0c4437ef3f/5112862 archive --format=tar --output=/home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-MQxYmN/source.tar 5112862367b76a2ea3875d0135ca4ca1ec602c32
[pinned-checkout] tar -xf /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-MQxYmN/source.tar -C /home/shawn/workspace2/suprnova-directory-starter/.upgrade-work/workspaces/pinned-checkout-MQxYmN/repository
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib -- --list --format terse
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib checkout::tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-stripe --lib idempotency_tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib -- --list --format terse
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib checkout::tests:: -- --nocapture --test-threads=1
[pinned-checkout] cargo test --locked -p suprnova-payments-paddle --lib idempotency_tests:: -- --nocapture --test-threads=1
[payments] node scripts/with-ssr.mjs node frontend/tests/paid-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs editorial
[editorial] cargo fmt --check
[editorial] cargo build --locked --bins
[editorial] bun install --frozen-lockfile --cwd frontend
[editorial] bun run --cwd frontend build
[editorial] bun run --cwd frontend build:ssr
[editorial] node scripts/with-ssr.mjs cargo test --locked --test editorial_workflows -- --nocapture
[editorial] node scripts/with-ssr.mjs node frontend/tests/editorial-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs administration
[administration] cargo fmt --check
[administration] cargo build --locked --bins
[administration] bun install --frozen-lockfile --cwd frontend
[administration] bun run --cwd frontend build
[administration] bun run --cwd frontend build:ssr
[administration] node scripts/with-ssr.mjs cargo test --locked --test administration_workflows -- --nocapture
[administration] node scripts/with-ssr.mjs node frontend/tests/administration-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs adoption
[adoption] cargo fmt --check
[adoption] cargo build --locked --bins
[adoption] bun install --frozen-lockfile --cwd frontend
[adoption] bun run --cwd frontend build
[adoption] bun run --cwd frontend build:ssr
[adoption] cargo test --locked --test demo_seed -- --nocapture
[adoption] node scripts/verify-adoption-install.mjs
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption install] cargo run
[adoption] node scripts/with-ssr.mjs cargo test --locked --test adoption_workflows -- --nocapture
[adoption] node scripts/with-ssr.mjs node frontend/tests/adoption-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs overview
[overview] cargo fmt --check
[overview] cargo build --locked --bins
[overview] bun install --frozen-lockfile --cwd frontend
[overview] bun run --cwd frontend build
[overview] bun run --cwd frontend build:ssr
[overview] node scripts/with-ssr.mjs cargo test --locked --test overview_workflows -- --nocapture
[overview] node scripts/with-ssr.mjs node frontend/tests/overview-workflows.mjs
[framework-regression] node scripts/verify-complete-starter.mjs seo
[seo] cargo fmt --check
[seo] cargo build --locked --bins
[seo] bun install --frozen-lockfile --cwd frontend
[seo] bun run --cwd frontend build
[seo] bun run --cwd frontend build:ssr
[seo] node scripts/with-ssr.mjs cargo test --locked --test seo_workflows -- --nocapture
[seo] node scripts/with-ssr.mjs node frontend/tests/seo-workflows.mjs
[framework-regression] node scripts/verify-appearance.mjs
[appearance] cargo fmt --check
[appearance] cargo build --locked --bins
[appearance] bun install --frozen-lockfile --cwd frontend
[appearance] bun run --cwd frontend build
[appearance] bun run --cwd frontend build:ssr
[appearance] cargo test --locked --lib appearance_tests
[appearance] node scripts/with-ssr.mjs cargo test --locked --test adoption_workflows adoption_workflows_contract -- --exact --nocapture
[appearance] /home/shawn/workspace2/suprnova-directory-starter/target/debug/console directory:demo
[appearance] node scripts/with-ssr.mjs node frontend/tests/appearance.mjs
[framework-regression] node scripts/verify-upgrade-guide.mjs
```
