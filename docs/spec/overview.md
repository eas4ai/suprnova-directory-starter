# Suprnova directory starter

Status: Draft

This is a free MIT-licensed starter for developers operating directories. Operators can charge for listings. The first delivery includes directory discovery, owner submissions, moderation, paid publishing through Stripe and Paddle, articles, taxonomy, RSS, SEO, administration, accounts, notifications, storage, demo data and adoption documentation.

The application uses Suprnova v1.3.7, Vue, Inertia and Vuetify 0. Suprnova owns authentication, payment-provider integration and RBAC. Application modules own directory permissions, ownership, moderation, plans and publication entitlements. One application and relational database keep installation and transactions understandable; this architecture remains a proposal.

Pulsar supplies candidate account/editorial/admin behavior and tests. The target retains its shared/HTTP bootstrap separation. The suprnova.app UI supplies implementation references; its branding is not selected. Approved listing revisions remaining public during review is still proposed behavior.

Review occurs before payment. Payment eligibility and moderation approval are separate. Stripe and Paddle administration/configuration are selected. NOWPayments is a separate next-iteration framework commitment. AI, Keygen, additional social login and speculative licensing are outside current scope.

## Spec map

| Domain | Record | Prefix and depth |
| --- | --- | --- |
| Build, accounts and UI foundation | foundation.md | FND; first commitment requirements |
| Provider configuration and paid publication | payments.md | PAY; selected scope, detailed policy pending |
| Directory discovery, ownership and moderation | directory.md | DIR; paid-directory draft |
| Editorial, taxonomy, RSS and SEO | content.md | CNT; complete-starter draft |
| Delegated administration and audit | administration.md | ADM; complete-starter draft |
| Notifications, media and starter adoption | adoption.md | KIT; complete-starter draft |

Read glossary.md, foundation.md, payments.md, directory.md, content.md, administration.md, adoption.md and roadmap.md after this overview. Paths in the map are repository-relative except sibling spec filenames, which are relative to docs/spec. Existing implementation evidence and unverified findings remain in docs/recon.md and docs/pulsar-assessment.md.
