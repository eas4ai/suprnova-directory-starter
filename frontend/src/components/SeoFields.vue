<script setup lang="ts">
import type { SeoOverrides } from '../types/seo'
const props = defineProps<{ modelValue: SeoOverrides; errors: Record<string, string>; prefix: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: SeoOverrides] }>()
function set<K extends keyof SeoOverrides>(key: K, value: SeoOverrides[K]) { emit('update:modelValue', { ...props.modelValue, [key]: value }) }
const fields = [{ key: 'title', label: 'Search title', max: 160 }, { key: 'description', label: 'Search description', max: 320 }, { key: 'image', label: 'Social image URL', max: 2048 }] as const
</script>
<template>
  <fieldset class="directory-fields"><legend>Search and social metadata</legend>
    <p class="field-help">Optional. Blank fields use the content title, summary and image, then site defaults. These changes follow the same publication rules as the content.</p>
    <div v-for="field in fields" :key="field.key"><label :for="prefix + '-seo-' + field.key" class="form-label">{{ field.label }}</label>
      <input :id="prefix + '-seo-' + field.key" :value="modelValue[field.key]" :maxlength="field.max" class="form-input" :aria-invalid="Boolean(errors['seo.' + field.key])" :aria-describedby="prefix + '-seo-' + field.key + '-error'" @input="set(field.key, ($event.target as HTMLInputElement).value)" />
      <p :id="prefix + '-seo-' + field.key + '-error'" class="form-error">{{ errors['seo.' + field.key] }}</p>
    </div>
    <label class="billing-check"><input type="checkbox" :checked="modelValue.noindex" @change="set('noindex', ($event.target as HTMLInputElement).checked)" />Exclude this page from search engines</label>
    <p class="field-help">The page remains public. It receives noindex and is excluded from sitemaps.</p>
  </fieldset>
</template>
