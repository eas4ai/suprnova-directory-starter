# Upgrade guidance

Status: Agreed 2026-10-05
Prefix: UPG

[UPG-001] The starter MUST provide an upgrade guide whose release identifiers, dependency pins, shell examples and named operating boundaries agree with the released application.
Falsifier: The guide omits or contradicts a manifest/lockfile version or direct framework/Markdown revision, contains a shell example that fails syntax validation or destructive fresh-install/seeding/key-replacement commands, or omits backup/key preservation, native RBAC timestamps, production rate limiting/mail, separate SSR startup, persisted worker status and rollback guidance.
Mechanism: upgrade-guidance
Status: Agreed 2026-10-05

## Scope

The developer confirmed creating upgrade guides for Pulsar, Nebula, this starter and the Markdown plugin. The original documentation-only commitment covered root UPGRADE.md and its verification. UPG-001 remains an acceptance obligation for the current framework upgrade: update the guide to match its final pins and operation. Deployment and operator data are outside this commitment. Existing release tags continue to identify their original code.
