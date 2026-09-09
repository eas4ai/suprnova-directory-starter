# Starter foundation

Status: Agreed 2026-09-08
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005

## Outcome

A fresh checkout builds and runs on Suprnova v1.3.7, initializes disposable account data, completes account flows, and presents the selected public/admin UI foundation. Relevant Pulsar behavior and tests may be adapted within this outcome. No wholesale foundation import is implied.

## Records and formats

Specification: docs/spec/foundation.md. Mechanisms: .cairn/mechanisms/foundation-*. Review: .cairn/reviews/starter-foundation.md. Check receipts and logs: .cairn/evidence/. Commands report each established requirement as `cairn: FND-001: pass` or `cairn: FND-001: fail`, substituting the actual identifier. Missing results remain unverified.

Implementation may affect Cargo manifests/locks, src, cmd, frontend, tests, public assets, configuration examples, README, handoff.md and verification scripts. The setup review reads handoff.md for prerequisite and verification-limit consistency with the setup guide. Select the account schema and disposable test engine from framework/scaffold evidence before implementation and record any meaningful choice.

## Checks and completion

The five mechanisms correspond to the five requirement blocks. Each new check gets a safe violating fixture and a corrected fixture, with the failure cause recorded in the review. Build success is not account-flow or UI evidence. External services and skipped tests remain explicitly unverified.

Done requires developer-confirmed requirements and falsifiers, implemented mechanisms with fresh passing Cairn evidence, and a review covering behavior the checks might miss. Provider configuration, listing functionality and NOWPayments are outside this first commitment.
