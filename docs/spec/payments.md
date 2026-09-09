# Payment scope

Status: Draft
Prefix: PAY

The developer confirmed PAY-001 and PAY-004 through PAY-008, their falsifiers and operating rules on 2026-09-09. PAY-002 and PAY-003 remain Draft. Entitlement duration, recurring plans, refunds, disputes, renewal failure and reinstatement remain open.

[PAY-001] The starter MUST allow an administrator with billing configuration permission to save and edit Stripe and Paddle configuration through the administration UI using Suprnova v1.3.7 adapters.
Falsifier: Either provider requires application source edits, saved configuration disappears after restart, or a guest or account without the permission can read or change billing configuration through a direct request.
Mechanism: Proposed provider-administration-access HTTP tests and provider-administration-browser journeys, including both providers, restart persistence, denied actors and invalid CSRF tokens.
Status: Agreed 2026-09-09

[PAY-002] The starter MUST restrict paid-listing checkout to approved submissions.
Falsifier: An owner can create a payable checkout for a draft, pending or rejected submission.
Mechanism: Proposed checkout lifecycle tests across moderation states for both providers.

[PAY-003] The starter MUST require verified payment eligibility before publishing an approved paid listing.
Falsifier: Approval alone or browser-return parameters publish an unpaid listing.
Mechanism: Proposed payment-forgery and publication visibility tests across search, category, detail and sitemap.

[PAY-004] The starter MUST provide an operator command to grant or revoke administrative access for an existing verified account through Suprnova RBAC.
Falsifier: The documented command cannot provision the first administrator, repeated grants create duplicate access, revocation leaves protected requests authorized, an unknown or unverified account receives access, or public registration can grant administrative permissions.
Mechanism: Proposed provider-administration-provisioning command and HTTP tests on a disposable database, including an existing session after revocation.
Status: Agreed 2026-09-09

[PAY-005] The starter MUST store provider API and webhook secrets encrypted under the operator-provided application key and accept replacements without returning stored secrets to the browser.
Falsifier: Stored database values, response bodies, validation errors or application logs contain submitted secret markers; an unrelated save erases an existing secret; or missing, default, incorrect or corrupt encryption material permits configuration use or silently overwrites unreadable values.
Mechanism: Proposed provider-administration-secrets tests with unique secret markers, captured responses and logs, database inspection, replacement and unrelated updates, and missing-key and corrupt-ciphertext cases.
Status: Agreed 2026-09-09

[PAY-006] The starter MUST keep separate test and live provider profiles with explicit enablement and at most one enabled default provider for each mode.
Falsifier: Selecting one mode uses credentials from the other, an incomplete profile can be enabled, a disabled provider remains the default, or a failed or stale concurrent update partially changes saved configuration.
Mechanism: Proposed provider-administration-state tests for both modes and providers, transactional failures, conflicting updates and adapter selection.
Status: Agreed 2026-09-09

[PAY-007] The starter MUST let a billing administrator map a local plan identifier to one provider price identifier per provider and mode.
Falsifier: A saved mapping cannot be edited or removed, duplicate entries for the same plan/provider/mode coexist, or resolving a missing mapping falls back to another provider or mode.
Mechanism: Proposed provider-administration-mappings persistence and browser tests covering duplicates, missing mappings, changes and removal.
Status: Agreed 2026-09-09

[PAY-008] The starter MUST reject locally invalid provider configuration with actionable errors before constructing an adapter.
Falsifier: Blank required values, invalid header characters or detectable key/mode mismatches cause a panic or a saved enabled profile; constructor failure alters the previous configuration; or the UI describes local validation as successful authentication or price verification with the external provider.
Mechanism: Proposed provider-administration-validation boundary tests and browser error journeys, with actual adapter construction for valid fixtures and controlled constructor failure.
Status: Agreed 2026-09-09

## Agreed operating rules

These details belong to PAY-001 and PAY-004 through PAY-008 and were confirmed with those requirements.

