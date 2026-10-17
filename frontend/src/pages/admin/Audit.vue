<script setup lang="ts">
import { Head, Link } from '@inertiajs/vue3'
import type { Pagination } from '../../types/articles'
type AuditEntry = { id: number; actor: string; actor_id: number; actor_type: string; target_type: string; target_id: number | string; action: string; summary: string; created_at: number }
const props = defineProps<{ entries: AuditEntry[]; pagination: Pagination }>()
const pageUrl = (page: number) => '/admin/audit?' + new URLSearchParams({ page: String(page), per_page: String(props.pagination.per_page) })
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString()
const displayDate = (timestamp: number) => date(timestamp).replace('T', ' ').replace('.000Z', ' UTC')
</script>
<template>
  <Head title="Audit history" />
  <section class="directory-page"><header class="directory-heading"><h1>Audit history</h1><p>Recorded administrative actions, including who acted and what changed.</p></header><p role="status">{{ pagination.total }} {{ pagination.total === 1 ? 'entry' : 'entries' }}</p>
    <ol v-if="entries.length" class="editorial-admin-list"><li v-for="entry in entries" :key="entry.id"><div><h2>{{ entry.action }}</h2><p>{{ entry.summary }}</p><p>{{ entry.actor || entry.actor_type }} · {{ entry.actor_type }} {{ entry.actor_id }}</p><p>Target: {{ entry.target_type }} {{ entry.target_id }}</p></div><time :datetime="date(entry.created_at)">{{ displayDate(entry.created_at) }}</time></li></ol>
    <div v-else class="directory-empty"><h2>No recorded actions yet</h2><p>Administrative changes will appear here.</p></div>
    <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Audit pages"><Link v-if="pagination.page > 1" :href="pageUrl(pagination.page - 1)" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }} of {{ Math.ceil(pagination.total / pagination.per_page) }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="pageUrl(pagination.page + 1)" class="button button-secondary">Next</Link></nav>
  </section>
</template>
