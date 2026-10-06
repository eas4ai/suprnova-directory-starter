# Upgrade guidance mechanism review

The check reads only the declared guide, Rust manifest/lockfile, frontend package metadata, backup-link target and its own source. It compares released package versions and all four direct Git revisions, validates shell syntax without executing deployment commands, rejects named fresh-install commands and checks the required operating boundaries.

Disposable copies demonstrated four failures: a wrong framework revision, overwriting `.env`, broken shell syntax and omitted database/media/key recovery. Restoring the original guide passed. Full output is in `upgrade-guidance-mechanism.out`; no operator data or repository inputs were altered by these cases.

These checks establish metadata consistency and the presence of named guidance. They cannot prove prose semantics, successful deployment or recovery in an operator environment. The final review must inspect the exact CLI/source behavior, startup order, encryption-key preservation, worker recovery and limits of historical release evidence.

The documentation check lives under `.cairn/checks/` with its own declared inputs. Application source and existing runtime-check inputs remain unchanged, so the previous release evidence remains current.
