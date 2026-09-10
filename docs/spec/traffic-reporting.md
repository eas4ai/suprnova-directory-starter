# Traffic, revenue and engagement reporting

Status: Agreed 2026-09-10
Prefix: TRF

[TRF-001] The starter MUST collect public page views once per completed browser navigation and keep Markdown fetches, prefetches, assets, private/account/admin routes and identifiable automated traffic out of browser traffic totals.
Falsifier: Initial rendering plus Inertia navigation records two views for one navigation, replaying an event ID increments totals, a prefetch counts as a view, or a private/Markdown request appears as browser engagement.
Mechanism: Proposed traffic-collection; real browser initial/Inertia visits, replay and prefetch fixtures, private-route requests and Markdown fetches with database assertions.

[TRF-002] Public link clicks MUST record source page, validated destination/content identity and link role separately from page views, while navigation remains usable if collection fails.
Falsifier: The listing website action accepts a caller-supplied destination, hidden/ineligible content can generate valid listing clicks, a page view counts as a click, published prose links are missed, or an analytics failure prevents navigation.
Mechanism: Proposed traffic-links; browser website/prose/internal-link clicks, forged destinations, ineligible revisions, replay and injected collection failure.

[TRF-003] Authorized administrators MUST have a dedicated Traffic section with Overview, Sources, Pages and Links views, bounded drill-downs and consistent date filters.
Falsifier: Sources, top pages or clicked destinations disagree with known events, renaming content splits its identity, a drill-down drops the date range, a missing referrer is asserted to prove a direct visit, or a referrer is presented as a search query.
Mechanism: Proposed traffic-reports; deterministic event fixtures, source classification, content renames, pagination and browser drill-down checks.

[TRF-004] The admin dashboard MUST show traffic, revenue and engagement charts with shared 7/30/90-day and bounded custom ranges, prior-period comparisons, visible reporting timezone and honest metric definitions.
Falsifier: Charts display invented data, aggregate counts are labeled unique people or a visitor funnel, a failed query looks like zero activity, or a chart/table/export disagrees for the same filters.
Mechanism: Proposed dashboard-reporting; deterministic series, empty/error states, period boundaries, chart/table/export comparisons and browser journeys.

[TRF-005] Revenue reports MUST use confirmed payment records and immutable purchase terms, separate currencies and live/test activity, and reconcile refunds/disputes without double counting or invented history.
Falsifier: Checkout initiation or a browser return creates revenue, repeated notifications duplicate a payment/refund, currencies are added together, test payments enter default totals, refunds exceed retained evidence, or collections are labeled profit or provider payout.
Mechanism: Proposed revenue-reporting; confirmed/failed/recurring payment fixtures, multiple currencies, sandbox mode, partial/full refunds, repeated delivery and disputes checked against the stored ledger.

[TRF-006] Engagement reports MUST count successful registrations, listing submissions and paid purchases from server-side records, and name the denominator for listing website click-through rates.
Falsifier: A rejected action increments a success count, a repeated request duplicates one stored success, unrelated aggregate counts become a person-level funnel, or a rate uses an unstated/inconsistent denominator.
Mechanism: Proposed engagement-reporting; successful/rejected/retried server actions, listing view/click fixtures and exact report assertions.

[TRF-007] Analytics collection, reports and exports MUST enforce permission, privacy, resource and retention boundaries.
Falsifier: Traffic access grants billing access, an unauthorized direct request or export reveals reports, raw IPs or credential-bearing URLs are retained, invalid/oversized events are accepted, reports scan an unbounded raw history, or configured cleanup leaves expired raw events indefinitely.
Mechanism: Proposed analytics-boundaries; permission matrix, hostile payload/URL fixtures, rate and size limits, cleanup/rollup tests and bounded query inspection.

[TRF-008] All report charts and controls MUST support keyboard use, accessible data tables, explicit units and readable light/dark mobile layouts.
Falsifier: A chart's data is unavailable without pointer hover, date controls cannot be operated by keyboard, the 320-pixel layout overflows, or tables/CSV totals differ from the visible chart.
Mechanism: Proposed reporting-accessibility; browser keyboard/mobile/theme checks and parsed chart/table/export equality.

## Metric and storage rules

Built-in reporting is the baseline; external analytics is optional later scope.
Default to the last 30 days. Custom ranges must have a documented finite maximum.
Use stable content identities, referral domains and allowlisted campaign
source/medium/name. Strip fragments, credentials and unapproved query parameters.
Keep visitor events separate from account records; no raw IP storage or implied
unique-visitor/session count. Label absent referrers direct/unknown.

Separate public internal links from outbound links. Resolve the main listing
website destination on the server from its public revision. Validate browser
event identities, bound payloads, deduplicate event IDs and rate-limit collection.
Document that blocking and automation affect coverage. Report collection failures
without breaking navigation or pretending analytics is a financial ledger.

Store indexed daily aggregates and provide bounded raw-event retention/cleanup.
Add separate analytics-view permission. Revenue also requires billing permission
on UI, JSON and CSV endpoints. Registration/submission metrics must not expose
individual account data merely because analytics access is granted.

Label amount_total as confirmed collections, not tax-exclusive sales or profit.
Show refunds and collections after refunds; disclose that processor fees are not
deducted. Show disputes separately. Cumulative refund evidence must be reconciled
idempotently. If dated movements are needed, create them at the authenticated
reconciliation boundary; label historical coverage without inventing old dates.
Keep currencies separate and exclude test mode by default.

Owner-facing private reports, visitor identity tracking, external-service
integration and ranking/search-query imports are not part of this commitment.
