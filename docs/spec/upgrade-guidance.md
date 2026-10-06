# Upgrade guidance

Status: Agreed 2026-10-05
Prefix: UPG

[UPG-001] The starter MUST provide an upgrade guide whose release identifiers, dependency pins, shell examples and named operating boundaries agree with the released application.
Falsifier: The guide omits or contradicts a manifest/lockfile version or direct framework/Markdown revision, contains a shell example that fails syntax validation or destructive fresh-install/seeding/key-replacement commands, or omits backup/key preservation, native RBAC timestamps, production rate limiting/mail, separate SSR startup, persisted worker status and rollback guidance.
Mechanism: upgrade-guidance; metadata/lock and shell-syntax checks plus explicit boundary checks, followed by recorded manual command/source and recovery review.

## Scope

The developer confirmed creating upgrade guides for Pulsar, Nebula, this starter and the Markdown plugin. This repository's commitment covers its root UPGRADE.md and the documentation verification/records necessary under Cairn. Runtime source, dependency pins, lockfiles, deployment and operator data remain unchanged. Keep the previously completed application's acceptance evidence current. Publish the guide on main after verification; existing release tags identify their original code.
