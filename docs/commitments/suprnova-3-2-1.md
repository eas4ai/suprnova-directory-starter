# Suprnova 3.2.1 upgrade and starter 1.0.1 release

Status: Agreed 2026-10-05
Requirements: SEO-001, SEO-002, SEO-003, SEO-004, SEO-005, SEO-006, FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, UI-001, UI-002, OVR-001, OVR-002, OVR-003

## Outcome

The developer requested the same upgrade-and-release workflow used for Pulsar and Nebula. Target the exact Suprnova 3.2.1 release for the framework, Stripe and Paddle adapters, and the Markdown prerequisite. Release this starter as 1.0.1. Preserve all 54 completed starter requirements and the operator database.

## Boundary and completion

Update dependency manifests/locks, exact-pin verification, active specification pins, and necessary operational documentation. Change application code only for demonstrated compatibility failures. Commit candidates before formal checks; record fresh passes for every inherited requirement and a final production review. Commit and push main with an annotated v1.0.1 tag after verification. No deployment, traffic-reporting implementation, real payment charge, or operator-data migration is requested.

## Todo

- Done: align and verify exact framework/Markdown pins, client/SSR and both binaries; review revised mechanisms and deliberate failures.
- Done: verify all 54 inherited requirements on committed inputs, verify 108 output hashes and record the production release review.

Publication follows the completed verification commitment: push main and create annotated v1.0.1, then verify both remote references. This operation is authorized by the developer; its outcome is reported after the push.
