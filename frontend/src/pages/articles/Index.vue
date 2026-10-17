<script setup lang="ts">
import { ref, watch } from 'vue'
import { Link } from '@inertiajs/vue3'
import PublicMetadata from '../../components/PublicMetadata.vue'
import type { PublicArticle, Term, Pagination, Seo } from '../../types/articles'
const props = defineProps<{ articles: PublicArticle[]; terms: Term[]; pagination: Pagination; q: string; category: string; tag: string; seo: Seo }>()
const query = ref(props.q), selectedCategory = ref(props.category), selectedTag = ref(props.tag)
watch(() => [props.q, props.category, props.tag], () => { query.value = props.q; selectedCategory.value = props.category; selectedTag.value = props.tag })
const pageUrl = (page: number) => '/articles?' + new URLSearchParams({ q: props.q, category: props.category, tag: props.tag, page: String(page) })
const termUrl = (term: Term) => '/articles?' + new URLSearchParams({ [term.kind]: term.slug })
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString().slice(0, 10)
</script>
<template>
  <PublicMetadata :seo="seo" />
  <section class="directory-page"><header class="directory-heading"><h1>Articles</h1><p>Guides, ideas and updates from the directory.</p></header>
    <form action="/articles" method="get" role="search" class="directory-search"><div><label for="public-article-query" class="form-label">Search articles</label><input id="public-article-query" v-model="query" name="q" type="search" maxlength="200" class="form-input" /></div><div><label for="public-article-category" class="form-label">Category</label><select id="public-article-category" v-model="selectedCategory" name="category" class="form-input"><option value="">All categories</option><option v-for="term in terms.filter(t => t.kind === 'category')" :key="term.id" :value="term.slug">{{ term.name }}</option></select></div><div><label for="public-article-tag" class="form-label">Tag</label><select id="public-article-tag" v-model="selectedTag" name="tag" class="form-input"><option value="">All tags</option><option v-for="term in terms.filter(t => t.kind === 'tag')" :key="term.id" :value="term.slug">{{ term.name }}</option></select></div><button type="submit" class="button button-primary">Search</button><Link v-if="q || category || tag" href="/articles" class="text-link">Reset</Link></form>
    <p role="status">{{ pagination.total }} {{ pagination.total === 1 ? 'article' : 'articles' }}</p>
    <ul v-if="articles.length" class="editorial-articles"><li v-for="item in articles" :key="item.id"><article><img v-if="item.media_url" :src="item.media_url" :alt="item.media_alt" loading="lazy" class="editorial-card-image" /><time :datetime="date(item.published_at)">{{ date(item.published_at) }}</time><h2><Link :href="'/articles/' + item.slug" class="text-link">{{ item.title }}</Link></h2><p>{{ item.summary }}</p><ul class="directory-tags"><li v-for="term in item.terms" :key="term.id"><Link :href="termUrl(term)">{{ term.name }}</Link></li></ul></article></li></ul>
    <div v-else class="directory-empty"><h2>{{ q || category || tag ? 'No articles match your search' : 'Articles are on their way' }}</h2><p>{{ q || category || tag ? 'Try another search or browse all articles.' : 'Published stories will appear here.' }}</p><Link v-if="q || category || tag" href="/articles" class="text-link">Browse all articles</Link></div>
    <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Article pages"><Link v-if="pagination.page > 1" :href="pageUrl(pagination.page - 1)" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }} of {{ Math.ceil(pagination.total / pagination.per_page) }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="pageUrl(pagination.page + 1)" class="button button-secondary">Next</Link></nav>
  </section>
</template>
