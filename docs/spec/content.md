# Editorial content and public metadata

Status: Draft
Prefix: CNT

[CNT-001] The starter MUST let an authorized editor create, revise, preview, publish and unpublish articles through administration.
Falsifier: An editor cannot complete the article workflow, another actor can mutate it by direct request, a draft leaks publicly, or stale editing overwrites a newer revision.
Mechanism: Proposed editorial HTTP/state tests and article-authoring browser journeys.

[CNT-002] The starter MUST provide public article search, pagination and category/tag navigation restricted to published content.
Falsifier: Draft or unpublished content appears in any public result, filtering is ignored, pagination is unbounded, or public article URLs fail to resolve or redirect after an intentional slug change.
Mechanism: Proposed editorial-discovery HTTP tests with publication and taxonomy fixtures.

[CNT-003] The starter MUST let authorized administrators manage listing categories and article categories/tags without orphaning existing content.
Falsifier: An unauthorized actor edits taxonomy, deleting an in-use term breaks public content, or unpublished content leaks through term counts or term pages.
Mechanism: Proposed taxonomy permission, relationship and visibility tests plus administration browser journeys.

[CNT-004] The starter MUST render public listing and article content with canonical metadata in the initial HTML response.
Falsifier: A crawler without JavaScript receives no meaningful public content, title, description, canonical URL or Open Graph metadata, or structured data describes an unpublished item.
Mechanism: Proposed public-HTML/SEO HTTP checks with JavaScript disabled and browser metadata checks.

[CNT-005] The starter MUST provide valid RSS, sitemap and robots output consistent with current publication eligibility.
Falsifier: A feed or sitemap contains private/unpublished/expired content, XML is invalid, canonical hosts come from untrusted request headers, or pagination omits eligible items from sitemap coverage.
Mechanism: Proposed feed/XML parsing and cross-surface visibility tests with enough records to cross page boundaries.

[CNT-006] The starter MUST render article Markdown and media without permitting active-content injection or private-media disclosure.
Falsifier: Script, executable links or unsafe uploads run in a public article or preview, or an unauthorized request fetches draft article media.
Mechanism: Proposed editorial-content fixtures and browser security/preview authorization checks reusing the verified media boundary.

## Proposed operating rules

- Articles have title, summary, Markdown body, stable slug, optional cover image and
  alternative text, categories and tags. Publishing is explicit; scheduled publishing
  and collaborative editing are not included. Published edits take effect only when
  the editor saves the publication action. Draft previews require editorial permission.
- Use the directory's media limits and safe Markdown rendering. Listing categories
  and editorial taxonomy are separate so one cannot silently repurpose the other.
- In-use taxonomy terms can be renamed or disabled; destructive removal requires
  an explicit reassignment/removal operation in one transaction.
- Initial public HTML supplies visible content and metadata. RSS includes the latest
  50 published articles. Sitemap output covers eligible listings, articles and useful
  public taxonomy pages with bounded pagination; private/account/admin URLs are absent.
- Metadata uses configured application origin and branding. Structured data uses
  appropriate Article, ItemList and breadcrumb shapes without inventing ratings or
  unsupported business claims. Robots exclusion is not an authorization mechanism.
