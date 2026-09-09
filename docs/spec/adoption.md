# Starter operations and distribution

Status: Agreed 2026-09-09
Prefix: KIT

[KIT-001] The starter MUST record owner-facing moderation and payment notifications durably for retryable delivery.
Falsifier: A committed rejection, approval, suspension or payment-state change loses its notification intent on restart, replay creates duplicate intents, or delivery failure silently discards pending work.
Mechanism: Proposed notification outbox, captured-mail and retry-command tests with controlled delivery failures.

[KIT-002] The starter MUST provide documented branding and storage configuration without application source edits.
Falsifier: The documented site identity fails to appear consistently on public/admin pages and metadata, invalid configuration prevents useful diagnosis, or the documented local media store loses files across restart.
Mechanism: Proposed configuration-override, storage persistence and browser branding checks in a disposable install.

[KIT-003] The starter MUST provide an idempotent demonstration seed that cannot silently alter an existing production installation.
Falsifier: Repeating the seed duplicates fixtures, it overwrites operator records or credentials, a production invocation runs without an explicit refusal/override, or demo purchases call an external provider.
Mechanism: Proposed demo-seed command tests on empty, previously seeded and production-mode databases.

[KIT-004] The starter MUST include the selected MIT license and preserve required notices for reused source and assets.
Falsifier: The distribution lacks the MIT text or copyright holder, copied third-party material loses its required notice, or proprietary reference source/assets enter the starter.
Mechanism: Proposed distribution inventory and provenance review, backed by license/notice presence checks.

[KIT-005] The starter MUST provide a repeatable installation and operations guide verified from a clean checkout.
Falsifier: The documented build, migrate, seed, serve, account, moderation, payment, notification or recovery commands fail in the disposable acceptance environment, or the guide labels untested external integrations as verified.
Mechanism: Proposed complete-starter clean-install browser journey, command checks and all prior commitment regressions.

## Agreed operating rules

- Notifications appear in owner status pages and are sent through Suprnova mail.
  Persist one intent per domain event; retry failed delivery through a bounded command.
  SMTP can redeliver after an acknowledgment is lost, so email delivery is at least
  once rather than an impossible exactly-once promise. Do not put credentials or raw
  payment payloads in mail.
- Branding includes site name, description, configured origin, logo and accent tokens.
  Provide a documented local media disk and explain framework-backed alternatives;
  cloud-provider deployment requires a separate operator smoke test.
- Demo data includes categories, listings in representative states and published/draft
  articles. Generate no fixed privileged password. Operator provisioning remains an
  explicit command against a verified account. Synthetic entitlements remain clearly
  identified demonstration data and cannot call payment APIs.
- The MIT copyright holder uses the repository owner's existing identity where
  established; otherwise the neutral project contributor attribution is proposed.
  Preserve Pulsar notices for reused MIT material. The purchased Laravel reference
  guides behavior only; do not redistribute its source or assets.
- Document production configuration, TLS/reverse-proxy requirements, workers/scheduling,
  database/media/key backup and restoration, permission recovery, reconciliation,
  webhook URLs and provider sandbox setup. Run local acceptance on SQLite. PostgreSQL,
  external SMTP, cloud storage and actual Stripe/Paddle accounts are reported separately
  from local acceptance; do not require production credentials to run the test suite.
- NOWPayments, AI, Keygen, additional social login, external search, ratings, favorites,
  listing claims, bulk imports and deployment to a hosting account remain outside these
  two commitments.
