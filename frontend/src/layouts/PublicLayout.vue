<script setup lang="ts">
import { usePage } from '@inertiajs/vue3'
import { useTheme } from '@vuetify/v0'
import type { SharedProps } from '../types/shared'
import SiteBrand from '../components/SiteBrand.vue'
import ShellNavigation from '../components/ShellNavigation.vue'
const page = usePage<SharedProps>()
const theme = useTheme()
</script>
<template>
  <div class="public-shell" data-shell="public" :data-theme="theme.isDark.value ? 'dark' : 'light'">
    <a href="#main-content" class="skip-link">Skip to content</a>
    <header class="public-header"><SiteBrand /><ShellNavigation /></header>
    <main id="main-content" tabindex="-1" class="public-content"><slot /></main>
    <footer class="site-footer"><span>{{ page.props.site.description }}</span><span>{{ page.props.site.name }}</span></footer>
  </div>
</template>
