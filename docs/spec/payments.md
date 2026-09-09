# Payment scope

Status: Draft
Prefix: PAY

The developer selected both providers and the review-before-payment ordering. The following requirement text and mechanisms await confirmation. Entitlement duration, recurring plans, refunds, disputes, renewal failure and reinstatement remain open.

[PAY-001] The starter MUST expose Stripe and Paddle administration and configuration through Suprnova's provider adapters.
Falsifier: An operator cannot configure either provider without editing application source, or an unauthorized account can change billing configuration.
Mechanism: Proposed provider-administration HTTP and browser tests for each provider and denied actors.

[PAY-002] The starter MUST restrict paid-listing checkout to approved submissions.
Falsifier: An owner can create a payable checkout for a draft, pending or rejected submission.
Mechanism: Proposed checkout lifecycle tests across moderation states for both providers.

[PAY-003] The starter MUST require verified payment eligibility before publishing an approved paid listing.
Falsifier: Approval alone or browser-return parameters publish an unpaid listing.
Mechanism: Proposed payment-forgery and publication visibility tests across search, category, detail and sitemap.

Provider enablement, default selection, test/live modes, plan mappings, secret storage and configuration precedence will be specified before the provider-administration commitment. Proposed details remain in docs/architecture-proposal.md. NOWPayments work is recorded in docs/commitments/nowpayments-framework-adapter.md.
