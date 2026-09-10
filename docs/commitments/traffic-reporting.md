# Traffic, revenue and engagement reporting

Status: Agreed 2026-09-10
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, UI-001, UI-002, OVR-001, OVR-002, OVR-003, SEO-001, SEO-002, SEO-003, SEO-004, SEO-005, SEO-006, TRF-001, TRF-002, TRF-003, TRF-004, TRF-005, TRF-006, TRF-007, TRF-008

## Outcome

Deliver built-in traffic and link collection, the dedicated Traffic section and the requested traffic, revenue and engagement dashboard charts.

Contract: docs/spec/traffic-reporting.md. Preserve every inherited starter requirement.

## Execution order

1. Declare collection, link, report, payment/engagement, permission and browser mechanisms before implementation.
2. Record the exact timezone, custom-range bound, retention window, event identity and rollup policies as decisions with observable failure conditions.
3. Implement bounded public-event storage, collection, replay protection, cleanup and daily aggregation; prove that prefetch/private/Markdown traffic is excluded.
4. Implement server-resolved listing clicks and validated published-prose/internal-link events with fail-open navigation tests.
5. Build Traffic Overview, Sources, Pages and Links with stable content identities, filters, drill-downs and CSV export.
6. Implement confirmed-payment, refund/dispute and successful server-action series; prove live/test separation, currency isolation and idempotency.
7. Integrate all three dashboard chart groups, period comparison, truthful empty/error states and accessible tables.
8. Run real browser collection/report journeys and hostile/permission fixtures; refresh inherited checks and record the final production review.

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
