# Paid directory review

## Mechanism review: FND-001 revised dependency contract

Examined foundation-build and scripts/verify-foundation.mjs verifyPin, build and
snapshot handling against the confirmed exact framework revision. The mechanism
still requires the old tag v1.3.7 and therefore rejects the agreed dependency.
It also checks only the framework dependency; the corrected assertion must check
both payment adapters and all locked Suprnova workspace packages. This is a
mechanism mismatch requiring a separate implementation action. No runtime code
was changed during this review. Preserve disposable builds and unchanged lock checks.

The separate repair now checks the framework, Stripe and Paddle dependency sources,
and every locked Suprnova package, against the agreed immutable revision. The safe
violating case ran the revised build check against the previous tag manifest and
failed at the exact-source assertion before compilation. After updating the three
dependencies and resolving Cargo.lock, the same check passed: both Rust binaries,
frozen Bun installation, Vue type checking, client and SSR builds, generated
artifacts and unchanged dependency locks. The disposable source copy was removed.
Ripwire edit-check found no verifyPin signature change or incompatible known caller.
This establishes the amended build contract; paid behavior remains to be built.

## Mechanism review: PAY-001 repaired adapter integration

Compared the provider declaration, full verifier, HTTP/persistence contract and
Chromium administration journeys with revised PAY-001. Only the immutable dependency
changed in this requirement. The inherited FND-001 check establishes that exact
revision; the provider mechanism constructs both actual adapters and exercises
authorized saves, reads after restart, direct-request denial and both provider forms.
No mechanism mismatch was found. No application code changed during this review.

The full verifier passed against the repaired revision, including controlled
constructor failure and its corrected case, invalid keys, failed writes, competing
saves, deployment-key restart probes and desktop/mobile browser interactions.
The earlier disposable permission-removal mutation and its failing 200-versus-403
assertion are recorded in docs/provider-administration-mechanism-review.md; the
unchanged denial assertions passed in this run. These are local synthetic checks,
not external provider authentication or a payment-processing claim.

## Reference findings

Larafast Directories is the selected working example. Its category sidebar/search,
card and compact results, submission form, owner status dashboard, listing-specific
plans and admin workspace guide the independently authored Vue implementation.
Product.php, CreateProduct.php, Products.php, ProductController.php, dashboard.blade.php
and plans.blade.php in that reference establish the flow. The reference primarily
uses Lemon Squeezy and pays before moderation; the agreed starter uses Stripe and
Paddle and reviews before checkout. Its unconstrained active/draft booleans and
browse/detail eligibility mismatch are not the agreed domain model.

Review remains open until implementation and failure-sensitive verification finish.

## Directory implementation and mechanism attack

The directory verifier builds both binaries, checks formatting, runs four image
boundary tests and the HTTP/database contract, then builds Vue client/SSR assets
and runs Chromium owner, moderator and public journeys in disposable workspaces.
The corrected implementation passed this full editing-time check. Browser checks
include reopening a saved draft after server restart, upload, rejection with reason,
revision and approval, stale editing with preserved input, public search and detail,
keyboard controls and a narrow viewport. Desktop moderation and mobile directory
screenshots were inspected; the pages use the existing semantic theme and fit their
viewports. These runs are not committed Cairn receipts.

Eight separate source copies changed one behavior each while retaining the same
integration assertions. Every copy compiled and failed at runtime with exit 101:

| Requirement | Safe source mutation | Observed failure |
| --- | --- | --- |
| DIR-001 | Remove the private listing owner filter. | Other owner's draft GET returned 200 instead of 404. |
| DIR-002 | Enable unsafe Markdown HTML. | Rendered description contained a script element. |
| DIR-003 | Permit moderation of an unsubmitted draft. | Decision returned 302 instead of 422. |
| DIR-004 | Build the public card from the current proposal pointer. | Eligible approved detail became unavailable after a proposed edit. |
| DIR-005 | Permit test-mode entitlements in the public predicate. | Search exposed a test purchase. |
| DIR-006 | Raise the page-size ceiling to 1000. | A 101-item request returned 200 instead of 422. |
| DIR-007 | Remove suspension from the public predicate. | Search exposed the suspended listing. |
| DIR-008 | Remove the approved unpaid owner's checkout action. | Status returned `none` instead of `checkout`. |

The disposable driver is retained as scripts/demonstrate-directory-failures.mjs.
It rejects compile failures and timeouts as demonstrations, removes each copy and
terminates any surviving process group. It never changes the application source.

DIR-008's status assertions are covered here, but its complete checkout journey
is deliberately assigned to the paid-lifecycle mechanism. The plans link alone
does not establish that the owner can check out. Payment integration, additional
metadata/feed surfaces and the final commitment review remain outstanding.

Ripwire quality-delta exited 2 and test-gate exited 4; neither is recorded as a
pass. Its scan included ignored generated bundles and marked ORM/handler macro
types and test functions dead despite compiled and exercised paths. Reviewed
source findings include explicit owner/moderator query similarities, domain-local
validation errors, controller extraction/redirect patterns, the sequential global
runtime integration test, the schema migration and owner status branching. These
keep distinct authorization and state rules visible. A swallowed browser startup
probe error was changed to preserve the last error as the timeout cause. The test
map names account, foundation UI and provider regressions; committed Cairn runs
must establish them after this implementation. Existing generated bundles are
excluded from the distribution inventory and are not source changes.
