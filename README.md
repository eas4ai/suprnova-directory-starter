# Suprnova directory starter

A free directory starter built on Suprnova, with owner submissions, moderation,
directory search and free or paid publication through Stripe and Paddle.
Licensed under [MIT](LICENSE), with [third-party notices](THIRD_PARTY_NOTICES.md).
See [the roadmap](docs/spec/roadmap.md) for the current verification status.

## Prerequisites

- Rust 1.94 or newer and Cargo; this checkout is verified with Rust 1.95.
- Bun 1.4.1 for frontend packages and database inspection.
- Node 24 for local key generation and verification. Cairn is needed only for
  specification lint and evidence recording, not to run the application.
- A C/C++ toolchain, pkg-config and OpenSSL development headers for native dependencies.
- Git and network access for the first dependency installation.
- Python 3 for XML parsing in the editorial verification suite.

## Install and migrate

Run from a fresh checkout. Keep an existing operator configuration when updating
an installation; the copy command below is only for a new checkout.

<!-- foundation:install -->
```sh
cp .env.example .env
node -e 'const fs=require("node:fs"); const crypto=require("node:crypto"); const p=".env"; fs.writeFileSync(p,fs.readFileSync(p,"utf8").replace(/^APP_KEY=.*$/m,"APP_KEY="+crypto.randomBytes(32).toString("base64url")))'
cargo build --locked --bins
cd frontend
bun install --frozen-lockfile
bun run build
bun run build:ssr
cd ..
cargo run --locked --bin directory -- migrate
```

The default database is `database.db` in this checkout. SQLite requires no database
service. The migration command creates account and billing settings tables and records completed
migrations. Running the same command again applies only pending migrations.
It is Suprnova's application migration entry point; the developer CLI's
`suprnova migrate` delegates to it.

## Run locally

Start Vite in one terminal, from the repository root:

<!-- foundation:vite -->
```sh
cd frontend
bun run dev
```

In a second terminal, run the public-page renderer from the repository root:

<!-- foundation:ssr -->
```sh
node --env-file=.env frontend/bootstrap/ssr/ssr.js
```

In a third terminal, from the repository root:

<!-- foundation:serve -->
```sh
cargo run --locked --bin directory -- serve --no-migrate
```

Open `http://localhost:8765`. The public directory and account forms should render.
Keep all three terminals running; stop each with Ctrl+C. Local mode loads modules from
Vite on port 5765. `bun run build` checks production assets but does not replace
Vite in local mode. The guide uses the application binaries directly, so the
separately installed Suprnova developer CLI is not required.

Public content and metadata are rendered in the initial HTML by the Vue SSR worker.
The worker binds to loopback port 13714. A stopped worker produces an explicit page
error instead of a blank public shell. Private account and administration forms
remain available. After changing Vue pages, run `bun run --cwd frontend build:ssr`
and restart the renderer. Supervise this worker alongside the application in production.

If either port is occupied, export `SERVER_PORT`, `VITE_PORT`, and `APP_URL` with
matching free ports in the relevant terminals before running the commands. The backend
reads these before `.env`, and Vite reads `VITE_PORT` from the terminal environment.
The setup verifier uses this documented override with ephemeral local ports.
If the renderer port is occupied, also export matching `SSR_PORT` and `SSR_URL`
in the renderer and application terminals. Keep the renderer on a private interface.

## Account mail

To register or recover accounts, run a local SMTP capture server on
`localhost:1025`, without authentication or encryption, as configured in
`.env.example`. The server and forms can start without it, but sending account
mail requires it. Use the capture server's inbox to open verification/reset links.
An external SMTP relay must be configured separately before deployment.

Account email uses the SMTP service on port 1025 from the example configuration.
Registration sends a verification link; sign in and open `/verify-email` to resend
it. Open the link while signed in to the account it belongs to. Password reset
at `/forgot-password` sends links only for verified accounts. Unknown and unverified
addresses receive the same public response. Resetting a password revokes existing
sessions and remember tokens. Mail links use `APP_URL`, so set it to the address
where the application is reachable.

## Administrator access

Register an account, verify its email, then read its account ID on **Your account**.
From the application host, replace `42` below with that exact ID:

```sh
cargo run --locked --bin console -- admin:access grant --user-id 42
```

