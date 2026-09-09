# Delegated starter administration

Status: Agreed 2026-09-09
Prefix: ADM

[ADM-001] The starter MUST enforce distinct Suprnova permissions for listing moderation, editorial work, taxonomy, account administration and billing.
Falsifier: Possession of one capability grants an unrelated administrative action, a hidden link is the only denial, or an unprivileged account can grant itself permissions.
Mechanism: Proposed administration permission matrix across direct HTTP requests and navigation.

[ADM-002] The starter MUST let an authorized account administrator manage account access without bypassing verification or losing operator recovery.
Falsifier: An unauthorized actor changes another account, suspension leaves protected mutations available in an existing session, a role edit bypasses email verification, or the documented host command cannot recover administrator access.
Mechanism: Proposed account-administration HTTP, existing-session revocation and operator-command tests.

[ADM-003] The starter MUST record protected administrative decisions with their actor, target, time and safe change summary.
Falsifier: A successful moderation, account-access, taxonomy, editorial-publication or billing-plan change lacks an attributable record, a failed transaction records a successful decision, or audit output exposes passwords or provider secrets.
Mechanism: Proposed administrative-audit transaction and secret-marker tests with permission-restricted audit pages.

## Agreed operating rules

- Seed explicit roles for administrator, moderator and editor through Suprnova RBAC.
  Role names themselves grant nothing. Operator provisioning grants the full starter
  administrator bundle; revocation removes it and memberships that confer it.
- Account administration supports bounded search, account details, suspension and
  reinstatement, and assignment of the predefined roles. It does not expose passwords,
  impersonation, arbitrary permission creation or bulk hard deletion.
- Suspension blocks protected actions and new sign-in and hides the account's public
  listings. Reinstatement does not override listing moderation or payment eligibility.
  Prevent accidental removal of the last active verified administrator in the UI;
  the host recovery command remains available to the operator.
- Keep administrative audit pages permission-protected and paginated. Audit summaries
  identify changed field names and relevant public identifiers, never secret values.
