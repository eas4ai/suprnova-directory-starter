# Notify owners only about retained payment effects

Level: Judged
Decided by: agent
Rests on: KIT-001
Would be wrong if: A rejected observation creates a misleading notice or accepted payment changes lose their durable notice.

## Decision

Resolve the recorded final-review finding by carrying accepted payment effects into notification construction, using retained adverse state for wording, and suppressing rejected stale observations. Add controlled reverse-commit regression coverage and rerun the inherited checks before final review.

## Realized by

- 2f2a9b8 Suppress payment notices for rejected stale observations