The command grants `admin.access`, `billing.configure`, `listings.moderate`,
`articles.manage`, `taxonomy.manage`, `accounts.manage`, `audit.view` and
`seo.manage` through
the explicit Suprnova administrator role. It also reinstates a suspended account.
It refuses unknown or unverified accounts. Repeating a grant is safe. Sign in and
open **Administration → Payment providers**. An existing session sees the grant
on its next request; registration itself never grants administrative access.

To revoke access, including access in an existing session:

```sh
cargo run --locked --bin console -- admin:access revoke --user-id 42
```

Revocation removes these direct permissions and any role memberships granting
any of them. Other role memberships remain. Roles and their permissions
are not deleted. The same host command can recover access if no administrators
remain; no public bootstrap endpoint is exposed.

**Administration → Accounts** provides bounded search, read-only verification status,
predefined roles, suspension and reinstatement. Moderators get listing moderation;
editors get article editing and publication. Taxonomy, billing, accounts, audit and SEO each
require their own permission. Role names alone carry no authority. Saving account access replaces starter direct
grants and memberships that confer starter permissions; unrelated roles remain.

**Administration → Overview** shows current publication counts and pending listing
reviews for the capabilities you hold. Published counts use the same eligibility
rules as the public directory; a published listing can also have changes awaiting
review. Editors see published article counts, and accounts with `audit.view` see
the five most recent administrative actions. Query failures remain errors rather
than appearing as zero activity. Navigation groups the available content, billing,
administrative and account destinations.

Role grants require verified email. The UI prevents removal or suspension of the last
active verified full administrator and rejects stale forms. The host command can
recover access without overriding email verification. Suspension blocks new sign-in,
existing sessions and protected actions and hides the owner's public listings.
Reinstatement preserves listing moderation and payment eligibility.

**Administration → Audit history** is paginated and requires `audit.view`. Decisions
record the acting account, target, time and changed fields in the same transaction.
Host commands are attributed to **Host operator**, not to the target account; this
records use of the command, not the operator's individual OS identity. Passwords,
provider credentials and owner-facing moderation reasons are excluded from summaries.

## Articles, taxonomy and public metadata

Editors use **Administration → Articles** to create an article, save a draft and
preview the saved version. Saving changes to a published article keeps the current
public version intact. Use **Publish saved changes** to replace it explicitly, or
**Unpublish article** to remove it from public pages, RSS and sitemap output.
Conflicting saves preserve your entered text and offer a link to reload the latest
revision. A deliberate published slug change redirects the old URL while the
article remains published; old slugs cannot be assigned to another article.

**Taxonomy** manages listing categories separately from article categories and tags.
Rename or disable an in-use term to preserve existing relationships. Removing an
in-use term is rejected. Article covers use the same private disk and safe image
limits as listings; draft previews and their media require editorial permission.
Markdown renders without raw HTML or executable links.

Public article search is at `/articles`, RSS at `/feed.xml`, the sitemap index at
`/sitemap.xml` and crawler instructions at `/robots.txt`. RSS contains the latest
50 published articles. The sitemap uses pages of 100 eligible URLs. Content and
canonical, Open Graph and structured metadata appear in initial HTML through the
SSR worker. Set `APP_URL` to the site's public origin; request headers never choose
canonical hosts.

Set `APP_NAME`, `SITE_DESCRIPTION`, `SITE_LOGO_URL` and `SITE_ACCENT` in the deployment
environment to customize both shells and metadata. A logo can use an absolute
HTTP(S) URL or a site-relative path such as `/brand/logo.png`. Quote the hex accent
in `.env`, for example `SITE_ACCENT="#146b56"`; it must contrast with white text by
at least 4.5:1. Invalid settings return a diagnostic error. Restart the application
after changing configuration.

For a quick color preset, set `SITE_THEME` to `zinc`, `blue`, `indigo`, `violet`,
`emerald`, `teal`, `rose` or `orange`. For example, `SITE_THEME=indigo` uses the
Tailwind indigo palette. Leave `SITE_THEME=` blank (the default) to retain the
current theme and your `SITE_ACCENT`. A selected preset takes precedence over the
custom accent. Restart the application after changing it.

