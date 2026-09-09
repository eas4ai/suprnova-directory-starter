# Verify the documented SQLite foundation through the application migration command

Level: Judged
Decided by: Codex
Rests on: FND-002,FND-005
Would be wrong if: The default SQLite configuration cannot initialize the account schema, or a check can touch an existing operator database.

## Decision

Keep the scaffold users schema and documented SQLite default for this foundation. Exercise cargo run --locked --bin directory -- migrate in a disposable checkout with an explicitly selected local database. Inspect account tables and migration history after first and repeated runs. This is SQLite proof only; it does not establish PostgreSQL parity. Use the framework migration entry point instead of introducing another migration command.

## Realized by

- 25e0b79a8d099275e019de6da0344782c84219ed Verify account migrations and data preservation on disposable SQLite
