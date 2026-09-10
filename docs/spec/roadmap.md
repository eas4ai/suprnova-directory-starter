# Roadmap

Status: Agreed 2026-09-08
Current: site-appearance

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

## Next framework iteration

NOWPayments adapter for the next Suprnova release: docs/commitments/nowpayments-framework-adapter.md. This is a separate framework workstream, not the next Current value in this repository.
