# Provider administration draft review

Reviewed 2026-09-09. Status: Draft; no runtime implementation or provider verification is claimed.

## Attacks and resolutions

- Authorization only through hidden navigation would miss direct requests. PAY-001 tests denied HTTP reads and writes; PAY-004 tests public privilege escalation and revocation with an existing session.
- Encrypted database columns alone would miss response and log leaks. PAY-005 checks unique markers across all three surfaces and failures. Public client credentials are explicitly distinguished from API and webhook secrets.
- A single global provider instance could retain old credentials or mix test and live configuration. PAY-006 tests selection by provider and mode, and the operating rules require committed configuration to govern resolution.
- Default selection and disabling could leave inconsistent state. PAY-006 includes atomic failure and conflicting update cases; disabling clears the default in the same update.
- Arbitrary price IDs cannot prove that a provider account has a usable price. PAY-007 limits this commitment to mappings; PAY-008 forbids claiming external verification from local validation.
- Provisioning could grow into general user administration. PAY-004 is limited to an operator command for existing verified accounts, backed by RBAC. The decision record explains this boundary.

## Pinned source observations

Inspected the local Suprnova repository at tag v1.3.7. StripeProvider::new in crates/suprnova-payments-stripe/src/lib.rs accepts explicit credentials and documents a panic for invalid HTTP header values. PaddleProvider::new in crates/suprnova-payments-paddle/src/lib.rs accepts explicit credentials and an environment and returns a Result. PAY-008 therefore requires validation before construction rather than assuming constructors validate every input.

PaymentProviderRegistry in framework/src/payments/registry.rs is process-global, supports replacement by name, and has no removal operation. It is not by itself a durable mode-aware configuration store. The proposed application configuration resolver must preserve mode and freshness without changing the provider protocol implementation.

No external service request was made. Proposed checks must establish their own failure sensitivity when implemented. Review found no remaining contradiction within this bounded draft; business payment policies remain explicitly deferred.

Validation: `node scripts/spec-lint.mjs docs/spec` passed. `git diff --check` passed. The production rules self-audit covered scope, boundary handling, persistence, secret exposure, maintainability and honest verification claims. This delivery contains specification documents only; implementation and its tests are not complete or claimed to pass.
