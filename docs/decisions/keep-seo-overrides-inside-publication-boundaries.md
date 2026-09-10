# Keep SEO overrides inside publication boundaries

Level: Judged
Decided by: Codex
Rests on: SEO-001 SEO-002 SEO-003 SEO-004 SEO-005 SEO-006
Would be wrong if: Draft metadata leaks, private content gains a Markdown twin, redirects hijack live routes, or SEO reports claim complete coverage from a truncated scan.

## Decision

Store optional metadata on listing/article revisions and taxonomy records; preserve existing publication pointers and taxonomy authorization. Version site defaults under a separate seo.manage permission and record successful changes without values in audit summaries. Extend the existing SSR metadata component with trusted-origin canonical, social and real structured data. Keep genuine pagination self-canonical; noindex search, combined facets and nondefault page sizes while allowing crawlers to read that instruction. Sitemaps exclude noindex and ineligible content and use stored public modification times only. Reserve existing route namespaces from manual redirects, require bounded same-site paths and live public targets, reject cycles and long chains, and preserve automatic article aliases. Aggregate only bounded nonsensitive path-only 404 records with retention and a hard row cap. Reports use bounded pages and explicitly disclose scope; previews share emitted metadata. Wrap the verified Markdown plugin with method/path and no-store response controls; the MarkdownSource rechecks eligible approved or published revisions and propagates database failure safely. Keep all verification isolated from operator data.

## Realized by

- a1b20086f7f165fa27571dda020344cfdf8a8899 Add revision-aware SEO controls and public Markdown twins
