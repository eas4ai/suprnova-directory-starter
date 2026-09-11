# Roadmap

Status: Agreed 2026-09-10
Current: seo-controls

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
`9aa7a482782ddc6eef92393ab2b0f36a2fefb7ec`. Starter integration belongs to
seo-controls. External analytics and Search Console data imports remain separate.

## Framework workstream

NOWPayments adapter for the next Suprnova release: docs/commitments/nowpayments-framework-adapter.md. This is a separate framework workstream, not the next Current value in this repository.
