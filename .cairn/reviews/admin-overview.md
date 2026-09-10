commit: ae39c010a26a7b1001f8db52002c7e48d36383ce
findings:
  - resolved: The OVR-002 negative demonstration now bypasses both listing permission guards in a disposable copy. The test rejects the editor receiving listing_summary. All three negative cases ran successfully after a2d706f, followed by fresh passing evidence for all 48 requirements.

## Review outcome

Reviewed the final overview controller, shared review predicate, Vue overview and navigation, HTTP and browser checks, mechanism inputs, README and failure demonstrations. No open implementation findings remain. Implementation is in 48dd7a2 and the shell compatibility correction in 1f852bd; the final failure demonstration is in a2d706f.

The overview counts eligible public listings, current submitted revisions and published articles. Proposed edits can overlap public listings; the copy explains that overlap. The public eligibility predicate is reused unchanged, including entitlement dates, mode, suspension and archive state. The queue and overview share the exact submitted-revision predicate. Counts use database aggregation; the audit query returns at most five rows with a joined actor name. It reuses the protected audit projection, excluding private reasons and redacting historical listing free text. There is no per-listing or per-actor query loop.

Each protected data branch checks its own capability before querying. Listing and editorial branches also apply the existing verified, active administrative boundary; audit performs the same check internally. The existing route middleware retains admin.access. A shell-only account still opens the empty shell without acquiring any protected data. FND-004 caught an initial regression in that behavior; 1f852bd fixed the application without weakening the inherited test. Unverified full administrators remain denied protected overview data.

Database failures propagate as errors, not zero counts. Vue escapes activity text. Navigation retains the existing permitted destinations and filters empty groups. Public navigation, appearance controls and mobile dialog behavior remain covered by inherited browser checks. The overview uses semantic tokens, definition-list counts, explicit link labels and UTC activity timestamps.

## Evidence and failure sensitivity

All 48 current requirements have fresh committed passes on the final inputs: OVR 3, FND 5, PAY 16, DIR 8, CNT 6, ADM 3, KIT 5 and UI 2. The last evidence run spans 20260910T145930523Z through 20260910T151059810Z. Cairn selected each mechanism and recorded actual output against a stable committed candidate.

The overview mechanism builds both Rust binaries, installs locked frontend dependencies, checks TypeScript, builds client and SSR, runs isolated HTTP fixtures and exercises a real production-mode server in Chromium. It verifies empty and populated data, expired/future/test/revoked entitlements, listing and owner suspension, archived submissions, duplicate entitlements, published articles with private edits, audit ordering/limit/redaction, direct permission denial and a real database failure. Browser journeys verify the actual queue destinations and row counts, delegated data/navigation, keyboard activation, narrow layout, dark appearance, text contrast and empty states reached through actual archive/unpublish workflows.

Private administration was already client rendered. An early draft check incorrectly expected private SSR text; it was corrected to match the recorded architecture, while the browser verifies the actual rendered content. Public SSR checks remain inherited and passing.

`scripts/demonstrate-overview-failures.mjs` ran against disposable application snapshots with filtered environment and fresh databases. Its three intentional violations were rejected for the intended reasons; output is recorded in `.cairn/reviews/admin-overview-failures.out`:

- OVR-001: counting all unarchived listings produced 4 public listings where 2 were eligible.
- OVR-002: bypassing both moderation guards exposed listing_summary to the editor and failed the permission assertion.
- OVR-003: directing review work to articles passed its build/fixture prerequisites, then failed the browser URL assertion.

The final corrected positive run passed all three overview requirements after these demonstrations. No operator database, real provider or external service was used for mutation tests.

## Tooling limits

Cargo formatting, JavaScript syntax, TypeScript and locked builds passed. Normal all-target Clippy completed successfully on the baseline and overview candidate with the same 125 located warnings and no warnings in the edited Rust files. Strict Clippy with `-D warnings` failed on existing migration and style warnings in unchanged files; it is not reported as passing and those unrelated files were not changed or suppressed.

Ripwire edit checks found no incompatible signature change. Its initial full quality scan included ignored generated frontend bundles; the focused source scan flagged structurally similar but distinct SQL predicates and review-queue churn. Those predicates belong to different domains and should not be merged; the genuinely shared review predicate was extracted. Static test reachability misses macro-registered HTTP handlers, so the runtime HTTP/browser evidence establishes route coverage. Later current-tree edit/quality/test scans exited successfully, but are not a substitute for the initial change review. The graph MCP was unavailable (transport closed); Ripwire and focused source reads supplied the discovery fallback.

## Production self-audit

1. Outcome and affected paths were mapped against the approved overview contract and existing publication/permission rules.
2. Changes stay within counts, review work, grouped navigation and their verification; no dependencies, schema or provider work was added.
3. Existing publication and audit helpers are reused; the queue predicate is shared with its actual caller.
4. Existing shell access and queue/filter contracts are preserved; serialized props and Vue types agree.
5. Errors propagate and protected audit text is excluded; disposable fixtures contain no operator secrets.
6. Authorization is enforced on direct requests and before data queries, with capability-specific navigation.
7. Production changes are read-only; no new persistence or retry protocol is introduced.
8. Database counts and a five-row audit limit bound application memory and query count. Counts are a current view, not a transactionally frozen financial report.
9. The commitment todo remained active through implementation and verification; completion is recorded only after review.
10. Success, boundary, error, denial, browser and deliberate-failure checks actually ran; inherited checks are fresh.
11. Failed intermediate checks and static-tool limitations are retained and explained; no failed check is called a pass.
12. Work follows the developer-approved commitment and stops before activating the next one.
13. Final source/evidence review found no remaining change needed for this contract; the recorded verification finding is resolved.
14. UI copy, README and delivery notes use the project's vocabulary and explain the observed behavior directly.

The admin-overview commitment is ready to close. SEO controls and traffic/revenue/engagement charts remain the following agreed commitments.
