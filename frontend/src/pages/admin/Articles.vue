<script setup lang="ts">
import { ref, watch } from 'vue'
import { Head, Link } from '@inertiajs/vue3'
import type { ArticleSummary, Pagination } from '../../types/articles'
const props = defineProps<{ articles: ArticleSummary[]; q: string; state: string; pagination: Pagination }>()
const query = ref(props.q)
const selected = ref(props.state)
watch(() => [props.q, props.state], () => { query.value = props.q; selected.value = props.state })
const pageUrl = (page: number) => '/admin/articles?' + new URLSearchParams({ q: props.q, state: props.state, page: String(page) })
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString().slice(0, 10)
</script>
<template>
  <Head title="Articles" />
  <section class="directory-page"><header class="directory-heading directory-heading-actions"><div><h1>Articles</h1><p>Write and publish guides for your directory.</p></div><Link href="/admin/articles/create" class="button button-primary">Create article</Link></header>
    <form action="/admin/articles" method="get" role="search" class="directory-search"><div><label for="article-query" class="form-label">Search articles</label><input id="article-query" v-model="query" name="q" type="search" maxlength="200" class="form-input" /></div><div><label for="article-state" class="form-label">Publication</label><select id="article-state" v-model="selected" name="state" class="form-input"><option value="">All articles</option><option value="draft">Drafts</option><option value="published">Published</option></select></div><button type="submit" class="button button-secondary">Filter</button><Link v-if="q || state" href="/admin/articles" class="text-link">Reset</Link></form>
    <p role="status">{{ pagination.total }} {{ pagination.total === 1 ? 'article' : 'articles' }}</p>
    <ul v-if="articles.length" class="editorial-admin-list"><li v-for="item in articles" :key="item.id"><div><h2><Link :href="'/admin/articles/' + item.id + '/edit'" class="text-link">{{ item.title }}</Link></h2><p>/articles/{{ item.slug }}</p></div><div><p>{{ item.status === 'draft' ? 'Private draft' : item.status === 'unpublished_changes' ? 'Published · draft changes' : 'Published' }}</p><time :datetime="date(item.updated_at)">Updated {{ date(item.updated_at) }} UTC</time></div></li></ul>
    <div v-else class="directory-empty"><h2>{{ q || state ? 'No articles match these filters' : 'Write your first article' }}</h2><p>{{ q || state ? 'Try another search or reset the filters.' : 'Create a private draft, preview it and publish when ready.' }}</p><Link v-if="q || state" href="/admin/articles" class="text-link">Show all articles</Link></div>
    <nav v-if="pagination.total > pagination.per_page" class="directory-pagination" aria-label="Article pages"><Link v-if="pagination.page > 1" :href="pageUrl(pagination.page - 1)" class="button button-secondary">Previous</Link><span>Page {{ pagination.page }} of {{ Math.ceil(pagination.total / pagination.per_page) }}</span><Link v-if="pagination.page * pagination.per_page < pagination.total" :href="pageUrl(pagination.page + 1)" class="button button-secondary">Next</Link></nav>
  </section>
</template>
