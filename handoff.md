# Suprnova directory starter handoff

Updated: 2026-09-08
Working repository: `/home/shawn/workspace2/suprnova-directory-starter`
Origin: https://github.com/eas4ai/suprnova-directory-starter.git

## Purpose and current stage

Build a free, MIT-licensed, Suprnova-specific directory starter. The developer owns Suprnova and wants a strong reusable product, informed by a purchased and subsequently abandoned Laravel directory kit. We have established a feature baseline and inspected the candidate foundations. We have not begun the application implementation beyond the explicitly requested framework version pin.

Continue here rather than in the Laravel reference checkout. Do not restart discovery or ask the developer to repeat the settled decisions. Read the records below, verify current Git state, then turn the baseline into concrete requirements, acceptance mechanisms and small commitments.

## Read in this order

1. `docs/feature-map.md` — expanded product baseline and proposed acceptance checks.
2. `docs/architecture-proposal.md` — module responsibilities, lifecycle and billing proposals, framework integration.
3. `docs/pulsar-assessment.md` — reuse evidence, differences to resolve, confirmed framework RBAC use, selected UI.
4. `docs/recon.md` — initial target state, inspection history and verification limits.
5. `../larafast-directories-master/docs/recon.md` — source-kit observations and unresolved findings. It is orientation, not a complete inventory or a new-product contract.

The docs are Draft/Observed. The developer accepted the general baseline, but has not confirmed individual normative requirements and falsifiers. Do not label all proposed behavior Agreed merely because the baseline was accepted.

## Settled developer direction

- Free MIT starter, including operator monetization: free source does not mean directory operators cannot charge for listings.
- First delivery includes listings, categories, bounded search, owner submissions/dashboard, moderation and paid publishing. Baseline also includes blog articles, taxonomy, RSS, SEO, our administration, accounts, notifications, storage, demo data, documentation and verification.
- Use Suprnova's authentication, payment provider module and RBAC. Application code supplies directory rules, permission names, plans and publication entitlements. Do not build parallel auth, payment or RBAC engines.
- Use Suprnova **v1.3.7** now. Developer says the CLI may lag. This pin is already changed in Cargo.toml.
- Developer says 2.0 largely preserves the API and adds Suprnova Live, RenderCache and many bug fixes. Consult development source for fixes before introducing application workarounds. Do not silently depend on unreleased code or assume every development API exists in 1.3.7.
- Evaluate/reuse **Pulsar** as the application-feature foundation. No foundation replacement or wholesale import has happened.
- UI is **Vue + Inertia + Vuetify 0**, following suprnova.app. Exact reference dependency: `@vuetify/v0` version `1.0.1`. This is distinct from Pulsar's `vuetify` 3 package.
- Separate theme from component behavior: shared semantic tokens for color, typography, spacing and branding; composable UI components; v0 interaction primitives. Apply the customization contract to both public and administration screens. The reference site's branding/dark-only theme is not automatically selected.
- Future AI features gated by self-hosted Keygen are **brainstorming only**. No AI implementation, Keygen deployment, activation flow, license-server dependency or speculative licensing system belongs in current scope.
- Durable documentation is a product/process requirement for this solo developer and future assistants. Record decisions, evidence, unresolved questions and verification honestly.

## Relevant repositories and inspected facts

| Repository | Role and useful evidence |
| --- | --- |
| `../suprnova` | Framework development source. Read its own AGENTS.md before work there. At inspection: `v1.3.7-613-gac5d1756`. Auth provider contract: `framework/src/auth/provider.rs:71`; payment contract: `framework/src/payments/traits/mod.rs:31`; payment registry: `framework/src/payments/registry.rs:75`. Manual includes Live and RenderCache. |
| `../Pulsar` | MIT application foundation candidate. At inspection: clean main, commit `222721a`, framework pin v1.3.1, Vue/Vuetify 3. Includes account flows, articles, RSS, taxonomy, admin surfaces and related tests. |
| `../suprnova.app` | Selected UI implementation reference. `frontend/package.json:18` pins v0; public/console layouts import its Dialog primitive. Custom components live under `frontend/src/components/ds`; styles are ordered from `frontend/src/main.ts`. Read its AGENTS.md when working there. |
| `../larafast-directories-master` | Purchased PHP feature reference. No Git repository at inspection; dependencies absent. Keep its implementation observations distinct from intended new behavior. No PHP has been copied into the MIT starter. |

Pulsar RBAC was explicitly rechecked after the developer questioned its age: `src/commands/users_promote.rs:4` imports suprnova::rbac helpers, `src/migrations/mod.rs:2` imports CreateRbacTables, the user implements HasRoles, and routes use PermissionMiddleware. Pulsar defines its own role/permission assignments on top; adapt these instead of inheriting unrelated community grants.

Foundation integration has real differences: Pulsar uses app_users and initializes Suprnova Magnetar; the target scaffold uses users. Pulsar combines HTTP and shared bootstrap; the target separates them. Preserve current framework conventions and select one consistent auth schema when integrating. Retain/adapt relevant tests rather than copying only features.

## Open decisions and proposals

