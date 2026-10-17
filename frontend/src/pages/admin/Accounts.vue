<script setup lang="ts">
import { ref, watch } from 'vue'
import { Head, Link } from '@inertiajs/vue3'
import type { Pagination } from '../../types/articles'
type AccountRow = { id: number; name: string; email: string; verified: boolean; suspended: boolean; version: number; roles: string[] }
const props = defineProps<{ accounts: AccountRow[]; q: string; pagination: Pagination }>()
const query = ref(props.q)
watch(() => props.q, value => { query.value = value })
const pageUrl = (page: number) => '/admin/accounts?' + new URLSearchParams({ q: props.q, page: String(page), per_page: String(props.pagination.per_page) })
</script>
<template>
  <Head title="Accounts" />
  <section class="directory-page">
    <header class="directory-heading"><h1>Accounts</h1><p>Review email verification, manage administrative roles and suspend account access.</p></header>
    <form action="/admin/accounts" method="get" role="search" class="directory-search"><div><label for="account-query" class="form-label">Search accounts</label><input id="account-query" v-model="query" name="q" type="search" maxlength="200" class="form-input" /></div><button type="submit" class="button button-secondary">Search</button><Link v-if="q" href="/admin/accounts" class="text-link">Reset</Link></form>
    <p role="status">{{ pagination.total }} {{ pagination.total === 1 ? 'account' : 'accounts' }}</p>
    <ul v-if="accounts.length" class="editorial-admin-list"><li v-for="account in accounts" :key="account.id"><div><h2><Link :href="'/admin/accounts/' + account.id" class="text-link">{{ account.name || account.email }}</Link></h2><p>{{ account.email }}</p><p>{{ account.roles.length ? account.roles.join(', ') : 'No administrative roles' }}</p></div><div><p>{{ account.verified ? 'Email verified' : 'Email not verified' }}</p><p>{{ account.suspended ? 'Suspended' : 'Account active' }}</p></div></li></ul>
    <div v-else class="directory-empty"><h2>{{ q ? 'No accounts match your search' : 'No accounts to display' }}</h2><p v-if="q">Try another name or email address.</p><Link v-if="q" href="/admin/accounts" class="text-link">Show all accounts</Link></div>
    <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Account pages"><Link v-if="pagination.page > 1" :href="pageUrl(pagination.page - 1)" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }} of {{ Math.ceil(pagination.total / pagination.per_page) }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="pageUrl(pagination.page + 1)" class="button button-secondary">Next</Link></nav>
  </section>
</template>
