# Site appearance review

commit: c8290c2afeab4b6a9f0e93a437583af55279f72d
findings:
  - resolved: UI-002 dark placeholders now use the shared muted text token; direct Chromium verification increased contrast from 4.11:1 to 7.76:1, and the browser mechanism now checks pseudo-elements.

## Review scope

Reviewed the committed change from 84b1a4c through 9d8b8e4d3e61d0655d87a8edc2ed7a717ba5f576, its 45 current
passing requirement receipts and the public, login, administration and Markdown
screenshots. No application code changed during this review.

Attacked blank/default compatibility, invalid preset and cookie inputs, CSS
injection boundaries, per-render theme ownership, persistence through navigation
and reload, SSR style ordering, mobile keyboard controls and nested typography.
The backend accepts only the documented preset names and two appearance outcomes.
The existing strict custom-accent validation remains intact. Each client or SSR
app creates its own Vuetify theme; concurrent light/dark HTTP requests were checked.
The blank light theme retains the original CSS tokens. No dependency, schema,
provider or authorization behavior changed.

## Mechanism failure sensitivity

- A private source copy changed only the blank light accent to white. The actual
  browser check failed with actual `#ffffff`, expected `#146b56`; the mutation was
  never applied to the source checkout. The corrected suite subsequently passed.
- During construction, equal-specificity head styles lost to the later base CSS.
  The browser rejected the resulting light appearance after a dark selection.
  `html:root` now overrides base tokens regardless of head order; client switching
  and no-JavaScript server rendering both passed.
- Invalid preset names and malformed or unrelated cookies are exercised directly.
  The browser checks all eight independently specified Tailwind color pairs,
  primary-button contrast, hover colors, 320px navigation and isolated SSR output.
  Transition completion is awaited before contrast measurement; screenshots finish
  finite transitions rather than capturing misleading intermediate colors.

## Finding recorded in the first review

The login screenshot exposed dim placeholder text. A direct Chromium check used
the real built CSS and actual SSR-rendered Login component, with JavaScript
disabled. Canvas conversion measured the placeholder as sRGB (106,114,130) over
(9,9,11), or 4.113697664745079:1. The check failed its 4.5:1 threshold.
Map dark placeholders to the shared muted token and extend the browser mechanism
to measure `::placeholder`; retain light-mode colors. Fix this as a separate action.

## Static analysis disposition

Ripwire edit checks reported no incompatible callers for appearance/siteTheme.
Quality-delta returned exit 2 for churn on AuthShare and shared prop types. These
are the intended boundary edits, not reasons to rename or relocate working code.
Its unused findings concern the called cookie parser, test entry points and
adapter interface methods; the compiler and real browser/request checks exercise
them. The reported `read` verbosity is a name-collision attribution to unchanged
billing code. Test-gate returned exit 4 with 18 routes unlinked to tests in its
static graph. The full HTTP, SSR and browser mechanisms ran those domains; no
static-tool pass is claimed. The final production audit is recorded below.

The earlier general design audit and its unrelated administration empty-state
copy issue remain separate from this bounded appearance update.

## Placeholder resolution

A separate implementation action maps dark `::placeholder` to `--text-muted`
with full opacity; light styles are untouched. The browser contrast helper now
accepts a pseudo-element and checks login placeholders. The same direct Chromium
check measured corrected sRGB (161,161,170) over (9,9,11), or
7.762805693682826:1, and passed. The frontend type/build check passed. Fresh Cairn
checks and a final review are required before completion.

## Final review and production self-audit

Re-reviewed the final placeholder diff and corrected login screenshot after all
45 requirements received fresh passing evidence. The last appearance receipts
are UI-001 and UI-002 at 20260910T010457289Z. No application code changed during
this review. Spec lint and whitespace checks passed. No unresolved finding remains
in this appearance commitment.

Applied all 14 production rules: the agreed scope and affected request/render
paths are documented; the change reuses installed dependencies and existing
semantic tokens; validated configuration, typed props and documentation agree.
The cookie contains only a bounded appearance choice and changes no permission,
payment or account state. Invalid configuration retains actionable errors and
introduces no secret exposure. Server renders use separate theme instances; the
preference survives navigation/reload without adding database or migration work.
Palette work is bounded and requires no network service. The todo list records
verified work, and both the negative demonstrations and corrected outcomes are
retained. The full build, database, auth, permissions, provider, directory,
payment, editorial, administration, adoption and appearance checks passed.
Static-tool limitations are disclosed above rather than reported as passes.
The developer's blank-default and light/dark choices are preserved, the review
finding was corrected before delivery, and the setup instructions use concrete
configuration examples. The appearance change is ready to deliver.

## Login link layout follow-up

Reviewed the developer-requested change in 5242836: the existing link container
uses a vertical flex layout with an 8px gap. Link targets, text, semantics and
keyboard order remain intact. The narrow-browser check measured separate lines
with an 8px gap; frontend type checking passed. All 45 requirements have fresh
passing evidence, ending with UI-001/UI-002 at 20260910T014229232Z.
No application code changed during this review and no new finding remains.
Rechecked the 14-rule self-audit above against this one-line presentation change;
no new state, dependency, boundary, security or performance concern is introduced.
The local runtime database is excluded from staging and publication.
