# Add optional color presets and remembered light and dark appearance

Level: Judged
Decided by: Shawn
Rests on: site-appearance UI-001 UI-002
Would be wrong if: Blank changes the existing light theme or a visitor preference leaks between server renders.

## Decision

The developer requested common Tailwind color presets, explicitly retained blank as the default, and approved adding light/dark. Use the installed Vuetify 0 palette and a theme instance per app. Preserve SITE_ACCENT when blank. Read a validated appearance cookie for initial server rendering and update it from the shared navigation switch. Execute the bounded plan in docs/commitments/site-appearance.md and retain prior regressions.

## Realized by

- d4266614733ce1bbb0d8349e76cf6496bd84cab8 Add optional color presets and remembered light and dark appearance
