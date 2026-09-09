# Complete-starter review

## Editorial mechanism construction — 2026-09-09

Reviewed CNT-001 through CNT-006 against `tests/editorial_workflows.rs`,
`frontend/tests/editorial-workflows.mjs`, `scripts/verify-complete-starter.mjs`
and the actual SSR wrapper. This is the editorial mechanism review, not the
commitment's final production review. Administration and adoption remain pending.

The runner copies declared application inputs into a disposable installation,
builds both locked Rust binaries and frozen Vue client/SSR assets, runs HTTP
and transactional assertions, then runs Chromium against the production server.
Each stage must exit successfully before any requirement is reported as passed.
It uses temporary SQLite/media, synthetic users and captured mail. The built
SSR worker binds to a reserved loopback port, has a startup deadline and is
stopped with its child command. The outer command also cleans its process group.

HTTP assertions cover guest/member/unverified and separate editor/moderator
capabilities; CSRF; safe upload limits; preview and cross-domain media denial;
draft/public revision isolation; concurrent saves; forced audit-insert rollback;
reserved slug redirects; active and in-use taxonomy; search and page bounds;
actual rendered prose; configured canonical hosts; valid XML; 50-item RSS; and
complete sitemap coverage across 105 articles and five listing eligibility states.
Browser assertions exercise authoring, media, preview, explicit publication,
private changes, stale-form text and focus, taxonomy protection, unpublication,
keyboard actions, mobile overflow, page errors, and content/metadata without
JavaScript. JSON-LD is parsed from the actual script element, not just its prop.

### Failure demonstrations

Six separate source copies compiled and then failed at the intended runtime
assertion. The source checkout and operator data were untouched. Each mutation
was restored before the next case; the temporary installation was removed.

| Requirement | Deliberate violation | Observed rejection |
| --- | --- | --- |
| CNT-001 | Remove editorial permission check from article save. | Moderator POST returns 302; expected 403. |
| CNT-002 | Replace category/tag filters with empty filters. | Private-only tag result count becomes 1; expected 0. |
| CNT-003 | Alias taxonomy permission to editorial permission. | Editor opens taxonomy with 200; expected 403. |
| CNT-004 | Remove SSR enablement from public-page configuration. | Initial response lacks rendered `<strong>Visible proof</strong>`. |
| CNT-005 | Reduce RSS request size from 50 to 49. | Parsed RSS item count is 49; expected 50. |
| CNT-006 | Return raw article body in place of the safe Markdown renderer. | Public body contains a script element. |

The first complete browser run passed. Adding a stronger actual-HTML JSON-LD
assertion then exposed an empty script element: Inertia Head ignores `innerHTML`
as element contents. Changed it to a text child containing the server's escaped
JSON. The corrected full HTTP/browser run passed that assertion. Public titles
use Head's escaped `title` prop, and the browser includes a hostile title to
exercise closing-tag injection as well as Markdown injection.

A seventh isolated copy restored the interpolated `<title>` element while
retaining the hostile-title browser fixture. Both builds and HTTP tests passed,
then the no-JavaScript browser saw a truncated title (`Browser editorial …`)
instead of the full literal text and site name. The new assertion rejected it.
This proves that the title check observes parsed document metadata, rather than
merely finding the hostile marker inside an Inertia data attribute.

### Structural review

`ripwire . --quality-delta` exited 2, with 190 gating rows across the broad scan.
This is not reported as a clean quality pass. Generated client/SSR bundles
dominated its 2,375 total rows. Inspected source rows separately: macro-generated
SeaORM types/routes are exercised by the runtime checks; same-name `read`, `save`
and `detail` collisions included untouched billing functions. Short-horizon churn
records the shared auth/config/test changes rather than a behavioral defect.