The sun/moon button in navigation switches between light and dark. Light is the
initial default; a first-party `site_appearance` cookie remembers the visitor's
choice for one year. Server-rendered public pages honor the cookie before
JavaScript runs. Dark mode uses lighter accent shades for readable links and
buttons. No extra dependency or theme editor is needed.

## SEO controls and Markdown publication

Use **Administration → SEO** for site defaults, verification values, findings,
search/social previews, manual redirects and the 404 report. Access requires both
`admin.access` and `seo.manage` on a verified, active account. The full
`admin:access grant` bundle includes this permission; existing installations can
rerun that command for their intended administrator after migration. Editors keep
using their existing listing, article and taxonomy permissions for content SEO.

Search titles, descriptions, social images and noindex flags are optional. Blank
values fall back to the content title, summary and cover, then site defaults.
Listing overrides become public through moderation; article overrides become public
through publishing. Saving a draft never changes public metadata. Taxonomy edits
apply immediately. Successful settings, redirect and taxonomy changes are audited;
verification values are omitted from audit summaries. Conflicting saves preserve
entered values and require reloading the saved version.

The title format requires `{title}` once and permits `{site}` once. Publisher and
social profiles describe the actual publisher; no reviews, ratings or business facts
are generated. Paste Google Search Console or Bing verification **values**, without
HTML tags, save, and complete verification in that service. For Search Console's
HTML-tag method, use a URL-prefix property matching `APP_URL`; domain properties use
DNS verification outside this form. Submit `/sitemap.xml` after verification. No
OAuth connection, search-query import or ranking score is provided.

Public canonical URLs use the configured `APP_URL`. Real pagination keeps its own
page number; tracking parameters are omitted. A single active taxonomy filter can
be indexed. Search queries, combined filters and non-default page sizes
receive noindex. Content or site noindex excludes affected URLs from sitemaps while
allowing crawlers to read the instruction. Whole-site noindex empties the sitemap
index. Article/listing sitemap dates use stored public revision dates; static and
taxonomy URLs omit modification dates. Noindex does not make content private.

Manual redirects return 301 and drop query strings. Sources must be plain site
paths outside existing route namespaces and public files; encoded paths and
Markdown paths are rejected. Destinations must currently resolve to a public
listing, article or index. Limits are 1,000 redirects and five steps per chain.
Cycles, stale edits and removing a destination with incoming redirects are rejected.
Published article slug changes retain their existing automatic redirects.

The 404 report stores path-only observations for 30 days, capped at 1,000 paths,
with 25 rows per page. It omits queries, credentials, sensitive namespaces, encoded
paths, Markdown paths and long segments. Counts include bots and are not unique
visitors. Findings compare saved metadata across eligible public content; previews
use the same resolver as public HTML. Search/social services may display different
text. External image availability is not checked by the server.

Eligible articles and listings advertise a `.md` alternate link in their HTML.
The pinned `suprnova-markdown` plugin supplies Markdown responses; the application
loads the current public revision and checks eligibility on every request.
Unpublishing, suspension, expiry or revoked eligibility removes the twin immediately
for subsequent requests. GET and HEAD are supported; responses use UTF-8
`text/markdown`, `X-Robots-Tag: noindex`, `nosniff` and `Cache-Control: no-store`.
HTML remains canonical. Noindex content stays publicly readable in both formats.
The dependency is pinned to `dcc6de06a177d66cfc1a9897ded584e19040f40c` and shares the
starter's framework revision; do not update one without checking the other.

Run `node scripts/verify-complete-starter.mjs seo` for disposable HTTP, SSR and
browser verification. Screenshots go to `target/seo-ui`. Run
`node scripts/demonstrate-seo-failures.mjs` to exercise deliberately broken copies.
These commands do not modify the operator database.

## Listings and moderation

After migrating, create the initial categories:

```sh
cargo run --locked --bin console -- directory:categories
```

Repeated runs preserve existing terms. Verified owners use **Your listings** to
save drafts, upload an optional image, and submit for review. Administrators use
**Listing reviews** to approve or reject the exact submitted revision. Rejected
owners see the reason and can save a revised draft before resubmitting. Existing
approved content stays public during editing when publication eligibility remains
valid. After approval, owners choose a publishing plan. Approval alone does not
grant publication. Free plans grant eligibility immediately; paid plans wait for
verified provider evidence.

