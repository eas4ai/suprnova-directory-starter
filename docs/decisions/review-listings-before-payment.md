# Review listings before payment

Level: Consequential
Decided by: Shawn
Rests on: docs/architecture-proposal.md, handoff.md
Would be wrong if: The developer requires checkout before moderation, or an owner can be charged for a submission before approval.

## Decision

On 2026-09-08 Shawn confirmed review before payment. An owner submits a listing, an administrator approves it, and the owner then checks out. Verified payment establishes paid publication eligibility; approval alone does not establish payment. This avoids charging for rejected submissions and adds a checkout handoff after approval. Provider selection, entitlement policy, and requirement-level falsifiers remain open.

## Realized by

(none yet: recorded, not built)