Retained separate article/listing entity loaders, validators and permission
checks: they share ORM shapes but enforce different ownership, fields, taxonomy
and publication rules. A generic persistence abstraction would conceal those
boundaries. Reused the existing image persistence/response and Markdown renderer.
The common HTTP helper grew to validate completed connection-task failures;
only a 413 response permits an underlying BrokenPipe during early body rejection.
Other task errors still fail, and pending tasks are canceled. Kept this explicit
test lifecycle rather than hiding unexpected panics. Long migration and HTTP
contract sequences remain sequential because they establish related state and
rollback behavior. These are reviewed trade-offs, not blanket metric exemptions.

`ripwire --edit-check` found all four callers of the shared image response with
no incompatible arity. `--test-gate` exited 4 and named the editorial, directory,
paid lifecycle, provider and foundation tests. Runtime routes it marks untested
are covered by HTTP and browser calls; prior commitment regressions remain
required through Cairn after this implementation is committed.

Inspected desktop published-article, mobile-editor and mobile-taxonomy screenshots.
Controls, labels, content wrapping and navigation were readable without horizontal
overflow. Images in these proofs are tiny synthetic black PNGs, not distribution
assets or purchased reference material. External SMTP, cloud storage, PostgreSQL
and real payment accounts are not established by this local editorial run.

### Corrected case

The final editing-time `node scripts/verify-complete-starter.mjs editorial`
exited 0 after the hostile-title fixture also exposed long-word mobile overflow.
Added wrapping to the editorial reading container. Both locked binaries, frozen
client and SSR builds, the HTTP contract, and the complete Chromium journey passed;
the runner emitted passes for CNT-001 through CNT-006. Inspected the resulting
mobile article screenshot: the literal hostile title wraps inside the viewport.
`git diff --check` also passed. Committed Cairn evidence remains a separate step.

## Shared accent regression — 2026-09-09

Fresh FND-001/002/003 evidence passed. FND-004 then rejected the editorial
implementation's per-shell inline accent: setting the existing root
`--brand-accent` left the admin mark green instead of red. Kept the original
browser assertion. Moved the validated deployment hex value into a shared
`--site-accent` root rule emitted by SiteBrand, and made `--brand-accent` use
that value with its existing palette fallback. Removed the per-shell overrides.
This preserves SSR configuration and the root token's effect on both shells.
The unchanged foundation UI mechanism passed in a disposable installation,
including builds, HTTP denial and Chromium keyboard/shared-token assertions.
`git diff --check` passed. The repeated broad Ripwire scan still reports generated
bundle findings; the source fix changes template bindings and the token fallback.

## Directory navigation regression check — 2026-09-09

The directory HTTP and unit contracts passed, but the browser still selected
the unused `design` category from public navigation. CNT's public taxonomy now
lists only terms attached to eligible public content, so that option correctly
disappeared. Updated the browser to assert its absence, choose populated
`software`, verify the two matching Pagination fixtures, then submit an unmatched
query and verify the mobile empty state. The HTTP assertions still independently
check that direct `design` filtering returns no records. No publication predicate
or permission denial was relaxed.
The corrected complete directory mechanism exited 0: locked builds, listing
unit/HTTP assertions and owner/moderator/public browser journeys all passed.
The broad Ripwire output again contained only generated-bundle rows after
filtering to the changed source paths; it was not treated as a clean scan.


## Delegated administration implementation and mechanism review — 2026-09-09

