export type BillingMode = 'test' | 'live'
export type BillingProvider = 'stripe' | 'paddle'

export type BillingProfile = {
  enabled: boolean
  public_key: string
  has_secrets: boolean
}

export type BillingSettings = {
  mode: BillingMode
  revision: number
  default_provider: BillingProvider | null
  stripe: BillingProfile
  paddle: BillingProfile
  mappings: { plan: string; stripe: string | null; paddle: string | null }[]
}

export type ProfileInput = {
  enabled: boolean
  public_key: string
  api_key: string
  webhook_key: string
  clear_secrets: boolean
}
