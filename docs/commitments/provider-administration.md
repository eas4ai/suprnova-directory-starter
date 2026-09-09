# Provider administration

Status: Draft
Requirements: PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008

## Outcome

An operator provisions an administrator, then configures Stripe and Paddle through the administration UI. Credentials remain protected, test and live profiles stay separate, and plan mappings persist. The current roadmap remains starter-foundation until the developer agrees to the requirement text and falsifiers in docs/spec/payments.md.

## Verification plan

The named checks in payments.md will cover provisioning, direct-request authorization, CSRF, secret handling, provider state, plan mappings and validation. Browser journeys cover both providers, keyboard operation, validation feedback and persistence. Existing foundation checks remain regression checks for the affected account and UI paths.

Each implemented mechanism needs declared inputs and a safe violating fixture that fails before its corrected counterpart passes. Proposed checks are not existing evidence. Use disposable data and the developer-selected scratch directory; clean up processes and artifacts after each run. Do not inspect or alter operator credentials or the operator database.

## Boundary and completion

Implementation may affect manifests and locks, application modules, command registration, migrations, frontend pages, generated types, tests, configuration examples, README and verification scripts. Declare exact mechanism inputs before implementation. Preserve Suprnova ownership of RBAC and provider protocols.

Done requires agreed requirements, fresh passing Cairn evidence and a recorded final review. External provider authentication, catalog verification, real payment processing, webhook delivery, checkout, entitlements, subscription policy and general account management are outside this commitment. Record discovered discrepancies in docs/discrepancies.md.
