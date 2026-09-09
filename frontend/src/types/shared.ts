export type Account = {
  name: string
  email: string
  verified: boolean
  can_admin: boolean
  can_billing: boolean
}

export type SharedProps = { auth: { user: Account | null } }
