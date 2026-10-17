<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
const page = usePage<SharedProps>()
</script>
<template>
  <Head title="Directory" />
  <section class="home-intro">
    <div class="intro-copy">
      <p class="eyebrow">Worth finding. Worth sharing.</p>
      <h1>A place for good discoveries.</h1>
      <p class="lead">Explore a growing collection of places, projects, and people worth knowing.</p>
      <Link :href="page.props.auth.user ? '/dashboard' : '/register'" class="button button-primary">{{ page.props.auth.user ? 'Your account' : 'Join the directory' }} <span aria-hidden="true">↗</span></Link>
    </div>
    <div class="discovery-illustration" aria-hidden="true"><span class="orbit orbit-one" /><span class="orbit orbit-two" /><span class="orbit-center" /><span class="orbit-point" /></div>
  </section>
  <section class="directory-empty" aria-labelledby="collection-title">
    <div class="section-heading"><h2 id="collection-title">The collection</h2><span>Just getting started</span></div>
    <div class="empty-copy"><h3>Good things are on their way.</h3><p>No listings have been published yet. Check back for the first discoveries.</p></div>
  </section>
</template>
