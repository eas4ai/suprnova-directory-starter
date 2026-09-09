<script setup lang="ts">
import { Head, Link } from '@inertiajs/vue3'
import type { EditorArticle } from '../../types/articles'
defineProps<{ article: EditorArticle }>()
</script>
<template>
  <Head :title="'Preview: ' + article.current.title" />
  <section class="directory-page editorial-reading"><p class="notice">Private preview of the saved draft. This may differ from the published article.</p><Link :href="'/admin/articles/' + article.id + '/edit'" class="text-link">Back to editor</Link><article><header class="directory-heading"><h1>{{ article.current.title }}</h1><p class="editorial-summary">{{ article.current.summary }}</p></header><ul class="directory-tags"><li v-for="term in article.current.terms" :key="term.id">{{ term.name }}</li></ul><img v-if="article.current.media_url" :src="article.current.media_url" :alt="article.current.media_alt" class="editorial-cover" /><div class="editorial-prose" v-html="article.current.body_html" /></article></section>
</template>
