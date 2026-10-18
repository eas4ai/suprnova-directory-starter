import { createThemePlugin, ThemeAdapter } from '@vuetify/v0/theme'
import { tailwind } from '@vuetify/v0/palettes/tailwind'
import type { SharedProps } from '../types/shared'

// Inertia Head in SiteBrand owns stylesheet rendering in both SSR and the browser.
class InertiaThemeAdapter extends ThemeAdapter {
  constructor() { super('site') }
  setup() {}
  update() {}
}

export function siteTheme(props: Pick<SharedProps, 'site' | 'appearance'>) {
  const preset = props.site.theme
  const light = preset ? `{palette.tw.${preset}.700}` : props.site.accent
  const dark = preset ? `{palette.tw.${preset}.400}` : `color-mix(in srgb, ${props.site.accent} 35%, white)`
  return createThemePlugin({
    default: props.appearance === 'dark' ? 'dark' : 'light',
    adapter: new InertiaThemeAdapter(),
    palette: { tw: tailwind },
    themes: {
      light: { dark: false, colors: { 'site-accent': light } },
      dark: { dark: true, colors: {
        'neutral-0': '{palette.tw.zinc.950}', 'neutral-50': '{palette.tw.zinc.900}',
        'neutral-100': '{palette.tw.zinc.800}', 'neutral-200': '{palette.tw.zinc.700}',
        'neutral-500': '{palette.tw.zinc.400}', 'neutral-700': '{palette.tw.zinc.300}',
        'neutral-900': '{palette.tw.zinc.50}', 'red-700': '{palette.tw.red.400}',
        'site-accent': dark,
      } },
    },
  })
}
