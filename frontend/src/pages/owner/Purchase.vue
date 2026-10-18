<script lang="ts">
interface PaddleEvent { name?: string }
interface PaddleApi {
  Initialized?: boolean
  Environment: { set(environment: 'sandbox'): void }
  Initialize(options: { token: string; eventCallback: (event: PaddleEvent) => void }): void
  Update(options: { eventCallback: (event: PaddleEvent) => void }): void
  Checkout: { open(options: { transactionId: string }): void; close(): void }
}
declare global { interface Window { Paddle?: PaddleApi } }

// Paddle.js persists across Inertia visits. Initialization is once per document;
// changing a token or environment requires a fresh document, not a second Initialize.
let paddleLoad: Promise<PaddleApi> | null = null
let initializedPaddleConfig: string | null = null
const paddleScriptUrl = 'https://cdn.paddle.com/paddle/v2/paddle.js'

function loadPaddle(): Promise<PaddleApi> {
  if (window.Paddle) return Promise.resolve(window.Paddle)
  if (paddleLoad) return paddleLoad
  paddleLoad = new Promise<PaddleApi>((resolve, reject) => {
    const script = document.createElement('script')
    script.src = paddleScriptUrl
    script.async = true
    const timer = window.setTimeout(() => fail(), 15_000)
    function cleanup() {
      window.clearTimeout(timer)
      script.onload = null
      script.onerror = null
    }
    function fail() {
      cleanup()
      script.remove()
      reject(new Error('Paddle checkout could not load.'))
    }
    script.onload = () => {
      if (!window.Paddle) { fail(); return }
      cleanup()
      resolve(window.Paddle)
    }
    script.onerror = fail
    document.head.appendChild(script)
  }).catch(error => {
    paddleLoad = null
    throw error
  })
  return paddleLoad
}
</script>

<script setup lang="ts">
import OwnerNotifications from '../../components/OwnerNotifications.vue'
import type { OwnerNotice } from '../../types/notifications'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { Head, Link, router, useForm } from '@inertiajs/vue3'
import { billingTypeLabel, publishingPrice, publishingTerms, type PublishingPurchase } from '../../types/publishing'

const props = defineProps<{ purchase: PublishingPurchase; notifications: OwnerNotice[] }>()
const actionForm = useForm({})
const errors = computed(() => actionForm.errors as Record<string, string>)
const errorSummary = ref<HTMLElement | null>(null)
const clientError = ref('')
const refreshNotice = ref('')
const refreshing = ref(false)
const openingCheckout = ref(false)
const checkoutOpen = ref(false)
const reloadRequired = ref(false)
const confirmCancel = ref(false)
const cancelButton = ref<HTMLButtonElement | null>(null)
const cancelConfirmation = ref<HTMLElement | null>(null)
let mounted = true
let openedPaddle: PaddleApi | null = null
let checkoutLoadTimer: number | undefined
const busy = computed(() => actionForm.processing || refreshing.value || openingCheckout.value || checkoutOpen.value)
const statusLabels: Record<string, string> = {
  reserved: 'Preparing checkout', customer_creating: 'Preparing checkout', customer_unknown: 'Confirming checkout details',
  checkout_creating: 'Preparing checkout', open: 'Ready for payment', awaiting_evidence: 'Verifying payment',
  active: 'Active', expired: 'Expired', canceled: 'Canceled', refunded: 'Refunded', disputed: 'Payment under dispute',
}
const paymentLabels: Record<string, string> = {
  none: 'Not paid', pending: 'Awaiting confirmation', unpaid: 'Not paid', free: 'Free plan',
  paid: 'Payment verified', active: 'Payment verified', test: 'Test payment verified', test_payment: 'Test payment only',
  expired: 'Paid period ended', canceled: 'Canceled', refunded: 'Refunded', disputed: 'Under dispute',
  revoked: 'Payment eligibility removed', failed: 'Payment unsuccessful', awaiting_payment: 'Awaiting payment',
  awaiting_evidence: 'Awaiting confirmation',
}
const displayState = computed(() => ['refunded', 'disputed', 'revoked', 'expired'].includes(props.purchase.payment_status)
  ? props.purchase.payment_status : props.purchase.state)
