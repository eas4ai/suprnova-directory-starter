<script setup lang="ts">
import { Head, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
import SiteBrand from '../components/SiteBrand.vue'
import ShellNavigation from '../components/ShellNavigation.vue'
const page = usePage<SharedProps>()
</script>
<template>
  <Head><meta head-key="robots" name="robots" content="noindex, nofollow" /></Head>
  <div class="admin-shell" data-shell="admin" :style="{ '--brand-accent': page.props.site.accent }">
    <a href="#main-content" class="skip-link">Skip to content</a>
    <aside class="admin-sidebar"><SiteBrand /><p class="sidebar-label">Administration</p><ShellNavigation admin /></aside>
    <div class="admin-workspace">
      <header class="admin-header"><span>Your workspace</span><span>{{ page.props.auth.user?.name }}</span></header>
      <main id="main-content" tabindex="-1" class="admin-content"><slot /></main>
    </div>
  </div>
</template>
