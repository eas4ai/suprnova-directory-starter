<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { Head, Link, useForm, usePage } from '@inertiajs/vue3'
import type { Paged } from '../../types/listings'
type Defaults = { title_format: string; description: string; image: string; publisher: string; profiles: string[]; google_verification: string; bing_verification: string; noindex: boolean }
type Redirect = { id: number; source: string; destination: string; version: number }
type Preview = { title: string; description: string; canonical: string; image: string | null; noindex: boolean }
type Finding = { path: string; preview: Preview; sitemap_included: boolean; findings: string[] }
const props = defineProps<{ settings: { version: number; defaults: Defaults }; report: { kind: string; rows: Finding[]; scope: string; pagination: Paged }; redirects: Redirect[]; not_found: { path: string; hits: number; first_seen: number; last_seen: number }[]; not_found_pagination: Paged }>()
const page = usePage()
const tabs = [{ key: 'settings', label: 'Site settings' }, { key: 'findings', label: 'Findings and previews' }, { key: 'redirects', label: 'Redirects' }, { key: 'missing', label: '404 report' }] as const
const tab = ref<string>(page.url.includes('missing_page=') ? 'missing' : page.url.includes('kind=') ? 'findings' : 'settings')
const initial = () => ({ ...props.settings.defaults, profiles: [...props.settings.defaults.profiles], version: props.settings.version, profiles_text: props.settings.defaults.profiles.join('\n') })
const form = useForm(initial())
const selected = ref<number | null>(null)
const redirect = useForm({ version: 0, source: '', destination: '' })
const removal = useForm({ version: 0 })
const busy = computed(() => form.processing || redirect.processing || removal.processing)
const errors = computed(() => ({ ...form.errors, ...redirect.errors, ...removal.errors }) as Record<string, string>)
const summary = ref<HTMLElement | null>(null)
const saved = ref('')
const focusErrors = async () => { await nextTick(); summary.value?.focus() }
watch(() => props.settings.version, () => { if (!form.isDirty) { form.defaults(initial()); form.reset() } })
function save() {
  saved.value = ''
  form.transform(value => ({ version: value.version, defaults: { title_format: value.title_format, description: value.description, image: value.image, publisher: value.publisher, profiles: value.profiles_text.split('\n').map(v => v.trim()).filter(Boolean), google_verification: value.google_verification, bing_verification: value.bing_verification, noindex: value.noindex } }))
    .post('/admin/seo', { preserveScroll: true, onError: focusErrors, onSuccess: () => { form.defaults(initial()); form.reset(); saved.value = 'SEO settings saved.' } })
}
function choose(row?: Redirect) {
  if (redirect.isDirty && !window.confirm('Discard your unsaved redirect changes?')) return
  selected.value = row?.id ?? null
  redirect.defaults({ version: row?.version ?? 0, source: row?.source ?? '', destination: row?.destination ?? '' }); redirect.reset(); redirect.clearErrors(); removal.clearErrors()
}
function saveRedirect() {
  saved.value = ''
  redirect.post('/admin/seo/redirects' + (selected.value === null ? '' : '/' + selected.value), { preserveScroll: true, onError: focusErrors, onSuccess: () => { selected.value = null; redirect.defaults({ version: 0, source: '', destination: '' }); redirect.reset(); saved.value = 'Redirect saved.' } })
}
function remove(row: Redirect) {
  if (!window.confirm(`Remove the redirect from ${row.source}?`)) return
  removal.version = row.version
  removal.post(`/admin/seo/redirects/${row.id}/remove`, { preserveScroll: true, onError: focusErrors, onSuccess: () => { saved.value = 'Redirect removed.'; if (selected.value === row.id) choose() } })
}
const fields = [{ key: 'title_format', label: 'Title format', max: 160 }, { key: 'description', label: 'Default description', max: 320 }, { key: 'image', label: 'Default social image URL', max: 2048 }, { key: 'publisher', label: 'Publisher name', max: 120 }, { key: 'google_verification', label: 'Google Search Console verification value', max: 256 }, { key: 'bing_verification', label: 'Bing verification value', max: 256 }] as const
const kinds = [{ key: 'listing', label: 'Listings' }, { key: 'article', label: 'Articles' }, { key: 'listing_category', label: 'Listing categories' }, { key: 'category', label: 'Article categories' }, { key: 'tag', label: 'Article tags' }]
const timestamp = (value: number) => new Date(value * 1000).toISOString().replace('T', ' ').replace('.000Z', ' UTC')
</script>
<template>
  <Head title="SEO" />
  <section class="seo-page"><header class="directory-heading"><h1>SEO</h1><p>Control search metadata, inspect public pages and maintain old links.</p></header>
    <nav class="seo-tabs" aria-label="SEO sections"><button v-for="item in tabs" :key="item.key" type="button" class="nav-link" :aria-pressed="tab === item.key" @click="tab = item.key">{{ item.label }}</button></nav>
    <div v-if="Object.keys(errors).length" ref="summary" tabindex="-1" role="alert" class="billing-errors"><h2>Changes were not saved</h2><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><p v-if="errors.version">Your edits are preserved. Copy anything you need before reloading.</p><Link v-if="errors.version" href="/admin/seo" class="text-link">Reload saved settings</Link></div>
    <p v-if="saved" role="status" class="billing-saved">{{ saved }}</p>
    <section v-show="tab === 'settings'" aria-labelledby="seo-settings-heading"><h2 id="seo-settings-heading">Site settings</h2>
      <p>Use <code>{title}</code> once and optionally <code>{site}</code> in the title format. Blank descriptions and publisher names use the site configuration. Content overrides take priority.</p>
      <form class="seo-form" :aria-busy="busy" @submit.prevent="save"><fieldset class="directory-fields" :disabled="busy"><legend class="sr-only">Site metadata</legend>
        <div v-for="field in fields" :key="field.key"><label :for="'seo-' + field.key" class="form-label">{{ field.label }}</label><input :id="'seo-' + field.key" v-model="form[field.key]" class="form-input" :maxlength="field.max" :aria-invalid="Boolean(errors[field.key])" :aria-describedby="'seo-' + field.key + '-error'" /><p :id="'seo-' + field.key + '-error'" class="form-error">{{ errors[field.key] }}</p></div>
        <p class="field-help">Paste verification values only, without HTML tags. After saving, complete verification in the search service. Search Console data imports are not connected here.</p>
        <div><label for="seo-profiles" class="form-label">Public social profiles</label><textarea id="seo-profiles" v-model="form.profiles_text" class="form-input" rows="4" maxlength="20500" aria-describedby="seo-profiles-help" /><p id="seo-profiles-help" class="field-help">Up to ten HTTPS profile URLs, one per line. Use profiles that belong to the publisher.</p></div>
        <label class="billing-check"><input v-model="form.noindex" type="checkbox" />Exclude the whole site from search engines</label><p class="field-help">Public pages remain accessible. They receive noindex, and the sitemap index is empty. Crawlers can still read these instructions.</p>
      </fieldset><button class="button button-primary" :disabled="busy">{{ form.processing ? 'Saving…' : 'Save SEO settings' }}</button></form>
    </section>
    <section v-show="tab === 'findings'" aria-labelledby="seo-findings-heading"><h2 id="seo-findings-heading">Findings and previews</h2><p>{{ report.scope }}</p>
      <nav class="seo-tabs" aria-label="SEO content type"><Link v-for="kind in kinds" :key="kind.key" :href="'/admin/seo?kind=' + kind.key" class="nav-link" :aria-current="report.kind === kind.key ? 'page' : undefined">{{ kind.label }}</Link></nav>
      <p>{{ report.pagination.total }} public pages in this type. Page {{ report.pagination.page }}.</p>
      <p v-if="!report.rows.length" class="directory-empty">No eligible public pages in this view.</p>
      <article v-for="row in report.rows" :key="row.path" class="seo-finding"><h3><a :href="row.path" class="text-link">{{ row.path }}</a></h3>
        <div class="seo-previews"><section class="seo-preview" aria-label="Search preview"><h4>Search preview</h4><p class="seo-preview-url">{{ row.preview.canonical }}</p><p class="seo-preview-title">{{ row.preview.title }}</p><p class="seo-preview-description">{{ row.preview.description }}</p></section>
          <section class="seo-preview" aria-label="Social preview"><h4>Social preview</h4><img v-if="row.preview.image" :src="row.preview.image" alt="Configured social image" referrerpolicy="no-referrer" loading="lazy" /><p v-else class="field-help">No social image configured.</p><p class="seo-preview-title">{{ row.preview.title }}</p><p class="seo-preview-description">{{ row.preview.description }}</p></section></div>
        <p>{{ row.preview.noindex ? 'Noindex' : 'Indexable' }} · {{ row.sitemap_included ? 'Included in sitemap' : 'Excluded from sitemap' }}</p>
        <ul v-if="row.findings.length"><li v-for="finding in row.findings" :key="finding">{{ finding }}</li></ul><p v-else>No findings from these checks.</p>
      </article>
      <nav class="seo-tabs" aria-label="SEO report pages"><Link v-if="report.pagination.page > 1" :href="`/admin/seo?kind=${report.kind}&page=${report.pagination.page - 1}`" class="button button-secondary">Previous findings</Link><Link v-if="report.pagination.page * report.pagination.per_page < report.pagination.total" :href="`/admin/seo?kind=${report.kind}&page=${report.pagination.page + 1}`" class="button button-secondary">Next findings</Link></nav>
    </section>
    <section v-show="tab === 'redirects'" aria-labelledby="seo-redirects-heading"><h2 id="seo-redirects-heading">Redirects</h2><p>Permanent same-site redirects for old paths. Existing route namespaces and public files are reserved. Destinations must resolve to a currently public page. At most 1,000 redirects and five steps per chain.</p>
      <form class="seo-form" :aria-busy="busy" @submit.prevent="saveRedirect"><fieldset class="directory-fields" :disabled="busy"><legend>{{ selected === null ? 'Create redirect' : 'Edit redirect' }}</legend><div><label for="redirect-source" class="form-label">Old path</label><input id="redirect-source" v-model="redirect.source" class="form-input" maxlength="240" required :readonly="selected !== null" placeholder="/old-guide" /></div><div><label for="redirect-destination" class="form-label">Destination path</label><input id="redirect-destination" v-model="redirect.destination" class="form-input" maxlength="240" required placeholder="/articles/current-guide" /></div></fieldset><div class="seo-tabs"><button class="button button-primary" :disabled="busy">Save redirect</button><button v-if="selected !== null" type="button" class="button button-secondary" :disabled="busy" @click="choose()">Cancel edit</button></div></form>
      <p v-if="!redirects.length" class="directory-empty">No manual redirects yet. Published article slug changes keep their automatic redirects.</p><ul class="seo-redirect-list"><li v-for="row in redirects" :key="row.id"><span>{{ row.source }} → {{ row.destination }}</span><div class="seo-tabs"><button type="button" class="button button-secondary" :disabled="busy" :aria-label="'Edit redirect ' + row.source" @click="choose(row)">Edit</button><button type="button" class="button button-quiet" :disabled="busy" :aria-label="'Remove redirect ' + row.source" @click="remove(row)">Remove</button></div></li></ul>
    </section>
    <section v-show="tab === 'missing'" aria-labelledby="seo-missing-heading"><h2 id="seo-missing-heading">404 report</h2><p>Path-only observations from the last 30 days, capped at 1,000 paths. Query strings, credentials, sensitive routes, encoded paths, Markdown and long segments are omitted. Counts include automated requests; they are not unique visitors.</p><p v-if="!not_found.length" class="directory-empty">No recent reportable missing pages.</p>
      <ul class="seo-missing-list"><li v-for="row in not_found" :key="row.path"><strong>{{ row.path }}</strong><span>{{ row.hits }} requests</span><time>{{ timestamp(row.last_seen) }}</time></li></ul>
      <nav class="seo-tabs" aria-label="404 report pages"><Link v-if="not_found_pagination.page > 1" :href="'/admin/seo?missing_page=' + (not_found_pagination.page - 1)" class="button button-secondary">Previous missing pages</Link><Link v-if="not_found_pagination.page * not_found_pagination.per_page < not_found_pagination.total" :href="'/admin/seo?missing_page=' + (not_found_pagination.page + 1)" class="button button-secondary">Next missing pages</Link></nav>
    </section>
  </section>
