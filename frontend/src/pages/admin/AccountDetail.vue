<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Head, Link, router, useForm } from '@inertiajs/vue3'
type AccountRow = { id: number; name: string; email: string; verified: boolean; suspended: boolean; version: number; roles: string[] }
const props = defineProps<{ account: AccountRow }>()
const initial = () => ({ version: props.account.version, roles: [...props.account.roles], suspended: props.account.suspended })
const form = useForm(initial())
const saved = ref(false)
const summary = ref<HTMLElement | null>(null)
const errors = computed(() => form.errors as Record<string, string>)
const roles = [{ value: 'administrator', label: 'Administrator', description: 'Manage the directory and its administrative access.' }, { value: 'moderator', label: 'Moderator', description: 'Review submitted listings.' }, { value: 'editor', label: 'Editor', description: 'Write and publish articles.' }]
const mayLeave = () => !form.isDirty || window.confirm('Discard your unsaved account access changes?')
const beforeUnload = (event: BeforeUnloadEvent) => { if (form.isDirty) { event.preventDefault(); event.returnValue = '' } }
let removeGuard: (() => void) | undefined
onMounted(() => { window.addEventListener('beforeunload', beforeUnload); removeGuard = router.on('before', event => { if (event.detail.visit.method === 'get' && !mayLeave()) event.preventDefault() }) })
onBeforeUnmount(() => { window.removeEventListener('beforeunload', beforeUnload); removeGuard?.() })
async function focusErrors() { await nextTick(); summary.value?.focus() }
function save() {
  if (form.processing) return
  saved.value = false
  form.post('/admin/accounts/' + props.account.id, { preserveScroll: true, onError: focusErrors, onSuccess: () => { form.defaults(initial()); form.reset(); saved.value = true } })
}
</script>
<template>
  <Head :title="'Account access: ' + account.name" />
  <section class="directory-page directory-editor">
    <Link href="/admin/accounts" class="text-link">Back to accounts</Link>
    <header class="directory-heading"><h1>Account access</h1><p>{{ account.name }} · {{ account.email }}</p></header>
    <dl class="directory-review-meta"><dt>Email verification</dt><dd>{{ account.verified ? 'Verified' : 'Not verified' }}</dd><dt>Saved account status</dt><dd>{{ account.suspended ? 'Suspended' : 'Active' }}</dd></dl>
    <p class="notice">Role grants require a verified email address. The account holder must complete email verification before receiving an administrative role.</p>
    <div v-if="Object.keys(errors).length" ref="summary" tabindex="-1" role="alert" class="billing-errors"><h2>Access was not saved</h2><ul><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul><p v-if="errors.version">Your selections are still here. Note the changes you want to keep before reloading the latest access settings.</p><Link v-if="errors.version" :href="'/admin/accounts/' + account.id" class="text-link">Reload latest access</Link></div>
    <p v-if="saved" role="status" class="billing-saved">Account access saved.</p>
    <form :aria-busy="form.processing" @submit.prevent="save"><fieldset class="directory-fields" :disabled="form.processing"><legend class="sr-only">Account access settings</legend>
      <fieldset class="directory-category-options" aria-describedby="account-roles-help account-roles-error"><legend class="form-label">Administrative roles</legend><p id="account-roles-help" class="field-help">Select only the access this person needs. Removing every role keeps their ordinary account.</p><label v-for="role in roles" :key="role.value" class="billing-check"><input v-model="form.roles" name="roles" type="checkbox" :value="role.value" :aria-invalid="Boolean(errors.roles)" :aria-describedby="'account-role-' + role.value + ' account-roles-error'" />{{ role.label }}<span :id="'account-role-' + role.value" class="field-help">{{ role.description }}</span></label><p id="account-roles-error" class="form-error">{{ errors.roles }}</p></fieldset>
      <div><label class="billing-check"><input v-model="form.suspended" name="suspended" type="checkbox" :aria-invalid="Boolean(errors.suspended)" aria-describedby="account-suspension-help account-suspension-error" />Suspend account</label><p id="account-suspension-help" class="field-help">Block this person's account access. Clear this option to restore access.</p><p id="account-suspension-error" class="form-error">{{ errors.suspended }}</p></div>
    </fieldset><button type="submit" class="button button-primary" :disabled="form.processing">{{ form.processing ? 'Saving…' : 'Save access' }}</button><p v-if="form.isDirty" class="field-help">You have unsaved access changes.</p></form>
    <section class="directory-archive"><h2>Host recovery</h2><p>If administrative access is lost, the host operator can restore it with <code>admin:access grant --user-id {{ account.id }}</code>.</p></section>
  </section>
</template>
