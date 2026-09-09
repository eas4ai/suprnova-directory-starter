# Payment scope

Status: Draft
Prefix: PAY

The developer selected both providers and the review-before-payment ordering. The following requirement text and mechanisms await confirmation. Entitlement duration, recurring plans, refunds, disputes, renewal failure and reinstatement remain open.

[PAY-001] The starter MUST allow an administrator with billing configuration permission to save and edit Stripe and Paddle configuration through the administration UI using Suprnova v1.3.7 adapters.
Falsifier: Either provider requires application source edits, saved configuration disappears after restart, or a guest or account without the permission can read or change billing configuration through a direct request.
Mechanism: Proposed provider-administration-access HTTP tests and provider-administration-browser journeys, including both providers, restart persistence, denied actors and invalid CSRF tokens.

[PAY-002] The starter MUST restrict paid-listing checkout to approved submissions.
Falsifier: An owner can create a payable checkout for a draft, pending or rejected submission.
Mechanism: Proposed checkout lifecycle tests across moderation states for both providers.

[PAY-003] The starter MUST require verified payment eligibility before publishing an approved paid listing.
Falsifier: Approval alone or browser-return parameters publish an unpaid listing.
Mechanism: Proposed payment-forgery and publication visibility tests across search, category, detail and sitemap.

[PAY-004] The starter MUST provide an operator command to grant or revoke administrative access for an existing verified account through Suprnova RBAC.
Falsifier: The documented command cannot provision the first administrator, repeated grants create duplicate access, revocation leaves protected requests authorized, an unknown or unverified account receives access, or public registration can grant administrative permissions.
Mechanism: Proposed provider-administration-provisioning command and HTTP tests on a disposable database, including an existing session after revocation.

[PAY-005] The starter MUST store provider API and webhook secrets encrypted under the operator-provided application key and accept replacements without returning stored secrets to the browser.
Falsifier: Stored database values, response bodies, validation errors or application logs contain submitted secret markers; an unrelated save erases an existing secret; or missing, default, incorrect or corrupt encryption material permits configuration use or silently overwrites unreadable values.
Mechanism: Proposed provider-administration-secrets tests with unique secret markers, captured responses and logs, database inspection, replacement and unrelated updates, and missing-key and corrupt-ciphertext cases.

[PAY-006] The starter MUST keep separate test and live provider profiles with explicit enablement and at most one enabled default provider for each mode.
Falsifier: Selecting one mode uses credentials from the other, an incomplete profile can be enabled, a disabled provider remains the default, or a failed or stale concurrent update partially changes saved configuration.
Mechanism: Proposed provider-administration-state tests for both modes and providers, transactional failures, conflicting updates and adapter selection.

[PAY-007] The starter MUST let a billing administrator map a local plan identifier to one provider price identifier per provider and mode.
Falsifier: A saved mapping cannot be edited or removed, duplicate entries for the same plan/provider/mode coexist, or resolving a missing mapping falls back to another provider or mode.
Mechanism: Proposed provider-administration-mappings persistence and browser tests covering duplicates, missing mappings, changes and removal.

[PAY-008] The starter MUST reject locally invalid provider configuration with actionable errors before constructing an adapter.
Falsifier: Blank required values, invalid header characters or detectable key/mode mismatches cause a panic or a saved enabled profile; constructor failure alters the previous configuration; or the UI describes local validation as successful authentication or price verification with the external provider.
Mechanism: Proposed provider-administration-validation boundary tests and browser error journeys, with actual adapter construction for valid fixtures and controlled constructor failure.

## Proposed operating rules

These details belong to the Draft requirements above and are part of the agreement request.

- The operator command grants or revokes the starter's administrative permission bundle by exact account identifier. It supports recovery if all administrators lose access. General account editing and delegated role management are later work.
- Saved database configuration is authoritative. Provider environment variables do not override it. The deployment supplies the application encryption key; it is never editable through the administration UI. Key replacement requires deliberate operator recovery and is documented; automatic key rotation is outside this commitment.
- Both modes start disabled. Stripe needs its secret key, publishable key and webhook signing secret. Paddle needs its API key, client token and webhook key. Test mode maps to Paddle Sandbox; live mode maps to Production. Public client keys may be displayed to the authorized administrator; API and webhook secrets are write-only.
- Blank secret inputs preserve existing values. Replacing a secret is explicit. Disabling a profile retains its stored configuration and clears its default selection atomically. Clearing credentials requires a disabled profile. Conflicting edits report a conflict instead of overwriting another administrator's save.
- Plan identifiers and price identifiers are entered manually. Mapping does not establish provider-side price existence, currency, recurrence or entitlement duration. Those policies belong to paid-directory. No purchase is offered by this commitment.
- Adapter resolution reads committed configuration by provider and mode. It does not mutate process environment variables or rely on an old process-global registration after a save. No production network call is needed to save configuration.

PAY-001 and PAY-004 through PAY-008 form the proposed provider-administration commitment. PAY-002 and PAY-003 remain future Draft requirements. NOWPayments work is recorded in docs/commitments/nowpayments-framework-adapter.md.