</template>
<style scoped>
.seo-page { min-width: 0; }
.seo-page section > p { margin-block: var(--space-4); }
.seo-tabs { display: flex; flex-wrap: wrap; gap: var(--space-2); margin-block: var(--space-6); }
.seo-tabs [aria-pressed="true"] { background: var(--surface-muted); color: var(--text-primary); font-weight: 600; }
.seo-form { max-width: 70ch; margin-block: var(--space-6); }
.seo-finding { border-block-start: 1px solid var(--border-default); padding-block: var(--space-6); overflow-wrap: anywhere; }
.seo-previews { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr)); gap: var(--space-6); margin-block: var(--space-6); }
.seo-preview { min-width: 0; padding: var(--space-6); border: 1px solid var(--border-default); border-radius: var(--radius-panel); }
.seo-preview h4 { font-size: var(--text-small); color: var(--text-secondary); margin-bottom: var(--space-4); }
.seo-preview-title { font-size: 1.25rem; font-weight: 600; }
.seo-preview-url { font-size: var(--text-small); color: var(--text-secondary); }
.seo-preview img { width: 100%; max-height: 220px; object-fit: contain; }
.seo-redirect-list, .seo-missing-list { padding: 0; list-style: none; }
.seo-redirect-list li, .seo-missing-list li { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-4); justify-content: space-between; padding-block: var(--space-4); border-bottom: 1px solid var(--border-default); overflow-wrap: anywhere; }
.seo-redirect-list span, .seo-missing-list strong { min-width: 0; overflow-wrap: anywhere; }
.seo-missing-list time { font-size: var(--text-small); color: var(--text-secondary); }
</style>
