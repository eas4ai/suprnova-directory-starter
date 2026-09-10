# Site appearance

Status: Agreed 2026-09-09
Requirements: FND-001, FND-002, FND-003, FND-004, FND-005, PAY-001, PAY-004, PAY-005, PAY-006, PAY-007, PAY-008, DIR-001, DIR-002, DIR-003, DIR-004, DIR-005, DIR-006, DIR-007, DIR-008, PAY-002, PAY-003, PAY-009, PAY-010, PAY-011, PAY-012, PAY-013, PAY-014, PAY-015, PAY-016, CNT-001, CNT-002, CNT-003, CNT-004, CNT-005, CNT-006, ADM-001, ADM-002, ADM-003, KIT-001, KIT-002, KIT-003, KIT-004, KIT-005, UI-001, UI-002

## Outcome

Implement the developer-approved optional color presets and remembered light/dark
switch in docs/spec/appearance.md. Preserve all completed starter contracts.

## Plan and todo

- Completed: implement and verify validated configuration, request-scoped Vuetify
  themes, shared head styles and the accessible switch.
- In progress: refresh all Cairn evidence after the reviewed dark-placeholder fix;
  browser presets, persistence, SSR isolation and documentation were already verified.
- Pending: complete the final review after the fresh checks.

The isolated appearance suite passed on 2026-09-09: configuration units, both
builds, eight presets in both shells/modes, cookie persistence, concurrent server
renders, no-JavaScript output, dark Markdown and 320px navigation. Documentation
and the example environment describe blank precedence and the visitor cookie.

src/config/site.rs owns the preset allowlist. AuthShare passes the validated
appearance cookie. frontend/src/lib/theme.ts constructs a theme for each app;
main.ts and ssr.ts install it. SiteBrand writes its CSS through Inertia Head for
both server HTML and browser navigation. ShellNavigation contains the switch.
Existing semantic tokens retain their names and default light values.

Verification covers blank and custom accents, all eight presets in both modes,
invalid presets/cookies, button contrast, keyboard use, navigation/reload,
JavaScript-disabled server HTML and independent light/dark requests. Run existing
Rust/frontend/build/browser mechanisms and the production self-audit.
