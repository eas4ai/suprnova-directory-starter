<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Category, ListingInput } from '../types/listings'
const props = defineProps<{ modelValue: ListingInput; categories: Category[]; errors: Record<string, string>; disabled: boolean; mediaUrl: string | null }>()
const emit = defineEmits<{ 'update:modelValue': [value: ListingInput]; uploading: [value: boolean] }>()
const value = computed(() => props.modelValue)
const preview = ref(props.mediaUrl)
watch(() => props.mediaUrl, url => { preview.value = url })
const uploadError = ref('')
const uploading = ref(false)
function set<K extends keyof ListingInput>(key: K, input: ListingInput[K]) { emit('update:modelValue', { ...value.value, [key]: input }) }
function toggleCategory(id: number, checked: boolean) { set('category_ids', checked ? [...value.value.category_ids, id] : value.value.category_ids.filter(item => item !== id)) }
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
    const response = await fetch('/dashboard/listings/media', { method: 'POST', credentials: 'same-origin', headers: { 'Content-Type': file.type, 'Accept': 'application/json', 'X-XSRF-TOKEN': decodeURIComponent(token) }, body: file })
    if (!response.ok) throw new Error(response.status === 413 ? 'The image is too large. Choose a file no larger than 5 MiB.' : 'The image could not be accepted. Use a valid JPEG, PNG or WebP, at most 5 MiB and 4,096 pixels per side, and try again.')
    const result: unknown = await response.json()
    if (!result || typeof result !== 'object' || !('id' in result) || !('url' in result) || typeof result.id !== 'string' || typeof result.url !== 'string') throw new Error('The upload response was incomplete. Try again.')
    set('media_id', result.id); preview.value = result.url
  } catch (error) { uploadError.value = error instanceof Error ? error.message : 'The upload failed. Try again.' }
  finally { uploading.value = false; emit('uploading', false); input.value = '' }
}
function removeImage() { set('media_id', null); preview.value = null }
const textFields = [
  { key: 'title', label: 'Title', max: 120, rows: 0, help: 'Up to 120 characters.' },
  { key: 'summary', label: 'Summary', max: 280, rows: 3, help: 'A short introduction, up to 280 characters.' },
  { key: 'description', label: 'Description', max: 20000, rows: 10, help: 'Markdown is supported. Raw HTML is disabled. Up to 20,000 characters.' },
  { key: 'url', label: 'Website URL', max: 2048, rows: 0, help: 'An absolute http:// or https:// address without a username or password.' },
] as const
</script>
<template>
  <fieldset class="directory-fields" :disabled="disabled || uploading">
    <legend class="sr-only">Listing content</legend>
    <div v-for="field in textFields" :key="field.key" class="directory-field">
      <label :for="`listing-${field.key}`" class="form-label">{{ field.label }}</label>
      <textarea v-if="field.rows" :id="`listing-${field.key}`" :value="value[field.key]" :name="field.key" :rows="field.rows" :maxlength="field.max" required class="form-input" :aria-invalid="Boolean(errors[field.key])" :aria-describedby="`listing-${field.key}-help listing-${field.key}-error`" @input="set(field.key, ($event.target as HTMLTextAreaElement).value)" />
      <input v-else :id="`listing-${field.key}`" :value="value[field.key]" :name="field.key" :type="field.key === 'url' ? 'url' : 'text'" :maxlength="field.max" required class="form-input" :aria-invalid="Boolean(errors[field.key])" :aria-describedby="`listing-${field.key}-help listing-${field.key}-error`" @input="set(field.key, ($event.target as HTMLInputElement).value)" />
      <p :id="`listing-${field.key}-help`" class="field-help">{{ field.help }}</p><p :id="`listing-${field.key}-error`" class="form-error">{{ errors[field.key] }}</p>
    </div>
    <fieldset class="directory-category-options" aria-describedby="listing-categories-help listing-categories-error"><legend class="form-label">Categories</legend><p id="listing-categories-help" class="field-help">Choose one to five categories.</p><label v-for="category in categories" :key="category.id" class="billing-check"><input type="checkbox" name="category_ids" :value="category.id" :checked="value.category_ids.includes(category.id)" :disabled="!value.category_ids.includes(category.id) && value.category_ids.length >= 5" :aria-invalid="Boolean(errors.category_ids)" @change="toggleCategory(category.id, ($event.target as HTMLInputElement).checked)" />{{ category.name }}</label><p v-if="!categories.length" class="field-help">No categories are available. Contact the directory operator.</p><p id="listing-categories-error" class="form-error">{{ errors.category_ids }}</p></fieldset>
    <div class="directory-field"><label for="listing-image" class="form-label">Listing image (optional)</label><input id="listing-image" type="file" accept="image/jpeg,image/png,image/webp" aria-describedby="listing-image-help listing-image-error" :aria-invalid="Boolean(uploadError || errors.media_id)" @change="upload" /><p id="listing-image-help" class="field-help">JPEG, PNG or WebP. Up to 5 MiB and 4,096 pixels per side. Images stay private until approved.</p><p id="listing-image-error" class="form-error" role="alert">{{ uploadError || errors.media_id }}</p><p v-if="uploading" role="status">Uploading image…</p><div v-if="preview" class="directory-image-preview"><img :src="preview" :alt="value.media_alt" /><button type="button" class="button button-secondary" @click="removeImage">Remove image</button></div></div>
    <div v-if="value.media_id" class="directory-field"><label for="listing-media-alt" class="form-label">Image alternative text</label><input id="listing-media-alt" :value="value.media_alt" name="media_alt" maxlength="280" required class="form-input" :aria-invalid="Boolean(errors.media_alt)" aria-describedby="listing-alt-help listing-alt-error" @input="set('media_alt', ($event.target as HTMLInputElement).value)" /><p id="listing-alt-help" class="field-help">Describe what the image shows for people who cannot see it.</p><p id="listing-alt-error" class="form-error">{{ errors.media_alt }}</p></div>
  </fieldset>
</template>
