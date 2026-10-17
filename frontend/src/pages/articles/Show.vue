<script setup lang="ts">
import { Link } from '@inertiajs/vue3'
import PublicMetadata from '../../components/PublicMetadata.vue'
import type { PublicArticleDetail, Seo, Term } from '../../types/articles'
defineProps<{ article: PublicArticleDetail; seo: Seo }>()
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString().slice(0, 10)
const termUrl = (term: Term) => '/articles?' + new URLSearchParams({ [term.kind]: term.slug })
</script>
<template>
  <PublicMetadata :seo="seo" />
  <section class="directory-page editorial-reading"><Link href="/articles" class="text-link">All articles</Link><article><header class="directory-heading"><h1>{{ article.title }}</h1><p class="editorial-summary">{{ article.summary }}</p><p class="editorial-date"><time :datetime="date(article.published_at)">Published {{ date(article.published_at) }}</time><span v-if="article.modified_at !== article.published_at"> · Updated {{ date(article.modified_at) }}</span></p><ul class="directory-tags"><li v-for="term in article.terms" :key="term.id"><Link :href="termUrl(term)">{{ term.name }}</Link></li></ul></header><img v-if="article.media_url" :src="article.media_url" :alt="article.media_alt" class="editorial-cover" /><div class="editorial-prose" v-html="article.body_html" /></article></section>
</template>
