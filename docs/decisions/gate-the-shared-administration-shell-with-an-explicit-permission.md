# Gate the shared administration shell with an explicit permission

Level: Judged
Decided by: Codex
Rests on: FND-004
Would be wrong if: Ordinary accounts can open the admin shell, or public and admin navigation diverge under branding and keyboard changes.

## Decision

Use Suprnova RBAC tables and PermissionMiddleware with admin.access for the first admin shell. No account receives this permission at registration. Keep a stable directory.user model discriminator. Public and administration layouts share Vue components and semantic CSS tokens; Vuetify 0 supplies modal behavior. Use honest empty states while listing and billing features remain outside this commitment. Start with a neutral system-font theme and teal accent, subject to the pending developer styling preference.

## Realized by

- fdb282a5ac2276a22996941b5ac6828e53b32ae0 Build shared public and permission-gated administration shells
