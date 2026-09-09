import './app.css'
import { createInertiaApp } from '@inertiajs/vue3'
import { createApp, createSSRApp, h } from 'vue'
import { initLang } from './lib/lang'
import { resolvePage } from './resolve'

// Inertia's HTTP client reads the current XSRF-TOKEN cookie on each request.
// Do not cache the page's meta token: authentication rotates the session.

createInertiaApp({
  resolve: resolvePage,
  async setup({ el, App, props, plugin }) {
    // `el` is `null` when `setup` runs server-side - @inertiajs/vue3's
    // `createInertiaApp` reuses this same callback for both the browser
    // bootstrap and an SSR render pass (see `ssr.ts`, which calls this
    // same `createInertiaApp` shape from inside `createServer`).
    // `initLang` does a `fetch()`, which has no business running on the
    // server (no absolute URL to fetch, no reason to block a render on
    // a network round trip there), so it's skipped entirely there;
    // `t()`'s documented raw-key fallback covers whatever renders
    // during that pass.
    //
    // Caution: awaiting `initLang` here, before mount/hydrate, means a
    // hydrating client's first paint carries real translations while
    // the server-rendered markup it hydrates against still has `t()`'s
    // untranslated fallback (SSR always skips `initLang` - see above) -
    // a hydration content mismatch on any translated string. This
    // scaffold accepts that trade-off (translate before first paint,
    // for the common case) rather than deferring the catalog load until
    // after hydration.
    if (el) {
      await initLang(props.initialPage)
    }

    // SSR-aware: when the server pre-rendered, the mount node carries
    // `data-server-rendered="true"` and we must hydrate. Without this
    // check, SSR markup gets destroyed and re-rendered on the client.
    if (el.hasAttribute('data-server-rendered')) {
      createSSRApp({ render: () => h(App, props) })
        .use(plugin)
        .mount(el)
    } else {
      createApp({ render: () => h(App, props) })
        .use(plugin)
        .mount(el)
    }
  },
})
