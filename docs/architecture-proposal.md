# Directory starter architecture proposal

Status: Draft
Date: 2026-09-08

Developer direction: monetization is included in the first delivery. Architecture and feature improvements are welcome. Recommendations below remain Draft except where developer decisions are explicitly recorded. Requirement-level agreement still requires confirmed falsifiers.

Subsequent developer direction: use Suprnova's own authentication and payment provider modules, inspect the development source in the sibling suprnova repository, and use v1.3.7 for the starter now. The developer reports that 2.0 largely preserves the current API and adds Suprnova Live, RenderCache and numerous bug fixes. That is developer-provided release direction, not a verified compatibility result.

## Framework integration

Authentication uses Suprnova's AuthManager, UserProvider and existing EloquentUserProvider binding. The scaffold already registers these in `src/bootstrap.rs:53` and calls Auth::attempt in `src/controllers/auth.rs:59`. Extend the app's pages, routes and directory authorization through the framework facilities; do not introduce a second authentication engine. The development source defines the provider contract in `../suprnova/framework/src/auth/provider.rs:71`.

Payments use Suprnova's PaymentProvider and PaymentProviderRegistry. The framework owns the provider abstraction; the starter owns plan selection and the link from verified payment to listing publication. The development source defines these in `../suprnova/framework/src/payments/traits/mod.rs:31` and `../suprnova/framework/src/payments/registry.rs:75`. Selecting an initial gateway means configuring a Suprnova adapter, not replacing this module.

For the future 2.0 upgrade, keep framework setup in bootstrap, provider configuration in the integration module, and publication decisions in domain actions. Use the framework's APIs directly where appropriate; avoid a speculative compatibility framework. Run the starter's authentication, checkout and publication tests against the actual 2.0 release before upgrading its pin.

The development manual now includes Live and RenderCache (`../suprnova/manual/live.md`, `../suprnova/manual/render-cache-generations.md`, `../suprnova/manual/render-cache-representations.md`). Evaluate them against concrete starter flows before choosing adoption scope. Bug fixes also make behavioral differences relevant even when API signatures remain stable: if a starter failure is already fixed upstream, record the release dependency and use a deliberate version decision instead of adding a permanent application workaround.

## Recommended structure

Scope update: the developer now includes article publishing, SEO, our own administration and Suprnova RBAC in a free MIT-licensed starter. Pulsar is a candidate foundation; see `docs/pulsar-assessment.md`. This expands the earlier directory-first proposal. AI integrations and self-hosted Keygen are future ideas only.

Use one Suprnova application and one relational database, with Vue/Inertia pages and the developer-selected Vuetify 0 UI primitives. Use the sibling suprnova.app custom components, layouts and ordered CSS as implementation references (`../suprnova.app/frontend/package.json:18`, `../suprnova.app/frontend/src/main.ts:1`). Keep directory, moderation and billing logic in focused application modules. Continue the scaffold's controller/model organization instead of introducing a package system or separate services. The existing entry point and frontend selection are in `cmd/main.rs:8` and `src/bootstrap.rs:132`.

| Option | Benefit | Cost | Recommendation |
| --- | --- | --- | --- |
| One application with explicit module responsibilities | Straightforward installation, transactions and testing; users can read and customize the starter. | Module boundaries need discipline. | Start here. |
| Reusable directory crate plus host application | Useful if several real applications need coordinated directory upgrades. | Public API compatibility and packaging work before those consumers exist. | Extract only when real reuse demonstrates the boundary. |
| Separate directory and billing services | Independent operations and deployment. | Distributed consistency and more infrastructure for every starter user. | No current requirement justifies this. |

## Responsibilities

| Module | Owns | Boundary |
| --- | --- | --- |
| Accounts | Identity, verified email, owner/admin authorization. | Every mutation checks the actor server-side. |
| Access control | Application permission names and default roles, implemented through Suprnova RBAC. | Permission gates and record ownership are both checked; hidden UI controls are not authorization. |
| Editorial | Articles, drafts, publication, taxonomy and RSS. | Public content excludes drafts; editorial permissions are distinct from listing moderation and billing. |
| Directory | Listings, categories, slugs, approved public content, bounded search queries. | Public reads use one publication predicate, including direct detail and sitemap. |
| Moderation | Submitted revisions, decisions, reasons, suspension and audit records. | Payment does not imply approval. |
| Billing | Local plans, checkout attempts, listing entitlements and reconciliation. | Provider prices and monetary values come from trusted server configuration. |
| Integrations | Suprnova payment adapter, mail and storage configuration. | Provider-specific payloads stay outside directory decisions. |

Suprnova's local manual documents `PaymentProviderRegistry`, `StartSessionRequest`, payment mirror tables and webhook recovery in `../suprnova/manual/payments.md:13`, `../suprnova/manual/payments.md:460` and `../suprnova/manual/payments.md:519`. Reuse those facilities rather than reproduce an entire billing framework. Verify the APIs and semantics in the exact framework version chosen for the starter before implementation.

## Listing lifecycle

Use explicit moderation states: draft, pending review, approved, rejected and suspended. Keep billing eligibility separate, including its validity interval and reason. Compute public visibility from approval, entitlement and owner/admin archival or suspension state. Avoid independent mutable booleans that can disagree.

The reference uses active/draft columns and reads payment relations in listing queries (`../larafast-directories-master/database/migrations/2024_04_12_111231_create_products_table.php:23`, `../larafast-directories-master/app/Livewire/Products.php:28`). The proposed split makes cases such as “approved, awaiting payment” and “paid, suspended” explainable.

