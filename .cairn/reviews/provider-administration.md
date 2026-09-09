commitment: provider-administration
commit: 8675e90e3d16b3ff021094c7e85dba211776d274
examined:
  - PAY-001 and PAY-004 through PAY-008, their falsifiers, operating rules, mechanism declaration and six fresh passing provider receipts.
  - HTTP authorization and CSRF, RBAC grant/revoke transactions, stored settings and encryption boundaries, local adapter validation, revision conflicts and failure atomicity.
  - Vue form lifecycle, secret replacement and clearing, mode navigation, permission-gated links, keyboard controls, mobile screenshots and operator instructions.
  - Shared snapshot isolation and process cleanup, dependency pins, generated props, failure demonstrations and Ripwire findings recorded in docs/provider-administration-mechanism-review.md.
findings:
  - open: Historical discrepancy notes still describe administrator provisioning and payment administration as future work without making the historical scope explicit. Update those statements to match the current implementation while preserving the earlier evidence.

## Review observations

No application source changed during this review. All six current provider
requirements have fresh passing Cairn receipts. Build, migration and account
regressions have also passed; shared UI and fresh-install regressions are still
being collected, so this review does not yet declare completion.

The billing controller checks both permissions on GET and POST in addition to the
outer authenticated administration group. Public registration cannot grant the
permission bundle. Grant and revoke use framework-owned tables in one transaction;
revocation removes administrative role memberships as well as direct grants and
preserves unrelated memberships. Requests recheck permissions, including sessions
created before revocation. The CLI requires an exact existing verified account.

Provider API/webhook secrets have no response DTO or Debug implementation that
prints their values. Responses expose public client keys and only secret-presence
flags. Deserializer and SDK failures become sanitized errors. Old ciphertext is
authenticated before replacement or clearing, so wrong deployment keys cannot
silently overwrite recoverable settings. Ciphertext is bound to provider and mode.
Keys are required explicitly even for local administration. Stored secrets remain
in process memory while used; this commitment does not claim memory zeroization.

Each mode has one bounded, schema-versioned record. A revision-filtered UPDATE
commits profiles, default selection and mappings together. A failed constructor,
failed database write or conflicting editor leaves the previous state intact.
Adapters read stored configuration by explicit mode/provider and do not rely on a
stale global registry. Missing prices and disabled providers fail without fallback.
Local token checks protect the released Stripe constructor from invalid headers;
real Stripe and Paddle constructors are exercised without network calls.

The unkeyed Inertia form does not remember replacement secrets in browser history.
Successful saves clear replacement fields using server-confirmed props. Errors
preserve edits and focus the error summary. Mode switches warn about unsaved edits;
clear operations require a disabled profile. Native fieldsets, labels, explicit
show/hide controls and responsive layout were inspected in rendered browser output.

The application remains one starter with framework-owned provider protocols and
RBAC. No checkout, publication entitlement or webhook handler was added. SQLite
integration, local adapter construction and built frontend behavior are the verified
scope. External authentication, price existence, SMTP delivery, PostgreSQL, runtime
SSR and production deployment remain unverified and are documented as such.
