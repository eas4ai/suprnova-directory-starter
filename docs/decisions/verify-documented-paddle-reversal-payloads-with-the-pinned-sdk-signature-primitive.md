# Verify documented Paddle reversal payloads with the pinned SDK signature primitive

Level: Judged
Decided by: Codex
Rests on: PAY-011 PAY-013 PAY-015 PAY-016
Would be wrong if: An invalid signature can be accepted, the original bytes are changed for verification, or the SDK schema gap still prevents recovery of a documented reversal.

## Decision

The pinned Paddle adapter verifies signatures through Paddle::unmarshal, which also deserializes the complete EventData schema. Its AdjustmentAction enum predates documented chargeback_warning_reverse and credit_reverse payloads. Keep the adapter as the first verifier. For documented reversal payloads rejected by that fixed schema, use the same pinned SDK Signature::verify on the exact original bytes with the retained or current signing key, the same five-second age limit, and strict bounded ASCII digest validation before its parser. Do not implement custom HMAC or alter provider bytes. Continue to correlate and fulfill only through current authenticated SDK resource reads; signature acceptance alone never grants access. Verify accepted real-adapter fixtures and reversal-specific valid, invalid, stale and malformed SDK signatures. Preserve the agreed framework revision and make no sibling-framework changes.

## Realized by

`src/billing/events.rs` applies the fallback only to the two named actions.
`tests/paid_lifecycle.rs::signature_contract` exercises both through HTTP, with
valid, wrong-key, stale, future, odd-length and Unicode signatures. Those cases
passed in the editing-time lifecycle run. Final evidence belongs to paid-lifecycle.
