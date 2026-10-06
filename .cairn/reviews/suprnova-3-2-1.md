commit: 2bfa8da71e633b9f72623763554861c7f629476b
findings: []

# Starter 1.0.1 release review on Suprnova 3.2.1

## Reviewed change

The developer requested the same verified upgrade, commit, main push and release-tag workflow as Pulsar and Nebula. Compared the complete change with starter 1.0.0 at 5be23a2. All seven locked framework packages and both direct payment adapters use the published Suprnova 3.2.1 commit 2bd4bd53d04fa4581fdbb152ebd70cfb682b342c. Verified that the remote annotated release tag peels to that commit. The Markdown prerequisite at published commit 565c2205b1d1d77aa05a29247f3a2416abc60791 targets the same framework identity; its four unit tests, two middleware tests and doctest passed before publication.

The only application code change is in src/accounts/roles.rs: permission and role seed timestamps remain native UTC DateTime values for the new framework model fields, with redundant clones removed. Insert conflict handling, role names, permission grants, transactions and application schema are unchanged. Both package versions are 1.0.1. Exact-pin checkers retain every assertion. Active specifications and documentation agree with the pins. The existing SEO sentence was split without changing its two obligations, and specification lint passes. Production documentation includes shared Redis rate limiting and the explicit single-process memory exception; default production guards remain enabled.

## Verification and evidence

All 54 requirements have new passing committed-tree receipts: FND 5, PAY 16, DIR 8, CNT 6, ADM 3, KIT 5, UI 2, OVR 3 and SEO 6. Verified every latest receipt's result and both captured output hashes: 108 hashes. Checks cover locked server/console builds, frozen frontend installation, TypeScript, client and SSR outputs, repeated migrations, accounts, provider configuration, paid publication/recovery, directory/media boundaries, editorial/RSS/SEO/Markdown, delegated administration, fresh adoption/demo operations, appearance and overview HTTP/browser workflows. The upstream Stripe/Paddle gate used a clean archive of the exact pinned release, enforced the required test names and checked checkout payloads, correlation, idempotency, retrieval and bounded timeouts. Neither dependency caches nor test assertions were patched.

Cargo check --locked --all-targets, formatting and JavaScript syntax checks passed. The all-target check reports unused methods in shared test fixtures; it is not a warning-free claim. No Clippy result is claimed. Old-pin and revised metadata/cache violations were rejected at their intended assertions, followed by corrected positive executions; see suprnova-3-2-1-mechanism.md and suprnova-3-2-1-failures.out. The combined demonstration transcript has only its trailing blank lines normalized; formal receipt output bytes remain untouched.

Ripwire reports zero quality regressions against 5be23a2 and no incompatible seed callers. Its test gate returns 4, identifying seven impacted test files and five graph-uncovered callers. All seven files ran through the acceptance mechanisms, including the paid browser fixture. Migration and administrator/account request paths ran through actual migrations, HTTP tests and repeated grant/revocation checks. This runtime evidence covers the macro/bootstrap paths the static graph does not establish; the test-gate status is not reported as a pass.

The installed Cairn entry point is now Sudus and cannot read this older tracked layout without initialization. Used the archived legacy Cairn implementation at 94965aaa to wake, check and record evidence against the existing agreement. No workflow migration or durable-ref initialization was performed.

## Production self-audit

Rules 1–4: the requested release, dependency identities and affected RBAC boundary were mapped; the change is limited to pins, versions, native timestamps and necessary verification/documentation; existing domain behavior and all public contracts remain covered. Rules 5–8: no new secrets, error suppression or authorization bypass was introduced; existing transaction, credential, publication, retry and bounded-work contracts passed their checks; no new hot-path work or helper was added. Rules 9–11: the work and failures were tracked, acceptance uses committed stable candidates, receipts and hashes were verified, and static-tool limits are stated accurately. Rules 12–14: implementation follows the authorized upgrade-and-release scope, the full diff was reviewed against the production standard, and the documentation explains operational changes directly. No remaining finding or revision is required for this release.

Verification uses disposable SQLite data, private test media, local SSR/browser requests and synthetic provider fixtures. It does not prove PostgreSQL deployment behavior, actual provider charges, production Redis connectivity or external mail delivery. The existing untracked operator database.db was excluded from snapshots and remains untouched. Traffic reporting and other planned framework work remain outside this release. Publication of main and annotated v1.0.1 follows this committed review; no deployment is requested.
