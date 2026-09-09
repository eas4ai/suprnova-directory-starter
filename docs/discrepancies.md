# Foundation discrepancy log

Observed during the starter-foundation commitment, 2026-09-08 (America/New_York).
These findings describe this checkout and its installed tools. A starter integration
mistake is not automatically an upstream framework defect. Cairn development notes
are in `.cairn/reviews/starter-foundation.md`; committed check output is under
`.cairn/evidence`. Failed editing-time checks are described here and in the review.

## Application and integration findings

| Finding | Evidence and effect | Correction / status |
| --- | --- | --- |
| Registration did not send verification mail; verification and password-reset routes were absent. | HTTP registration captured zero messages; subsequent route tests returned 404. | Added released Suprnova account-flow calls and routes. Account journeys pass, including invalid, expired, reused and wrong-owner tokens. |
| Compiled frontend assets were not served. | The production-asset browser check requested the generated assets and received 404; shell rendering failed. | Added the framework's public static-file fallback in `src/routes.rs`. Browser rendering passes. |
| The scaffold cached its initial CSRF meta token. | Login rotated session state; later Inertia logout returned 419 `CSRF token mismatch`. | Removed the override in `frontend/src/main.ts`. Installed Inertia 3.7 already reads the current XSRF cookie. Browser login followed by logout passes. Server token rejection remains enabled. |
| The scaffold's CSRF comment described the wrong installed transport behavior. | It said Inertia used native fetch and needed manual token forwarding. Installed `@inertiajs/core` 3.7 uses its XHR client and automatically sends X-XSRF-TOKEN. | Replaced the stale comment and removed redundant forwarding. An intermediate manual XSRF header produced a combined 82-byte value instead of one 40-byte token and was removed. |
| CSRF cookie configuration was not synchronized with session configuration. | The scaffold used `CsrfMiddleware::new()` independently of `SessionConfig`; the released middleware defaults its cookie to Secure, while `.env.example` selects local HTTP. | Use the existing `with_session_config` builder. This keeps cookie path, lifetime, domain, SameSite and Secure settings aligned. It is an application wiring correction, not a missing framework API. |
| Login and registration explicitly supplied `errors: null`. | The fresh-setup browser check submitted invalid credentials. The server returned a validation redirect, but the page displayed no error. | Removed the explicit error props; forms read `useForm().errors`. The corrected fresh-setup browser check displays the error. Generated prop types were refreshed. |
| Invalid reset tokens initially produced raw JSON for an Inertia form. | The account test observed 400 where the form needed a validation redirect. | Translate only the framework's bad-request token failure into field validation. Database and revocation errors continue to propagate. |
| Mobile navigation initially had no sign-out action. | The desktop action container is hidden on small screens; inspecting the mobile navigation found no replacement. | Added a mobile sign-out form and verified it by keyboard, followed by denied dashboard access. |
| README stopped after installation/migration. | It did not tell a fresh operator to run Vite and the application, generate a persistent local key, or install Chromium for browser verification. | README now has executable install/Vite/server blocks, prerequisite and mail limits, port overrides, and browser setup. Disposable execution passes. Removing the migration command in a safe fixture makes the check fail at the missing database assertion. |
| The handoff described the initial scaffold as the current state. | It still said requirements were unconfirmed and no builds or implementation had occurred. | Replaced it with the agreed contract, implemented foundation, verification limits and current Cairn entry point. Earlier recon documents remain historical. |

## Verification-harness findings

| Finding | Evidence | Correction / status |
| --- | --- | --- |
| SQLite probe used incompatible Bun connection options. | `readonly: false, create: false` produced SQLITE_MISUSE in a focused disposable-file experiment. | Use `readwrite: true, create: false`. Repeat migrations and the persisted data probe pass. |
| HTTP harness initially omitted framework configuration and encryption initialization. | The first test failed during bootstrap before checking account behavior. | Initialize the real configuration and a fresh test key before the database and HTTP stack. No application failure was inferred from these setup failures. |
| A modal focus assertion rejected browser chrome. | After tabbing past modal controls, focus could move to browser UI while the native dialog stayed open; the page body was inactive. | Permit focus outside the document, but still reject focus on background page controls. |
| The dialog test raced an Inertia navigation. | It opened the old public page's dialog before the admin page mounted, then treated the old component's removal as failed focus restoration. | Wait for the admin shell and URL before using its controls. Removed attempted custom focus patches; native dialog Escape, reopen and confirmation pass. |
| The setup verifier imported Playwright before installing dependencies. | The disposable copy failed with ERR_MODULE_NOT_FOUND before README installation ran. | Import Playwright dynamically after executing the guide's install block. |
| Earlier temporary copies used `/tmp`. | This was the original verifier default; the developer requested workspace scratchpads. | Default to the sibling `scratchpads` directory, allow FOUNDATION_SCRATCH_DIR, direct child temporary data there, and remove each run's copy. No task-owned `/tmp/directory-foundation-*` directories or processes remained at the cleanup check. |

## Tool behavior to keep in mind

The installed `suprnova generate-types --help` invocation actually scanned the
checkout and regenerated `frontend/src/types/inertia-props.ts`, returning exit 0;
it did not display help. Generation was relevant to the current changed props,
and the result was inspected. Treat this subcommand's help behavior as a CLI
follow-up, not a reason to assume all `--help` invocations are read-only.
It also emitted extra blank lines at EOF, which the Git whitespace check rejected;
the generated file was normalized. No upstream CLI change was made under this
starter commitment.

Graph line metadata was stale in some inspected files. Focused source reads were
used where the returned snippet did not match the released tag or current file.
Framework development source is ahead of v1.3.7; integration decisions use the
exact released tag rather than assuming unreleased APIs are available.

## Limits, not defects established by these tests

- SQLite is verified; PostgreSQL has not been exercised.
- Captured account mail proves generated flow behavior, not external SMTP delivery.
- Client and SSR bundles build; runtime SSR and deployment are not verified.
- Listing publication, payment administration and NOWPayments are later work.
- The public theme is a neutral light theme with teal accent. The selected UI
  stack is Vue/Inertia/Vuetify 0; matching suprnova.app's dark-only branding was
  not an agreed requirement.

Append new findings with observed evidence and an explicit status. Do not erase
failed experiments or call an untested limitation a confirmed product defect.
