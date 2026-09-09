export type PublishingBillingType = 'free' | 'one_time' | 'monthly' | 'annual'
export type PublishingProvider = 'stripe' | 'paddle'

export interface PublishingPlan {
  key: string
  name: string
  description: string
  enabled: boolean
  billing_type: PublishingBillingType
  amount: number
  currency: string
  version: number
}

export interface PublishingOffer {
  key: string
  name: string
  description: string
  billing_type: PublishingBillingType
  amount: number
  currency: string
  providers: { id: PublishingProvider; label: string; default: boolean }[]
}

export type PublishingCheckout =
  | { type: 'stripe'; url: string }
  | { type: 'paddle'; transaction_id: string; client_token: string; environment: 'sandbox' | 'production' }

export interface PublishingPurchase {
  id: string
  listing_id: number
  listing_title: string
  plan_name: string
  amount: number
  currency: string
  billing_type: PublishingBillingType
  provider: 'free' | PublishingProvider
  mode: 'free' | 'test' | 'live'
  state: string
  error: string | null
  payment_status: string
  paid_through: number | null
  cancel_requested: boolean
  cancel_at: number | null
  can_cancel: boolean
  can_retry: boolean
  checkout: PublishingCheckout | null
}

export const billingTypeLabel: Record<PublishingBillingType, string> = {
  free: 'Free', one_time: 'One-time payment', monthly: 'Monthly', annual: 'Annual',
}

export function publishingAmount(amount: number, currency: string): string {
  try {
    const formatter = new Intl.NumberFormat('en-US', { style: 'currency', currency, currencyDisplay: 'code' })
    const decimals = formatter.resolvedOptions().maximumFractionDigits ?? 2
    return formatter.format(amount / 10 ** decimals)
  } catch {
    return `${amount} minor units (${currency})`
  }
}

export function publishingPrice(plan: Pick<PublishingPlan, 'billing_type' | 'amount' | 'currency'>): string {
  if (plan.billing_type === 'free') return 'Free'
  const amount = publishingAmount(plan.amount, plan.currency)
  if (plan.billing_type === 'monthly') return `${amount} / month`
  if (plan.billing_type === 'annual') return `${amount} / year`
  return `${amount} once`
}

export function publishingTerms(type: PublishingBillingType): string {
  if (type === 'free') return 'No payment required. Publication remains subject to listing approval.'
  if (type === 'one_time') return 'One payment with no expiry date. Approval, refund and dispute rules still apply.'
  return 'Renews until canceled. Publication continues through the last verified paid period.'
}
