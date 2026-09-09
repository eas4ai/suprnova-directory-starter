<script setup lang="ts">
import { Head, Link } from '@inertiajs/vue3'
import type { OwnerListing, Paged } from '../../types/listings'
defineProps<{ listings: OwnerListing[]; pagination: Paged }>()
</script>
<template>
  <Head title="Listing review queue" />
  <section class="directory-page"><header class="directory-heading"><h1>Listing review queue</h1><p>Review the exact revision an owner submitted. Approval does not grant payment eligibility.</p></header>
    <div v-if="!listings.length" class="directory-empty"><h2>No listings awaiting review</h2><p>Submitted revisions will appear here.</p></div>
    <ul v-else class="directory-owner-list"><li v-for="listing in listings" :key="listing.id" class="directory-owner-row"><h2><Link :href="`/admin/listings/${listing.id}`">{{ listing.current.title }}</Link></h2><p>{{ listing.current.summary }}</p><div class="directory-toolbar"><span>{{ listing.approved ? 'Changes to an approved listing' : 'First submission' }} · Revision {{ listing.current.id }}</span><Link :href="`/admin/listings/${listing.id}`" class="button button-secondary">Review listing</Link></div></li></ul>
    <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Review queue pages"><Link v-if="pagination.page > 1" :href="`/admin/listings?page=${pagination.page - 1}`" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="`/admin/listings?page=${pagination.page + 1}`" class="button button-secondary">Next</Link></nav>
  </section>
</template>