const statusLabel = computed(() => displayState.value === 'revoked' ? 'Payment eligibility removed' : statusLabels[displayState.value] ?? 'Status unavailable')
const paymentLabel = computed(() => paymentLabels[props.purchase.payment_status] ?? 'Awaiting confirmation')
const stateExplanation = computed(() => {
  switch (displayState.value) {
    case 'active': return props.purchase.provider === 'free'
      ? 'Your free publishing plan is active. Your listing must remain approved to appear in the directory.'
      : 'Payment has been verified. Your listing must remain approved to appear in the directory.'
    case 'open': return 'Your checkout is ready. Review the final total with your payment provider before paying.'
    case 'awaiting_evidence': return 'We are waiting for verified payment confirmation. You can refresh this page without starting another purchase.'
    case 'customer_unknown': return 'The payment provider has not confirmed the earlier request yet. We are keeping the same purchase while it is checked.'
    case 'expired': return 'This purchase no longer provides current publishing access. Review your listing to see the next available action.'
    case 'canceled': return 'This purchase has been canceled. Review any paid-through date below and your listing’s current publication status.'
    case 'refunded': return 'A full refund has removed the publishing access supplied by this payment.'
    case 'disputed': return 'Publishing access from this payment is suspended while the dispute is resolved.'
    case 'revoked': return 'The provider confirmed a lost dispute. This payment no longer provides publishing access.'
    default: return 'Your purchase is saved. Checkout preparation or payment confirmation may take a moment.'
  }
})
const paidThrough = computed(() => {
  if (props.purchase.paid_through === null) return null
  const date = new Date(props.purchase.paid_through * 1000)
  if (!Number.isFinite(date.getTime())) return null
  return {
    iso: date.toISOString(),
    label: `${new Intl.DateTimeFormat('en-US', { dateStyle: 'medium', timeStyle: 'short', timeZone: 'UTC' }).format(date)} UTC`,
  }
})
const stripeUrl = computed(() => {
  if (props.purchase.checkout?.type !== 'stripe') return null
  try {
    const url = new URL(props.purchase.checkout.url)
    return url.protocol === 'https:' ? url.href : null
  } catch { return null }
})
const hasErrors = computed(() => !!clientError.value || !!props.purchase.error || Object.keys(errors.value).length > 0)

async function focusErrors() {
  await nextTick()
  if (mounted) errorSummary.value?.focus()
}

function refreshStatus() {
  if (refreshing.value || actionForm.processing || !mounted) return
  refreshing.value = true
  refreshNotice.value = ''
  router.reload({
    only: ['purchase', 'notifications'],
    onSuccess: () => { refreshNotice.value = 'Status refreshed.'; if (!reloadRequired.value) clientError.value = '' },
    onError: () => { clientError.value = 'We couldn’t refresh your payment status. Please try again.'; void focusErrors() },
    onNetworkError: () => failedRequest('We couldn’t refresh your payment status. Your saved purchase is unchanged; please try again.'),
    onHttpException: () => failedRequest('We couldn’t refresh your payment status. Reload the page and check that you are signed in.'),
    onFinish: () => { refreshing.value = false },
  })
}

function failedRequest(message: string) {
  clientError.value = message
  void focusErrors()
  return false
}

function continuePurchase() {
  if (!props.purchase.can_retry || busy.value) return
  clientError.value = ''
  refreshNotice.value = ''
  actionForm.clearErrors()
  actionForm.post(`/dashboard/purchases/${encodeURIComponent(props.purchase.id)}/continue`, {
    preserveScroll: true, onError: focusErrors,
    onNetworkError: () => failedRequest('We couldn’t confirm the request. Refresh the status before continuing the same purchase.'),
    onHttpException: () => failedRequest('This purchase could not be continued. Refresh the status and try again.'),
  })
}

async function showCancelConfirmation() {
  confirmCancel.value = true
  await nextTick()
  cancelConfirmation.value?.focus()
}
async function hideCancelConfirmation() {
  confirmCancel.value = false
  await nextTick()
  cancelButton.value?.focus()
}
function cancelSubscription() {
  if (!props.purchase.can_cancel || !confirmCancel.value || busy.value) return
  clientError.value = ''
  actionForm.clearErrors()
  actionForm.post(`/dashboard/purchases/${encodeURIComponent(props.purchase.id)}/cancel`, {
    preserveScroll: true,
    onError: focusErrors,
    onNetworkError: () => failedRequest('We couldn’t confirm cancellation. Refresh the status before trying again.'),
    onHttpException: () => failedRequest('Cancellation could not be confirmed. Refresh the status and try again.'),
    onSuccess: () => { confirmCancel.value = false },
  })
}

