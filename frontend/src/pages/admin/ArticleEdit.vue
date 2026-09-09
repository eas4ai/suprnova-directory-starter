<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Head, Link, router, useForm } from '@inertiajs/vue3'
import ArticleForm from '../../components/ArticleForm.vue'
import type { ArticleInput, EditorArticle, Term } from '../../types/articles'
const props = defineProps<{ article: EditorArticle | null; terms: Term[] }>()
function initial(): ArticleInput {
  const r = props.article?.current
  return { version: props.article?.version ?? 0, slug: r?.slug ?? '', title: r?.title ?? '', summary: r?.summary ?? '', body: r?.body ?? '', media_id: r?.media_id ?? null, media_alt: r?.media_alt ?? '', term_ids: r?.terms.map(t => t.id) ?? [] }
}
const form = useForm(initial())
const action = useForm({ version: props.article?.version ?? 0 })
const uploading = ref(false)
const saved = ref(false)
const errorSummary = ref<HTMLElement | null>(null)
const errors = computed(() => ({ ...form.errors, ...action.errors }) as Record<string, string>)
const busy = computed(() => form.processing || action.processing || uploading.value)
const focusErrors = async () => { await nextTick(); errorSummary.value?.focus() }
const mayLeave = () => !(form.isDirty || uploading.value) || window.confirm('Leave this editor? Unsaved changes and any upload in progress will be lost.')
const beforeUnload = (event: BeforeUnloadEvent) => { if (form.isDirty || uploading.value) { event.preventDefault(); event.returnValue = '' } }
let removeGuard: (() => void) | undefined
onMounted(() => {
  window.addEventListener('beforeunload', beforeUnload)
  removeGuard = router.on('before', event => { if (event.detail.visit.method === 'get' && !mayLeave()) event.preventDefault() })
})
onBeforeUnmount(() => { window.removeEventListener('beforeunload', beforeUnload); removeGuard?.() })
function update(value: ArticleInput) { Object.assign(form, value); saved.value = false }
function resetSaved() { form.defaults(initial()); form.reset(); saved.value = true }
function save() {
  if (busy.value) return
  action.clearErrors(); saved.value = false
  form.post(props.article ? '/admin/articles/' + props.article.id : '/admin/articles', { preserveScroll: true, onError: focusErrors, onSuccess: resetSaved })
}
function mutate(kind: 'publish' | 'unpublish') {
  if (!props.article || busy.value || form.isDirty) return
  if (kind === 'unpublish' && !window.confirm('Unpublish this article? It will disappear from public articles and feeds. Your draft and revision history will remain.')) return
  action.version = props.article.version; form.clearErrors()
  action.post('/admin/articles/' + props.article.id + '/' + kind, { preserveScroll: true, onError: focusErrors, onSuccess: resetSaved })
}
</script>
<template>
  <Head :title="article ? 'Edit article' : 'Create article'" />
  <section class="directory-page editorial-editor">
    <Link href="/admin/articles" class="text-link">Back to articles</Link>
    <header class="directory-heading"><h1>{{ article ? 'Edit article' : 'Create article' }}</h1><p>Write privately. Save a draft, preview it, then publish when it is ready.</p></header>
    <div v-if="Object.keys(errors).length" ref="errorSummary" tabindex="-1" role="alert" class="billing-errors"><h2>Changes were not saved</h2><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><p v-if="errors.version">Your edits are still here. Copy the changes you want to keep before reloading the latest revision.</p><Link v-if="errors.version && article" :href="'/admin/articles/' + article.id + '/edit'" class="text-link">Reload latest revision</Link></div>
    <p v-if="saved" role="status" class="billing-saved">Changes saved.</p>
    <div class="editorial-workspace">
      <form id="article-editor" :aria-busy="busy" @submit.prevent="save"><ArticleForm :model-value="form.data()" :terms="terms" :errors="errors" :disabled="busy" :media-url="article?.current.media_url ?? null" @update:model-value="update" @uploading="uploading = $event" /></form>
      <aside class="editorial-publication" aria-labelledby="publication-heading">
        <h2 id="publication-heading">Publication</h2>
        <p class="editorial-status">{{ !article || article.status === 'draft' ? 'Private draft' : article.status === 'unpublished_changes' ? 'Published · draft changes' : 'Published' }}</p>
        <p>{{ article?.published_revision_id ? 'The published revision stays public while you edit this draft.' : 'Only authorized editors can see this draft and its preview.' }}</p>
        <button form="article-editor" type="submit" class="button button-primary" :disabled="busy">{{ form.processing ? 'Saving…' : 'Save draft' }}</button>
        <p v-if="form.isDirty" class="field-help">Unsaved changes. Save before publishing or previewing the new revision.</p>
        <template v-if="article">
          <a :href="'/admin/articles/' + article.id + '/preview'" target="_blank" rel="noopener" class="text-link">Preview saved draft (new tab)</a>
          <button type="button" class="button button-secondary" :disabled="busy || form.isDirty" @click="mutate('publish')">{{ action.processing ? 'Updating…' : article.published_revision_id ? 'Publish saved changes' : 'Publish article' }}</button>
          <a v-if="article.public_url" :href="article.public_url" class="text-link" target="_blank" rel="noopener">View published article (new tab)</a>
          <button v-if="article.published_revision_id" type="button" class="button button-quiet" :disabled="busy || form.isDirty" @click="mutate('unpublish')">Unpublish article</button>
        </template>
      </aside>
    </div>
  </section>
</template>
