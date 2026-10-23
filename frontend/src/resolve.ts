import type { DefineComponent } from 'vue'
import PublicLayout from './layouts/PublicLayout.vue'
import AdminLayout from './layouts/AdminLayout.vue'

// Client navigation and SSR must choose the same shell.
const pages = import.meta.glob<{ default: DefineComponent }>('./pages/**/*.vue')
export async function resolvePage(name: string) {
  const load = pages[`./pages/${name}.vue`]
  if (!load) throw new Error(`Unknown page: ${name}`)
  const module = await load()
  const page = module.default as DefineComponent & { layout?: unknown }
  page.layout ??= name.startsWith('admin/') ? AdminLayout : PublicLayout
  return page
}
