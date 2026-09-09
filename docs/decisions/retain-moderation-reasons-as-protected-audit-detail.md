# Retain moderation reasons as protected audit detail

Level: Judged
Decided by: Codex
Rests on: DIR-003 DIR-007 ADM-003
Would be wrong if: A suspension or reinstatement loses its recorded reason, or broad audit output reveals private moderation free text.

## Decision

The administration regression exposed a real loss of suspension reasons when audit summaries changed to safe field names. Add nullable private moderation detail to each audit record. Listing decisions save their reason and actor atomically; the shared audit DTO exposes only safe summaries. Preserve historical summary text in storage and copy it into the protected detail during migration. Keep the existing reason assertions against this dedicated field and add a broad audit-page exclusion assertion.

## Realized by

(none yet: recorded, not built)