Images are decoded and re-encoded as PNG, with a 5 MiB input limit and maximum
dimensions of 4096 × 4096. Set `DIRECTORY_MEDIA_ROOT` to a private local directory
outside `public/`; the default is `storage/private/directory`. The process needs
write access. Back up that directory with the database. Draft images require owner
or moderator access, and public image routes check current listing eligibility.

## Payment provider configuration

Select **Test mode** or **Live mode**, enter a provider's three credentials, enable
it if desired, then save. Both modes start disabled. Stripe needs its secret API
key, publishable key and webhook signing secret. Paddle needs its API key, client
token and webhook key; test mode uses Paddle Sandbox. Use current mode-prefixed
credentials. Legacy Paddle API keys without a mode prefix are not accepted.

API and webhook secrets are encrypted with Suprnova's context-bound AES-GCM under
`APP_KEY`. They are never returned to the browser. Leave replacement fields blank
to preserve saved secrets. To remove credentials, disable the provider and select
**Clear credentials when saved**. Clearing also removes the public client key.
A disabled provider cannot remain the default. Disabling alone preserves its
credentials and mappings.

Database settings are authoritative: `STRIPE_*` and `PADDLE_*` environment variables
do not override them. `APP_KEY` must be generated and explicitly configured even
for local billing administration. The application's temporary local development
key is insufficient. Keep the key in deployment secret storage and back it up
separately from the database. If the key is lost or the stored ciphertext is
damaged, the UI refuses edits; restore the matching key/database and restart.
Changing the key does not automatically migrate existing billing credentials.
This starter does not provide an automatic key-rotation command.

Map each local plan identifier to a Stripe `price_…` and/or Paddle `pri_…` identifier.
Mappings are separate by mode; a blank price has no fallback. The limit is 100
plans per mode. Plan identifiers use 1–64 lowercase letters, digits, hyphens or
underscores; credential and price fields are bounded to 512 characters.
Remove a mapping row and save to delete it. Saving a stale form reports a conflict;
reload the saved settings before applying your changes again.

Saving validates local format and adapter construction. It does **not** contact
Stripe or Paddle, verify authentication or price availability, or create a checkout.
Create the corresponding local plan under **Publishing plans**. Choose free,
one-time, monthly or annual billing, enter the advertised amount in minor units
and currency, and enable it. Existing purchases retain their original terms when
plans, mappings or provider configuration change.

## Paid publication and recovery

Set `BILLING_CHECKOUT_MODE=test` or `live` explicitly. Only the selected profile
is offered to owners; test payments never appear in the public directory.
Configure the matching provider webhook destination:

| Provider | Test path | Live path |
| --- | --- | --- |
| Stripe | `/billing/webhooks/stripe/test` | `/billing/webhooks/stripe/live` |
| Paddle | `/billing/webhooks/paddle/test` | `/billing/webhooks/paddle/live` |

Use your HTTPS application origin before each path. Supply each destination's
signing secret in its matching profile. Subscribe to checkout/transaction,
invoice, subscription, refund and dispute/adjustment events relevant to your
provider. The endpoints authenticate and retain evidence, then return 202;
acceptance does not mean fulfillment succeeded.

Run the following from a scheduler every minute, with the same database and
`APP_KEY` as the web process. It processes at most 25 due events, with a 120-second
batch deadline and a 90-second per-event deadline. Expired worker leases are
recoverable. Automatic retries stop after eight failed attempts.

```sh
cargo run --locked --bin console -- billing:reconcile --limit 25
cargo run --locked --bin console -- billing:reconcile --status --limit 100
```

Monitor command failures and unresolved event IDs. Correct a price, currency,
customer, settlement or credential problem at its source, then retry the retained
event. The `--event` value is the local event ID printed by the status command:

```sh
cargo run --locked --bin console -- billing:reconcile --event EVENT_ID
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID
```

A missing webhook can be recovered from the purchase's saved session or
subscription. If the create response was lost, look up its `purchase_id`
metadata/custom data in the provider dashboard and supply the existing resource:

```sh
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID --resource PROVIDER_RESOURCE_ID
```

