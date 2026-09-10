# Useful admin overview

Status: Agreed 2026-09-10
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, UI-001, UI-002, OVR-001, OVR-002, OVR-003

## Outcome

Replace the placeholder overview with actual publication counts and moderation work, enforce permissions on data and actions, and group the existing navigation. Charts arrive in traffic-reporting after their data sources exist.

Contract: docs/spec/admin-overview.md. Preserve every inherited starter requirement.

## Execution order

1. Read the administration and publication decisions and map the overview, permissions and public eligibility queries.
2. Declare an admin-overview mechanism with application, frontend, migration, dependency and verification inputs before implementation.
3. Demonstrate the placeholder/count and permission failures using disposable fixtures; implement counts, queue links and grouped navigation.
4. Run direct HTTP permission tests and desktop/mobile keyboard journeys in both appearance modes.
5. Commit the candidate, refresh inherited checks including the stale README build evidence, demonstrate failure sensitivity and record the final production review.

## Boundary and completion

Application manifests/locks, configuration, migrations, domain modules, routes,
frontend/SSR, commands, tests, verification scripts and documentation may change
only as needed for the named requirements. Declare complete mechanism inputs
before implementation. Keep framework authentication, RBAC and payment protocols
in Suprnova. Preserve the operator database; verification uses disposable data.

Done requires fresh passing evidence for all named requirements, meaningful
failure demonstrations, browser checks where specified, and the recorded final
self-audit. No deployment or unrelated provider integration is implied. Cairn
stops at Done; activating the following commitment remains the developer's choice.

## Todo

- In progress: establish the commitment and its falsifiable specification.
- Pending: declare and implement the overview mechanism and application changes.
- Pending: complete committed verification, inherited checks and final review.
