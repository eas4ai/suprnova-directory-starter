# Keep proposed listing revisions separate from publication eligibility

Level: Judged
Decided by: Codex
Rests on: DIR-001 DIR-002 DIR-003 DIR-004 DIR-005 PAY-003 PAY-012
Would be wrong if: Concurrent edits can replace reviewed content, private media can bypass publication rules, or payment evidence cannot update eligibility transactionally.

## Decision

Use relational listing, immutable revision, revision-category, media and audit records. Listing rows hold the current proposed and approved revision identifiers and an optimistic version. Owner saves create a new revision; submission and moderation compare the exact current revision and version in one transaction. One shared publication query combines approved content, archive and suspension state, and a separate durable entitlement. Payment purchases later reference listings and preserve immutable terms. Private media is stored outside public assets and served only after the same listing eligibility or explicit owner/moderator authorization. Bounded HTTP and browser tests exercise competing edits, cross-owner requests and the public visibility matrix. This implements the confirmed policies without changing their scope.

## Realized by

(none yet: recorded, not built)
