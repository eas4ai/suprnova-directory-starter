<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
const page = usePage<SharedProps>()
</script>
<template>
  <Head>
    <meta head-key="theme-color" name="theme-color" :content="page.props.site.accent" />
    <component :is="'style'" head-key="site-accent">{{ `:root { --site-accent: ${page.props.site.accent}; }` }}</component>
  </Head>
  <Link class="site-brand" href="/" :aria-label="`${page.props.site.name} home`">
    <img v-if="page.props.site.logo_url" :src="page.props.site.logo_url" alt="" width="32" height="32" />
    <span v-else class="brand-mark" data-brand-mark aria-hidden="true"><span /></span>
    {{ page.props.site.name }}<span class="brand-period">.</span>
  </Link>
</template>
