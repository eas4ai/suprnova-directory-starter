commit: 8a1e12db60a33e4817351c229e60a61cdfc2d48f
findings:
  - resolved: The inherited provider fixture expected seven administrator capabilities. bc35eef requires all eight, including seo.manage, and keeps repeat-grant, revocation and denial assertions. All 54 requirements have fresh passes after that correction.

## Retained README work review — 2026-09-11

### FND-001 mechanism review for the requested 2.0.0 upgrade

The developer requested upgrading the starter pin to Suprnova 2.0.0 after the release discrepancy audit. Inspected the revised FND-001 requirement/falsifier, foundation-build declaration, verifyPin, and build. The only check change is the expected exact revision, now 3229aa9af542c991196274fa3c235cdce88a68e2. It still checks all three direct framework/adapter sources, every locked Suprnova source, both binary names, locked Rust builds, frozen frontend installation, client/SSR outputs, and unchanged dependency locks. No mismatch remains between this mechanism and the revised requirement.

A disposable fixture carrying the old committed Cargo.toml and Cargo.lock from 4670ede was rejected at the exact-revision assertion, exit 1. The first fixture mistakenly selected the already-upgraded commit and reached compilation; it was stopped, its processes terminated and disposable tree removed, and the corrected fixture asserted the old pin before execution. The corrected current candidate passed node scripts/verify-foundation.mjs build, including both binaries, TypeScript/client build and SSR build. These are editing-time demonstrations; fresh Cairn evidence follows this recorded review. The earlier formal failure against the old expected pin remains preserved.

Ripwire found one verifyPin caller and no incompatible call. Its script quality delta and test gate returned 0 against the already-committed tree (zero changed symbols); these do not replace the actual negative and positive mechanism executions above. No production application code changed in this mechanism review.

Reviewed the changes since the completed SEO delivery at 4ddf866: the README feature list, framework link and developer-supplied header image, followed by the scope approval and refreshed evidence. No application, dependency, mechanism or test code changed. The developer explicitly confirmed retaining assets/card.jpg in loop-035. The image matches the supplied artwork, exists as a JPEG, and the relative README link and descriptive alternative text agree with it. The framework link uses the developer-corrected https://suprnova.app address. The feature list describes the implemented starter and does not claim the pending traffic reports.

All 54 requirements passed again after the committed retention approval, from 2026-09-11T14:07:49.300Z through 2026-09-11T14:23:00.663Z. Examined the latest receipts and verified the SHA-256 hashes of both captured output files for every receipt. Historical failure demonstrations and the implementation review below remain applicable because their code and mechanisms are unchanged. No new implementation finding arose from this documentation-only retention review.

Reviewed the retained change against all 14 production rules: scope and intent match the developer's request; the diff is limited and readable; no runtime contracts, secrets, security boundaries, persistent state or performance paths changed; verification and the active task todo are recorded; claims distinguish completed tests from unverified work; and this review requires no further correction. Static code analysis was not rerun for a README and JPEG change. These checks exercise the existing pinned framework, not Suprnova 2.0.0. The newly requested 2.0.0 discrepancy audit remains separate and has not been performed. No push or deployment occurred during this refresh.

## Original implementation review outcome

Reviewed the committed implementation, mechanism inputs, actual failure demonstrations, fresh evidence and operator documentation. No open implementation findings remain for this commitment. The feature is implemented in a1b2008; the inherited fixture and administrator documentation correction is bc35eef. This review changed no application or test code.

Site defaults are versioned and require admin.access plus seo.manage on a verified active account. Settings and redirect writes recheck permission inside the transaction. Audit records share that transaction and omit metadata/verification values. Stale versions fail without overwriting edits. Content SEO stays on the existing listing/article revision records; taxonomy retains its existing permission and immediate-update policy. Full administrator grants now include eight explicit permissions. Delegated editor and SEO-only fixtures prove these capabilities do not imply one another.