function paddleEvent(event: PaddleEvent) {
  if (!mounted) return
  if (['checkout.loaded', 'checkout.completed', 'checkout.closed', 'checkout.error'].includes(event.name ?? '')) {
    window.clearTimeout(checkoutLoadTimer)
  }
  if (event.name === 'checkout.completed' || event.name === 'checkout.closed') {
    checkoutOpen.value = false
    refreshStatus()
  } else if (event.name === 'checkout.error') {
    checkoutOpen.value = false
    clientError.value = 'Paddle couldn’t open this checkout. Refresh the status, then try opening checkout again.'
    void focusErrors()
  }
}

async function openPaddle() {
  const checkout = props.purchase.checkout
  if (checkout?.type !== 'paddle' || busy.value) return
  const purchaseId = props.purchase.id
  openingCheckout.value = true
  clientError.value = ''
  reloadRequired.value = false
  try {
    const paddle = await loadPaddle()
    if (!mounted || props.purchase.id !== purchaseId || props.purchase.checkout?.type !== 'paddle'
      || props.purchase.checkout.transaction_id !== checkout.transaction_id
      || props.purchase.checkout.client_token !== checkout.client_token
      || props.purchase.checkout.environment !== checkout.environment) return
    const config = `${checkout.environment}:${checkout.client_token}`
    if ((initializedPaddleConfig !== null && initializedPaddleConfig !== config)
      || (paddle.Initialized && initializedPaddleConfig === null)) {
      reloadRequired.value = true
      clientError.value = 'Checkout settings changed. Reload this page, then open checkout again.'
      await focusErrors()
      return
    }
    const eventCallback = (event: PaddleEvent) => {
      if (props.purchase.id === purchaseId) paddleEvent(event)
    }
    if (initializedPaddleConfig === null) {
      if (checkout.environment === 'sandbox') paddle.Environment.set('sandbox')
      paddle.Initialize({ token: checkout.client_token, eventCallback })
      initializedPaddleConfig = config
    } else {
      paddle.Update({ eventCallback })
    }
    openedPaddle = paddle
    checkoutOpen.value = true
    checkoutLoadTimer = window.setTimeout(() => {
      if (!mounted || props.purchase.id !== purchaseId) return
      checkoutOpen.value = false
      clientError.value = 'Paddle checkout is taking too long to open. Check your connection, refresh the status and try again.'
      void focusErrors()
    }, 15_000)
    paddle.Checkout.open({ transactionId: checkout.transaction_id })
  } catch {
    window.clearTimeout(checkoutLoadTimer)
    checkoutOpen.value = false
    clientError.value = 'Paddle checkout couldn’t load. Check your connection and try again.'
    await focusErrors()
  } finally {
    openingCheckout.value = false
  }
}

function reloadPage() { window.location.reload() }
watch(() => props.purchase.id, () => {
  window.clearTimeout(checkoutLoadTimer)
  if (checkoutOpen.value && openedPaddle) openedPaddle.Checkout.close()
  checkoutOpen.value = false
  confirmCancel.value = false
  clientError.value = ''
  refreshNotice.value = ''
  actionForm.clearErrors()
})
watch(() => props.purchase.can_cancel, allowed => { if (!allowed) confirmCancel.value = false })
onBeforeUnmount(() => {
  mounted = false
  window.clearTimeout(checkoutLoadTimer)
  if (checkoutOpen.value && openedPaddle) openedPaddle.Checkout.close()
})
</script>

