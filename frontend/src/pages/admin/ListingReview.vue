<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { Head, Link, useForm } from '@inertiajs/vue3'
import type { Category, OwnerListing } from '../../types/listings'
const props = defineProps<{ listing: OwnerListing; categories: Category[] }>()
const decision = useForm({ version: props.listing.version, revision_id: props.listing.current.id, decision: 'approve' as 'approve' | 'reject', reason: '' })
const suspension = useForm({ version: props.listing.version, suspended: !props.listing.suspended, reason: '' })
const errorSummary = ref<HTMLElement | null>(null)
const errors = computed(() => ({ ...decision.errors, ...suspension.errors }) as Record<string, string>)
const busy = computed(() => decision.processing || suspension.processing)
const categoryNames = (ids: number[]) => ids.map(id => props.categories.find(category => category.id === id)?.name ?? `Category ${id}`).join(', ')
const focusErrors = async () => { await nextTick(); errorSummary.value?.focus() }
function review() {
  suspension.clearErrors()
  decision.version = props.listing.version; decision.revision_id = props.listing.current.id
  decision.post(`/admin/listings/${props.listing.id}/decision`, { preserveScroll: true, onError: focusErrors, onSuccess: () => decision.reset('reason') })
}
function changeSuspension() {
  decision.clearErrors(); suspension.version = props.listing.version; suspension.suspended = !props.listing.suspended
  suspension.post(`/admin/listings/${props.listing.id}/suspension`, { preserveScroll: true, onError: focusErrors, onSuccess: () => suspension.reset('reason') })
}
</script>
<template>
  <Head :title="`Review ${listing.current.title}`" />
  <section class="directory-page"><Link href="/admin/listings" class="text-link">Back to review queue</Link><header class="directory-heading"><h1>Review listing</h1><p>Review the submitted content before recording your decision.</p></header>
    <div v-if="Object.keys(errors).length" ref="errorSummary" class="billing-errors" tabindex="-1" role="alert"><h2>The action was not saved</h2><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><Link v-if="errors.version || errors.revision_id" :href="`/admin/listings/${listing.id}`" class="text-link">Reload the latest revision before deciding</Link></div>
    <div class="directory-review-columns"><section aria-labelledby="current-revision"><h2 id="current-revision">Current revision · {{ listing.current.status }}</h2><h3>{{ listing.current.title }}</h3><p>{{ listing.current.summary }}</p><dl class="directory-review-meta"><dt>Website</dt><dd><a :href="listing.current.url" target="_blank" rel="noopener noreferrer" class="text-link">{{ listing.current.url }} <span class="sr-only">(opens in a new tab)</span></a></dd><dt>Categories</dt><dd>{{ categoryNames(listing.current.category_ids) }}</dd><dt>Image alternative text</dt><dd>{{ listing.current.media_alt || 'No image description' }}</dd></dl><img v-if="listing.current.media_url" :src="listing.current.media_url" :alt="listing.current.media_alt" class="directory-detail-image" /><h3>Description (Markdown source)</h3><pre class="directory-source">{{ listing.current.description }}</pre></section>
      <section v-if="listing.approved" aria-labelledby="approved-revision"><h2 id="approved-revision">Last approved revision</h2><h3>{{ listing.approved.title }}</h3><p>{{ listing.approved.summary }}</p><dl class="directory-review-meta"><dt>Website</dt><dd>{{ listing.approved.url }}</dd><dt>Categories</dt><dd>{{ categoryNames(listing.approved.category_ids) }}</dd><dt>Image alternative text</dt><dd>{{ listing.approved.media_alt || 'No image description' }}</dd></dl><img v-if="listing.approved.media_url" :src="listing.approved.media_url" :alt="listing.approved.media_alt" class="directory-detail-image" /><h3>Description (Markdown source)</h3><pre class="directory-source">{{ listing.approved.description }}</pre></section></div>
    <form v-if="listing.current.status === 'submitted' && !listing.archived" class="directory-review-action" @submit.prevent="review"><h2>Record review decision</h2><p>Approval accepts this revision. Publication also requires an eligible publishing plan.</p><fieldset :disabled="busy"><legend class="sr-only">Decision</legend><label class="billing-check"><input v-model="decision.decision" type="radio" name="decision" value="approve" />Approve revision</label><label class="billing-check"><input v-model="decision.decision" type="radio" name="decision" value="reject" />Reject revision</label><label for="review-reason" class="form-label">{{ decision.decision === 'reject' ? 'Reason for rejection' : 'Review note (optional)' }}</label><textarea id="review-reason" v-model="decision.reason" name="reason" rows="3" class="form-input" :required="decision.decision === 'reject'" :aria-invalid="Boolean(decision.errors.reason)" aria-describedby="review-reason-error" /><p id="review-reason-error" class="form-error">{{ decision.errors.reason }}</p><button type="submit" class="button button-primary">{{ decision.processing ? 'Recording…' : 'Record decision' }}</button></fieldset></form>
    <p v-else class="notice">Only a submitted revision can receive a review decision.</p>
    <form class="directory-review-action" @submit.prevent="changeSuspension"><h2>{{ listing.suspended ? 'Reinstate listing' : 'Suspend listing' }}</h2><p>{{ listing.suspended ? 'Reinstatement removes the moderation suspension. It does not grant payment eligibility.' : 'Suspension removes the listing from public surfaces without deleting its history.' }}</p><fieldset :disabled="busy"><label for="suspension-reason" class="form-label">Reason for {{ listing.suspended ? 'reinstatement' : 'suspension' }}</label><textarea id="suspension-reason" v-model="suspension.reason" name="reason" rows="3" required class="form-input" :aria-invalid="Boolean(suspension.errors.reason)" aria-describedby="suspension-reason-error" /><p id="suspension-reason-error" class="form-error">{{ suspension.errors.reason }}</p><button type="submit" class="button button-secondary">{{ suspension.processing ? 'Saving…' : listing.suspended ? 'Reinstate listing' : 'Suspend listing' }}</button></fieldset></form>
  </section>
</template>
