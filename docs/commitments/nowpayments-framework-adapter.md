# NOWPayments adapter for Suprnova

Status: Planned by developer; acceptance details Draft
Iteration: Next, separate from the current starter work
Target: Next Suprnova release; version number not yet assigned
Implementation repository: sibling `suprnova`
Date: 2026-09-08

## Goal and authorization

The developer requested a reusable NOWPayments adapter for Suprnova's next release. The current starter supports Stripe and Paddle. This commitment schedules framework work; it does not add NOWPayments checkout to the starter's current scope.

Decision: `docs/decisions/support-stripe-and-paddle-now-nowpayments-next.md`.

## Proposed delivery

- A NOWPayments adapter crate integrated with Suprnova's payment-provider contracts and registry.
- Invoice checkout, authenticated payment-status retrieval, webhook signature verification, and mapping into framework payment events.
- Explicit capability behavior for unsupported operations, including subscription management, customer management, capture and promotions. Verify actual provider capabilities before specifying these; do not simulate recurring billing.
- Provider documentation, configuration examples, and tests delivered with the adapter.

## Reuse evidence

Nation X implements an HTTP invoice client and checkout service in `../nation-x_com/src/domain/monetization/nowpayments.rs:93`, payment reconciliation at line 504, and webhook verification/parsing in `../nation-x_com/src/domain/monetization/webhooks/nowpayments.rs:29`.

Its implementation uses Nation X storefront, support-event, provider and checkout-session models. Extract provider behavior without bringing those application models into Suprnova. Source inspection does not establish passing tests or redistribution permission; confirm the source license before copying implementation.

Suprnova v1.3.7 requires Checkout, Subscription, CustomerStore and WebhookHandler in its PaymentProvider contract (`../suprnova/framework/src/payments/traits/mod.rs:31`). Inspect the actual next-release contract before implementation. Keep directory publication entitlements in the starter.

## Proposed acceptance mechanisms

These checks are Draft, not Agreed requirements or implemented Cairn mechanisms.

| Behavior | Failure that the check must detect | Proposed mechanism |
| --- | --- | --- |
| Checkout and status retrieval | Incorrect invoice fields, untrusted redirect URLs, unbounded responses, or silent provider failures | Adapter tests using a local fake HTTP server for success, rejection, timeout and malformed responses |
| Webhook authentication | A changed payload or invalid signature is accepted | Signed fixtures plus tampered-payload and missing-signature cases |
| Payment-state mapping | Pending, partial, failed or expired payment grants completed-payment status | Table-driven event mapping tests, including payment completion and replay |
| Recovery | Duplicate or reordered events duplicate fulfillment, or ambiguous invoice creation retries create a second invoice | Framework payment-pipeline tests with controlled interruption, replay and reconciliation |
| Capability reporting | An unsupported operation reports success | Contract tests for each implemented or explicitly unsupported capability |
| Framework integration | The adapter cannot register or compile against the release candidate | Adapter compilation, framework integration tests and the repository release gate |
| Provider interoperability | Fake-server assumptions differ from provider behavior | Documented sandbox verification using the provider's available test facilities; record any unverified path |

## Done when

The developer has confirmed the final requirements and falsifiers. The adapter and documentation are in the Suprnova release candidate. Agreed mechanisms have passed, including safe failing fixtures demonstrating that the checks catch their stated violations. The framework's release checks and review are recorded. Publication of the release follows the framework's release process.

## Handoff into the framework iteration

Carry this commitment into Suprnova's own planning records before implementation. Assign requirement identifiers and mechanism declarations there after contract and capability review. This record does not establish an active Cairn loop in either repository.
