# Directory and moderation

Status: Agreed 2026-09-09
Prefix: DIR

This domain owns listings and their approved public revisions. Payment eligibility
is owned by payments.md. The operating rules below were confirmed for the remaining starter.

[DIR-001] The starter MUST let a verified owner create, save, edit and archive their own listing drafts through the owner UI.
Falsifier: A valid draft cannot be reopened after restart, another owner can read a private draft or mutate it by direct request, or an unverified account can submit a mutation.
Mechanism: Proposed directory-ownership HTTP and browser journeys with two owners, a guest and an unverified account.

[DIR-002] The starter MUST validate listing content and media before committing a revision.
Falsifier: Invalid fields or unsafe media are accepted, a failed submission leaves a partial revision, unapproved media is publicly accessible, or rendered content executes injected script.
Mechanism: Proposed directory-content boundary tests, media fixtures and browser injection checks.

[DIR-003] The starter MUST let an authorized moderator approve or reject a submitted revision with a recorded decision.
Falsifier: A draft can be approved without submission, a non-moderator can decide through a direct request, a rejection lacks an owner-visible reason, or a stale decision overwrites a newer revision.
Mechanism: Proposed directory-moderation state, permission and concurrent-decision tests plus review-queue browser journeys.

[DIR-004] The starter MUST retain the last approved public revision while an owner's proposed changes await review.
Falsifier: Unreviewed changes appear on any public surface, rejecting an edit removes the previously eligible revision, or approval publishes content other than the revision reviewed.
Mechanism: Proposed directory-revisions tests across edit, submit, reject, approve and competing edits.

[DIR-005] The starter MUST restrict every public listing surface to the same current publication eligibility rules.
Falsifier: A draft, rejected initial submission, archived or suspended listing, expired paid listing, or test-mode purchase appears in search, category, detail, metadata, feeds, media or sitemap output.
Mechanism: Proposed directory-visibility matrix over public HTTP routes with a controllable clock and payment fixtures.

[DIR-006] The starter MUST provide bounded public search, category filtering and pagination with stable listing URLs.
Falsifier: Results ignore the requested query/category, pagination repeats or skips unchanged records, an unbounded result request succeeds, or renaming a listing breaks its previously published URL without a redirect.
Mechanism: Proposed directory-discovery database/HTTP tests and desktop/mobile browser journeys.

[DIR-007] The starter MUST let an authorized moderator suspend and reinstate a listing without granting payment eligibility.
Falsifier: Suspension leaves a public surface visible, a non-moderator can suspend a listing, reinstatement publishes an unpaid or expired listing, or the action lacks an audit reason and actor.
Mechanism: Proposed directory-suspension permission and visibility tests across paid, free and expired listings.

[DIR-008] The starter MUST show owners their listing's moderation, payment and publication status with the available next action.
Falsifier: An owner cannot distinguish pending review from awaiting payment, an approved unpaid listing has no checkout action, a rejected revision hides its reason, or another owner's status is disclosed.
Mechanism: Proposed directory-owner-dashboard HTTP and keyboard/browser journeys through the complete lifecycle.

## Agreed operating rules

- Listing fields: title (1–120 characters), summary (1–280), Markdown description
  (1–20,000), absolute HTTP(S) destination URL (at most 2,048), one to five active
  categories, and an optional image with alternative text. Reject URL credentials
  and executable schemes. Do not fetch owner-supplied URLs on the server.
- Accept JPEG, PNG and WebP images up to 5 MiB and 4,096 pixels per dimension after
  decoding. Re-encode accepted images, strip metadata, use generated storage names,
  and reject SVG and invalid image data. Draft media stays private until approved;
  public media requests follow listing eligibility. Local disk is the supported
  default; operators can configure framework storage through documented settings.
- Owners archive rather than hard-delete records with moderation or payment history.
  Editing a live listing creates a proposed revision. Rejection permits revision and
  resubmission. Only the exact submitted revision can receive a decision.
- A public listing requires an approved revision, no owner archive or administrator
  suspension, and a valid free or live-mode paid entitlement. A test purchase is
  visible only in authorized owner/admin status pages, never the public directory.
- Stable slugs survive title edits. Deliberate administrator slug changes retain
  redirects; old slugs are not reassigned to unrelated listings.
- Search is case-insensitive over title and summary, with an optional category and
  deterministic newest-first ordering plus identifier tie-break. Query length is
  at most 200 characters; page size defaults to 24 and cannot exceed 100. No external
  search service is required.
- Public descriptions disable raw HTML and sanitize rendered Markdown and links.
  Categories are managed through the complete-starter taxonomy UI; paid-directory
  includes an idempotent operator seed/command for the categories its flow needs.
