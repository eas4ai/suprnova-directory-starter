<script setup lang="ts">
import { Head } from '@inertiajs/vue3'
import type { Seo } from '../types/articles'
defineProps<{ seo: Seo }>()
</script>
<template>
  <Head :title="seo.title">
    <meta head-key="description" name="description" :content="seo.description" />
    <link head-key="canonical" rel="canonical" :href="seo.canonical" />
    <meta head-key="og:type" property="og:type" :content="seo.kind" />
    <meta head-key="og:title" property="og:title" :content="seo.title" />
    <meta head-key="og:description" property="og:description" :content="seo.description" />
    <meta head-key="og:url" property="og:url" :content="seo.canonical" />
    <meta v-if="seo.image" head-key="og:image" property="og:image" :content="seo.image" />
    <!-- The server serializes structured data and escapes HTML delimiters. -->
    <component :is="'script'" head-key="structured-data" type="application/ld+json">{{ seo.structured_data }}</component>
  </Head>
</template>
