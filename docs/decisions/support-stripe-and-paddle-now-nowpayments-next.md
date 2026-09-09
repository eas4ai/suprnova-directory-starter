# Support Stripe and Paddle now; NOWPayments next

Level: Consequential
Decided by: Shawn
Rests on: docs/feature-map.md, docs/architecture-proposal.md
Would be wrong if: The starter supports only one selected gateway, or the NOWPayments adapter is treated as part of the current starter implementation.

## Decision

On 2026-09-08 Shawn selected Stripe and Paddle admin and configuration support for the starter. This replaces the proposal to select one initial provider. Shawn also requested a separate next-iteration commitment to create a NOWPayments adapter for the next Suprnova release, using Nation X as implementation evidence. The release number is not specified. Detailed configuration behavior and acceptance mechanisms remain Draft until reviewed.

## Realized by

- a4cadb95e469f8467953e71c32a36035c63806e6 Establish Cairn specifications and draft foundation commitment

This scope decision is realized by the two-provider specification and separate NOWPayments commitment. Runtime provider administration and the framework adapter remain future implementation; no provider behavior is claimed complete.