Public output resolves approved listing or published article data and the configured APP_URL. Request Host cannot select the canonical origin. Public and report previews use the same resolver; title placeholders are expanded only in the format, not in user text. Vue escapes HTML attributes, and serialized JSON-LD escapes HTML delimiters. Article structured data uses visible titles, summaries and covers, stored publication/modification dates and the configured publisher. It invents no rating or business facts. Initial HTML, no-JavaScript rendering and Inertia head replacement are covered by actual browser requests.

Sitemap selection applies both existing publication eligibility and revision noindex. Static/taxonomy paths use bounded eligible taxonomy queries; their 1,000-record limits match enforced taxonomy capacity. Content dates use stored public dates, including historical fixtures distinct from newer private revisions; static/taxonomy URLs omit lastmod. Whole-site noindex leaves crawling allowed so crawlers can read the instruction and empties the sitemap index. Pagination keeps its page; tracking is omitted, while search, combined facets and nondefault page sizes receive noindex. Noindex does not grant privacy or remove readable content.

Redirect writes reject external and encoded paths, credentials/query syntax, reserved route namespaces, existing public files, loops, stale versions and chains beyond five steps. A transaction serializes validation and capacity checks. Incoming references prevent deletion of a needed target. Resolution is bounded and checks the final target's current public eligibility. The existing published-article alias route remains separate and working. The 404 observer runs only after a reportable GET actually returns 404. It stores no query string, omits sensitive/encoded paths and long segments, evicts at 1,000 paths and hides observations older than 30 days. An observational write error is logged generically and preserves the original 404; protected report query failure remains an error.

The report returns bounded database pages. Duplicate aggregation compares displayed values across eligible content and taxonomy; private drafts are excluded. It reports missing social images, duplicate metadata and intentional noindex, with a known-good empty-findings fixture. It does not claim external image availability or search-engine rankings. Authorized operators can configure HTTP(S) image references; the server does not fetch them. External publication and search-service verification remain operator actions, not claimed integration results.

Markdown uses the exact verified plugin revision dcc6de06a177d66cfc1a9897ded584e19040f40c. Cargo.lock contains one suprnova framework identity at 107e6e7a122d5145160ea1547ca90ddc37459c27. The source checks eligibility on every request and selects the public revision pointer. Private, expired, suspended, revoked and unpublished content cannot produce twins. Only canonical listing/article slugs are advertised. The wrapper enforces GET/HEAD, plain bounded paths, UTF-8 text/markdown, default noindex, nosniff and no-store; source failures produce generic uncached errors. It does not use the plugin's public-cache default. In-flight requests observe the revision selected for that request; subsequent requests recheck current eligibility.

## Evidence

All 54 current requirements have fresh committed passes on the final inputs: SEO 6, FND 5, PAY 16, DIR 8, CNT 6, ADM 3, KIT 5, UI 2 and OVR 3. The final evidence span is 2026-09-10T16:27:58.066Z through 2026-09-10T16:40:21.493Z. Cairn selected each mechanism, captured its output against a stable committed candidate, and required the resulting receipts/output to be committed before continuing.

The six deliberately broken copies failed at their intended runtime assertions: discarded settings, missing Twitter metadata, noindex sitemap leakage, wrong redirect target, inaccurate preview and public Markdown caching. See seo-controls-failures.out and seo-controls-mechanism.md beside this review. The final corrected positive suite includes added capacity, historical-date, database-failure, blank-fallback and browser publication-boundary checks. Failed intermediate provider receipts remain in history; no failed requirement was waived.

Client/SSR builds, TypeScript, formatting and JavaScript syntax passed. All-target Clippy completed with the same 125 located warning messages/files as the preceding overview candidate after introduced style warnings were fixed. Existing warnings remain; this is not a strict warning-free claim. The final provider fixture change was compiled and exercised by its complete HTTP/CLI/browser mechanism.

