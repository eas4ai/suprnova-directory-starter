<script setup lang="ts">
import { ref, watch } from 'vue'
import { Link } from '@inertiajs/vue3'
import PublicMetadata from '../../components/PublicMetadata.vue'
import type { Seo } from '../../types/articles'
import type { Category, Paged, PublicCard } from '../../types/listings'
const props = defineProps<{ heading?: string; listings: PublicCard[]; categories: Category[]; q: string; category: string; pagination: Paged; seo: Seo }>()
const query = ref(props.q)
const selected = ref(props.category)
const compact = ref(false)
watch(() => [props.q, props.category], () => { query.value = props.q; selected.value = props.category })
const resultsUrl = (category: string, page = 1) => `/listings?${new URLSearchParams({ q: props.q, category, page: String(page) })}`
</script>
<template>
  <PublicMetadata :seo="seo" />
  <section class="directory-page">
    <header class="directory-heading"><h1>{{ heading ?? 'Explore the directory' }}</h1><p>Find a listing by title, summary or category.</p></header>
    <form action="/listings" method="get" role="search" class="directory-search">
      <div><label for="directory-query" class="form-label">Search listings</label><input id="directory-query" v-model="query" name="q" type="search" maxlength="200" class="form-input" /></div>
      <div class="directory-mobile-category"><label for="directory-category" class="form-label">Category</label><select id="directory-category" v-model="selected" name="category" class="form-input"><option value="">All categories</option><option v-for="item in categories" :key="item.id" :value="item.slug">{{ item.name }}</option></select></div>
      <button class="button button-primary" type="submit">Search</button>
    </form>
    <div class="directory-layout">
      <nav class="directory-categories" aria-label="Categories"><h2>Categories</h2><Link :href="resultsUrl('')" class="nav-link" :aria-current="!category ? 'page' : undefined">All categories</Link><Link v-for="item in categories" :key="item.id" :href="resultsUrl(item.slug)" class="nav-link" :aria-current="category === item.slug ? 'page' : undefined">{{ item.name }}</Link></nav>
      <div class="directory-results">
        <div class="directory-toolbar"><p role="status">{{ pagination.total }} {{ pagination.total === 1 ? 'listing' : 'listings' }}</p><div class="directory-actions" role="group" aria-label="Result layout"><button type="button" class="button button-quiet" :aria-pressed="!compact" @click="compact = false">Grid</button><button type="button" class="button button-quiet" :aria-pressed="compact" @click="compact = true">Compact</button></div></div>
        <div v-if="!listings.length" class="directory-empty"><h2>{{ q || category ? 'No matching listings' : 'The directory is getting started' }}</h2><p>{{ q || category ? 'Try another search or browse all categories.' : 'Approved, eligible listings will appear here.' }}</p><Link v-if="q || category" href="/listings" class="text-link">Clear filters</Link><Link v-else href="/dashboard/listings/create" class="text-link">Create a listing</Link></div>
        <ul v-else class="directory-grid" :class="{ 'directory-compact': compact }"><li v-for="listing in listings" :key="listing.id" class="directory-card"><img v-if="listing.media_url" :src="listing.media_url" :alt="listing.media_alt" loading="lazy" class="directory-card-image" /><div class="directory-card-body"><h2><Link :href="`/listings/${listing.slug}`">{{ listing.title }}</Link></h2><p>{{ listing.summary }}</p><ul class="directory-tags" aria-label="Listing categories"><li v-for="item in listing.categories" :key="item.id">{{ item.name }}</li></ul></div></li></ul>
        <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Results pages"><Link v-if="pagination.page > 1" :href="resultsUrl(category, pagination.page - 1)" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="resultsUrl(category, pagination.page + 1)" class="button button-secondary">Next</Link></nav>
      </div>
    </div>
  </section>
</template>
