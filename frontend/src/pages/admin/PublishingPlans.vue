<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Head, Link, router, useForm } from '@inertiajs/vue3'
import { billingTypeLabel, publishingPrice, type PublishingPlan } from '../../types/publishing'

const props = defineProps<{ plans: PublishingPlan[] }>()
const freshPlan = (): PublishingPlan => ({
  key: '', name: '', description: '', enabled: false, billing_type: 'free', amount: 0, currency: 'USD', version: 0,
})
const form = useForm<PublishingPlan>(freshPlan())
const editing = ref<string | null>(null)
const formElement = ref<HTMLFormElement | null>(null)
const errorSummary = ref<HTMLElement | null>(null)
const savedNotice = ref('')
const requestError = ref('')
const errors = computed(() => form.errors as Record<string, string>)
const fieldLabels: Record<string, string> = {
  key: 'Plan identifier', name: 'Name', description: 'Description', enabled: 'Availability',
  billing_type: 'Billing type', amount: 'Amount', currency: 'Currency', version: 'Saved version',
}
const mayDiscard = () => !form.isDirty || window.confirm('Discard your unsaved plan changes?')

async function selectPlan(plan: PublishingPlan | null) {
  if (form.processing || !mayDiscard()) return
  editing.value = plan?.key ?? null
  form.defaults(plan ? { ...plan } : freshPlan())
  form.reset()
  form.clearErrors()
  savedNotice.value = ''
  requestError.value = ''
  await nextTick()
  document.getElementById(plan ? 'publishing-name' : 'publishing-key')?.focus()
}

async function focusErrors() {
  await nextTick()
  const invalid = formElement.value?.querySelector<HTMLElement>('[aria-invalid="true"]:not(:disabled)')
  ;(invalid ?? errorSummary.value)?.focus()
}

function save() {
  savedNotice.value = ''
  requestError.value = ''
  form.transform(data => ({
    ...data, key: data.key.trim(), name: data.name.trim(), description: data.description.trim(),
    currency: data.currency.trim().toUpperCase(), amount: data.billing_type === 'free' ? 0 : data.amount,
  })).post(editing.value ? `/admin/plans/${encodeURIComponent(editing.value)}` : '/admin/plans', {
    preserveScroll: true,
    onError: focusErrors,
    onNetworkError: () => failedRequest('We couldn’t confirm that the plan was saved. Reload saved plans before trying again.'),
    onHttpException: () => failedRequest('The plan could not be saved. Reload saved plans and check your access before trying again.'),
    onSuccess: async () => {
      const key = form.key.trim()
      await nextTick()
      const saved = props.plans.find(plan => plan.key === key)
      if (saved) {
        editing.value = saved.key
        form.defaults({ ...saved })
        form.reset()
      } else {
        form.defaults()
      }
      form.clearErrors()
      savedNotice.value = 'Plan saved. Existing purchases keep their original terms.'
    },
  })
}

function failedRequest(message: string) {
  requestError.value = message
  void focusErrors()
  return false
}
let skipLeavePrompt = false
function reloadPlans() {
  if (!mayDiscard()) return
  skipLeavePrompt = true
  router.reload({ only: ['plans'], onNetworkError: () => failedRequest('Saved plans could not be refreshed. Your edits are still here.'),
    onHttpException: () => failedRequest('Saved plans could not be refreshed. Check your access and try again.'), onSuccess: () => {
    const saved = props.plans.find(plan => plan.key === editing.value)
    form.defaults(saved ? { ...saved } : freshPlan())
    if (!saved) editing.value = null
    form.reset()
    form.clearErrors()
    requestError.value = ''
  } })
  skipLeavePrompt = false
}

watch(() => form.billing_type, type => { if (type === 'free') form.amount = 0 })
watch(() => form.isDirty, dirty => { if (dirty) savedNotice.value = '' })
const beforeUnload = (event: BeforeUnloadEvent) => {
  if (form.isDirty) { event.preventDefault(); event.returnValue = '' }
}
let stopBeforeVisit: (() => void) | undefined
onMounted(() => {
  window.addEventListener('beforeunload', beforeUnload)
  stopBeforeVisit = router.on('before', event => {
    if (!skipLeavePrompt && event.detail.visit.method === 'get' && form.isDirty && !mayDiscard()) event.preventDefault()
  })
})
onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', beforeUnload)
  stopBeforeVisit?.()
})
</script>

