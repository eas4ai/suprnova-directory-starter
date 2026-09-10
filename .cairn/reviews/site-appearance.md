# Site appearance review

commit: 9d8b8e4d3e61d0655d87a8edc2ed7a717ba5f576
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

## Open finding

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
static-tool pass is claimed. The final production audit awaits fresh evidence after the placeholder fix.

The earlier general design audit and its unrelated administration empty-state
copy issue remain separate from this bounded appearance update.

## Placeholder resolution

A separate implementation action maps dark `::placeholder` to `--text-muted`
with full opacity; light styles are untouched. The browser contrast helper now
accepts a pseudo-element and checks login placeholders. The same direct Chromium
check measured corrected sRGB (161,161,170) over (9,9,11), or
7.762805693682826:1, and passed. The frontend type/build check passed. Fresh Cairn
checks and a final review are required before completion.