Recovery only reads existing provider state; it never creates charges. Stripe
checkout retries retain the original idempotency key within the supported window.
An ambiguous Paddle create remains pending: do not create another transaction to
work around a lost response. Recover the original transaction by correlation.
Browser return parameters cannot grant publication.

One-time payment has no expiry. Recurring access ends at the last verified paid
period, without a grace period for failed renewal. Owner cancellation requests
stop future renewals only after provider confirmation; scheduled cancellation
retains the paid period. Full refunds remove the affected payment's eligibility,
partial refunds preserve it, open disputes suspend it, and confirmed wins restore
it subject to moderation. Initiate refunds in the provider dashboard.

Disabling a provider stops new checkout while existing purchases remain
recoverable. Purchases retain encrypted credentials and immutable price terms;
clearing a disabled profile is permitted only when retained recovery material is
readable. Old purchase signing keys and the currently configured key can overlap
during replacement. Authenticated retained events replay without rejecting their
now-old signature timestamps. If the old API key is revoked, install the new key
in the same account/mode and explicitly select it for read-only recovery:

```sh
cargo run --locked --bin console -- billing:reconcile --purchase PURCHASE_ID --use-current-credentials
```

Do not change provider accounts to recover an existing purchase. Keep the
database and matching `APP_KEY` together in backups; clearing current settings
does not erase the encrypted material retained with purchases. Raw authenticated
events remain private database records and may contain provider customer data.

Local verification uses synthetic provider responses and the actual pinned
adapters' signature checks. Before accepting real money, use operator-owned
sandbox accounts and matching prices to exercise an approved test listing,
checkout, actual webhook delivery, reconciliation, renewal/cancellation and
refund/dispute outcomes. Confirm test listings stay private. This external smoke
test is separate from the local suite and has not been performed by the starter's
local verifier.

## Verification

```sh
node scripts/spec-lint.mjs docs/spec
node scripts/verify-foundation.mjs build
node scripts/verify-foundation.mjs database
node scripts/verify-foundation.mjs accounts
node scripts/verify-foundation.mjs ui
node scripts/verify-foundation.mjs setup
node scripts/verify-provider-administration.mjs
node scripts/verify-paid-directory.mjs directory
node scripts/verify-paid-directory.mjs payments
```

The verifiers copy application source into a disposable directory under the sibling
`scratchpads` directory. Set
`FOUNDATION_SCRATCH_DIR` to choose another scratch root. Each run removes its
own disposable directory on completion, including failed checks. It never copies
the checkout's `.env` or existing database. The build check uses committed locks;
the database check runs migration twice against its own SQLite file and compares
the account schema and migration history. Compiled Rust artifacts are cached under
the source checkout's ignored `target` directory. Editing-time checks include new,
unignored source files; Cairn evidence requires committing all declared inputs first.

The database check establishes SQLite behavior only. PostgreSQL behavior has not
been verified. Use `cairn wake` for the next action; receipts and logs remain tracked
under `.cairn/evidence`.

The account check exercises the actual HTTP router and middleware with an isolated
database and a captured mail transport. It checks registration, verification,
login, logout, reset, CSRF rejection, token expiry/reuse/ownership, and session
revocation. It does not establish delivery through an external SMTP service.

The UI and setup checks need Chromium installed once:

```sh
cd frontend
bunx playwright install chromium
cd ..
```

On Linux, install the browser's system dependencies with
`bunx playwright install-deps chromium` from `frontend` if Playwright reports
missing libraries. The UI check runs built assets against a disposable server,
checks permissions and keyboard controls, and changes a shared branding token.
Screenshots are written to the ignored `target/foundation-ui` directory.

The setup check executes the marked README command blocks in a disposable copy,
creates a new local key, migrates the example database, starts Vite and the
application, and opens the public and sign-in pages in Chromium. It also checks
an invalid login returns a visible error. It does not send SMTP mail or establish
external delivery. No operator configuration or database is read or copied.

Both shells use the semantic tokens in `frontend/src/styles/tokens.css`.
Administration requires the explicit `admin.access` permission; an account or
an `administrator` role name alone grants no access.

