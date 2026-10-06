# Suprnova 3.2.1 mechanism review

## FND-001 preliminary review

Inspected the revised requirement and falsifier, foundation-build declaration and verifyPin/build functions. Only the exact release revision changes in the checker. The lock check still rejects mixed framework identities and checks both binaries, frozen Bun installation, client and SSR outputs, and unchanged dependency locks. A disposable old manifest/lock was rejected at the intended revision assertion (exit 1). The initial negative fixture lacked the Bun lock and was corrected before this demonstration.

The corrected-pin build reached application compilation and found four timestamp type mismatches in src/accounts/roles.rs: the new framework models require DateTime<Utc>, while seed supplies RFC3339 strings. No mechanism mismatch was found. Record this review before a separate compatibility fix; the mechanism review marker awaits the corrected positive run.

## FND-001 completed review

The separate compatibility fix keeps the UTC timestamp values native and removes redundant clones. The corrected positive foundation mechanism passed: both locked binaries, frozen Bun install, TypeScript/client/SSR builds, generated manifests and unchanged locks. The old-pin rejection remains at the exact expected assertion. All checker assertions and binary names are retained; no requirement/mechanism mismatch remains. Fresh formal evidence follows the review marker.

## PAY-001 completed review

Inspected the revised exact-adapter requirement, unchanged permission/CSRF/persistence falsifier, provider-administration declaration and script. The foundation checker still enforces all three direct sources and every framework lock source. The disposable old framework/adapter pin was rejected by the pinned-checkout verifier at its intended assertion. The corrected provider mechanism passed HTTP and persistence, repeated administrator grants, unknown-user rejection, key-policy failure/corrected cases, constructor failure/correction, client/SSR and browser journeys for both provider forms. No assertion was removed or weakened. Fresh formal evidence follows the marker.
