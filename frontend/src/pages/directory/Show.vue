<script setup lang="ts">
import { Link } from '@inertiajs/vue3'
import PublicMetadata from '../../components/PublicMetadata.vue'
import type { Seo } from '../../types/articles'
import type { PublicDetail } from '../../types/listings'
defineProps<{ listing: PublicDetail; seo: Seo }>()
</script>
<template>
  <PublicMetadata :seo="seo" />
  <article class="directory-page directory-detail">
    <Link href="/listings" class="text-link">Back to directory</Link>
    <header class="directory-heading"><h1>{{ listing.title }}</h1><p>{{ listing.summary }}</p><ul class="directory-tags" aria-label="Categories"><li v-for="category in listing.categories" :key="category.id"><Link :href="`/listings?category=${encodeURIComponent(category.slug)}`">{{ category.name }}</Link></li></ul></header>
    <img v-if="listing.media_url" :src="listing.media_url" :alt="listing.media_alt" class="directory-detail-image" />
    <!-- description_html is Markdown rendered and sanitized by the server. -->
    <div class="prose directory-description" v-html="listing.description_html" />
    <a :href="listing.url" target="_blank" rel="noopener noreferrer" class="button button-primary">Visit website <span class="sr-only">(opens in a new tab)</span></a>
  </article>
</template>