The provider verifier exercises both real adapters without external requests,
direct HTTP access and CSRF failures, the administrator command, encrypted storage,
wrong/missing/default keys, corrupted ciphertext, concurrent and failed writes,
mapping changes, and browser administration journeys. It captures output to check
for secret markers. These tests use synthetic credentials and establish local
integration only; actual provider authentication and delivery remain unverified.

The paid-directory verifier exercises signed ingress, rejected evidence,
transaction interruption, replay, renewals, refunds, disputes and recovery in both
modes. It runs local HTTP contracts for the pinned SDKs and checkout adapters, then
browser journeys for plans and purchases. Payment screenshots go to
`target/payments-ui`; directory screenshots go to `target/directory-ui`.

## Demonstration installation

Use a fresh local database for demonstrations. The seed creates clearly identified
categories, listings in representative states and published/draft articles. It
creates no fixed privileged password. Provision your own verified administrator
with the host command above. Demo entitlements are synthetic; no payment API is
called. Repeating these commands preserves operator records and avoids duplicates.

<!-- adoption:seed -->
```sh
cargo run --locked --bin console -- directory:categories
cargo run --locked --bin console -- directory:demo
```

`directory:demo` refuses production mode unless the host operator deliberately adds
`--allow-production`. That override authorizes demo records in the production
database; prefer a separate demonstration installation.

## Owner notifications

Owners can read their latest 20 moderation and payment notifications on listing
edit and purchase status pages. Domain changes and notification intents commit together. A failed mail send
retains the intent; restarting does not discard it. Run the delivery command every
minute using the same database, application origin and SMTP configuration as the
web process. The batch defaults to 25 intents; `--limit` accepts 1 through 100.
Each send has a 15-second timeout, each lease lasts 60 seconds, and a batch has a
120-second deadline. The worker checks the remaining budget before starting the
next send. Automatic delivery stops after eight failed attempts. Retry delays are
`30 × 2^attempt` seconds: the first is 60 seconds and the eighth is 7,680 seconds.

<!-- adoption:operations -->
```sh
cargo run --locked --bin console -- billing:reconcile --limit 25
cargo run --locked --bin console -- billing:reconcile --status --limit 100
cargo run --locked --bin console -- notifications:deliver --limit 25
cargo run --locked --bin console -- notifications:deliver --status --limit 100
```

After correcting SMTP configuration, explicitly retry a retained notification by
its local ID. This renews the attempt budget only when it is unsent and has no
live worker lease:

```sh
cargo run --locked --bin console -- notifications:deliver --retry ID
```

Monitor failures and exhausted attempts. SMTP delivery is at least once: a lost
acknowledgment can produce a duplicate email. History remains available even when
mail delivery fails. Never put provider credentials or raw webhook payloads in mail.

## Production operation

Build both frontend bundles and the Rust binaries from the locked dependencies
before deployment. Set `APP_ENV=production`, `APP_DEBUG=false`, an explicit stable
`APP_KEY`, `APP_URL=https://your-domain.example`, `SESSION_SECURE=true` and
`SESSION_COOKIE_PREFIX=__Host-`. Use an absolute writable `DATABASE_URL` and private
`DIRECTORY_MEDIA_ROOT`; retain them outside replaceable release directories.
Configure authenticated encrypted SMTP and a real sender; the example local
unencrypted capture server is not a production relay.

Terminate TLS at a reverse proxy and forward requests to the application bound
on loopback or a private network. Keep the SSR worker private; never expose its
port publicly. Forward the original host and HTTPS scheme only from the trusted
proxy, strip client-supplied forwarding headers, and restrict backend access to
that proxy. Set the canonical origin explicitly through `APP_URL`. Check secure
cookies, login redirects, uploads and canonical URLs through the actual proxy.
Serve the production assets from the built release and do not run Vite in production.

Supervise the application and `node --env-file=.env frontend/bootstrap/ssr/ssr.js`
with restart-on-failure and bounded logs. The application command is
`cargo run --locked --bin directory -- serve --no-migrate`; an installed release
may run its compiled `directory serve --no-migrate` directly. Run migrations once
before starting the new release. Schedule the two bounded processing commands in
`adoption:operations` every minute and inspect their status commands during recovery.
These commands process persisted work; no separate undocumented queue daemon is
required. Run all processes as the dedicated application OS user.

