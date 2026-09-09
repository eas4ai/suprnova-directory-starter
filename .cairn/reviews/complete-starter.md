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
