# SEO controls and Markdown publication

Status: Agreed 2026-09-10
Prefix: SEO

[SEO-001] Authorized operators MUST manage site metadata defaults and search-engine verification values; authorized content editors MUST manage optional search titles, descriptions, social images and indexability for listings, articles and taxonomy.
Falsifier: An unauthorized direct request reads or mutates protected settings, saved overrides do not appear in public output, blank overrides do not use content defaults, or an unapproved revision changes public metadata.
Mechanism: Proposed seo-controls; settings and revision lifecycle HTTP tests, permission matrix and initial-HTML assertions.

[SEO-002] Public HTML MUST emit consistent canonical, Open Graph, Twitter card and structured data derived from the current public revision and configured origin.
Falsifier: Metadata duplicates after Inertia navigation, uses an attacker-supplied host, invents reviews or business facts, exposes draft content, or disagrees with visible content and stored publication dates.
Mechanism: Proposed public-seo-controls; initial HTML, Inertia/browser navigation, hostile host/text fixtures and parsed JSON-LD assertions.

[SEO-003] Robots, sitemaps and public indexability MUST agree with current publication eligibility, metadata settings and a documented pagination/filter policy.
Falsifier: A draft, private, expired or noindex record appears in a sitemap, real pagination canonicalizes to the wrong page, tracking parameters split canonical URLs, or sitemap modification dates are fabricated.
Mechanism: Proposed seo-discovery; sitemap/robots/canonical matrix with publication, expiry, suspension, pagination, search and filter fixtures.

[SEO-004] Authorized SEO administrators MUST manage bounded same-site redirects and inspect a bounded 404 report without creating open redirects, loops or conflicts with live routes.
Falsifier: A caller can choose an external destination, a cycle or live-route conflict is accepted, automatic old-slug redirects break, or report input exposes credentials or unbounded query data.
Mechanism: Proposed seo-redirects; direct request tests for valid aliases, chains, loops, collisions, encoded paths, external targets and bounded report retrieval.

[SEO-005] An authorized operator MUST receive actionable SEO findings and accurate search/social previews without a fabricated ranking score.
Falsifier: Missing or duplicate metadata and inconsistent sitemap/indexing state are not reported, a preview disagrees with emitted metadata, or a user without SEO access can retrieve findings or change SEO settings.
Mechanism: Proposed seo-report; known-good and violating fixtures, permission tests and browser preview checks.

[SEO-006] The starter MUST use suprnova-markdown to publish Markdown twins only for eligible public listing/article revisions and advertise only existing twins.
Falsifier: A twin exposes unpublished, suspended, expired or private content; differs from the public revision; permits an unsupported method/path; breaks Unicode; lacks text/markdown or default noindex headers; or shared caching keeps serving newly ineligible content.
Mechanism: Proposed markdown-publication; actual HTTP requests through the middleware, eligibility transitions, malformed paths, method tests, alternate-link escaping and cache-header assertions.

## Operating rules

Site defaults include title format, description, social image, publisher identity,
public social-profile links and verification values. Content overrides use the
existing revision boundary. Add a separate SEO-management permission and audit
successful protected changes without storing secrets in audit summaries.

Keep the existing SSR metadata component. Reuse behavior from suprnova.app without
copying its branding or older response rewriting. Structured data must describe
real visible content. HTML remains canonical; Markdown is non-indexable by
default. Apply publication checks within MarkdownSource, and do not accept the
plugin's default one-hour public cache for eligibility-sensitive content.

Use the verified plugin commit dcc6de06a177d66cfc1a9897ded584e19040f40c from
https://github.com/eas4ai/suprnova-markdown.git. It matches the starter's framework
revision. Preserve one framework dependency identity. The plugin supplies routing
and response metadata; the starter supplies approved content.

Search Console verification/setup is included. OAuth imports of search queries or
rankings, ranking guarantees and invented rich-result eligibility are excluded.
