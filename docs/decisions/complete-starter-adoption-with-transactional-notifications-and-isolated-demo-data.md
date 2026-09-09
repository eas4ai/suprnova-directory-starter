# Complete starter adoption with transactional notifications and isolated demo data

Level: Judged
Decided by: Codex
Rests on: KIT-001 KIT-002 KIT-003 KIT-004 KIT-005
Would be wrong if: Committed owner events lose their notification intent, demo seeding changes operator data, or the clean-install guide claims untested integrations.

## Decision

Record owner notification intents in the same database transaction as moderation and payment changes, using domain event identities for deduplication. Deliver through Suprnova mail with bounded retries and recoverable leases; retain owner status history independently of delivery. Keep configured branding and the existing private local image store. Seed explicitly identified demonstration records atomically and idempotently, refuse production without an explicit override, preserve operator accounts and credentials, and never invoke payment APIs. Complete MIT and provenance records and prove documented local installation and operations in disposable SQLite with captured mail. External services remain separately reported operator smoke tests.

## Realized by

(none yet: recorded, not built)
