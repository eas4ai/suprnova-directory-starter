export type Account = {
  name: string
  email: string
  verified: boolean
  can_admin: boolean
  can_billing: boolean
  can_moderate: boolean
  can_edit: boolean
  can_taxonomy: boolean
  can_accounts: boolean
  can_audit: boolean
  can_seo: boolean
}

export type Site = { name: string; description: string; origin: string; logo_url: string | null; accent: string; theme: string }
export type SharedProps = { auth: { user: Account | null }; site: Site; appearance: 'light' | 'dark' }