- Added distinct account-management and audit capabilities to the full starter bundle. Seeded explicit web-guard administrator, moderator and editor role permissions. Account edits replace starter grants, preserve unrelated roles and never change verification. The host grant command reinstates a verified account for recovery.
- Suspension lives outside the authentication model, so whole-row framework password/verification updates cannot restore access. The delegating provider rejects new, existing-session and remembered authentication; the shared public listing predicate checks owner suspension. Human listing, editorial, taxonomy, plan and initial purchase writes recheck access under the account-change lock. Capability changes use the same lock where administrative domain writes occur.
- Account forms have version conflicts and last-active-verified-full-administrator protection. A concurrent self-demotion test permits one commit, rejects the other with the last-administrator validation error, leaves one administrator and writes one audit. Account search and audit pagination use bounded DTOs and joined/batched labels rather than per-row queries.
- Shared transactional audit recording distinguishes the host operator from the target account. Summaries contain safe role/suspension state or changed field names. Historical listing reasons remain in storage but are excluded from audit-page output. Controlled audit failures roll back accounts, taxonomy, article publication and billing plans and leave no successful audit.
- Manual complete administration verifier passed on the final application implementation: Rust format/build, Vue type/client/SSR builds, real HTTP permission and write matrix, password and remember denial, stale-user/password-update suspension persistence, last-admin concurrency, attributable audit/rollback and real browser journeys. Browser proof includes keyboard role assignment, changed capabilities in an existing session, suspension/reinstatement, focused stale-form errors, desktop/mobile account and audit pages, and no overflow/page errors. Inspected account-detail-mobile and audit-desktop screenshots. Log: /tmp/complete-administration-proof.log.
- Failure sensitivity used four isolated source copies, each failing contract assertions rather than setup: remove the last-admin guard (302 instead of 422), overgrant the moderator role (200 instead of 403), bypass provider suspension filtering (missing login redirect), and skip account audit insertion (302 instead of the controlled 500). Each process returned failure; corrected source passed. Logs: /tmp/administration-violation-{last-administrator,role-isolation,suspended-session,transactional-audit}.log. Copies were removed after each run.
- An intermediate audit-test helper failed while cycling a SQLite trigger name. The final helper uses one controlled fault window for all three domain targets, checks each rollback and then verifies the trigger was removed. The final verifier passed; the intermediate run is not claimed as a pass.
- Ripwire impact/uses/edit-check/test-gate and quality-delta were run. The edit check returned 0; test-gate returned 4 with regression obligations, to be discharged by the inherited Cairn checks. Quality-delta returned 2, including generated bundles and 56 source flags. The substantive duplicate audit insertion was consolidated into the shared recorder. Remaining source flags concern typed HTTP/DTO boilerplate, framework trait/macro dispatch, isolated test fixtures, the intentionally serial integration scenario and expected shared permission/navigation changes. No flags were suppressed and no clean quality-tool result is claimed. Source inspection and real HTTP/browser tests cover the trait/route paths that its graph misses. git diff --check passed.


### Foundation regression after predefined role seeding

FND-004 failed because its old fixture assigned the now-seeded administrator role and expected it to carry no permissions. The agreed role now explicitly carries the bundle. The disposable fixture removes its role-permission rows before assigning the same administrator name; the original exact name-only denial and later direct-permission success assertions remain unchanged. No production permission was relaxed. The full manual foundation UI verifier then passed, including browser rendering, denied ordinary access, keyboard dialogs and shared branding (/tmp/complete-admin-foundation-ui-fix.log).


### Preserve moderation reasons while restricting shared audit output

The inherited directory check caught a real regression: replacing listing audit summaries with field names had discarded the required suspension and reinstatement reasons. A dedicated nullable private_reason column now retains those details in the same audit transaction, with actor and action. The migration copies historical listing summary text into that detail without overwriting the historical summary; rollback retains newly recorded reasons. The model excludes private_reason from serialization, and the explicit shared audit DTO still redacts listing free text. Directory assertions retain the exact required reason values in the dedicated field and verify their absence from the shared audit response. This satisfies both the recorded-reason and safe-summary contracts; the failure was repaired in code.

The first manual run passed HTTP but exposed an independent browser-test timing error: its two-card assertion could match the old document before a search navigation finished, so the next input was overwritten by navigation/hydration. It now waits for the exact submitted query URL and settled document before the next search. The final whole directory verifier passed Rust format/build, media tests, HTTP and browser journeys (/tmp/complete-private-reason-directory-final.log). A separate cargo check --locked --tests passed with the serialization guard (/tmp/complete-private-reason-check.log); only existing shared test-helper dead-code warnings remained. Focused audit edit-check and git diff --check returned 0.

## Directory hydration regression — 2026-09-09

