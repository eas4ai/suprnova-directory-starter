commit: 2f9cb86a9caa617c589886b7d7768dac1c374dec
findings: []

# Starter upgrade guidance review

## Reviewed behavior and recovery

Compared the guide with the released Rust/frontend versions, all framework/adapter revisions and the original compatible Markdown revision. The plugin's newer tag is explicitly a separate consumer change. The application source, manifests, lockfiles, existing workflow checks and operator database were not modified.

Inspected the CLI help for migrate and serve --no-migrate and the released framework's production rate-limit/mail configuration. The guide separates one migration run from web startup and a continuously running private SSR worker. It preserves the environment, APP_KEY, database, media and encrypted billing credentials, pauses scheduled writers, directs operators to the existing consistent SQLite backup/restore procedure and checks persisted work before resuming bounded schedules. It avoids fresh-install seeding, configuration replacement, key generation and blind payment replay. Rollback restores matching artifacts and state rather than promising reversible data migrations.

Checked the build, SSR and worker command names against the release's existing README and acceptance mechanisms. The guide distinguishes historical SQLite/local-browser/synthetic-provider evidence from PostgreSQL deployment, real provider charges, production Redis connectivity and external mail delivery. No live deployment or recovery operation was run for this documentation change.

## Verification and mechanism limits

UPG-001 passed against committed candidate 689198a. The check verifies release metadata, lock revisions, named operating boundaries, the backup link and shell syntax without running deployment commands. Its four disposable violations fail and the corrected guide passes; see upgrade-guidance-mechanism.md and its transcript. Assertions cannot prove prose meaning or production recovery; the operating sequence and limitations above were reviewed manually.

Verified all 55 latest passing receipts (the existing 54 application requirements plus UPG-001), 110 captured output hashes and every recorded input hash against the current tree. Existing release evidence remains current; it was not rerun or rewritten for this documentation-only commitment. Specification lint, checker syntax and whitespace validation pass.

Ripwire reports zero quality regressions. Its test gate returns 4 for a graph-uncovered README heading reached by the new file-reading helper; the guide/link checks and explicit failure demonstrations exercise this path. This static result is not reported as a pass. The legacy Cairn implementation at 94965aaa records evidence using the repository's existing agreement; no Sudus initialization or workflow migration was performed.

## Production self-audit

Rules 1–4: the documented release and affected operator steps were mapped; changes are limited to guidance and its required verification/records; dependency and application contracts are unchanged. Rules 5–8: the guide preserves secrets, authorization, state and bounded scheduled work; no runtime error handling or hot path changed. Rules 9–11: work was tracked, positive and negative checks ran, committed evidence and hashes were verified, and check limits are stated. Rules 12–14: the developer confirmed these guides and publication; the complete change meets the production standard and uses direct operating instructions. No finding remains. Publish documentation on main; leave v1.0.1 unchanged.