Developer-selected on 2026-09-08: review before payment. The owner submits, an administrator approves, and the owner then checks out. Verified payment permits paid publication. This avoids charging for rejected submissions and adds a checkout handoff after approval. See `docs/decisions/review-listings-before-payment.md`. This confirms the ordering; detailed requirements and their falsifiers remain Draft.

For edits to a live listing, keep the approved revision public while a proposed revision is reviewed. Rejected changes do not erase the paid listing. An admin suspension still removes the public listing immediately. This adds a small revision model but closes the path where an owner replaces approved content with unreviewed content.

## Plans and payment safety

Propose plans for one-time publishing and recurring publishing, with one plan attached to a listing entitlement. A one-time plan grants publication without a recurring billing end date, subject to moderation and refund policy. A recurring plan grants publication through a verified paid-through date. A free plan can use the same entitlement rules with no provider checkout.

Store local plan IDs and immutable checkout snapshots. Correlate provider records with internal listing and checkout IDs, never mutable slugs. Keep plan selection, currency, price references, return URLs and ownership checks on the server. Give each checkout attempt a stable idempotency key.

Browser return pages show payment progress; they do not establish payment. Publication follows verified provider state. Keep payment mirror processing separate from the application fulfillment receipt: a processed framework webhook does not by itself prove that the listing entitlement was granted. Design an idempotent reconciliation path so an interruption between the two cannot strand a purchase.

Test duplicate and out-of-order events, partial failures, provider outages and recovery. Use a scheduled reconciliation command to repair delayed events. Derive cancellation access from the paid-through date. Proposed defaults are access through the paid period on scheduled cancellation, no extension after renewal failure, and suspension of paid entitlement after a full refund or dispute. Partial refunds and reinstatement need explicit rules before billing implementation.

The framework manual documents mock-provider testing (`../suprnova/manual/payments-stripe.md:626`). Application tests still need to prove entitlement and publication behavior, and adapter sandbox checks need to cover the selected integration.

## Starter features worth adding

First delivery recommendations:

- Owner dashboard that explains draft, review, payment and publication status separately.
- Admin review queue, rejection reasons and an audit trail for publication and billing changes.
- Paginated search and categories, canonical URLs, metadata and a sitemap that excludes unpublished content.
- Branding and plan configuration, demo data, a setup guide and repeatable verification commands.
- Safe rendered descriptions and constrained image uploads; avoid fetching arbitrary image URLs by default.

Article publishing, RSS, SEO and custom administration are now included in the starter by developer direction. Use Suprnova RBAC for access control and evaluate Pulsar's existing implementations before building replacements.

Later candidates: CSV import/export, listing claims, favorites, public reviews, featured placements and external search engines. These remain outside the current scope until explicitly selected.

## Future commercial ideas — no current implementation

The developer may later offer licensed AI integrations using self-hosted Keygen. Record the idea only: no Keygen deployment, license checks, paid AI dependency, activation flow or licensing abstraction is part of this free starter. Keep directory billing (operators charging for listings) conceptually separate from future product licensing (access to commercial add-ons). No licensing-provider investigation is needed until that work is selected.

## Proposed verification

Use isolated database tests for lifecycle transitions and ownership; HTTP tests for public visibility, forged checkout requests and invalid webhook signatures; payment-provider fakes for retry/reconciliation behavior; and browser tests for submission through paid publication. Run the visibility assertions against search, category, detail and sitemap together so one forgotten endpoint cannot leak an unpublished listing.

Demonstrate each new mechanism with a safe violating fixture and a corrected fixture before relying on it. Compilation, frontend checks and formatting establish a build baseline; they do not prove the business flow. Framework tests do not replace starter tests.

## Version and open decisions

The developer selected v1.3.7, now pinned in `Cargo.toml:29`. Any added framework adapter should use the matching tag. On 2026-09-08 the inspected local framework described itself as `v1.3.7-613-gac5d1756`; its documentation is not proof that every API exists in the release pin. The v1.3.7 tag contains the authentication provider contract and Stripe adapter crate. Full compilation and integration behavior remain unverified. Do not silently depend on local unpublished changes.

Resolved UI choice: Vuetify 0, following suprnova.app; the earlier Pulsar Vuetify 3 versus Tailwind choice is closed. Resolved lifecycle choice: review before payment. Resolved provider scope on 2026-09-08: support both Stripe and Paddle through starter administration and configuration. The one-provider proposal is superseded. NOWPayments is scheduled separately in `docs/commitments/nowpayments-framework-adapter.md` for the next Suprnova release. Detailed billing policy and requirement falsifiers will be presented together after those choices. No application changes, commits or deployments are authorized by this proposal alone.

## Provider administration and configuration

Developer-selected on 2026-09-08: the starter supports Stripe and Paddle, using Suprnova adapters. This supersedes selecting one initial gateway. See `docs/decisions/support-stripe-and-paddle-now-nowpayments-next.md`.

Proposed configuration behavior remains Draft: administrators with billing-management permission can enable or disable each provider, select the checkout default, configure test/live mode, and map local plans to provider price identifiers. Secrets are write-only in administration responses; configuration uses the application's protected configuration facilities. Checkout offers only enabled, configured providers with a valid mapping for the selected plan. Disabling new checkout does not discard webhook processing for existing purchases.

Proposed checks: permission tests reject unauthorized configuration changes; response checks detect secret disclosure; checkout tests reject disabled or unmapped providers; provider-selection tests cover both gateways; webhook tests cover purchases created before a gateway is disabled. Secret storage and environment-versus-admin precedence need a concrete design before implementation.

The NOWPayments adapter belongs to the next framework iteration. Its planned commitment is `docs/commitments/nowpayments-framework-adapter.md`; current starter support remains Stripe and Paddle.
