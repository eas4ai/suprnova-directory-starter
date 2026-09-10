# SEO mechanism construction

The declared command builds locked Rust binaries, checks and builds Vue client and SSR assets, runs a real HTTP fixture through the SSR middleware, then starts a production-mode application for Chromium journeys. It uses a filtered environment and disposable source/database snapshots. The operator database is excluded.

Two editing-time complete runs passed all six SEO outputs. The second also covered article noindex, blank-content fallback, known-good findings, redirect capacity, observational database failures and browser editor/owner publication boundaries. Historical sitemap/JSON-LD date assertions were added afterward and compiled successfully; committed execution will verify them before completion.

`scripts/demonstrate-seo-failures.mjs` successfully ran six deliberate production mutations in disposable copies. `.cairn/reviews/seo-controls-failures.out` retains the actual assertion failures:

- SEO-001 discards the site description during validation; the persisted settings comparison rejects the missing value.
- SEO-002 removes the Twitter card from the SSR component; the initial HTML assertion rejects its absence.
- SEO-003 ignores the approved listing revision's noindex flag; the sitemap exclusion assertion rejects the URL.
- SEO-004 changes the resolved redirect target; the actual 301 location assertion rejects the wrong page.
- SEO-005 changes only the report preview title; comparison with public metadata rejects the disagreement.
- SEO-006 restores an hour of public caching; the actual Markdown response assertion rejects the cache header.

All mutations compile and reach their intended runtime assertions. The demonstration script checks the assertion reason as well as the failed test and Cargo status. The SSR wrapper translates Cargo 101 to process status 1; the initial demonstration harness expected 101 directly and was corrected without changing any application requirement.

The HTTP fixture checks direct permissions, protected audit redaction, stale and malformed writes, listing/article draft isolation, configured-origin metadata, escaped JSON-LD, saved dates, pagination/filter policy, actual sitemap traversal, eligibility transitions, taxonomy noindex, redirect collisions/chains/cycles/caps, bounded path-only 404 reporting, and real Markdown methods/headers/error responses. Browser checks cover settings persistence and errors, focused error summaries, desktop/mobile and light/dark output, delegated navigation, real redirects, accurate previews, initial HTML without JavaScript, Inertia head replacement, Markdown discovery, and editor/owner publication boundaries.

## Static tooling

Formatting and JavaScript syntax passed. All-target Clippy completed successfully with the same 125 located warning messages/files as the preceding overview candidate; new warnings in the SEO middleware and taxonomy projection were corrected. Existing warnings remain and are not reported as a strict warning-free pass.

Ripwire's source-focused quality scan returned 2, and its test gate returned 4. These results are not called passes. The full findings were inspected: common unqualified names merge unrelated `load`, `preview`, `save`, and `list` definitions; billing/settings and accounts were not edited. Macro/trait-registered handlers and serialized ORM types are marked dead despite successful runtime coverage. The scanner sees no tests when scoped to src, and does not model the HTTP/browser runner edges. The inherited mechanisms supply those checks.

The real growth is the public metadata override argument, sitemap eligibility/date logic, migration columns/tables, and the plugin response wrapper. These are required boundary behavior. Similar short controllers, SQL eligibility predicates, paginated ORM reads, and migration steps remain explicit in their distinct domains; combining them would couple unrelated workflows. No blanket acknowledgement or warning suppression was added. The metadata edit check found four updated callers and no incompatible calls. Middleware is correctly a new trait implementation. Taxonomy's reported definition-count change comes from the new unrelated redirect list function; its own two-argument signature is unchanged.

This file records mechanism construction, not the final production review. Fresh committed checks and the final review are still required.

## Inherited fixture correction

The first inherited provider check failed because its explicit full-administrator count remained seven. SEO adds an eighth capability by contract. The fixture now expects eight and independently requires `seo.manage` alongside every prior capability; repeated grant, full revoke, unverified denial and registration denial assertions remain intact. The administrator-access README list now includes the new capability. No production authorization was changed for this correction. The complete provider editing-time suite then passed all six provider outputs, including CLI and browser journeys. The failed committed receipts remain in history; fresh committed evidence is required after this input change.