- The operator command grants or revokes the starter's administrative permission bundle by exact account identifier. It supports recovery if all administrators lose access. General account editing and delegated role management are later work.
- Saved database configuration is authoritative. Provider environment variables do not override it. The deployment supplies the application encryption key; it is never editable through the administration UI. Key replacement requires deliberate operator recovery and is documented; automatic key rotation is outside this commitment.
- Both modes start disabled. Stripe needs its secret key, publishable key and webhook signing secret. Paddle needs its API key, client token and webhook key. Test mode maps to Paddle Sandbox; live mode maps to Production. Public client keys may be displayed to the authorized administrator; API and webhook secrets are write-only.
- Blank secret inputs preserve existing values. Replacing a secret is explicit. Disabling a profile retains its stored configuration and clears its default selection atomically. Clearing credentials requires a disabled profile. Conflicting edits report a conflict instead of overwriting another administrator's save.
- Plan identifiers and price identifiers are entered manually. Mapping does not establish provider-side price existence, currency, recurrence or entitlement duration. Those policies belong to paid-directory. No purchase is offered by this commitment.
- Adapter resolution reads committed configuration by provider and mode. It does not mutate process environment variables or rely on an old process-global registration after a save. No production network call is needed to save configuration.

PAY-001 and PAY-004 through PAY-008 form the provider-administration commitment. PAY-002 and PAY-003 remain future Draft requirements. NOWPayments work is recorded in docs/commitments/nowpayments-framework-adapter.md.

## Remaining payment lifecycle

The following requirements and operating rules are Draft. They complete PAY-002
and PAY-003 for the proposed paid-directory commitment.

[PAY-009] The starter MUST let a billing administrator manage local free, one-time and recurring publishing plans without altering existing purchase terms.
Falsifier: A non-billing actor can change plans, an invalid or disabled plan can start checkout, or editing a plan changes a previously created purchase's price reference, currency or entitlement policy.
Mechanism: Proposed paid-plans HTTP, persistence and immutable-purchase-snapshot tests.

[PAY-010] The starter MUST create checkout attempts from server-owned listing, plan, provider and mode data with retry-safe correlation.
Falsifier: Browser price/currency/customer/return-URL overrides are trusted, another owner can start or inspect checkout, a disabled or unmapped provider is offered, or repeating an ambiguous request creates an unrelated payable session.
Mechanism: Proposed paid-checkout tests with capturing provider fakes, concurrent requests, network failures and real adapter boundary fixtures.

[PAY-011] The starter MUST authenticate and correlate payment evidence before granting a listing entitlement.
Falsifier: A bad signature, unsettled invoice, unknown purchase, wrong provider/mode/customer/price/currency, or a browser return parameter grants or extends publication eligibility.
Mechanism: Proposed signed Stripe/Paddle webhook fixtures and payment-evidence tests, including Paddle billed-but-unsettled events and forged return pages.

[PAY-012] The starter MUST apply payment events and entitlement changes idempotently despite duplicate delivery, reordering and interrupted processing.
Falsifier: Replaying an event grants a second entitlement or extends a period twice, an older event reverses a newer authoritative state, or an interruption permanently loses fulfillment after recording the event.
Mechanism: Proposed paid-fulfillment transaction, replay, concurrency and controlled-crash tests with recovery runs.

[PAY-013] The starter MUST enforce the agreed renewal, cancellation, refund and dispute policy on publication eligibility.
Falsifier: An unpaid renewal extends access, scheduled cancellation removes an already paid period early, a full refund or open dispute leaves the affected entitlement usable, or partial-refund and reinstatement outcomes differ from the policy below.
Mechanism: Proposed paid-lifecycle state matrix for both providers with controlled time and reordered event fixtures.

[PAY-014] The starter MUST provide a bounded operator reconciliation command that repairs recoverable payment state without creating new charges.
Falsifier: A missed event cannot be recovered from available authoritative provider state, a replayed reconciliation duplicates fulfillment, an unavailable provider destroys prior evidence, or the command creates a new payment session or charge.
Mechanism: Proposed paid-reconciliation command tests with missed events, retained webhook replay, provider failures and a second corrected run.

[PAY-015] The starter MUST preserve recovery for existing purchases when new checkout is disabled or provider configuration changes.
Falsifier: Disabling checkout rejects valid events for an existing purchase, clearing credentials silently strands an unresolved purchase, or a replacement signing key makes retained events unusable without an explicit recovery path.
Mechanism: Proposed paid-provider-lifecycle tests for disabled providers, credential replacement/clearing and pending purchase recovery.

