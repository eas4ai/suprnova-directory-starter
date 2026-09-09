# Remaining starter draft review

Reviewed 2026-09-09 following the developer's request to complete the remaining starter.

## Existing contract and evidence

- `cairn wake` reports provider-administration Done. Its six requirements have fresh
  passes and a clean review; the five foundation regressions also passed.
- docs/spec/roadmap.md already orders paid-directory then complete-starter.
  docs/feature-map.md and docs/architecture-proposal.md describe the broader product
  direction but explicitly leave detailed policy Draft. Their original inventories
  are historical, not current feature or build status.
- src/routes.rs has account and provider administration routes, but no listing,
  article or fulfillment route. The completed code does not already implement the
  next delivery's requirements.
- The licensing backlog is partly resolved by the administrator command. MIT source
  licensing and provenance remain necessary distribution work in KIT-004.

## Contracts attacked

The review traced owner submission through moderation, checkout, payment evidence,
publication, revision, suspension and recovery. It checked for payment-before-review,
unreviewed content changes, test-purchase publication and inconsistent visibility
across direct URLs, search, media, metadata and feeds. DIR-003 through DIR-007 and
PAY-002/003/011 keep these independent gates explicit.

It checked free, one-time and recurring terms; immutable purchase snapshots; ambiguous
network failures; duplicate and out-of-order events; expiration, cancellation, refund
and dispute outcomes. It distinguishes full from partial refunds and payment recovery
from moderator reinstatement. Plan upgrades, trials and proration are excluded to keep
the initial billing contract bounded, while retaining both one-time and recurring plans.

It checked editorial drafts, taxonomy references, capability separation, existing-session
account suspension, last-administrator recovery, notification retry semantics, asset
provenance and adoption checks. Initial HTML is required for SEO; a client bundle build
does not establish crawler-visible content. Exactly-once SMTP delivery is not promised.

Each normative block has an observable falsifier and named proposed mechanism.
These are proposed checks, not existing evidence. The two commitments can be verified
in order without treating future editorial UI as a prerequisite for paid-directory.
The final starter commitment includes all prior regressions.

## Original released adapter findings

Before the adapter repair, inspected exact tag `v1.3.7` in the sibling Suprnova checkout:

- framework/src/payments/dto/session.rs declares idempotency keys and metadata on
  StartSessionRequest and distinguishes complete-but-unpaid from settled checkout.
- crates/suprnova-payments-stripe/src/checkout.rs does not use request idempotency
  keys or metadata. Its session_status implementation can retrieve known sessions.
  Request encoding needs a wire-level test; serde_json serialization tests alone do
  not establish form encoding accepted by Stripe.
- crates/suprnova-payments-paddle/src/checkout.rs sends price references and customer,
  but does not use request idempotency keys or metadata and does not implement
  session_status. The trait default returns NotSupported. Its client is crate-private.
- crates/suprnova-payments-paddle/src/event_map.rs maps transaction.billed to InvoicePaid
  and adjustment-created/updated events to PaymentRefunded. Domain fulfillment must
  inspect actual settlement/refund outcome, not blindly trust those neutral labels.

These observations do not imply the providers themselves support identical retry
features. PAY-016 requires truthful adapter behavior and safe ambiguity handling.
The developer subsequently chose framework repairs and authorized merging and
pushing them to Suprnova main. These tag-specific findings remain historical
evidence; they are not the current behavior of the repaired main branch.

## Review corrections and agreement boundary

Removed an initial overpromise to verify the provider catalog before checkout: the
pinned provider-neutral surface does not expose that operation, and the existing
configuration contract explicitly accepts manually entered mappings. Fulfillment
still checks immutable purchase terms against authenticated available evidence and
rejects mismatches. External sandbox testing remains separately identified.

Agreement covers the requirement text, falsifiers and proposed operating rules in
the five domain files plus the two commitment boundaries. The developer's broad
request selects completion of the remaining roadmap, but it does not establish
agreement with payment-policy details that had not yet been written. No Draft was
promoted and the current completed commitment remains unchanged pending that agreement.

## Resumption after adapter repair

Suprnova commit `a77f3d7c` repairs checkout metadata, form encoding, Stripe
PaymentIntent retrieval, Paddle transaction retrieval, settlement/refund event
classification, capture timestamps and bounded Paddle requests. Commit `107e6e7a`
synchronizes the manual translations. Both are on the pushed main branch.
The default push gate passed on `107e6e7a`, including 7,998 workspace tests and
860 browser tests. The later full release-gate run was interrupted after the
developer clarified that it had not been requested. No release tag was created.

The starter still uses v1.3.7. The paid-directory draft proposes the exact revision
`107e6e7a122d5145160ea1547ca90ddc37459c27` for the framework and adapters, with a
lockfile update and affected foundation/provider regressions. This explicitly
amends the old dependency constraint only after agreement. A moving main branch
or local sibling path is not the proposed distribution dependency.

Reviewed the draft again for cross-domain ordering: listing categories can be
seeded before the later taxonomy UI; revision and entitlement states remain
independent; account suspension in complete-starter adds to the common public
visibility rule; notification delivery is later work but owner status is available
in paid-directory. No new provider behavior or automatic retry guarantee is assumed.
The remaining agreement concerns the written product policies and their falsifiers,
not another adapter repair or release gate.