Ripwire edit checks found no incompatible calls. Its source-focused quality delta returned 2 and its test gate returned 4; these are not reported as passes. Findings and deliberate trade-offs are documented in seo-controls-mechanism.md. Common names conflate unrelated functions, macros/traits obscure runtime reachability, and the src-only corpus does not include test files. Required metadata/sitemap/plugin-wrapper growth and separate domain predicates were inspected directly, while the runtime suites establish the HTTP/browser coverage. No blanket acknowledgement, warning suppression or metric-driven refactor was used. Graph MCP discovery was unavailable (transport closed), so focused Ripwire and source reads supplied the fallback.

Desktop light/dark, 320-pixel layouts and no-JavaScript screenshots were generated. The SEO desktop and narrow findings screenshots were visually inspected; controls, metadata previews and long paths remain legible without horizontal overflow. Browser assertions verify focused errors, keyboard interaction and delegated navigation.

## Production self-audit

1. The approved SEO contract and existing publication/permission paths were mapped before implementation.
2. Changes stay within metadata, discovery, redirects, reporting, Markdown and required compatibility/verification work. No traffic collection or external provider integration was added.
3. Existing eligibility, revision, audit, taxonomy and framework RBAC behavior is reused; public and report metadata share one resolver.
4. Migration defaults preserve existing rows; request defaults, serialized projections, Vue types/forms, permission fixtures and documentation agree.
5. Database/SSR failures remain errors. Audits omit values, 404 storage omits query data, and all fixtures use disposable synthetic data.
6. Direct denial, CSRF, injection, path, redirect and publication-boundary tests ran through actual request handlers and middleware.
7. Settings/redirect/taxonomy writes use transactions, version checks and bounded serialized validation; existing revision publication transactions remain intact. Migration adds empty override defaults and seeds permissions without rewriting content pointers. No-store prevents shared caches retaining newly ineligible twins.
8. Redirects, report pages, taxonomy, 404 storage and Markdown inputs are bounded. Database aggregation and batched taxonomy access avoid per-content report queries; redirect resolution has a fixed maximum depth.
9. The todo remained active during implementation and verification; completion follows this review and fresh evidence.
10. Success, denial, error, stale-write, lifecycle, browser, SSR, historical-date and deliberate-violation checks ran. All inherited requirements were refreshed after the final fixture correction.
11. Intermediate failures and static-tool limits are retained and explained. No failing check is called a pass.
12. Work follows the developer-authorized SEO commitment and stops before the next commitment.
13. The final review is satisfied that this implementation meets the agreed contract and production standard; no further revision is required for this scope.
14. UI copy, README, decision records and delivery notes explain actual behavior and limits in the project's vocabulary.

Local acceptance covers SQLite, disposable media/mail, real local SSR/browser requests and synthetic payment fixtures. It does not establish external Search Console/Bing verification, ranking outcomes or deployment behavior. The operator database was preserved; no deployment or push was performed.

### PAY-001 mechanism review for Suprnova 2.0.0 — 2026-09-11

Reviewed the revised PAY-001 requirement, provider-administration declaration and verification script. Only the required adapter revision changed; permission, CSRF, persistence, deployment-key and browser checks remain applicable. The mechanism builds locked binaries and exercises both providers through HTTP and browser journeys, including denied actors and restart persistence. Its 15:37:50 passing receipt already used the 2.0.0 lockfile. Exact revision enforcement remains covered by foundation-build and the pinned-checkout verifier used by paid checkout. No mismatch found.

For the changed revision contract, a disposable manifest and lockfile from commit 4670ede were rejected by the current pinned-checkout verifier with exit 1: “suprnova must use the recorded checkout revision.” Output: target/upgrade-checkout-wrong-pin.out. The corrected current manifest passed the verifier against a clean archive of release commit 3229aa9af542c991196274fa3c235cdce88a68e2, including the required Stripe and Paddle checkout, idempotency, retrieval and timeout tests. No application code changed during this review. Fresh committed-tree checks follow this declaration review.