- Provider scope selected on 2026-09-08: support both Stripe and Paddle through starter administration and configuration, using Suprnova adapters. Lemon Squeezy is not a shipped Suprnova adapter. The earlier one-provider proposal is superseded.
- Next iteration: create a NOWPayments adapter for the next Suprnova release, with Nation X as implementation evidence. See `docs/commitments/nowpayments-framework-adapter.md`. Acceptance details remain Draft; this is separate from current starter implementation.
- Review before payment was selected by the developer on 2026-09-08: submit, approve, then checkout. See `docs/decisions/review-listings-before-payment.md`. Detailed billing requirements and falsifiers remain unconfirmed.
- One-time and recurring plans are proposed; exact entitlement duration, failed-renewal handling, cancellation, full/partial refunds, disputes and reinstatement need explicit rules.
- Recommended architecture: one application/database with focused modules; moderation and billing eligibility separate; approved revisions remain public while edits await review; idempotent fulfillment and reconciliation. These are proposals awaiting requirement-level review.
- Exact foundational integration sequence and release timing remain to be set. The developer was waiting for the framework development release before updating Pulsar/Nebula; do not infer authorization to upgrade those sibling projects.

## Actual changes and checks

At latest status, main tracks origin/main. Last verified base was `a49cb8e841c6124381e12e7dea89ac544cee0916` (`first commit`). Recheck before acting because the developer has committed during the conversation.

- Modified: Cargo.toml, Suprnova tag v1.3.5 → v1.3.7.
- New, uncommitted: docs/recon.md, docs/feature-map.md, docs/architecture-proposal.md, docs/pulsar-assessment.md, and this handoff.
- No implementation imports, frontend migration, new application features, commits or pushes by the assistant.
- Existing .env was not read or modified. Do not expose it.
- Passed: manifest TOML parsing; `cargo metadata --no-deps --format-version 1` confirmed the v1.3.7 dependency source; `git diff --check`; documentation path/line-bound and scope consistency checks. Citation bounds do not establish behavior.
- No target compilation, dependency resolution, runtime tests, browser tests, migrations or seeds ran. No target lockfiles or installed frontend dependencies existed at initial inspection.
- No Pulsar tests ran; their presence is not a passing result.
- In the PHP reference, `php artisan test` failed before tests due to missing vendor/autoload.php; frontend build failed because Vite was absent.
- `cairn wake` reported no docs/spec/roadmap.md. No glossary, normative spec set, mechanisms or commitment exists yet. scripts/spec-lint.mjs is absent; no spec-lint pass has been established.

## Operating rules and next move

The session began with the existing-project skill; continue the adoption/specification workflow at the selected feature baseline. Read `/home/shawn/.agents/skills/existing-project/SKILL.md` and its referenced new-project standing rules/template when preparing the specs. Observed is not Agreed; agreement includes falsifiers and named mechanisms. Do not install the working agreement midway and claim a functioning Cairn loop without its required records.

The machine-wide user instructions import `/home/shawn/.codex/RTK.md`, `/home/shawn/.codex/TILTH.md`, `/home/shawn/.codex/PARTNERSHIP.md` and `/home/shawn/.claude/BEST_PRACTICES.md`. A repository-local BEST_PRACTICES.md overrides the shared production rules. None was found in the target at inspection. Keep exactly one todo in progress and verify before completion claims. Prefer codebase-memory graph discovery; target, Pulsar, suprnova and suprnova.app are indexed, but stale line metadata was observed, so narrow source inspection may be needed. Use rtk-prefixed shell commands. Preserve concurrent user changes.

Current todo:

- Complete: inspect source/target/foundations and document the baseline with verification limits.
- Complete: record selected framework and UI, future-only ideas, and prepare this handoff.
- In progress: turn the baseline into requirements and acceptance checks, resolving the outstanding billing decisions.
- Pending: agree the relevant requirements and prepare the first verifiable commitment, then implement within its scope.

Suggested opening request in the new session: “Read handoff.md and the linked project records. Continue defining the Suprnova directory starter requirements and first commitment from the agreed feature baseline.”

Session update (2026-09-08): review-before-payment and Stripe/Paddle support are recorded developer decisions. NOWPayments is recorded as a planned next-iteration commitment. No active roadmap or Agreed normative requirement set exists yet. No adapter or starter application implementation was performed during these decisions.

## Cairn adoption update

The developer requested Cairn establishment on 2026-09-08. AGENTS.md now contains the working agreement verbatim. docs/spec contains the Draft overview, glossary, foundation, payment scope and roadmap; Current is starter-foundation. Five foundation mechanism declarations exist; scripts/verify-foundation.mjs is not implemented yet. scripts/spec-lint.mjs delegates to the installed Cairn checkout. Spec lint and its safe violation/correction demonstration passed. No application checks have run. Requirement/falsifier agreement is pending; do not promote these Draft blocks based on earlier general scope approval.

Current todo:
- Complete: prepare Cairn records and verify spec structure.
- In progress: obtain confirmation of FND-001 through FND-005 and their falsifiers.
- Pending: implement and demonstrate the foundation mechanisms, then execute the Cairn loop.

Foundation execution update: the developer confirmed FND-001 through FND-005 and their falsifiers. The agreement escalation is answered and committed. Current action is FND-001 implementation. Cargo.lock and frontend/bun.lock are generated; the disposable build verifier is being implemented. The initial Cairn receipt is unverified because that verifier did not exist at baseline, not a reproduced application defect. Account, database and UI mechanisms remain to be implemented.
