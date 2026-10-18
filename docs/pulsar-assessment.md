# Pulsar foundation assessment

Status: Observed findings; Draft recommendation
Date: 2026-09-08

The developer suggested the sibling Pulsar kit as a possible foundation. This inspection evaluates that option; it does not replace or merge the current starter. Paths beginning ../Pulsar refer to that sibling repository from this repository root.

## What Pulsar provides

| Capability | Source evidence | Assessment |
| --- | --- | --- |
| MIT source license | `../Pulsar/LICENSE:1` | Compatible starting material for the requested MIT kit; preserve its notice when reusing files. |
| Suprnova authentication and account management | `../Pulsar/src/routes.rs:29`, `../Pulsar/src/bootstrap.rs:87`, `../Pulsar/src/bootstrap.rs:119` | Verification, password-reset and profile routes already exist; bootstrap initializes Suprnova Magnetar and the users provider. |
| Suprnova RBAC | `../Pulsar/src/models/user.rs:186`, `../Pulsar/src/routes.rs:69`, `../Pulsar/tests/rbac_flows.rs:10` | Uses HasRoles and PermissionMiddleware, with role/permission tests. Reuse the framework integration and select directory-specific permissions. |
| Blog and articles | `../Pulsar/src/controllers/articles.rs:46`, `../Pulsar/src/routes.rs:15`, `../Pulsar/src/routes.rs:69` | Public published-article queries, detail pages, and permission-gated article creation/update/publish routes. |
| RSS and content rendering | `../Pulsar/src/routes.rs:17`, `../Pulsar/src/controllers/feed.rs:16`, `../Pulsar/src/content/articles.rs:19` | Useful editorial infrastructure already implemented. |
| Taxonomy administration | `../Pulsar/src/routes.rs:90` | Category, topic and tag administration routes guarded by taxonomy.manage. Decide which taxonomy concepts the directory needs rather than exposing all automatically. |
| User administration and moderation | `../Pulsar/src/routes.rs:134`, `../Pulsar/src/routes.rs:138` | Existing management entry points to extend into our administration. Their presence does not establish complete directory or billing administration. |
| Test coverage to carry forward | `../Pulsar/tests/article_flows.rs`, `../Pulsar/tests/auth_flows.rs`, `../Pulsar/tests/rbac_flows.rs`, `../Pulsar/tests/http_flows.rs` | Tests exist for relevant behavior. They were inspected, not executed in this assessment. |

## Differences to resolve

RBAC follow-up: the developer raised the possibility that Pulsar predates framework RBAC and owns a separate implementation. The inspected checkout delegates its RBAC machinery to Suprnova: `../Pulsar/src/commands/users_promote.rs:4` imports framework role/permission helpers; `../Pulsar/src/migrations/mod.rs:2` imports the framework CreateRbacTables migration; `../Pulsar/src/models/user.rs:186` implements HasRoles; and `../Pulsar/src/routes.rs:73` applies PermissionMiddleware. Pulsar owns the application-specific role names and permission assignments in users_promote.rs. This finding describes the current checkout, not every historical version. Directory-specific grants still need review; framework reuse does not establish that Pulsar's role policy fits this starter.

| Difference | Evidence | Consequence |
| --- | --- | --- |
| Pulsar pins Suprnova v1.3.1; target pins v1.3.7. | `../Pulsar/Cargo.toml:23`, `Cargo.toml:29` | Integrate against the selected 1.3.7 release and run the inherited tests. Do not downgrade the target. |
| Pulsar uses Vue/Vuetify; target uses Vue/Tailwind. | `../Pulsar/frontend/package.json:12`, `frontend/package.json:13` | Retaining Vuetify offers more UI reuse. Keeping Tailwind requires adapting those pages. This is an actual foundation choice, not a mechanical copy. |
| Pulsar's app_users model differs from the target users model. | `../Pulsar/src/models/user.rs:49`, `src/models/user.rs:18` | Choose one consistent authentication schema and its matching migrations before adding listing ownership. |
| Target separates shared bootstrap from HTTP bootstrap; Pulsar wires both in one register function. | `src/bootstrap.rs:49`, `src/bootstrap.rs:84`, `../Pulsar/src/bootstrap.rs:82` | Preserve the newer target's worker/HTTP separation while integrating Pulsar behavior. |
| Public article index fetches all published results. | `../Pulsar/src/controllers/articles.rs:46` | Add bounded pagination for a reusable directory/content kit. |
| SEO completeness is not established by article descriptions or RSS. | `../Pulsar/src/content/articles.rs:19`, `../Pulsar/src/routes.rs:10` | Explicitly specify and test canonical URLs, metadata, sitemap, structured data and indexing exclusions; do not claim a complete SEO layer yet. |

## Recommendation

Use Pulsar as the application foundation for accounts, editorial content, RBAC and administration, adapted to Suprnova 1.3.7 and the current bootstrap conventions. Retain its relevant tests. Add directory listings, publication entitlements and billing in the destination repository. Keep the Laravel checkout as behavioral reference, not source to copy into the MIT distribution.

UI decision (2026-09-08): the developer selected Vuetify 0, following suprnova.app. Its manifest pins `@vuetify/v0` to 1.0.1 (`../suprnova.app/frontend/package.json:18`); its console and public layouts compose v0 Dialog primitives with custom components and CSS (`../suprnova.app/frontend/src/layouts/ConsoleLayout.vue:4`, `../suprnova.app/frontend/src/layouts/PublicLayout.vue:4`). This supersedes the earlier options of retaining Pulsar's Vuetify 3 or rebuilding on Tailwind.

Reuse Pulsar's relevant backend behavior and tests, and use suprnova.app as the UI reference. Adapt branding and directory-specific navigation; the UI library choice does not automatically select the reference site's visual identity. No source files, dependency manifests or history in Pulsar or suprnova.app were changed. No foundation import has been performed.

## Inspection limits

Pulsar was clean on main tracking origin/main at inspection. Latest log entry was `222721a release: v0.2.1 on Suprnova v1.3.1`. No local AGENTS.md or BEST_PRACTICES.md was found by direct checks. Graph results showed stale line metadata for part of the user model; direct source inspection established the cited HasRoles implementation. No build, seed command, test run or database mutation was performed.
