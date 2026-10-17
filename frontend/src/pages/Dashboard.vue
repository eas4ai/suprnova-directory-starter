<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../types/shared'
const page = usePage<SharedProps>()
defineProps<{ user: { id: number; name: string; email: string } }>()
</script>
<template>
  <Head title="Your account" />
  <section class="account-page">
    <h1>Your account</h1>
    <p class="lead">Welcome back, {{ page.props.auth.user?.name }}.</p>
    <dl class="account-details"><div><dt>Email address</dt><dd>{{ page.props.auth.user?.email }}</dd></div><div><dt>Email status</dt><dd>{{ page.props.auth.user?.verified ? 'Verified' : 'Awaiting verification' }}</dd></div></dl>
    <p class="muted">Account ID: <span data-account-id>{{ user.id }}</span></p>
    <div v-if="!page.props.auth.user?.verified" class="notice"><h2>Confirm your email address</h2><p>Use the link in your inbox to verify your account.</p><Link href="/verify-email" class="button button-primary">Verify your email</Link></div>
    <div class="directory-actions"><Link href="/dashboard/listings" class="button button-primary">Your listings</Link><Link href="/listings" class="text-link">Explore the directory <span aria-hidden="true">↗</span></Link></div>
  </section>
</template>
