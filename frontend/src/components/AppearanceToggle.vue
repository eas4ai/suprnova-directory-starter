<script setup lang="ts">
import { useTheme } from '@vuetify/v0'
const theme = useTheme()
function toggle() {
  const next = theme.isDark.value ? 'light' : 'dark'
  theme.select(next)
  document.cookie = `site_appearance=${next}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === 'https:' ? '; Secure' : ''}`
}
</script>

<template>
  <button type="button" class="button button-secondary appearance-toggle" :aria-pressed="theme.isDark.value" :title="theme.isDark.value ? 'Switch to light mode' : 'Switch to dark mode'" aria-label="Dark mode" @click="toggle">
    <svg v-if="theme.isDark.value" aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75"><circle cx="12" cy="12" r="4" /><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5" /></svg>
    <svg v-else aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75"><path d="M20 15.5A8.5 8.5 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5Z" /></svg>
  </button>
</template>
