<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../../types/shared'
type RecentActivity = { id: number; actor: string; action: string; summary: string; target_type: string; target_id: string; created_at: number }
defineProps<{
  listing_summary: { published: number; awaiting_review: number; total: number } | null
  article_summary: { published: number; total: number } | null
  recent_activity: RecentActivity[] | null
}>()
const page = usePage<SharedProps>()
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString()
const displayDate = (timestamp: number) => date(timestamp).replace('T', ' ').replace('.000Z', ' UTC')
</script>
<template>
  <Head title="Administration" />
  <section class="admin-overview">
    <h1>Administration</h1>
    <p class="lead">Welcome, {{ page.props.auth.user?.name }}. Review publication and the work you manage.</p>
    <div class="workspace-summary"><p>Current directory status</p><Link href="/listings" class="button button-primary">Open the directory</Link></div>
    <dl v-if="listing_summary || article_summary" class="overview-metrics">
      <div v-if="listing_summary" class="overview-metric" data-metric="published-listings">
        <dt>Published listings</dt><dd class="overview-count">{{ listing_summary.published }}</dd>
        <dd><p v-if="listing_summary.published === 0">No listings are public right now.</p>
        <p v-else>Approved and eligible for publication right now.</p>
        <p class="muted">{{ listing_summary.total }} listings in the workspace, excluding archived listings.</p>
        <Link href="/listings" class="text-link">View published listings</Link></dd>
      </div>
      <div v-if="listing_summary" class="overview-metric" data-metric="pending-reviews">
        <dt>Awaiting review</dt><dd class="overview-count">{{ listing_summary.awaiting_review }}</dd>
        <dd><p v-if="listing_summary.awaiting_review === 0">No listings are waiting for review.</p>
        <p v-else>New submissions and proposed changes need a decision.</p>
        <p class="muted">Proposed changes can belong to a listing that is already published.</p>
        <Link href="/admin/listings" class="text-link">Review pending listings</Link></dd>
      </div>
      <div v-if="article_summary" class="overview-metric" data-metric="published-articles">
        <dt>Published articles</dt><dd class="overview-count">{{ article_summary.published }}</dd>
        <dd><p v-if="article_summary.published === 0">No articles are public right now.</p>
        <p v-else>Published revisions are visible to readers.</p>
        <p class="muted">{{ article_summary.total }} articles saved. Unpublished edits do not replace the public revision.</p>
        <Link href="/admin/articles?state=published" class="text-link">View published articles</Link></dd>
      </div>
    </dl>
    <section v-if="recent_activity" class="overview-activity" aria-labelledby="overview-activity-heading">
      <div class="section-heading"><h2 id="overview-activity-heading">Recent activity</h2><Link href="/admin/audit" class="text-link">View all activity</Link></div>
      <ol v-if="recent_activity.length">
        <li v-for="entry in recent_activity" :key="entry.id">
          <div><h3>{{ entry.action.replace(/_/g, ' ') }}</h3><p>{{ entry.summary }}</p><p class="muted">{{ entry.actor }} · {{ entry.target_type }} {{ entry.target_id }}</p></div>
          <time :datetime="date(entry.created_at)">{{ displayDate(entry.created_at) }}</time>
        </li>
      </ol>
      <p v-else class="overview-empty">No recorded activity yet.</p>
    </section>
    <section v-if="!listing_summary && !article_summary && !recent_activity" class="workspace-empty">
      <h2>Your workspace</h2><p>Use the navigation to open the areas you manage.</p>
    </section>
  </section>
</template>

<style scoped>
.overview-metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 230px), 1fr)); gap: var(--space-6); margin: 0 0 var(--space-12); }
.overview-metric { padding: var(--space-6); border: 1px solid var(--border-default); border-radius: var(--radius-panel); background: var(--surface-muted); min-width: 0; }
.overview-metric dt { color: var(--text-secondary); font-weight: 600; }
.overview-count { font-size: 2.5rem; line-height: 1.2; font-weight: 600; margin-block: var(--space-3) var(--space-6); font-variant-numeric: tabular-nums; }
.overview-metric p { font-size: var(--text-small); margin-top: var(--space-3); overflow-wrap: anywhere; }
.overview-metric .text-link { margin-top: var(--space-4); }
.overview-activity ol { list-style: none; padding: 0; margin: 0; }
.overview-activity li { display: flex; justify-content: space-between; gap: var(--space-6); padding-block: var(--space-6); border-bottom: 1px solid var(--border-default); overflow-wrap: anywhere; }
.overview-activity h3 { font-size: 1rem; text-transform: capitalize; }
.overview-activity p { margin-top: var(--space-2); font-size: var(--text-small); }
.overview-activity time { color: var(--text-secondary); font-size: var(--text-small); flex-shrink: 0; }
.overview-empty { padding-block: var(--space-6); color: var(--text-secondary); }
@media (max-width: 600px) { .overview-activity li { flex-direction: column; gap: var(--space-3); } }
</style>