Cairn DIR-001 through DIR-007 at 20260909T213115953Z failed in the browser
keyboard journey: Compact remained false. The HTTP and moderation-reason
assertions had passed. Source inspection confirmed SSR emitted enabled buttons
while main.ts still awaited the locale catalog before mounting event handlers.
Disable layout controls until onMounted. Native search remains available.
The browser test now holds JavaScript behind an explicit promise, asserts the
SSR control is disabled, releases the bundle, then checks activation and compact
layout. The later keyboard assertion waits for the submitted document and the
enabled control. This fixes a user-visible lost-click window.

The first manual run passed the delayed-bundle check but exposed a new test URL
expectation missing the native form's empty category parameter. Corrected the
expectation to the actual submitted URL. The full corrected directory verifier
passed Rust formatting/build, Vue type/client/SSR builds, four media tests, HTTP
contracts and the production browser journey (/tmp/directory-hydration-final.log).
An isolated copy with only the readiness guards removed failed specifically at
toBeDisabled with Received: enabled (/tmp/directory-hydration-negative.log).
The negative harness exited zero only after asserting that failure. No violating
source entered this checkout. Ripwire test-gate exited zero; quality-delta exited
two with 133 generated-output findings, none outside public assets or generated
SSR. Edit-check could not resolve the Vue component by Index; source inspection,
Vue checks and the runtime browser assertions cover this small change. Git diff
whitespace check passed. This is a focused regression repair, not final acceptance.

## Adoption mechanism construction — 2026-09-09

Reviewed KIT-001 through KIT-005 against the new notification modules and all
moderation, checkout, cancellation and fulfillment call sites; demo seed/schema;
site configuration and private media implementation; owner status components;
LICENSE/THIRD_PARTY_NOTICES; README; and the actual adoption runner/tests.
The grouped baseline at 20260909T214340258Z failed because adoption verification
was deliberately unimplemented. No acceptance criterion was relaxed.

The runner builds both locked binaries and frozen Vue client/SSR assets in a
source-only disposable install. It runs five real demo-console tests, executes
README commands and actual database/media/key backup/restoration, exercises HTTP
and transaction notification contracts and separate configuration/media processes,
then opens the real production application in Chromium with the SSR worker.
License and notice files are declared inputs and included in source snapshots.
No local environment, operator database, credentials or external provider account
is copied. The helper refuses a preexisting environment or installation database.

The first combined run failed because repeating directory:categories advanced
SQLite's internal listing_categories sequence from 6 to 10. Instrumented SQL dumps
showed no other changed line. Seed comparisons now exclude exactly that internal
category counter; all schema, business records, other sequences, migration replay
and backup/restore comparisons remain exact. The demo seed itself rolls back its
repeat lock and preserves all persisted data. It refuses unknown/production modes
without the explicit override, detects reserved-key collisions, serializes two
seeds and rolls back a controlled late write failure. It generates one unverified,
unprivileged synthetic owner with a random unprinted password and no payment or
notification calls. Existing operator records and credentials are preserved.

Notification intents share the domain transaction and a unique event identity.
Tests make intent insertion fail and require moderation and payment/receipt writes
to roll back; replay must retain one intent. A new console process sees the same
pending rows. Controlled mail failure retains safe failure codes and pending work,
respects backoff and eight attempts, and explicit retry delivers through Suprnova's
captured transport. Two workers cannot own the same live lease; expired leases
recover. Success acknowledgment retains the intent. A lost SMTP acknowledgment
can redeliver, as documented. Owner DTOs are independently scoped and limited to
20 rows. Reasons are plain escaped text. Payment messages contain no raw payloads,
credentials or provider customer references. Internal ordering uses microseconds
while owner timestamps remain Unix seconds, preventing misleading order for
several checkout steps in one second. Live lease ownership is rechecked before
send. Payment wording was separated from fulfillment state-transition logic.

Configured public/admin branding and metadata passed raw HTML and JavaScript-off
browser checks. Invalid name/accent/origin/logo values fail with the relevant
configuration field. A fresh process reads the encoded image persisted earlier.
Desktop/mobile owner history and both shells fit and have no browser exceptions;
viewed the owner mobile artifact and corrected one undefined border token.
The browser restarts the application and requires unchanged owner history.

