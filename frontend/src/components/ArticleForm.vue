<script setup lang="ts">
import SeoFields from './SeoFields.vue'
import { computed, ref, watch } from 'vue'
import type { Term, ArticleInput } from '../types/articles'
const props = defineProps<{ modelValue: ArticleInput; terms: Term[]; errors: Record<string, string>; disabled: boolean; mediaUrl: string | null }>()
const emit = defineEmits<{ 'update:modelValue': [value: ArticleInput]; uploading: [value: boolean] }>()
const value = computed(() => props.modelValue)
const preview = ref(props.mediaUrl)
watch(() => props.mediaUrl, url => { preview.value = url })
const uploadError = ref('')
const uploading = ref(false)
function set<K extends keyof ArticleInput>(key: K, input: ArticleInput[K]) { emit('update:modelValue', { ...value.value, [key]: input }) }
function toggleTerm(id: number, checked: boolean) { set('term_ids', checked ? [...value.value.term_ids, id] : value.value.term_ids.filter(item => item !== id)) }
async function upload(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  uploadError.value = ''
  if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type) || file.size > 5 * 1024 * 1024) {
    uploadError.value = 'Choose a JPEG, PNG or WebP image no larger than 5 MiB.'; input.value = ''; return
  }
  uploading.value = true; emit('uploading', true)
  try {
    const token = document.cookie.split('; ').find(row => row.startsWith('XSRF-TOKEN='))?.slice('XSRF-TOKEN='.length)
    if (!token) throw new Error('Your session could not be verified. Reload the page before uploading an image.')
    const response = await fetch('/admin/articles/media', { method: 'POST', credentials: 'same-origin', signal: AbortSignal.timeout(30000), headers: { 'Content-Type': file.type, 'Accept': 'application/json', 'X-XSRF-TOKEN': decodeURIComponent(token) }, body: file })
    if (!response.ok) throw new Error(response.status === 413 ? 'The image is too large. Choose a file no larger than 5 MiB.' : 'The image could not be accepted. Use a valid JPEG, PNG or WebP, at most 5 MiB and 4,096 pixels per side, and try again.')
    const result: unknown = await response.json()
    if (!result || typeof result !== 'object' || !('id' in result) || !('url' in result) || typeof result.id !== 'string' || typeof result.url !== 'string') throw new Error('The upload response was incomplete. Try again.')
    set('media_id', result.id); preview.value = result.url
  } catch (error) { uploadError.value = error instanceof Error ? error.message : 'The upload failed. Try again.' }
  finally { uploading.value = false; emit('uploading', false); input.value = '' }
}
function removeImage() { emit('update:modelValue', { ...value.value, media_id: null, media_alt: '' }); preview.value = null }
const textFields = [
  { key: 'title', label: 'Title', max: 160, rows: 0, help: 'Up to 160 characters.' },
  { key: 'summary', label: 'Summary', max: 320, rows: 3, help: 'A short introduction, up to 320 characters.' },
  { key: 'body', label: 'Article body', max: 50000, rows: 22, help: 'Write in Markdown. Raw HTML is disabled. Up to 50,000 characters.' },
  { key: 'slug', label: 'URL slug', max: 120, rows: 0, help: 'Lowercase letters, numbers and hyphens. Old published addresses redirect after a slug change.' },
] as const
</script>
<template>
  <fieldset class="directory-fields" :disabled="disabled || uploading">
    <legend class="sr-only">Article content</legend>
    <div v-for="field in textFields" :key="field.key" class="directory-field">
      <label :for="`article-${field.key}`" class="form-label">{{ field.label }}</label>
      <textarea v-if="field.rows" :id="`article-${field.key}`" :value="value[field.key]" :name="field.key" :rows="field.rows" :maxlength="field.max" required class="form-input" :aria-invalid="Boolean(errors[field.key])" :aria-describedby="`article-${field.key}-help article-${field.key}-error`" @input="set(field.key, ($event.target as HTMLTextAreaElement).value)" />
      <input v-else :id="`article-${field.key}`" :value="value[field.key]" :name="field.key" type="text" :maxlength="field.max" required class="form-input" :aria-invalid="Boolean(errors[field.key])" :aria-describedby="`article-${field.key}-help article-${field.key}-error`" @input="set(field.key, ($event.target as HTMLInputElement).value)" />
      <p :id="`article-${field.key}-help`" class="field-help">{{ field.help }}</p><p :id="`article-${field.key}-error`" class="form-error">{{ errors[field.key] }}</p>
    </div>
    <p class="field-help">Choose up to 15 categories and tags combined. Remove disabled terms before saving.</p>
    <fieldset v-for="kind in (['category', 'tag'] as const)" :key="kind" class="directory-category-options" aria-describedby="article-terms-error"><legend class="form-label">{{ kind === 'category' ? 'Categories' : 'Tags' }}</legend><label v-for="term in terms.filter(item => item.kind === kind)" :key="term.id" class="billing-check"><input type="checkbox" name="term_ids" :value="term.id" :checked="value.term_ids.includes(term.id)" :disabled="!value.term_ids.includes(term.id) && (!term.active || value.term_ids.length >= 15)" @change="toggleTerm(term.id, ($event.target as HTMLInputElement).checked)" />{{ term.name }}{{ term.active ? '' : ' (disabled)' }}</label><p v-if="!terms.some(item => item.kind === kind)" class="field-help">No {{ kind === 'category' ? 'categories' : 'tags' }} available.</p></fieldset><p id="article-terms-error" class="form-error">{{ errors.term_ids }}</p>
    <div class="directory-field"><label for="article-image" class="form-label">Cover image (optional)</label><input id="article-image" type="file" accept="image/jpeg,image/png,image/webp" aria-describedby="article-image-help article-image-error" :aria-invalid="Boolean(uploadError || errors.media_id)" @change="upload" /><p id="article-image-help" class="field-help">JPEG, PNG or WebP. Up to 5 MiB and 4,096 pixels per side. Images stay private until published.</p><p id="article-image-error" class="form-error" role="alert">{{ uploadError || errors.media_id }}</p><p v-if="uploading" role="status">Uploading image…</p><div v-if="preview" class="directory-image-preview"><img :src="preview" :alt="value.media_alt" /><button type="button" class="button button-secondary" @click="removeImage">Remove image</button></div></div>
    <div v-if="value.media_id" class="directory-field"><label for="article-media-alt" class="form-label">Image alternative text</label><input id="article-media-alt" :value="value.media_alt" name="media_alt" maxlength="280" required class="form-input" :aria-invalid="Boolean(errors.media_alt)" aria-describedby="article-alt-help article-alt-error" @input="set('media_alt', ($event.target as HTMLInputElement).value)" /><p id="article-alt-help" class="field-help">Describe what the image shows for people who cannot see it.</p><p id="article-alt-error" class="form-error">{{ errors.media_alt }}</p></div>
    <SeoFields :model-value="value.seo" :errors="errors" prefix="article" @update:model-value="set('seo', $event)" />
  </fieldset>
</template>
