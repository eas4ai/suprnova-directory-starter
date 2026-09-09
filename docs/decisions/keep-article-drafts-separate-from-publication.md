# Keep article drafts separate from publication

Level: Judged
Decided by: Codex
Rests on: CNT-001 CNT-002 CNT-003 CNT-004 CNT-005 CNT-006
Would be wrong if: Saving a draft changes published content, a historical slug leaks private material, or public HTML depends on browser JavaScript.

## Decision

Store immutable article revisions behind separate current and published pointers with a version-checked transaction. Reserve published slug aliases and redirect only while the article is published. Keep editorial categories and tags separate from listing categories; retain in-use relationships when disabling terms and reject destructive removal without explicit reassignment. Reuse the tested Markdown and private image boundary with editorial authorization. Render public Vue pages through the existing Suprnova SSR worker with bounded requests and an explicit error if unavailable, and derive metadata and paginated XML outputs from configured origin and shared publication queries. Private administration remains client rendered. Verify direct requests, draft isolation, stale saves, injection, taxonomy relationships, redirects, XML coverage and actual no-JavaScript HTML.

## Realized by

- 61ee2c7f69a8ac95d5dcdb08a5b7de36d7779b5d — separate article revisions and publication, protected taxonomy/media, real public SSR/metadata/XML, and editorial HTTP/browser verification with failure demonstrations.
