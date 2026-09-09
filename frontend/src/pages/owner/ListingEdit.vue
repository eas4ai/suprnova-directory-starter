<script setup lang="ts">
import OwnerNotifications from '../../components/OwnerNotifications.vue'
import type { OwnerNotice } from '../../types/notifications'
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Head, Link, useForm } from '@inertiajs/vue3'
import ListingForm from '../../components/ListingForm.vue'
import type { Category, ListingInput, OwnerListing } from '../../types/listings'
const props = defineProps<{ listing: OwnerListing | null; categories: Category[]; notifications: OwnerNotice[] }>()
function initial(): ListingInput {
  const revision = props.listing?.current
  return { version: props.listing?.version ?? 0, title: revision?.title ?? '', summary: revision?.summary ?? '', description: revision?.description ?? '', url: revision?.url ?? '', category_ids: [...(revision?.category_ids ?? [])], media_id: revision?.media_id ?? null, media_alt: revision?.media_alt ?? '' }
}
const form = useForm(initial())
const action = useForm({ version: props.listing?.version ?? 0 })
const uploading = ref(false)
const saved = ref(false)
const errorSummary = ref<HTMLElement | null>(null)
const errors = computed(() => ({ ...form.errors, ...action.errors }) as Record<string, string>)
const busy = computed(() => form.processing || action.processing || uploading.value)
const focusErrors = async () => { await nextTick(); errorSummary.value?.focus() }
const mayLeave = () => !form.isDirty || window.confirm('Discard your unsaved listing changes?')
const beforeUnload = (event: BeforeUnloadEvent) => { if (form.isDirty) { event.preventDefault(); event.returnValue = '' } }
onMounted(() => window.addEventListener('beforeunload', beforeUnload))
onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
function update(value: ListingInput) { Object.assign(form, value); saved.value = false }
function save() {
  action.clearErrors(); saved.value = false
  form.post(props.listing ? `/dashboard/listings/${props.listing.id}` : '/dashboard/listings', { preserveScroll: true, onError: focusErrors, onSuccess: () => { form.defaults(initial()); form.reset(); saved.value = true } })
}
function mutate(kind: 'submit' | 'archive') {
  if (!props.listing || busy.value) return
  if (kind === 'archive' && !window.confirm('Archive this listing? It will be removed from the public directory. Its review and payment history will be kept.')) return
  action.version = props.listing.version; form.clearErrors()
  action.post(`/dashboard/listings/${props.listing.id}/${kind}`, { preserveScroll: true, onError: focusErrors, onSuccess: () => { form.defaults(initial()); form.reset() } })
}
</script>
<template>
  <Head :title="listing ? 'Edit listing' : 'Create listing'" />
  <section class="directory-page directory-editor">
    <Link href="/dashboard/listings" class="text-link" :on-before="mayLeave">Back to your listings</Link>
    <header class="directory-heading"><h1>{{ listing ? 'Edit listing' : 'Create listing' }}</h1><p>Save your content, then submit the saved revision for review.</p></header>
    <p v-if="listing?.approved" class="notice">Your last approved revision stays public while changes await review, as long as the listing remains eligible for publication.</p>
    <p v-if="listing?.current.reason" class="notice"><strong>Review feedback:</strong> {{ listing.current.reason }}<span v-if="listing.current.status === 'rejected'"> Update the content and save a new draft before submitting it again.</span></p>
    <p v-if="listing?.archived" class="notice">This listing is archived and is not public.</p>
    <div v-if="Object.keys(errors).length" ref="errorSummary" tabindex="-1" role="alert" class="billing-errors"><h2>Changes were not saved</h2><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><p v-if="errors.version">Your edits are still here. Copy any changes you want to keep before reloading the latest revision.</p><Link v-if="errors.version && listing" :href="`/dashboard/listings/${listing.id}/edit`" :on-before="mayLeave" class="text-link">Reload latest revision</Link></div>
    <p v-if="saved" role="status" class="billing-saved">Draft saved.</p>
    <form @submit.prevent="save"><ListingForm :model-value="form.data()" :categories="categories" :errors="errors" :disabled="busy || Boolean(listing?.archived)" :media-url="listing?.current.media_url ?? null" @update:model-value="update" @uploading="uploading = $event" /><div class="directory-actions"><button type="submit" class="button button-primary" :disabled="busy || listing?.archived">{{ form.processing ? 'Saving…' : 'Save draft' }}</button><button v-if="listing && !listing.archived && listing.current.status === 'draft'" type="button" class="button button-secondary" :disabled="busy || form.isDirty" @click="mutate('submit')">Submit for review</button></div><p v-if="form.isDirty && listing" class="field-help">Save your changes before submitting for review.</p><p v-if="listing?.current.status === 'submitted'" class="field-help">This revision is awaiting review. Saving changes creates a new draft to submit.</p></form>
    <section v-if="listing && !listing.archived" class="directory-archive"><h2>Archive listing</h2><p>Remove this listing from the directory while keeping its history.</p><button type="button" class="button button-secondary" :disabled="busy || form.isDirty" @click="mutate('archive')">Archive listing</button></section>
    <OwnerNotifications :notifications="notifications" />
  </section>
</template>
