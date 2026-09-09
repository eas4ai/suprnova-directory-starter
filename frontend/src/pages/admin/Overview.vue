<script setup lang="ts">
import { Head, Link, usePage } from '@inertiajs/vue3'
import type { SharedProps } from '../../types/shared'
const page = usePage<SharedProps>()
</script>
<template>
  <Head title="Administration" />
  <section class="admin-overview">
    <h1>Administration</h1>
    <p class="lead">Welcome, {{ page.props.auth.user?.name }}. This is your directory workspace.</p>
    <div class="workspace-summary"><div><span class="status-dot" />Signed in as an administrator</div><Link href="/dashboard" class="text-link">Your account ↗</Link></div>
    <section class="workspace-empty"><h2>A fresh start.</h2><p>Your directory has no published listings yet.</p><p class="muted">Take a look around, then make this space your own.</p><Link href="/" class="button button-primary">Open the directory</Link></section>
    <dl class="account-details"><div><dt>Signed in</dt><dd>{{ page.props.auth.user?.email }}</dd></div><div><dt>Email status</dt><dd>{{ page.props.auth.user?.verified ? 'Verified' : 'Awaiting verification' }}</dd></div></dl>
  </section>
</template>
