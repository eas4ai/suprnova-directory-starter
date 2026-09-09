export type Account = {
  name: string
  email: string
  verified: boolean
  can_admin: boolean
}

export type SharedProps = { auth: { user: Account | null } }
