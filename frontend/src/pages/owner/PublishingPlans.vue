<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { Head, Link, useForm } from '@inertiajs/vue3'
import { publishingPrice, publishingTerms, type PublishingOffer, type PublishingProvider } from '../../types/publishing'

const props = defineProps<{
  listing: { id: number; title: string; slug: string }
  plans: PublishingOffer[]
  mode: 'test' | 'live'
  purchase_id?: string | null
}>()
const form = useForm({ plan_key: '', provider: null as PublishingProvider | null })
const providers = ref<Record<string, PublishingProvider | ''>>(Object.create(null))
const errorSummary = ref<HTMLElement | null>(null)
const requestError = ref('')
const errors = computed(() => form.errors as Record<string, string>)
watch(() => props.plans, plans => {
  const next: Record<string, PublishingProvider | ''> = Object.create(null)
  for (const plan of plans) {
    const previous = providers.value[plan.key]
    next[plan.key] = plan.providers.find(provider => provider.id === previous)?.id
      ?? plan.providers.find(provider => provider.default)?.id ?? plan.providers[0]?.id ?? ''
  }
  providers.value = next
}, { immediate: true })

function choose(plan: PublishingOffer) {
  if (form.processing || props.purchase_id) return
  form.plan_key = plan.key
  form.provider = plan.billing_type === 'free' ? null : providers.value[plan.key] || null
  if (plan.billing_type !== 'free' && !form.provider) return
  form.clearErrors()
  requestError.value = ''
  form.post(`/dashboard/listings/${encodeURIComponent(props.listing.id)}/checkout`, {
    onError: async () => { await nextTick(); errorSummary.value?.focus() },
    onNetworkError: failedRequest,
    onHttpException: failedRequest,
  })
}
function failedRequest() {
  requestError.value = 'We couldn’t confirm the checkout request. Your saved purchase will be reused when you try again.'
  void nextTick(() => errorSummary.value?.focus())
  return false
}
</script>

<template>
  <Head title="Choose a publishing plan" />
  <section class="directory-page publishing-owner">
    <header class="directory-heading">
      <Link :href="`/dashboard/listings/${listing.id}/edit`" class="text-link">Back to your listing</Link>
      <h1>Choose a publishing plan</h1>
      <p>For the approved revision of <strong>{{ listing.title }}</strong>.</p>
    </header>
    <p v-if="mode === 'test'" class="notice" role="status"><strong>Test checkout.</strong> Test payments do not publish your listing in the public directory.</p>
    <section v-if="purchase_id" class="notice" aria-labelledby="existing-purchase-heading">
      <h2 id="existing-purchase-heading">Your publishing purchase is saved</h2>
      <p>Continue the existing purchase to check payment or manage your plan. A listing can have only one active publishing purchase.</p>
      <Link :href="`/dashboard/purchases/${purchase_id}`" class="button button-primary">Continue existing purchase</Link>
    </section>
    <p class="publishing-note">Review comes before checkout. A paid listing becomes eligible only after payment is verified. Provider checkout shows the final tax and any available discount.</p>
    <div v-if="Object.keys(errors).length || requestError" ref="errorSummary" class="billing-errors" role="alert" tabindex="-1">
      <h2>We couldn’t start your checkout</h2>
      <p v-if="requestError">{{ requestError }}</p>
      <ul v-if="Object.keys(errors).length"><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul>
    </div>
    <div v-if="!plans.length && !purchase_id" class="directory-empty"><h2>No publishing plans available</h2><p>Your approval is saved. Check again later or contact the directory administrator.</p><Link href="/dashboard/listings" class="button button-secondary">Your listings</Link></div>
    <ul v-if="plans.length && !purchase_id" class="publishing-offers">
      <li v-for="plan in plans" :key="plan.key" class="publishing-offer">
        <div class="publishing-offer-copy">
          <h2 :id="`offer-${plan.key}`">{{ plan.name }}</h2>
          <p v-if="plan.description">{{ plan.description }}</p>
          <p class="field-help">{{ publishingTerms(plan.billing_type) }}</p>
        </div>
        <div class="publishing-offer-action">
          <p class="publishing-offer-price">{{ publishingPrice(plan) }}</p>
          <form :aria-labelledby="`offer-${plan.key}`" :aria-busy="form.processing && form.plan_key === plan.key" @submit.prevent="choose(plan)">
            <div v-if="plan.billing_type !== 'free' && plan.providers.length" class="directory-field">
              <label :for="`provider-${plan.key}`" class="form-label">Payment provider</label>
              <select :id="`provider-${plan.key}`" v-model="providers[plan.key]" name="provider" class="form-input" :disabled="form.processing" :aria-invalid="form.plan_key === plan.key && !!errors.provider" :aria-describedby="form.plan_key === plan.key && errors.provider ? `provider-error-${plan.key}` : undefined">
                <option v-for="provider in plan.providers" :key="provider.id" :value="provider.id">{{ provider.label }}{{ provider.default ? ' (default)' : '' }}</option>
              </select>
              <p v-if="form.plan_key === plan.key && errors.provider" :id="`provider-error-${plan.key}`" class="form-error">{{ errors.provider }}</p>
            </div>
            <p v-if="plan.billing_type !== 'free' && !plan.providers.length" class="field-help">Payment is unavailable for this plan right now.</p>
            <button type="submit" class="button button-primary" :disabled="form.processing || (plan.billing_type !== 'free' && !plan.providers.length)">
              {{ form.processing && form.plan_key === plan.key ? 'Preparing your plan…' : plan.billing_type === 'free' ? 'Choose free plan' : 'Continue to checkout' }}
            </button>
          </form>
        </div>
      </li>
    </ul>
  </section>
</template>
