# Prefer current publishing eligibility in owner status

Level: Judged
Decided by: Codex
Rests on: DIR-008 PAY-013
Would be wrong if: An older refunded period hides a valid current period, or test eligibility is exposed publicly.

## Decision

Owner dashboard status must prefer a currently valid free or live entitlement, then a currently valid test entitlement, before historical expired or adverse periods. The existing query only prioritizes current free/live periods; in test mode a more recently updated old refund can incorrectly replace the current status. Add a focused regression to the existing lifecycle contract and correct the owner projection ordering without changing public eligibility or fulfillment policy.

## Realized by

(none yet: recorded, not built)
