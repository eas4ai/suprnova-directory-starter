# Suprnova directory starter

Status: Agreed 2026-09-09
Prefix: OVW

This is a free MIT-licensed starter for developers operating directories. Operators can charge for listings. The first delivery includes directory discovery, owner submissions, moderation, paid publishing through Stripe and Paddle, articles, taxonomy, RSS, SEO, administration, accounts, notifications, storage, demo data and adoption documentation.

The application uses Suprnova revision `2bd4bd53d04fa4581fdbb152ebd70cfb682b342c`, Vue, Inertia and Vuetify 0. Suprnova owns authentication, payment-provider integration and RBAC. Application modules own directory permissions, ownership, moderation, plans and publication entitlements. One application and relational database keep installation and transactions understandable; this architecture is agreed.

Larafast Directories is the working reference for directory workflows, page composition and administration. Reimplement the agreed behavior in Suprnova without redistributing the purchased reference source or assets. Earlier Pulsar investigation remains historical groundwork, not the selected directory reference. The target retains its shared/HTTP bootstrap separation. The suprnova.app UI supplies implementation references; its branding is not selected. Approved listing revisions remain public during review when otherwise eligible.

Review occurs before payment. Payment eligibility and moderation approval are separate. Stripe and Paddle administration/configuration are selected. NOWPayments is a separate next-iteration framework commitment. AI, Keygen, additional social login and speculative licensing are outside current scope.

## Spec map

| File | Prefix | Domain |
| --- | --- | --- |
| foundation.md | FND | Build, accounts and UI foundation |
| payments.md | PAY | Provider configuration and paid publication |
| directory.md | DIR | Directory discovery, ownership and moderation |
| content.md | CNT | Editorial, taxonomy, RSS and SEO |
| administration.md | ADM | Delegated administration and audit |
| adoption.md | KIT | Notifications, media and starter adoption |
| appearance.md | UI | Optional color presets and visitor appearance |
| admin-overview.md | OVR | Actual admin overview and navigation |
| seo-controls.md | SEO | Metadata controls and public Markdown |
| traffic-reporting.md | TRF | Traffic, revenue and engagement reports |
| upgrade-guidance.md | UPG | Operator upgrade and recovery guidance |

Read glossary.md, foundation.md, payments.md, directory.md, content.md, administration.md, adoption.md, appearance.md, admin-overview.md, seo-controls.md, traffic-reporting.md and roadmap.md after this overview. Paths in the map are repository-relative except sibling spec filenames, which are relative to docs/spec. Existing implementation evidence and unverified findings remain in docs/recon.md and docs/pulsar-assessment.md.