Distribution review compared tracked source hashes with the Pulsar and purchased
Laravel references, excluding dependencies/builds. Six exact Pulsar file matches
are listed with its full MIT notice; no exact Laravel file match or tracked stock
photo/font/logo assets were found. This hash inventory supplements source review;
it does not alone establish provenance. MIT uses the established repository owner
Shawn McAllister. External SMTP, PostgreSQL, cloud storage, actual Stripe/Paddle
accounts and proxy/hosting deployment are explicitly unverified local boundaries.

Failure sensitivity was demonstrated in isolated source copies using unchanged
assertions. Omitting intent recording, discarding delivery errors, ignoring the
configured name and bypassing production demo refusal each compiled and failed
its targeted runtime test (exit 101, named test FAILED). Removing LICENSE failed
the distribution check; replacing the documented notification command with an
unknown command failed actual README execution. The harness required these exact
failure modes, removed every copy, and never put violating source in this checkout.
Logs: /tmp/adoption-negative-{outbox,delivery,branding,demo,license,guide}.log;
summary /tmp/adoption-negative-summary.log. All six demonstrations passed.

The full corrected adoption runner passed at /tmp/adoption-final-source-proof.log:
formatting, both binaries, Vue type/client/SSR builds, five demo tests, README
commands/restoration, notification/configuration/storage tests and browser journey.
A previous focused runner deliberately skipped install/demo stages to debug the
notification/browser work; its group labels are not acceptance evidence.
Ripwire edit-check(record) and git whitespace check passed. Quality-delta exited
two (158 findings, many ignored generated bundles); the substantive growth in
fulfillment complexity was resolved by separating mail wording. Remaining source
flags are typed ORM/DTO query similarities, schema/trait/command macro dispatch,
explicit test fixtures, local browser lifecycle helpers and required cross-domain
notification calls. No suppression or clean-gate claim is made. Test-gate exited
four, names six regression tests and misses framework/controller dispatch; all
named regressions must pass again before final commitment acceptance.

The final focused notification contract also passed after adding direct refund
and open-dispute assertions. Each reconciled event must have its own retained
notice with the corresponding safe explanation, alongside the settlement replay
and transaction rollback checks. /tmp/adoption-refund-dispute-positive.log records
one passed test, zero failed; the external driver asserted exit zero. Its reused
console summary label says "violating copy" but this positive run replaced no
source behavior and checked success, not an expected failure.

## Directory selector integration repair — 2026-09-09

Cairn DIR-001..007 at 20260909T221240250Z failed only in the browser: the
rejection reason now appears in review feedback and retained notification history,
so the old unrestricted substring locator matched twice. HTTP and media checks
passed. Scoped the original assertion to the review-feedback notice and added an
independent assertion against the named notification region. Both intended
surfaces must retain the exact reason. No application source changed.

An initial editing command missed the old locator's explicit exact:false option;
its replacement assertion stopped without changing the file, and the already
started manual verifier repeated the same known failure. After replacing the
actual locator, the full verifier passed formatting/build, Vue type/client/SSR,
media, HTTP and browser journeys (/tmp/directory-notification-selector-final.log).
Node syntax and git whitespace checks passed. Ripwire test-gate exited zero;
quality-delta exited two with 145 generated-only findings and no source findings.


## Final review finding — stale payment notices — 2026-09-09

Reviewed payment notification transaction boundaries, unique event identities,
delivery leases and retries, owner scoping and safe message content against the
fulfillment observation ordering rules. All 43 requirement checks currently pass,
but the review found a KIT-001 correctness gap: adverse() rejects an older
same-source observation while apply() forwards the original rejected fact to
notification wording. An older open-dispute read committed after a newer resolved
read can therefore produce a false newest suspension notice. Cancellation has
the same unfiltered-fact path. Existing sequential and replay checks miss this
overlap. Final acceptance is withheld pending a separate repair and controlled
reverse-commit regression. No application code changed during this review.
