# Persist each billing mode as a versioned settings record

Level: Judged
Decided by: Codex
Rests on: PAY-005, PAY-006, PAY-007, PAY-008
Would be wrong if: The bounded provider configuration requires independent plan or profile writes at a scale that makes one versioned record per mode contentious.

## Decision

Store one schema-versioned settings document per test/live mode in a relational row, with a revision used for compare-and-swap updates. The document has exactly the two supported providers and unique plan keys mapped by provider. Bound plan counts and field lengths. Encrypt API and webhook credentials with the released Crypt context-bound API; keep only public client credentials and mappings in cleartext. Read and validate before adapter construction, then atomically update the row if its revision still matches. This avoids a process-local configuration cache and makes concurrent edits report conflicts. Use Suprnova RBAC entities for atomic permission-bundle grants and revocation because the released helper surface has no revoke operation.

## Realized by

(none yet: recorded, not built)
