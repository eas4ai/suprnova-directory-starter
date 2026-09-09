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

FND-002 development review: the application migrate entry point initializes the SQLite example URL without needing a new command. The mechanism runs the documented cargo invocation twice and compares schema SQL, migration receipts and a seeded account probe. A disposable migrated database with its users table removed failed at Missing account table: users; restoring the correct database passed. The full repeat-migration mechanism also passed. This is SQLite evidence only.
