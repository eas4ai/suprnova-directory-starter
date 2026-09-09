# Paid directory review

## Mechanism review: FND-001 revised dependency contract

Examined foundation-build and scripts/verify-foundation.mjs verifyPin, build and
snapshot handling against the confirmed exact framework revision. The mechanism
still requires the old tag v1.3.7 and therefore rejects the agreed dependency.
It also checks only the framework dependency; the corrected assertion must check
both payment adapters and all locked Suprnova workspace packages. This is a
mechanism mismatch requiring a separate implementation action. No runtime code
was changed during this review. Preserve disposable builds and unchanged lock checks.

## Reference findings

Larafast Directories is the selected working example. Its category sidebar/search,
card and compact results, submission form, owner status dashboard, listing-specific
plans and admin workspace guide the independently authored Vue implementation.
Product.php, CreateProduct.php, Products.php, ProductController.php, dashboard.blade.php
and plans.blade.php in that reference establish the flow. The reference primarily
uses Lemon Squeezy and pays before moderation; the agreed starter uses Stripe and
Paddle and reviews before checkout. Its unconstrained active/draft booleans and
browse/detail eligibility mismatch are not the agreed domain model.

Review remains open until implementation and failure-sensitive verification finish.
