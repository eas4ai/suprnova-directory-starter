# Roadmap

Status: Agreed 2026-09-10
Prefix: MAP
Current: suprnova-4-2

## First starter delivery

1. starter-foundation — reproducible builds, account flows, migrations and the selected public/admin UI foundation.
2. provider-administration — Stripe and Paddle administration/configuration, protected credentials and plan mappings.
3. paid-directory — owner listings, bounded discovery, review-before-payment, checkout, publication and payment recovery for both providers.
4. complete-starter — editorial content, taxonomy, RSS, SEO, full administration, notifications, storage, demo data and adoption documentation.

The foundation and provider administration are complete. The developer confirmed paid-directory and complete-starter, including their requirement text, falsifiers and operating policies, on 2026-09-09. Implement them in this order. Paid publishing remains in the first delivery.

## Appearance update

5. site-appearance — optional Tailwind color presets with blank retaining the
   current theme, plus a remembered visitor light/dark switch. The developer
   confirmed this bounded update on 2026-09-09. See docs/spec/appearance.md.

## Admin, SEO and traffic update

The developer approved proceeding with the reviewed proposal on 2026-09-10.
Deliver these linked commitments in order; each preserves the preceding contracts.

6. admin-overview — actual publication counts and review work, permission-safe
   overview actions and grouped navigation. See docs/spec/admin-overview.md.
7. seo-controls — editable metadata, indexing, redirects, SEO findings and public
   Markdown twins. See docs/spec/seo-controls.md.
8. traffic-reporting — public page/link collection, Traffic source/page/link
   reports and the traffic, revenue and engagement chart groups. See
   docs/spec/traffic-reporting.md.

The Markdown plugin compatibility prerequisite is complete and published at
`565c2205b1d1d77aa05a29247f3a2416abc60791`. Starter integration belongs to
seo-controls. External analytics and Search Console data imports remain separate.

## Framework workstream

NOWPayments adapter for the next Suprnova release: `3d568ec154d1f6bcd1015225e9546450249d66ba:docs/commitments/nowpayments-framework-adapter.md` (historical Git record). This is a separate framework workstream, not the next Current value in this repository.

## Framework 3.2.1 maintenance release

On 2026-10-05 the developer requested upgrading this starter after Pulsar and Nebula, including the same verified commit, push and release-tag workflow. That completed maintenance commitment is recorded in `3d568ec154d1f6bcd1015225e9546450249d66ba:docs/commitments/suprnova-3-2-1.md` (historical Git record). Traffic reporting remains planned.

The Markdown prerequisite for Suprnova 3.2.1 is verified and published at `565c2205b1d1d77aa05a29247f3a2416abc60791`.

## Upgrade documentation

The developer requested and confirmed an upgrade guide for each upgraded repository. That completed commitment is `3d568ec154d1f6bcd1015225e9546450249d66ba:docs/commitments/upgrade-guidance.md` (historical Git record); the starter guide is [UPGRADE.md](../../UPGRADE.md). The completed Suprnova 3.2.1 implementation and its 54 acceptance requirements remain unchanged.


## suprnova-4-2

Status: Agreed 2026-10-17

Requirements: ADM-001, ADM-002, ADM-003, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, FND-001, FND-002, FND-003, FND-004, FND-005, FWU-001, FWU-002, FWU-003, FWU-004, FWU-005, FWU-006, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, OVR-001, OVR-002, OVR-003, PAY-001, PAY-002, PAY-003, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, SEO-001, SEO-002, SEO-003, SEO-004, SEO-005, SEO-006, UI-001, UI-002, UPG-001

Deliver the Suprnova 4.2 compatibility upgrade defined in framework-upgrade.md,
using the published Markdown v0.4.0 prerequisite and preserving all 55 delivered
requirements. Traffic reporting remains planned. The developer confirmed this
scope on 2026-10-17.

Done when every named requirement passes its declared mechanism, the upgrade
guide matches the final build and migration behavior, and production and
independent reviews have no unresolved findings.
