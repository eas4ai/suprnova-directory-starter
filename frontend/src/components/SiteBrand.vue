<script setup lang="ts">
import { Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
const page = usePage<SharedProps>()
</script>
<template>
  <Link class="site-brand" href="/" :aria-label="`${page.props.site.name} home`">
    <img v-if="page.props.site.logo_url" :src="page.props.site.logo_url" alt="" width="32" height="32" />
    <span v-else class="brand-mark" data-brand-mark aria-hidden="true"><span /></span>
    {{ page.props.site.name }}<span class="brand-period">.</span>
  </Link>
</template>