<template>
  <Head title="Publishing payment" />
  <section class="directory-page publishing-purchase">
    <header class="directory-heading">
      <Link href="/dashboard/listings" class="text-link">Your listings</Link>
      <h1>Publishing payment</h1>
      <p>{{ purchase.listing_title }}</p>
    </header>
    <p v-if="purchase.mode === 'test'" class="notice" role="status"><strong>Test purchase.</strong> This payment cannot publish your listing in the public directory.</p>

    <section class="publishing-payment-status" aria-labelledby="purchase-status-heading" :aria-busy="refreshing" aria-live="polite">
      <p class="field-help">Current status</p>
      <h2 id="purchase-status-heading">{{ statusLabel }}</h2>
      <p>{{ stateExplanation }}</p>
      <p v-if="purchase.cancel_at !== null && purchase.state !== 'canceled'" class="publishing-cancel-note">Cancellation is scheduled. Your already-paid period is retained.</p>
      <p v-else-if="purchase.cancel_requested && purchase.state !== 'canceled'" class="publishing-cancel-note">Cancellation was requested. We are waiting for provider confirmation.</p>
    </section>

    <dl class="publishing-purchase-details">
      <div><dt>Plan</dt><dd>{{ purchase.plan_name }}</dd></div>
      <div><dt>Advertised price</dt><dd>{{ publishingPrice(purchase) }}</dd></div>
      <div><dt>Billing</dt><dd>{{ billingTypeLabel[purchase.billing_type] }}</dd></div>
      <div><dt>Payment</dt><dd>{{ paymentLabel }}</dd></div>
      <div><dt>Provider</dt><dd>{{ purchase.provider === 'free' ? 'No payment required' : purchase.provider === 'stripe' ? 'Stripe' : 'Paddle' }}</dd></div>
      <div v-if="paidThrough"><dt>Verified paid through</dt><dd><time :datetime="paidThrough.iso">{{ paidThrough.label }}</time></dd></div>
    </dl>
    <p class="publishing-note">{{ publishingTerms(purchase.billing_type) }} Provider checkout shows the final total, including tax and discounts.</p>
    <p class="publishing-note">Returning from checkout does not confirm payment. This page shows the status saved by the server.</p>

    <div v-if="hasErrors" ref="errorSummary" class="billing-errors" role="alert" tabindex="-1">
      <h2>Your payment needs attention</h2>
      <p v-if="purchase.error">{{ purchase.error }}</p>
      <p v-if="clientError">{{ clientError }}</p>
      <ul v-if="Object.keys(errors).length"><li v-for="(message, field) in errors" :key="field">{{ message }}</li></ul>
      <button v-if="reloadRequired" type="button" class="button button-secondary" @click="reloadPage">Reload page</button>
    </div>

    <div class="directory-actions publishing-checkout-actions">
      <a v-if="stripeUrl" :href="stripeUrl" class="button button-primary">Open Stripe checkout</a>
      <button v-if="purchase.checkout?.type === 'paddle'" type="button" class="button button-primary" :disabled="busy || reloadRequired" @click="openPaddle">{{ openingCheckout ? 'Loading Paddle checkout…' : checkoutOpen ? 'Paddle checkout is open' : 'Open Paddle checkout' }}</button>
      <form v-if="purchase.can_retry" @submit.prevent="continuePurchase">
        <button type="submit" class="button" :class="purchase.checkout ? 'button-secondary' : 'button-primary'" :disabled="busy">{{ actionForm.processing ? 'Checking purchase…' : purchase.checkout ? 'Check existing checkout' : 'Continue purchase' }}</button>
      </form>
      <button type="button" class="button button-secondary" :disabled="busy" @click="refreshStatus">{{ refreshing ? 'Refreshing status…' : 'Refresh status' }}</button>
      <Link :href="`/dashboard/listings/${purchase.listing_id}/edit`" class="text-link">View your listing</Link>
    </div>
    <p v-if="purchase.checkout?.type === 'stripe' && !stripeUrl" class="form-error" role="alert">The checkout link is unavailable. Refresh the status or contact the directory administrator.</p>
    <p v-if="refreshNotice" class="publishing-refresh-notice" role="status">{{ refreshNotice }}</p>

    <section v-if="purchase.can_cancel" class="directory-review-action" aria-labelledby="cancel-heading">
      <h2 id="cancel-heading">Subscription renewal</h2>
      <p>Cancel future renewals and retain your already-paid publishing period.</p>
      <button v-if="!confirmCancel" ref="cancelButton" type="button" class="button button-secondary" :disabled="busy" aria-controls="cancel-confirmation" :aria-expanded="false" @click="showCancelConfirmation">Cancel renewal</button>
      <div v-else id="cancel-confirmation" ref="cancelConfirmation" class="publishing-cancel-confirmation" tabindex="-1" role="group" aria-labelledby="cancel-confirmation-heading" @keydown.esc.prevent="hideCancelConfirmation">
        <h3 id="cancel-confirmation-heading">Cancel future renewals?</h3>
        <p>Your already-paid period is retained. This does not request a refund.</p>
        <form class="directory-actions" @submit.prevent="cancelSubscription">
          <button type="submit" class="button button-primary" :disabled="busy">{{ actionForm.processing ? 'Requesting cancellation…' : 'Confirm cancellation' }}</button>
          <button type="button" class="button button-secondary" :disabled="actionForm.processing" @click="hideCancelConfirmation">Keep renewal</button>
        </form>
      </div>
    </section>
    <OwnerNotifications :notifications="notifications" />
  </section>
</template>
