<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Head, Link, useForm } from '@inertiajs/vue3'
import BillingCredentialField from '../../components/BillingCredentialField.vue'
import type { BillingMode, BillingProvider, BillingSettings, ProfileInput } from '../../types/billing'

const props = defineProps<{ mode: BillingMode; settings: BillingSettings | null; configuration_error: string | null }>()
const profiles: { id: BillingProvider; name: string; publicLabel: string }[] = [
  { id: 'stripe', name: 'Stripe', publicLabel: 'Stripe publishable key' },
  { id: 'paddle', name: 'Paddle', publicLabel: 'Paddle client token' },
]
const profileInput = (provider: BillingProvider): ProfileInput => ({
  enabled: props.settings?.[provider].enabled ?? false,
  public_key: props.settings?.[provider].public_key ?? '',
  api_key: '', webhook_key: '', clear_secrets: false,
})
const initial = () => ({
  revision: props.settings?.revision ?? 0,
  default_provider: props.settings?.default_provider ?? null as BillingProvider | null,
  stripe: profileInput('stripe'), paddle: profileInput('paddle'),
  mappings: props.settings?.mappings.map(row => ({ plan: row.plan, stripe: row.stripe ?? '', paddle: row.paddle ?? '' })) ?? [],
})
// Unkeyed form: replacement secrets are never remembered in browser history.
const form = useForm(initial())
const errorSummary = ref<HTMLElement | null>(null)
const errors = computed(() => form.errors as Record<string, string>)
const savedNotice = ref(false)
const modeName = computed(() => props.mode === 'test' ? 'Test' : 'Live')

watch(() => props.mode, () => {
  form.defaults(initial())
  form.reset()
  form.clearErrors()
})
watch(() => form.isDirty, dirty => { if (dirty) savedNotice.value = false })
for (const provider of ['stripe', 'paddle'] as const) {
  watch(() => form[provider].enabled, enabled => {
    if (!enabled && form.default_provider === provider) form.default_provider = null
    if (enabled) form[provider].clear_secrets = false
  })
}

function save() {
  savedNotice.value = false
  form.post(`/admin/billing?mode=${props.mode}`, {
    preserveScroll: true,
    onSuccess: () => {
      form.defaults(initial())
      form.reset()
      form.clearErrors()
      savedNotice.value = true
    },
    onError: async () => { await nextTick(); errorSummary.value?.focus() },
  })
}
const mayLeave = () => !form.isDirty || window.confirm('Discard your unsaved billing changes?')
const beforeUnload = (event: BeforeUnloadEvent) => {
  if (form.isDirty) { event.preventDefault(); event.returnValue = '' }
}
onMounted(() => window.addEventListener('beforeunload', beforeUnload))
onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
async function addMapping() {
  form.mappings.push({ plan: '', stripe: '', paddle: '' })
  await nextTick()
  document.getElementById(`plan-${form.mappings.length - 1}`)?.focus()
}
</script>

