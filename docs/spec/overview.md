# Suprnova directory starter

Status: Agreed 2026-09-09

This is a free MIT-licensed starter for developers operating directories. Operators can charge for listings. The first delivery includes directory discovery, owner submissions, moderation, paid publishing through Stripe and Paddle, articles, taxonomy, RSS, SEO, administration, accounts, notifications, storage, demo data and adoption documentation.

The application uses Suprnova revision `107e6e7a122d5145160ea1547ca90ddc37459c27`, Vue, Inertia and Vuetify 0. Suprnova owns authentication, payment-provider integration and RBAC. Application modules own directory permissions, ownership, moderation, plans and publication entitlements. One application and relational database keep installation and transactions understandable; this architecture is agreed.

Larafast Directories is the working reference for directory workflows, page composition and administration. Reimplement the agreed behavior in Suprnova without redistributing the purchased reference source or assets. Earlier Pulsar investigation remains historical groundwork, not the selected directory reference. The target retains its shared/HTTP bootstrap separation. The suprnova.app UI supplies implementation references; its branding is not selected. Approved listing revisions remain public during review when otherwise eligible.

Review occurs before payment. Payment eligibility and moderation approval are separate. Stripe and Paddle administration/configuration are selected. NOWPayments is a separate next-iteration framework commitment. AI, Keygen, additional social login and speculative licensing are outside current scope.

## Spec map

| Domain | Record | Prefix and depth |
| --- | --- | --- |
| Build, accounts and UI foundation | foundation.md | FND; first commitment requirements |
| Provider configuration and paid publication | payments.md | PAY; provider administration and paid lifecycle |
| Directory discovery, ownership and moderation | directory.md | DIR; paid-directory requirements |
| Editorial, taxonomy, RSS and SEO | content.md | CNT; complete-starter requirements |
| Delegated administration and audit | administration.md | ADM; complete-starter requirements |
| Notifications, media and starter adoption | adoption.md | KIT; complete-starter requirements |
| Optional color presets and visitor appearance | appearance.md | UI; site-appearance requirements |
| Actual admin overview and navigation | admin-overview.md | OVR; admin-overview requirements |
| Metadata controls and public Markdown | seo-controls.md | SEO; seo-controls requirements |
| Traffic, revenue and engagement reports | traffic-reporting.md | TRF; traffic-reporting requirements |

Read glossary.md, foundation.md, payments.md, directory.md, content.md, administration.md, adoption.md, appearance.md, admin-overview.md, seo-controls.md, traffic-reporting.md and roadmap.md after this overview. Paths in the map are repository-relative except sibling spec filenames, which are relative to docs/spec. Existing implementation evidence and unverified findings remain in docs/recon.md and docs/pulsar-assessment.md.
