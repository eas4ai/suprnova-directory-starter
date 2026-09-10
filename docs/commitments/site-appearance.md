# Site appearance

Status: Agreed 2026-09-09
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, UI-001, UI-002

## Outcome

Implement the developer-approved optional color presets and remembered light/dark
switch in docs/spec/appearance.md. Preserve all completed starter contracts.

## Plan and todo

- In progress: implement validated configuration, request-scoped Vuetify theme,
  shared head styles and accessible switch; verify configuration and rendering.
- Pending: verify browser journeys for presets, persistence, SSR isolation and
  public/account/admin surfaces; document operator configuration.
- Pending: commit, run all Cairn mechanisms, review and resolve findings.

src/config/site.rs owns the preset allowlist. AuthShare passes the validated
appearance cookie. frontend/src/lib/theme.ts constructs a theme for each app;
main.ts and ssr.ts install it. SiteBrand writes its CSS through Inertia Head for
both server HTML and browser navigation. ShellNavigation contains the switch.
Existing semantic tokens retain their names and default light values.

Verification covers blank and custom accents, all eight presets in both modes,
invalid presets/cookies, button contrast, keyboard use, navigation/reload,
JavaScript-disabled server HTML and independent light/dark requests. Run existing
Rust/frontend/build/browser mechanisms and the production self-audit.
