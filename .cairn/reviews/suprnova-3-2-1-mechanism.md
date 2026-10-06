# Suprnova 3.2.1 mechanism review

## FND-001 preliminary review

Inspected the revised requirement and falsifier, foundation-build declaration and verifyPin/build functions. Only the exact release revision changes in the checker. The lock check still rejects mixed framework identities and checks both binaries, frozen Bun installation, client and SSR outputs, and unchanged dependency locks. A disposable old manifest/lock was rejected at the intended revision assertion (exit 1). The initial negative fixture lacked the Bun lock and was corrected before this demonstration.

The corrected-pin build reached application compilation and found four timestamp type mismatches in src/accounts/roles.rs: the new framework models require DateTime<Utc>, while seed supplies RFC3339 strings. No mechanism mismatch was found. Record this review before a separate compatibility fix; the mechanism review marker awaits the corrected positive run.
