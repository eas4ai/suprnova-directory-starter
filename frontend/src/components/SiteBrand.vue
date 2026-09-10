<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import { computed } from 'vue'
import { useTheme } from '@vuetify/v0'
import type { SharedProps } from '../types/shared'
const page = usePage<SharedProps>()
const theme = useTheme()
const colors = computed(() => theme.colors.value[theme.isDark.value ? 'dark' : 'light'] ?? {})
// Outrank the base :root tokens regardless of the server head/asset order.
const stylesheet = computed(() => `html:root { ${Object.entries(colors.value).map(([key, value]) => `--${key}: ${value};`).join(' ')} color-scheme: ${theme.isDark.value ? 'dark' : 'light'}; }`)
</script>
<template>
  <Head>
    <meta head-key="theme-color" name="theme-color" :content="theme.isDark.value ? colors['neutral-0'] : colors['site-accent']" />
    <component :is="'style'" head-key="site-accent">{{ stylesheet }}</component>
  </Head>
  <Link class="site-brand" href="/" :aria-label="`${page.props.site.name} home`">
    <img v-if="page.props.site.logo_url" :src="page.props.site.logo_url" alt="" width="32" height="32" />
    <span v-else class="brand-mark" data-brand-mark aria-hidden="true"><span /></span>
    {{ page.props.site.name }}<span class="brand-period">.</span>
  </Link>
</template>