[PAY-016] The starter MUST use payment adapters whose observable checkout and recovery behavior meets the declared application contract.
Falsifier: An adapter silently drops required purchase correlation, claims an unsupported retry guarantee, cannot recover a recorded checkout's authoritative state, or fails to encode the configured price in a valid provider request.
Mechanism: Proposed adapter wire-contract tests with a local HTTP fixture server and signed webhook fixtures, including timeouts and unsupported operations.

## Proposed paid-directory operating rules

- Offer free, one-time and recurring plans. One-time payment grants publication
  without an expiry date, subject to moderation, refund and dispute rules. Recurring
  plans use monthly or annual provider prices and grant access only through a
  verified paid-through timestamp. No trial, prorated upgrade, coupon UI, quantity
  pricing or plan switching during an active entitlement is included.
- Local plans contain a stable key, display name, description, enabled flag, billing
  type, advertised amount in minor units and currency. Tax remains provider-owned.
  Operators configure matching provider prices, as in provider administration.
  The starter does not promise catalog verification at configuration save. Checkout
  sends the configured price and validates available payment evidence against the
  immutable purchase terms; a mismatch prevents fulfillment and produces an operator
  error. Browser values never decide the payable amount. Provider checkout is the
  final display of tax and any provider-supported discount before purchase.
- Review still precedes checkout. Owners choose among eligible plans and configured
  providers. A free plan grants eligibility after approval without a fake payment.
  Approval of later content revisions retains the listing's existing entitlement.
- Operators select the checkout mode explicitly through deployment configuration;
  no browser parameter chooses test versus live. Both webhook modes have distinct
  authenticated ingress paths. Test fulfillment never grants public eligibility.
- One listing has at most one active publishing purchase. Repeated requests reuse
  a pending attempt and its idempotency key. Ambiguous provider failures remain
  pending until reconciled; retries do not blindly create another session. A new
  attempt follows only an authoritative expired/failed/canceled result.
- Browser completion and cancel pages display status only. The server grants access
  from authenticated settled-payment evidence tied to its purchase snapshot. Neutral
  event labels alone are insufficient. Verify settlement and the purchase terms;
  an issued invoice does not prove collection.
- Scheduled cancellation retains access until the paid-through date. Immediate
  cancellation ends access at the provider-confirmed effective time. Failed renewal
  adds no time and has no grace period; the already paid period remains valid.
- A full refund revokes eligibility supplied by that payment. Partial refunds leave
  it unchanged. An open dispute suspends the affected entitlement. A resolved dispute
  restores it only when authoritative evidence confirms the payment was retained;
  lost disputes revoke it. Neither restoration nor a new payment overrides moderation.
- Refunds are initiated in the provider dashboard; this starter consumes their
  verified outcomes. Owner cancellation uses the provider's supported management
  flow or a server-authorized cancellation action. No application refund console
  or cross-provider subscription migration is promised.
- Persist a fulfillment receipt separately from webhook acceptance. Retain failed
  work for replay and expose an operator command with bounded batches, attempts,
  timeouts and errors. Provider outages do not erase prior paid evidence; recurring
  access still expires at its last verified boundary.
- Retain the credential material needed by unresolved purchases, encrypted and
  scoped by provider/mode. Block destructive clearing when safe recovery would be
  lost. Disabling new checkout leaves authenticated event handling active. Deliberate
  key replacement has documented overlap/replay and recovery behavior.
- Verification uses synthetic provider responses plus signatures verified by the
  real pinned adapters. A real sandbox smoke test is documented separately and
  requires operator-owned accounts, prices and webhook delivery. Local passes do
  not claim an external provider account was successfully charged.
- The developer selected and completed framework adapter repairs in Suprnova.
  The proposed integration pins Suprnova and its adapters to commit
  `107e6e7a122d5145160ea1547ca90ddc37459c27`, which contains the fixes, until a
  reviewed release tag includes them. This is a proposed amendment to the v1.3.7
  constraint in FND-001 and PAY-001, not a claim that the starter already uses it.
  Never invent a provider idempotency feature: when unsupported, retain an
  ambiguous attempt and recover through verified correlation/state or an explicit
  operator recovery path instead of issuing another create request. Verify the exact
  dependency revision and rerun affected foundation/provider checks after integration.
