# Provider administration mechanism review

Reviewed 2026-09-09 against PAY-001 and PAY-004 through PAY-008 and their agreed operating rules.

The mechanism runs the actual application HTTP router, middleware, relational storage,
released adapter constructors, operator binary and Chromium forms. It also starts
separate processes with original, missing, default and wrong deployment keys. Every
run uses a disposable SQLite database and synthetic credentials under the workspace
scratch directory. No operator environment or database is copied.

## Failure demonstrations

Each deliberate defect below was applied separately to a disposable source copy.
The unchanged `provider_administration_contract` compiled, ran and failed with exit
101 at the expected assertion. These were runtime failures, not compilation failures.
The source checkout retained the corrected implementation throughout.

| Requirement | Deliberate defect | Observed failing assertion |
| --- | --- | --- |
| PAY-001 | Remove controller billing permission checks. | Shell-only administrator reads billing with 200 instead of 403. |
| PAY-004 | Return success before revoking permissions. | Two direct permissions remain instead of zero. |
| PAY-005 | Return serialized credentials instead of encrypting them. | Database payload contains a plaintext secret marker. |
| PAY-006 | Bypass the stale revision check and conditional update revision filter. | Both competing saves succeed instead of exactly one. |
| PAY-007 | Resolve a missing plan using the first saved mapping. | Resolving `missing` succeeds instead of returning an error. |
| PAY-008 | Bypass credential token validation. | A live Stripe key saves in test mode with 302 instead of 422. |

The corrected full provider verifier passed format, Rust build, HTTP/persistence,
CLI grant/idempotency/unknown account, deployment key probes, controlled constructor
failure followed by success, frontend type/build/SSR, and browser journeys. Fresh
committed runs are recorded separately by Cairn. The constructor fixture also proves
that an adapter construction failure leaves the previous row unchanged.

## Structural checks

Ripwire ran against a disposable source-only checkout based at `4504855e2`, with
the pending implementation overlaid. Generated assets and operator files were absent.
`--quality-delta` exited 2: 45 findings, including six gating findings. This is not
a clean static-analysis claim. Each gating finding was examined:

- Two migration `down` methods use the same required drop-table idiom. Keep these
  independent migration contracts; sharing them would couple migration history.
- Billing and account validation helpers both construct framework validation errors.
  Their short domain-specific wrappers return different boundary errors; no shared
  error layer is warranted. The billing validation wrapper returns `BillingError`.
- The two browser suites repeat login and keyboard traversal helpers. The journeys
  use different fixtures and failure messages; these small local harness helpers
  keep each standalone runner explicit.
- Shared test setup changed to initialize the configured Crypt key. This is required
  by billing tests and retains account setup behavior; account regression checks apply.
- The remember-token migration grew only because `cargo fmt` expanded expressions.
  Its schema and execution behavior did not change.

New-symbol findings were also inspected. Trait/macro dispatch, serde types, generated
props and test entry points explain the dead-code reports. The profile merge branches
encode replacement, preservation, clearing and incomplete-input rules; their explicit
form is retained. Provisioning stays in one transaction, and the long HTTP contract
intentionally follows state revisions through competing and failed writes. Encryption
and decryption have separate authenticated operations and error messages.

`--test-gate` exited 4 and named the two foundation Rust suites plus both provider
suites. Its three untested symbols are exercised outside its resolved graph: provider
resolution by integration tests, the application entry point by browser/setup runners,
and the billing migration by fresh database setup. Fresh foundation regression checks
subsequently passed with committed Cairn evidence. `--edit-check` for billing save and administrator
access returned 0 with no incompatible callers; name-based graph results are advisory.

## Limits

These checks establish local integration on SQLite. They do not establish external
Stripe/Paddle authentication, catalog existence, webhook delivery, payment processing,
PostgreSQL behavior or deployment. Browser screenshots were visually inspected at
desktop and mobile widths. No secret marker appeared in the corrected captured
responses, database payloads or application output.
