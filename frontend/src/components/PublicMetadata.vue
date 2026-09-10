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
    <meta head-key="robots" name="robots" :content="seo.robots" />
    <meta head-key="og:site_name" property="og:site_name" :content="seo.site_name" />
    <meta head-key="twitter:card" name="twitter:card" :content="seo.image ? 'summary_large_image' : 'summary'" />
    <meta head-key="twitter:title" name="twitter:title" :content="seo.title" />
    <meta head-key="twitter:description" name="twitter:description" :content="seo.description" />
    <meta v-if="seo.image" head-key="twitter:image" name="twitter:image" :content="seo.image" />
    <meta v-if="seo.google_verification" head-key="google-site-verification" name="google-site-verification" :content="seo.google_verification" />
    <meta v-if="seo.bing_verification" head-key="msvalidate.01" name="msvalidate.01" :content="seo.bing_verification" />
    <link v-if="seo.markdown" head-key="markdown" rel="alternate" type="text/markdown" :href="seo.markdown" />
    <!-- The server serializes structured data and escapes HTML delimiters. -->
    <component :is="'script'" head-key="structured-data" type="application/ld+json">{{ seo.structured_data }}</component>
  </Head>
</template>