The implemented media store is local private storage. Suprnova offers other storage
adapters, but switching this starter to one requires an application integration and
a provider smoke test; setting cloud credentials alone does not move existing media.
Keep read/write access limited to the application user and backup operator.

## Backup, restore and permission recovery

Stop the web process, notification/reconciliation schedules and any other database
writers before this procedure. Set `BACKUP_DIR` to a new private directory on
protected backup storage. The following SQLite recipe uses Python's online backup
API, includes the media tree, and saves the matching key separately with mode 0600.
It assumes `DATABASE_URL` is a plain `sqlite://` file URL without query options;
use your database tooling for other URL forms. Export the same values used by the
application first. Never print `APP_KEY` or commit backups.

<!-- adoption:backup -->
```sh
python3 - <<'PYBACKUP'
import os, pathlib, shutil, sqlite3
backup = pathlib.Path(os.environ['BACKUP_DIR'])
backup.mkdir(mode=0o700, parents=True, exist_ok=False)
url = os.environ['DATABASE_URL']
assert url.startswith('sqlite://') and '?' not in url, 'Use a plain SQLite file URL'
with sqlite3.connect(url[9:]) as source, sqlite3.connect(backup / 'database.db') as target:
    source.backup(target)
shutil.copytree(os.environ['DIRECTORY_MEDIA_ROOT'], backup / 'media')
key = backup / 'APP_KEY'
key.write_text(os.environ['APP_KEY'])
key.chmod(0o600)
PYBACKUP
```

Restore into new paths while writers remain stopped. Set `RESTORE_DIR` to a new
private directory, then run:

<!-- adoption:restore -->
```sh
python3 - <<'PYRESTORE'
import os, pathlib, shutil, sqlite3
backup = pathlib.Path(os.environ['BACKUP_DIR'])
restore = pathlib.Path(os.environ['RESTORE_DIR'])
restore.mkdir(mode=0o700, parents=True, exist_ok=False)
shutil.copy2(backup / 'database.db', restore / 'database.db')
shutil.copytree(backup / 'media', restore / 'media')
shutil.copy2(backup / 'APP_KEY', restore / 'APP_KEY')
with sqlite3.connect(restore / 'database.db') as db:
    assert db.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
PYRESTORE
```

Configure the restored database/media paths and load the saved key into deployment
secret storage without logging it. Restore ownership to the application OS user;
private directories require owner read/write/traverse access, files owner read/write.
Keep parent directories inaccessible to public serving. Start the restored release,
run migration, inspect notification/payment status, and verify an existing account,
private image and encrypted billing settings before reopening traffic. Resume
schedules only after that inspection. Preserve the old deployment until recovery
is confirmed. A database without its matching key cannot recover encrypted provider
credentials. A database without its matching media backup cannot recover images.

If all administrator permissions were removed, use `admin:access grant --user-id 42`
against the restored verified account, substituting its actual ID. This also
reinstates a suspended account. Do not modify verification state with SQL or create
a shared emergency password. Revoke temporary recovery access after restoring the
intended administrators. Backups contain account and payment data: encrypt off-host
copies, limit access, define retention and rehearse restoration regularly.

## Complete-starter acceptance and external limits

Run `node scripts/verify-complete-starter.mjs adoption` from the source checkout.
It creates a disposable source snapshot, uses locked build inputs and executes the
marked seed, operations, backup and restore blocks above. The helper
`scripts/verify-adoption-install.mjs` runs only inside that supplied disposable
workspace; it does not read the checkout's operator configuration. Foundation
setup checks execute the install/serve blocks and browser account and moderation
journeys exercise the real application. The complete-starter groups are
`editorial`, `administration`, `adoption`, `overview` and `seo`; run the earlier verification commands
as regressions as well. Passing editing-time checks are not Cairn receipts.

Local acceptance establishes SQLite, captured mail, local persistent media and
synthetic Stripe/Paddle behavior. PostgreSQL, external SMTP, cloud storage, TLS
proxy deployment and real Stripe/Paddle accounts remain **untested externally**.
For each deployment, verify the public HTTPS flow and authenticated relay delivery,
restore a backup on the actual storage/database service, and exercise both selected
provider sandboxes with real webhook deliveries before enabling live payments.
No production credentials are required or expected in the local acceptance suite.
