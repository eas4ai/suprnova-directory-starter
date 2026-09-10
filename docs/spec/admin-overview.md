# Useful administration

Status: Agreed 2026-09-10
Prefix: OVR

This is the first stage of the approved admin, SEO and traffic update. It replaces
the placeholder overview and establishes navigation for the later reports. The
traffic-reporting commitment delivers the traffic, revenue and engagement charts;
this stage does not show simulated activity or links to unfinished reports.

[OVR-001] The admin overview MUST show actual listing and article publication counts, pending moderation work and a bounded recent administrative activity list from the application database, with truthful empty and error states.
Falsifier: A populated directory claims it has no published listings, an expired or suspended listing counts as publicly visible, an unpublished article counts as published, a pending revision is omitted from review work, activity exceeds the five-entry overview limit or exposes protected audit details, or a failed query is presented as zero activity.
Mechanism: Proposed admin-overview; isolated database fixtures for empty, published, pending, expired and suspended content, plus direct HTTP and browser assertions.

[OVR-002] Overview data and actions MUST respect the existing administrative permissions on direct requests as well as in navigation.
Falsifier: An administrator without moderation, editorial or audit permission receives the corresponding protected data or action, an ordinary account receives overview data, or a restricted action succeeds when called directly.
Mechanism: Proposed admin-overview; permission matrix for overview JSON, rendered links and destination requests.

[OVR-003] The admin shell MUST group its available navigation and make overview counts and review work usable by keyboard and at narrow widths in both appearance modes.
Falsifier: A permitted destination disappears, a forbidden destination appears, a count links to the wrong queue, keyboard users cannot reach a visible action, or the overview/navigation overflows at 320 pixels or loses readable contrast in dark mode.
Mechanism: Proposed admin-overview; desktop/mobile browser journeys, keyboard checks, both themes and count-to-queue assertions.

## Boundaries

Reuse Vue, Inertia, Vuetify 0 and the shared semantic tokens. Keep the existing
permission model and publication predicate. Do not add a second UI framework or
replace authentication. Group existing destinations by their purpose; add SEO and
Traffic destinations only when their commitments implement them. Keep existing
filters and pagination on linked queues. Database queries must remain bounded
and avoid loading every listing merely to count it.