<template>
  <Head title="Payment providers" />
  <section class="billing-page">
    <header class="billing-heading">
      <div><h1>Payment providers</h1><p>Manage credentials and plan prices for Stripe and Paddle.</p></div>
      <nav class="billing-modes" aria-label="Billing mode">
        <Link v-for="option in (['test', 'live'] as const)" :key="option" :href="`/admin/billing?mode=${option}`"
          class="nav-link" :aria-current="mode === option ? 'page' : undefined" :on-before="mayLeave">{{ option === 'test' ? 'Test mode' : 'Live mode' }}</Link>
      </nav>
    </header>
    <div v-if="configuration_error" class="notice" role="alert">
      <h2>Billing configuration is unavailable</h2><p>{{ configuration_error }}</p>
      <Link :href="`/admin/billing?mode=${mode}`" class="button button-secondary">Try again</Link>
    </div>
    <form v-else-if="settings" novalidate class="billing-form" @submit.prevent="save">
      <div class="billing-savebar">
        <div><strong>{{ modeName }} mode</strong><p>{{ mode === 'test' ? 'Separate credentials and prices for testing.' : 'Credentials and prices for your live provider accounts.' }}</p></div>
        <button type="submit" class="button button-primary" :disabled="form.processing">{{ form.processing ? 'Saving…' : `Save ${mode} settings` }}</button>
      </div>
      <p v-if="savedNotice" role="status" class="billing-saved">Settings saved. Provider authentication and price availability have not been verified.</p>
      <div v-if="Object.keys(errors).length" ref="errorSummary" tabindex="-1" role="alert" class="billing-errors">
        <h2>Changes were not saved</h2>
        <ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul>
        <Link v-if="errors.configuration" :href="`/admin/billing?mode=${mode}`" class="text-link" :on-before="mayLeave">Reload saved settings</Link>
      </div>
      <p class="billing-note">Saving checks format only. It does not contact the provider, verify prices or offer checkout.</p>
      <fieldset v-for="provider in profiles" :key="provider.id" class="billing-section" :disabled="form.processing">
        <legend>{{ provider.name }}</legend>
        <div class="provider-controls">
          <label class="billing-check"><input v-model="form[provider.id].enabled" type="checkbox" class="form-checkbox" />Enable {{ provider.name }} in {{ mode }} mode</label>
          <span class="field-help">{{ settings[provider.id].has_secrets ? 'Credentials saved' : 'Not configured' }}</span>
        </div>
        <div class="credential-grid">
          <BillingCredentialField v-model="form[provider.id].api_key" :id="`${provider.id}-api-key`" :label="`${provider.name} API key`" secret
            :error="errors[`${provider.id}.api_key`]" :disabled="form[provider.id].clear_secrets"
            :hint="settings[provider.id].has_secrets ? 'Leave blank to keep the saved API key.' : `Enter the API key for ${mode} mode.`" />
          <BillingCredentialField v-model="form[provider.id].public_key" :id="`${provider.id}-public-key`" :label="provider.publicLabel"
            :error="errors[`${provider.id}.public_key`]" :disabled="form[provider.id].clear_secrets" />
          <BillingCredentialField v-model="form[provider.id].webhook_key" :id="`${provider.id}-webhook-key`" :label="`${provider.name} webhook key`" secret
            :error="errors[`${provider.id}.webhook_key`]" :disabled="form[provider.id].clear_secrets"
            :hint="settings[provider.id].has_secrets ? 'Leave blank to keep the saved webhook key.' : 'Enter the signing secret for this mode’s webhook endpoint.'" />
        </div>
        <div v-if="settings[provider.id].has_secrets" class="clear-credentials">
          <label class="billing-check"><input v-model="form[provider.id].clear_secrets" type="checkbox" class="form-checkbox" :disabled="form[provider.id].enabled" />Clear {{ provider.name }} credentials when saved</label>
          <p class="field-help">Disable this profile first. Clearing removes all three credentials; you will need to enter them again.</p>
          <p v-if="errors[`${provider.id}.clear_secrets`]" class="form-error">{{ errors[`${provider.id}.clear_secrets`] }}</p>
        </div>
      </fieldset>
      <fieldset class="billing-section" :disabled="form.processing">
        <legend>Default provider</legend>
        <p class="field-help">Choose an enabled provider for {{ mode }} mode. Disabling it also clears this selection.</p>
        <div class="default-options">
          <label class="billing-check"><input v-model="form.default_provider" type="radio" name="default-provider" :value="null" />None</label>
          <label v-for="provider in profiles" :key="provider.id" class="billing-check"><input v-model="form.default_provider" type="radio" name="default-provider" :value="provider.id" :disabled="!form[provider.id].enabled" />{{ provider.name }}</label>
        </div>
      </fieldset>
      <section class="billing-section" aria-labelledby="mapping-heading">
        <div class="mapping-heading"><div><h2 id="mapping-heading">Plan prices</h2><p class="field-help">Use the same local plan identifier in each mode. A blank provider price has no fallback.</p></div>
          <button type="button" class="button button-secondary" :disabled="form.processing || form.mappings.length >= 100" @click="addMapping">Add plan</button></div>
        <p v-if="!form.mappings.length" class="mapping-empty">No plans mapped in {{ mode }} mode. Add a plan and copy its price identifiers from your provider accounts.</p>
        <p v-if="form.mappings.length >= 100" class="field-help">This mode has reached its limit of 100 plan mappings.</p>
        <div v-for="(mapping, index) in form.mappings" :key="index" class="mapping-row">
          <div><label :for="`plan-${index}`" class="form-label">Plan identifier</label><input :id="`plan-${index}`" v-model="mapping.plan" class="form-input" maxlength="64" :disabled="form.processing" autocapitalize="none" :spellcheck="false" /></div>
          <div><label :for="`stripe-price-${index}`" class="form-label">Stripe price (optional)</label><input :id="`stripe-price-${index}`" v-model="mapping.stripe" class="form-input" maxlength="512" :disabled="form.processing" autocapitalize="none" :spellcheck="false" /></div>
          <div><label :for="`paddle-price-${index}`" class="form-label">Paddle price (optional)</label><input :id="`paddle-price-${index}`" v-model="mapping.paddle" class="form-input" maxlength="512" :disabled="form.processing" autocapitalize="none" :spellcheck="false" /></div>
          <button type="button" class="button button-quiet" :disabled="form.processing" :aria-label="`Remove plan ${mapping.plan || index + 1}`" @click="form.mappings.splice(index, 1)">Remove</button>
        </div>
        <p v-if="errors.mappings" class="form-error">{{ errors.mappings }}</p>
      </section>
    </form>
  </section>
</template>
