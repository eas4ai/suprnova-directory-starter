# Keep account access separate from authentication records

Level: Judged
Decided by: Codex
Rests on: ADM-001 ADM-002 ADM-003
Would be wrong if: Suspension can be undone by recovery, unrelated capabilities leak, simultaneous changes remove the last administrator, or audit writes are not atomic.

## Decision

Use an application account-access table for versioned suspension, checked by a delegating Suprnova user provider and protected domain operations. Seed administrator, moderator and editor with explicit web permissions. Account changes serialize on a database row, replace starter permissions and role memberships, and protect the last active verified full administrator. Host admin:access grants the full bundle and reinstates a verified account for recovery. Account and host decisions share transactional audit storage with a distinct host operator attribution. Add bounded account search and permission-protected paginated audit pages. Keep authentication, password reset and RBAC protocols in Suprnova.

## Realized by

(none yet: recorded, not built)
