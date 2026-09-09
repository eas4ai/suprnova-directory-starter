# Use framework verification and verified-account password reset

Level: Judged
Decided by: Codex
Rests on: FND-003,FND-005
Would be wrong if: The configured user provider cannot complete verification, password rotation and session revocation through the framework APIs.

## Decision

Keep the existing Eloquent user provider. Send verification after registration and let authenticated users resend and consume their own links. Use the framework verified-account password-reset path, including its explicit revocation outcome. Build mail links from configured APP_URL. An unverified account can sign in and resend verification; reset requests for unknown or unverified addresses return the same response without mail. Do not introduce a second authentication engine for this foundation.

## Realized by

(none yet: recorded, not built)
