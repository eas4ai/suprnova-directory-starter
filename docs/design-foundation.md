# Foundation design

The public shell helps readers discover the directory and enter their account.
The administration shell gives authorized operators a quiet workspace. Neither
surface invents listings, payments, metrics or actions before those features exist.

The working theme uses a system sans-serif, neutral surfaces and one teal accent.
The circular brand and active-navigation marker are shared across both shells.
Motion is limited to short hover changes, with reduced-motion support.

`frontend/src/styles/tokens.css` defines primitive values and semantic roles.
`--brand-accent` is the shared branding entry point; surfaces, text, borders,
spacing and control radii have their own roles. Button tokens reference these roles.
`shells.css` implements the shell geometry and responsive layout. CSS imports stay
ordered in `app.css`. The initial theme is light; dark mode is not part of this
foundation commitment.

Vue layouts use one page resolver for client and SSR. Vuetify 0 supplies native
modal semantics for navigation and sign-out dialogs. Controls use visible focus,
semantic navigation, skip links, and labeled forms. Tests exercise Tab, Enter,
Escape, focus trapping/restoration, and a shared branding-token change.
