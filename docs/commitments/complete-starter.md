# Complete starter

Status: Agreed 2026-09-09
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005

## Outcome

Complete the selected free MIT starter with articles, taxonomy, crawler-readable
public pages, feeds, delegated administration, durable notifications, configurable
branding/storage, safe demo data and verified adoption documentation.

Contracts live in docs/spec/content.md, docs/spec/administration.md and
docs/spec/adoption.md. This commitment follows paid-directory and preserves its
payment and visibility guarantees.

## Execution and verification

1. Complete distinct administrative capabilities and audit records using Suprnova RBAC.
2. Complete article publishing and taxonomy with direct-request permission checks.
3. Establish initial-HTML content/metadata, RSS, sitemap and robots behavior. Reuse
   the publication predicate across every public output.
4. Complete notification delivery/retry, branding, media configuration and demo seed.
5. Add MIT/provenance records and verify the clean-install and operations guide.
6. Demonstrate failures, rerun prior commitment regressions and record the final review.

Proposed mechanisms: editorial-workflow, editorial-discovery, taxonomy, public-seo,
public-feeds, editorial-content, administration-permissions, account-administration,
administrative-audit, notification-delivery, starter-configuration, demo-seed,
distribution-inventory and complete-starter-install. Each mechanism declares inputs
before implementation and establishes concrete requirement results on committed code.

Local verification uses disposable SQLite, captured mail, temporary media and
provider fixtures. Browser journeys cover desktop/mobile layouts, keyboard access,
errors and the public pages with JavaScript disabled. Actual provider accounts,
external SMTP and cloud storage require separately reported operator smoke tests.

## Boundary and completion

Implementation may update application manifests/locks, schema, domain modules,
commands, routes, frontend, server rendering, configuration, tests, verification
scripts, license/notices and documentation. Preserve framework ownership of
authentication, RBAC, mail, storage and provider protocols.

Done requires every named requirement Agreed, current passing evidence, prior
commitment regressions, demonstrated failure sensitivity and a clean production
self-audit. The documentation states exactly which environments were verified.
Deployment, NOWPayments, AI, Keygen and the other excluded candidates in adoption.md
are not implied by completion of this starter.
