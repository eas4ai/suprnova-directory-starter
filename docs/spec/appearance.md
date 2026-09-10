# Site appearance

Status: Agreed 2026-09-09
Prefix: UI

[UI-001] The starter MUST provide optional Tailwind color presets with blank preserving the current theme and existing SITE_ACCENT behavior.
Falsifier: A blank preset changes the current light colors, a supported preset fails to color both shells, invalid configuration is silently accepted, or primary button text has less than 4.5:1 contrast.
Mechanism: site-appearance; configuration checks and browser rendering of every supported preset.

[UI-002] Visitors MUST be able to switch between light and dark appearance and retain their choice across navigation and reload.
Falsifier: The control cannot be operated by keyboard, surfaces or text become unreadable, reload loses the choice, initial public HTML renders the wrong appearance, or one visitor changes another visitor's theme.
Mechanism: site-appearance; public, account and administration browser checks plus isolated server renders.

## Agreed scope

SITE_THEME defaults to blank. Supported presets are zinc, blue, indigo, violet,
emerald, teal, rose and orange, using the installed Vuetify 0 Tailwind palette.
The visitor switch defaults to light and stores a validated light/dark cookie.
A preset overrides the custom accent; blank retains SITE_ACCENT. Dark appearance
uses readable accent and neutral variants. No theme editor or dependency upgrade.
