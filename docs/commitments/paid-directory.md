# Paid directory

Status: Draft
Requirements: DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016

## Outcome

A verified owner submits a listing, a moderator reviews its exact revision, and
the owner selects a free or paid plan after approval. Verified live payment permits
publication; visitors can browse eligible listings. Edits, failed payments,
cancellations, refunds, disputes and interrupted processing have explicit outcomes.

Directory rules live in docs/spec/directory.md; payment rules and policy defaults
live in docs/spec/payments.md. Provider administration is the completed predecessor.

## Execution and verification

1. Integrate the repaired Suprnova revision proposed in payments.md and establish
   the adapter wire contracts. Do not rely on mocks to hide adapter limitations.
2. Build listings, revisions, media, ownership and moderation with direct-request
   and public-visibility checks. Supply the minimal operator category setup.
3. Build plans, checkout attempts, authenticated evidence and idempotent fulfillment.
   Exercise Stripe and Paddle boundaries and keep test fulfillment private.
4. Build lifecycle recovery, status pages and complete browser journeys.
5. Demonstrate safe violations, collect committed Cairn evidence, rerun affected
   foundation/provider checks, and record an independent final self-review.

Proposed mechanisms: directory-ownership, directory-content, directory-moderation,
directory-revisions, directory-visibility, directory-discovery, directory-suspension,
directory-owner-dashboard, paid-plans, paid-checkout, paid-fulfillment, paid-lifecycle,
paid-reconciliation and adapter-wire-contracts. Implementations may combine related
checks under one declared command while reporting each established requirement.
Declare every input before implementation and commit before recording evidence.

Use disposable SQLite data, synthetic provider fixtures and local HTTP services in
workspace scratchpads. Clean up the processes and source copies. Real provider
sandbox acceptance is a separate documented operator exercise, not a local test claim.

## Boundary and completion

Application changes may include manifests/locks, framework integration, schema,
domain actions, routes, controllers, operator commands, frontend pages/styles/types,
media handling, scripts, tests and documentation. Framework adapter repairs were
authorized and completed separately. The proposed exact dependency revision is
part of this draft agreement; no further sibling changes or release are implied.

Done requires every named requirement Agreed, current passing evidence, demonstrated
failure sensitivity and a clean final review. This commitment does not require the
later editorial or delegated administration UI. The remaining starter follows in
complete-starter after its draft is agreed. No production deployment is included.
