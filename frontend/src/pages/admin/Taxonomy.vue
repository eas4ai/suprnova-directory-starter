<script setup lang="ts">
import SeoFields from '../../components/SeoFields.vue'
import { emptySeo } from '../../types/seo'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Head, Link, router, useForm } from '@inertiajs/vue3'
import type { TaxonomyItem } from '../../types/articles'
const props = defineProps<{ terms: TaxonomyItem[]; kind: 'listing_category' | 'category' | 'tag' }>()
const kinds = [{ value: 'listing_category', label: 'Listing categories' }, { value: 'category', label: 'Article categories' }, { value: 'tag', label: 'Article tags' }] as const
const selected = ref<number | null>(null)
const form = useForm({ seo: emptySeo(), version: 0, slug: '', name: '', active: true })
const removal = useForm({ version: 0 })
const summary = ref<HTMLElement | null>(null)
const nameInput = ref<HTMLInputElement | null>(null)
const saved = ref(false)
const errors = computed(() => ({ ...form.errors, ...removal.errors }) as Record<string, string>)
const busy = computed(() => form.processing || removal.processing)
const mayLeave = () => !form.isDirty || window.confirm('Discard your unsaved taxonomy changes?')
const focusErrors = async () => { await nextTick(); summary.value?.focus() }
function populate(item?: TaxonomyItem) {
  selected.value = item?.id ?? null
  form.defaults({ seo: { ...(item?.seo ?? emptySeo()) }, version: item?.version ?? 0, slug: item?.slug ?? '', name: item?.name ?? '', active: item?.active ?? true }); form.reset(); form.clearErrors(); removal.clearErrors()
}
async function choose(item?: TaxonomyItem) { if (busy.value || !mayLeave()) return; populate(item); saved.value = false; await nextTick(); nameInput.value?.focus() }
watch(() => props.kind, () => populate())
const beforeUnload = (event: BeforeUnloadEvent) => { if (form.isDirty) { event.preventDefault(); event.returnValue = '' } }
let removeGuard: (() => void) | undefined
onMounted(() => { window.addEventListener('beforeunload', beforeUnload); removeGuard = router.on('before', event => { if (event.detail.visit.method === 'get' && !mayLeave()) event.preventDefault() }) })
onBeforeUnmount(() => { window.removeEventListener('beforeunload', beforeUnload); removeGuard?.() })
function save() {
  if (busy.value) return
  removal.clearErrors(); saved.value = false
  const id = selected.value
  form.post('/admin/taxonomy/' + props.kind + (id === null ? '' : '/' + id), { preserveScroll: true, onError: focusErrors, onSuccess: () => { populate(id === null ? undefined : props.terms.find(t => t.id === id)); saved.value = true } })
}
function remove() {
  if (selected.value === null || busy.value || form.isDirty) return
  if (!window.confirm('Remove this unused term? Terms attached to content cannot be removed. Disable them to preserve their relationships.')) return
  removal.version = form.version
  removal.post('/admin/taxonomy/' + props.kind + '/' + selected.value + '/remove', { preserveScroll: true, onError: focusErrors, onSuccess: () => { populate(); saved.value = true } })
}
</script>
<template>
  <Head title="Categories and tags" />
  <section class="directory-page"><header class="directory-heading"><h1>Categories and tags</h1><p>Organize listings and articles without changing existing relationships.</p></header>
    <nav class="editorial-taxonomy-nav" aria-label="Taxonomy type"><Link v-for="item in kinds" :key="item.value" :href="'/admin/taxonomy?kind=' + item.value" class="nav-link" :aria-current="kind === item.value ? 'page' : undefined">{{ item.label }}</Link></nav>
    <div class="editorial-taxonomy-workspace"><section><div class="directory-heading-actions"><h2>{{ kinds.find(item => item.value === kind)?.label }}</h2><button type="button" class="button button-secondary" :disabled="busy" @click="choose()">Create term</button></div><ul v-if="terms.length" class="editorial-term-list"><li v-for="term in terms" :key="term.id"><button type="button" :aria-pressed="selected === term.id" :disabled="busy" @click="choose(term)"><strong>{{ term.name }}</strong><span>{{ term.slug }} · {{ term.active ? 'Active' : 'Disabled' }}</span></button></li></ul><p v-else class="directory-empty">No terms yet. Create one using the form.</p></section>
      <section class="editorial-term-editor"><h2>{{ selected === null ? 'Create term' : 'Edit term' }}</h2>
        <div v-if="Object.keys(errors).length" ref="summary" tabindex="-1" role="alert" class="billing-errors"><h3>Changes were not saved</h3><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><p v-if="errors.version">Your edits are preserved. Copy any changes to keep before reloading.</p><Link v-if="errors.version" :href="'/admin/taxonomy?kind=' + kind" class="text-link">Reload latest terms</Link></div><p v-if="saved" role="status" class="billing-saved">Taxonomy updated.</p>
        <form :aria-busy="busy" @submit.prevent="save"><fieldset class="directory-fields" :disabled="busy"><legend class="sr-only">Term details</legend><div><label for="term-name" class="form-label">Name</label><input id="term-name" ref="nameInput" v-model="form.name" name="name" required maxlength="120" class="form-input" :aria-invalid="Boolean(errors.name)" aria-describedby="term-name-error" /><p id="term-name-error" class="form-error">{{ errors.name }}</p></div><div><label for="term-slug" class="form-label">URL slug</label><input id="term-slug" v-model="form.slug" name="slug" required maxlength="120" :readonly="selected !== null" class="form-input" :aria-invalid="Boolean(errors.slug)" aria-describedby="term-slug-help term-slug-error" /><p id="term-slug-help" class="field-help">{{ selected === null ? 'Lowercase letters, numbers and hyphens. This cannot change after creation.' : 'The slug stays fixed to preserve existing links.' }}</p><p id="term-slug-error" class="form-error">{{ errors.slug }}</p></div><div><label class="billing-check"><input v-model="form.active" name="active" type="checkbox" aria-describedby="term-active-help" />Active</label><p id="term-active-help" class="field-help">Disable a term to stop new use while keeping its content relationships.</p><p class="form-error">{{ errors.active }}</p></div><SeoFields v-model="form.seo" :errors="errors" prefix="taxonomy" /></fieldset><button type="submit" class="button button-primary" :disabled="busy">{{ form.processing ? 'Saving…' : selected === null ? 'Create term' : 'Save term' }}</button></form>
        <section v-if="selected !== null" class="directory-archive"><h3>Remove unused term</h3><p>Terms used by content cannot be removed. Disable them instead to keep their relationships intact.</p><button type="button" class="button button-secondary" :disabled="busy || form.isDirty" @click="remove">{{ removal.processing ? 'Removing…' : 'Remove term' }}</button><p v-if="form.isDirty" class="field-help">Save your changes before removing this term.</p></section>
      </section>
    </div>
  </section>
</template>