<template>
  <Head title="Publishing plans" />
  <section class="directory-page publishing-admin">
    <header class="directory-heading directory-heading-actions">
      <div><h1>Publishing plans</h1><p>Set the plans owners can choose after their listings are approved.</p></div>
      <Link href="/admin/billing" class="button button-secondary">Provider price mappings</Link>
    </header>
    <p class="publishing-note">Saving a plan does not verify its price with Stripe or Paddle. Map matching provider prices before offering a paid plan.</p>
    <div class="publishing-admin-layout">
      <section class="publishing-plan-index" aria-labelledby="saved-plans-title">
        <div class="publishing-section-heading"><h2 id="saved-plans-title">Saved plans</h2><span>{{ plans.length }} / 100</span></div>
        <p v-if="!plans.length" class="publishing-empty">No plans yet. Create a free or paid plan to offer approved listings.</p>
        <ul v-else class="publishing-plan-list">
          <li v-for="plan in plans" :key="plan.key">
            <button type="button" class="publishing-plan-select" :aria-pressed="editing === plan.key" :disabled="form.processing" @click="selectPlan(plan)">
              <span class="publishing-plan-name">{{ plan.name }}</span>
              <span class="publishing-plan-price">{{ publishingPrice(plan) }}</span>
              <span class="publishing-plan-detail">{{ plan.key }} · {{ plan.enabled ? 'Enabled' : 'Disabled' }}</span>
            </button>
          </li>
        </ul>
        <button type="button" class="button button-secondary" :disabled="form.processing || plans.length >= 100" @click="selectPlan(null)">New plan</button>
        <p v-if="plans.length >= 100" class="field-help">The 100-plan limit is reached. Edit an existing plan.</p>
      </section>

      <form ref="formElement" class="publishing-plan-editor" :aria-busy="form.processing" @submit.prevent="save">
        <h2>{{ editing ? 'Edit plan' : 'Create plan' }}</h2>
        <p class="field-help">Changes apply to new purchases. Existing purchases keep their saved price and publishing terms.</p>
        <div v-if="Object.keys(errors).length || requestError" ref="errorSummary" class="billing-errors" role="alert" tabindex="-1">
          <h3>Check your plan</h3>
          <p v-if="requestError">{{ requestError }}</p>
          <ul v-if="Object.keys(errors).length"><li v-for="(message, field) in errors" :key="field">{{ fieldLabels[field] ?? field }}: {{ message }}</li></ul>
          <button v-if="errors.version || requestError" type="button" class="button button-secondary" :disabled="form.processing" @click="reloadPlans">Reload saved plans</button>
        </div>
        <p v-if="savedNotice" class="billing-saved" role="status">{{ savedNotice }}</p>
        <fieldset class="directory-fields" :disabled="form.processing">
          <div class="directory-field">
            <label for="publishing-key" class="form-label">Plan identifier</label>
            <input id="publishing-key" v-model="form.key" name="key" class="form-input" required maxlength="64" :readonly="editing !== null" autocapitalize="none" :spellcheck="false" :aria-invalid="!!errors.key" aria-describedby="publishing-key-help publishing-key-error" />
            <p id="publishing-key-help" class="field-help">Use this identifier in provider price mappings. It cannot change after creation.</p>
            <p v-if="errors.key" id="publishing-key-error" class="form-error">{{ errors.key }}</p>
          </div>
          <div class="directory-field">
            <label for="publishing-name" class="form-label">Name</label>
            <input id="publishing-name" v-model="form.name" name="name" class="form-input" required :aria-invalid="!!errors.name" :aria-describedby="errors.name ? 'publishing-name-error' : undefined" />
            <p v-if="errors.name" id="publishing-name-error" class="form-error">{{ errors.name }}</p>
          </div>
          <div class="directory-field">
            <label for="publishing-description" class="form-label">Description</label>
            <textarea id="publishing-description" v-model="form.description" name="description" class="form-input" rows="3" :aria-invalid="!!errors.description" :aria-describedby="errors.description ? 'publishing-description-error' : undefined" />
            <p v-if="errors.description" id="publishing-description-error" class="form-error">{{ errors.description }}</p>
          </div>
          <div class="directory-field">
            <label for="publishing-type" class="form-label">Billing type</label>
            <select id="publishing-type" v-model="form.billing_type" name="billing_type" class="form-input" :aria-invalid="!!errors.billing_type" :aria-describedby="errors.billing_type ? 'publishing-type-error' : undefined">
              <option v-for="(label, type) in billingTypeLabel" :key="type" :value="type">{{ label }}</option>
            </select>
            <p v-if="errors.billing_type" id="publishing-type-error" class="form-error">{{ errors.billing_type }}</p>
          </div>
          <div class="publishing-money-fields">
            <div class="directory-field">
              <label for="publishing-amount" class="form-label">Amount in minor units</label>
              <input id="publishing-amount" v-model.number="form.amount" name="amount" class="form-input" type="number" inputmode="numeric" min="0" step="1" required :disabled="form.billing_type === 'free'" :aria-invalid="!!errors.amount" aria-describedby="publishing-amount-help publishing-amount-error" />
              <p id="publishing-amount-help" class="field-help">For example, USD 10.00 is 1000. Free plans use 0.</p>
              <p v-if="errors.amount" id="publishing-amount-error" class="form-error">{{ errors.amount }}</p>
            </div>
            <div class="directory-field">
              <label for="publishing-currency" class="form-label">Currency</label>
              <input id="publishing-currency" v-model="form.currency" name="currency" class="form-input" required minlength="3" maxlength="3" autocapitalize="characters" :spellcheck="false" placeholder="USD" :aria-invalid="!!errors.currency" :aria-describedby="errors.currency ? 'publishing-currency-error' : undefined" />
              <p v-if="errors.currency" id="publishing-currency-error" class="form-error">{{ errors.currency }}</p>
            </div>
          </div>
          <div class="directory-field">
            <label class="billing-check"><input v-model="form.enabled" name="enabled" type="checkbox" :aria-invalid="!!errors.enabled" aria-describedby="publishing-enabled-help publishing-enabled-error" />Offer this plan to owners</label>
            <p id="publishing-enabled-help" class="field-help">Disabling stops new purchases. Existing purchases stay unchanged.</p>
            <p v-if="errors.enabled" id="publishing-enabled-error" class="form-error">{{ errors.enabled }}</p>
          </div>
        </fieldset>
        <div class="directory-actions">
          <button type="submit" class="button button-primary" :disabled="form.processing || (!editing && plans.length >= 100)">{{ form.processing ? 'Saving plan…' : editing ? 'Save plan' : 'Create plan' }}</button>
          <span v-if="form.isDirty" class="field-help">Unsaved changes</span>
        </div>
      </form>
    </div>
  </section>
</template>
