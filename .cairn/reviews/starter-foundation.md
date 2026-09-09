commitment: starter-foundation
examined:
  - Draft requirements were checked against the selected version, UI and provider decisions.
  - Build-only proof was rejected for account, database and UI behavior; separate mechanisms cover those failures.
  - Paid publishing remains in first delivery but is not required to complete this foundation.
  - Account and UI falsifiers include denied actors and failed token paths rather than success paths alone.
findings:
  - resolved: The developer confirmed FND-001 through FND-005 and their falsifiers, with glossary and roadmap context, on 2026-09-08.
  - open: Application mechanisms are declared but not implemented or demonstrated; no runtime pass is claimed.

This is a specification review, not the final implementation review. The final review needs a committed candidate and fresh mechanism evidence.

Setup verification: spec lint passed over docs/spec. A disposable Agreed DEMO-001 fixture without a Falsifier line exited 1 with SPEC-002; adding the line exited 0. This proves the lint wrapper invokes the installed checker and detects that structural violation. It does not prove any application requirement.

FND-001 mechanism development: a disposable Git fixture pinned to v1.3.5 reached the framework-version assertion and reported FND-001 fail. A separate frontend fixture changed the Vue dependency without updating bun.lock; bun install --frozen-lockfile rejected it with the expected frozen-lockfile error. The original client and SSR builds passed. The pinned Rust binary build and full corrected build mechanism are still running/pending; these partial checks are not FND-001 completion.

FND-001 corrected case: node scripts/verify-foundation.mjs build passed in a disposable copy, including both Rust binaries, frozen Bun installation, client and SSR builds, artifact checks and unchanged lockfile assertions. The preceding direct Rust build passed in 8m30s. This completes the wrong-tag/frozen-lock failure demonstration for the build mechanism; Cairn receipt recording follows its implementation commit.

FND-002 development review: the application migrate entry point initializes the SQLite example URL without needing a new command. The mechanism runs the documented cargo invocation twice and compares schema SQL, migration receipts and a seeded account probe. A disposable migrated database with its users table removed failed at Missing account table: users; restoring the correct database passed. The initial schema-only repeat-migration mechanism passed. The later data-probe extension failed before its assertion because the writable Bun SQLite connection needed readwrite: true; this is a verifier setup defect, not a migration defect. A focused disposable-file experiment reproduced SQLITE_MISUSE with readonly: false/create: false and succeeded with readwrite: true/create: false. Full corrected verification follows. This is SQLite evidence only.

FND-002 corrected verification passed: both migration invocations, schema/history equality, and preservation of the seeded account probe. The connection-mode correction follows Bun SQLite options documented at https://bun.sh/docs/runtime/sqlite and the local failing/passing experiment.

FND-003 mechanism development: after correcting missing test bootstrap configuration and encryption initialization, the HTTP journey failed because registration sent zero verification messages. Adding framework mail dispatch passed that case. Extending the journey then failed with 404 for verification and reset routes; implementing each framework-backed route made its case pass. An Inertia invalid-reset-token request failed with raw 400 instead of a 303 form redirect; translating only framework bad-request errors to field validation corrected it. Infrastructure errors still propagate. The final editing-time journey passed with registration validation, verification mail/resend, cross-account denial, expired and reused verification/reset tokens, unknown and unverified reset-address behavior, password confirmation, invalid credentials, CSRF denial, logout with a copied cookie, and password-reset session revocation. These real missing-feature cases and corrected runs demonstrate failure sensitivity. The harness uses the actual global middleware registry, fresh externally configured SQLite, and captured mail; it does not prove external SMTP delivery. Client typecheck/build passed after the three new account pages. Cairn evidence follows the implementation commit.
