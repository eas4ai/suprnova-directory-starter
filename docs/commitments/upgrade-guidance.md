# Starter upgrade guidance

Status: Agreed 2026-10-05
Requirements: UPG-001

## Outcome and boundary

The developer requested and confirmed an upgrade guide for each upgraded repository. Document starter 1.0.0 to 1.0.1 using its existing verified pins, preservation/build/migration steps, production mail/rate limits, private SSR, worker recovery, checks and rollback. Add a focused documentation mechanism, demonstrate rejection of a wrong pin and unsafe installation step in disposable copies, and record a final review. Keep runtime inputs and the existing database untouched. Other repositories' guides are committed in their own histories.

## Todo

- Done: write and validate UPGRADE.md and the focused mechanism; positive and four disposable failure cases passed.
- In progress: collect committed-tree evidence and record the production review.

Publish documentation on main after verification; no new release tag is needed for documentation alone.
