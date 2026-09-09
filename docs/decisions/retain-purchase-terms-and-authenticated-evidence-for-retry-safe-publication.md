# Retain purchase terms and authenticated evidence for retry-safe publication

Level: Judged
Decided by: Codex
Rests on: DIR-008 PAY-002 PAY-003 PAY-009 PAY-010 PAY-011 PAY-012 PAY-013 PAY-014 PAY-015 PAY-016
Would be wrong if: A retry can create another payable session, signed evidence cannot establish final purchase terms, credentials are lost for existing purchases, or replay changes an entitlement twice.

## Decision

Persist immutable publishing purchase terms, an exclusive listing purchase slot, exact checkout correlation, retained encrypted provider credentials, authenticated event acceptance, payment-specific entitlements and separate fulfillment receipts. Compare and update workflow versions transactionally; never hold a database transaction across provider calls. Create at most one payable session for an attempt; ambiguous requests remain pending, and unsupported Paddle idempotency is never invented. Use the pinned Suprnova adapters for checkout, customer, signature and subscription operations. Add narrow read-only evidence retrieval through the same pinned provider SDK versions where neutral adapter DTOs omit final price or settlement fields: Stripe expanded checkout line items and Paddle transaction detail. This application verification is needed for the agreed wrong-price and missed-event falsifiers and does not modify the framework. Verify provider, deployment-selected mode, customer, purchase correlation, final price, quantity, currency, advertised base amount and settlement before fulfillment. Retain raw authenticated events separately from atomic idempotent fulfillment; bounded recovery replays retained evidence and queries authoritative existing resources, never creates charges. Fixed provider/mode webhook ingress uses bounded untrusted identifiers solely to select candidate retained credentials, verifies before recording or applying anything, and preserves existing purchase recovery when checkout is disabled or keys change. Owner pages expose safe statuses and server-owned actions; admin plan changes do not rewrite existing purchases.

## Realized by

a45de12131a40da5cc1b3a373fc8e29b5cf37881 Implement verified paid publication and recovery for Stripe and Paddle

The publishing migration and `src/billing/{plans,checkout,events,collect,fulfillment,reconcile}.rs`
implement the saved terms, exclusive slot, authenticated ingress and transactional
receipts. `gateway.rs` uses the pinned adapters for mutations and pinned SDKs for
rich read-only evidence. Protected publishing controllers, owner/admin Vue pages
and the `billing:reconcile` command expose the workflow. Lifecycle, SDK wire and
browser tests use disposable data; final verification remains in progress.
