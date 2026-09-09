# Directory starter feature map

Status: Draft
Date: 2026-09-08

## Product direction

Build a reusable directory starter application with Suprnova in this repository. The developer supplied the purchased Laravel directory application as a feature foundation. The target uses Vue and Inertia. The developer selected Vuetify 0 for its UI, following the sibling suprnova.app implementation. The intended users of the starter are developers building their own directories. A particular directory subject is not required to define this starter.

The developer confirmed on 2026-09-08 that monetization belongs in the first delivery and invited architecture and feature improvement suggestions. That confirms the delivery boundary; individual rules and acceptance mechanisms below remain proposals. Reference paths resolve from the sibling `larafast-directories-master` checkout. Its complete recon is at `../larafast-directories-master/docs/recon.md` relative to this repository root.

## UI baseline

Developer-selected: Vue, Inertia and Vuetify 0 (`@vuetify/v0`). The reference app pins version 1.0.1 in `../suprnova.app/frontend/package.json:18` and uses v0 Dialog primitives with custom components and CSS in `../suprnova.app/frontend/src/layouts/ConsoleLayout.vue:4`. This supersedes retaining Pulsar’s Vuetify 3 presentation or the target scaffold’s Tailwind presentation. Pulsar remains the application-feature reference; suprnova.app supplies the UI implementation reference. No dependency or UI migration has been performed yet.

## Feature map

Expanded developer direction (2026-09-08): deliver a free MIT-licensed, Suprnova-specific starter with blog articles, SEO, custom administration and Suprnova RBAC. Evaluate the sibling Pulsar kit as a foundation. Future AI features and self-hosted Keygen licensing are brainstorming only; they add no implementation scope now. Free licensing of the starter does not remove the previously requested ability for directory operators to charge for listings.

| Capability | Observed reference | Target state | Proposed treatment |
| --- | --- | --- | --- |
| Browse and search | Reference `app/Livewire/Products.php:28`, `app/Models/Product.php:78`: search, categories, cards/list selection and eligibility filters. | Welcome page only: `frontend/src/pages/Home.vue:8`. | Core: searchable, paginated public directory with category filters and stable URLs. |
| Listing detail | Reference `app/Http/Controllers/ProductController.php:11`: slug detail, Markdown description, category loading, inactive 404. | No listing route: `src/routes.rs:6`. | Core: public approved listing detail, image and outbound URL. |
| Owner submissions | Reference `app/Livewire/CreateProduct.php:50`: fields, image upload, draft creation, category assignment, plans redirect. | Account model only: `src/models/user.rs:18`. | Core: create, edit and submit owned listings; validate fields and uploads. Owner editing is a proposed addition, not established reference parity. |
| Moderation and administration | Reference `app/Filament/Resources/ProductResource.php:22`, `app/Http/Controllers/ProductController.php:25`: admin forms, activation and moderation submission. | No admin routes or directory schema: `src/routes.rs:6`, `src/migrations/mod.rs:12`. | Core: our own administration for listings, articles, taxonomy, users and billing, using Suprnova RBAC and record-level authorization. Evaluate Pulsar's existing admin pages as the starting point. |
| Accounts | Reference `routes/web.php:28`: custom registration, social and magic-link entry points. | Password registration/login and dashboard; verification/reset traits exist without routes: `src/controllers/auth.rs:115`, `src/models/user.rs:124`, `src/routes.rs:6`. | Core: finish and test password authentication, reset, verification and owner access. Additional login methods can follow. |
| Monetization | Reference `routes/web.php:69`, `app/Models/User.php:18`, `app/Models/Product.php:53`: several provider routes, Lemon Squeezy billing relations. | No payment route: `src/routes.rs:6`. | Included in first delivery by developer direction: Stripe and Paddle with admin/configuration options, paid listing checkout and tested publishing entitlements. One-time and recurring plan support are proposed. |
| Notifications and media | Reference `app/Observers/ProductObserver.php:20`, `app/Services/OgImageService.php:97`. | No directory lifecycle yet: `src/migrations/mod.rs:12`. | Core moderation feedback and image storage; evaluate social-image generation separately. |
| SEO and editorial content | Reference `routes/web.php:43`, `routes/web.php:107`: Open Graph images, sitemap and blog. | Home page only for public content: `src/routes.rs:8`. | Included in the expanded starter: article authoring, draft/publish workflow, categories/tags, blog, RSS, metadata, canonical URLs, sitemap and structured data. Evaluate Pulsar's article/content modules and add the SEO behavior they do not already establish. |
| Role-based access control | Pulsar uses Suprnova HasRoles and PermissionMiddleware: `../Pulsar/src/models/user.rs:186`, `../Pulsar/src/routes.rs:69`. | Basic authenticated/guest gates: `src/routes.rs:10`. | Use Suprnova's RBAC for permission checks, role assignments and seeded defaults. Apply ownership rules in addition to permissions. No custom RBAC engine. |
| Free distribution | Developer direction: MIT-licensed starter; Pulsar includes an MIT license: `../Pulsar/LICENSE:1`. | No target license added yet. | Include source, administration, blog, SEO and paid-listing integration in the free kit. Future commercial features do not impose a license-server dependency on core behavior. |
| Starter adoption | Reference README is framework boilerplate: reference `README.md:10`. | No setup guide or lockfiles: `docs/recon.md`, inspection record. | Deliver install instructions, demo data, branding configuration, repeatable checks and a documented extension path. These are new starter requirements. |

## Proposed delivery order

1. Establish a tested, reproducible scaffold baseline at an explicitly selected Suprnova version.
2. Deliver one end-to-end paid directory: owner creates a listing, admin reviews it, checkout establishes publishing eligibility, and a visitor finds and opens it. Include authorization and rejection paths.
3. Complete payment recovery and lifecycle verification within that same delivery: webhook replay, failed payment, cancellation, refund and reconciliation coverage.
4. Complete the expanded starter's article publishing, SEO and administration, using Suprnova RBAC throughout. Reuse verified Pulsar capabilities where they fit.
5. Consider additional social login and generated-image features by value; AI and Keygen remain future brainstorming.

This is an ordering proposal, not a Cairn roadmap. No Current commitment or Agreed requirement exists yet. Steps 1–3 are internal milestones of the first delivery, not permission to call a free-only directory complete. Architecture and behavior proposals are in `docs/architecture-proposal.md`.

The expanded starter includes step 4; the exact commitment breakdown remains to be agreed. Foundation comparison and observed Pulsar capabilities are in `docs/pulsar-assessment.md`.

## Proposed acceptance checks for the first directory flow

- An owner can save a draft and submit it for review.
- Another ordinary user cannot edit, submit or delete that owner's listing.
- An administrator can approve or reject a submitted listing.
- Visitors see only published, approved listings in search, category pages and direct detail requests.
- Paid listings become public only after verified payment establishes the selected plan's entitlement; return-page query parameters cannot grant access.
- Repeated checkout requests and duplicate webhooks do not create duplicate fulfillment or extend entitlement twice.
- Failed payment, cancellation, expiration, refund and delayed webhooks produce consistent publication status and useful owner/admin explanations.
- Invalid fields and uploads return useful errors without leaving a half-created listing.
- A fresh checkout can install, migrate, seed and run from the documented steps.

These checks need concrete mechanisms and developer agreement before becoming commitment requirements.

Provider scope update (2026-09-08): the developer selected both Stripe and Paddle. NOWPayments is a separate next-iteration framework commitment in `docs/commitments/nowpayments-framework-adapter.md`.
