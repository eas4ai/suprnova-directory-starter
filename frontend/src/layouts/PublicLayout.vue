<script setup lang="ts">
import SiteBrand from '../components/SiteBrand.vue'
import ShellNavigation from '../components/ShellNavigation.vue'
</script>
<template>
  <div class="public-shell" data-shell="public">
    <a href="#main-content" class="skip-link">Skip to content</a>
    <header class="public-header"><SiteBrand /><ShellNavigation /></header>
    <main id="main-content" tabindex="-1" class="public-content"><slot /></main>
    <footer class="site-footer"><span>A little curiosity goes a long way.</span><span>Directory</span></footer>
  </div>
</template>
