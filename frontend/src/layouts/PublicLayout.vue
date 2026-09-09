<script setup lang="ts">
import { Head, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
import SiteBrand from '../components/SiteBrand.vue'
import ShellNavigation from '../components/ShellNavigation.vue'
const page = usePage<SharedProps>()
</script>
<template>
  <Head><meta head-key="theme-color" name="theme-color" :content="page.props.site.accent" /></Head>
  <div class="public-shell" data-shell="public" :style="{ '--brand-accent': page.props.site.accent }">
    <a href="#main-content" class="skip-link">Skip to content</a>
    <header class="public-header"><SiteBrand /><ShellNavigation /></header>
    <main id="main-content" tabindex="-1" class="public-content"><slot /></main>
    <footer class="site-footer"><span>{{ page.props.site.description }}</span><span>{{ page.props.site.name }}</span></footer>
  </div>
</template>
