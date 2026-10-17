<script setup lang="ts">
import { computed, ref } from 'vue'
import { Link, useForm, usePage } from '@inertiajs/vue3'
import { Dialog } from '@vuetify/v0'
import type { SharedProps } from '../types/shared'
import AppearanceToggle from './AppearanceToggle.vue'
import SignOutDialog from './SignOutDialog.vue'

const props = defineProps<{ admin?: boolean }>()
const page = usePage<SharedProps>()
const user = computed(() => page.props.auth.user)
const open = ref(false)
const logout = useForm({})
const groups = computed(() => props.admin
  ? [
      { label: 'Workspace', links: [{ href: '/admin', label: 'Overview' }] },
      { label: 'Content', links: [
        ...(user.value?.can_moderate ? [{ href: '/admin/listings', label: 'Listing reviews' }] : []),
        ...(user.value?.can_edit ? [{ href: '/admin/articles', label: 'Articles' }] : []),
        ...(user.value?.can_taxonomy ? [{ href: '/admin/taxonomy', label: 'Categories and tags' }] : []),
      ] },
      { label: 'Billing', links: user.value?.can_billing
        ? [{ href: '/admin/plans', label: 'Publishing plans' }, { href: '/admin/billing', label: 'Payment providers' }] : [] },
      { label: 'Administration', links: [
        ...(user.value?.can_seo ? [{ href: '/admin/seo', label: 'SEO' }] : []),
        ...(user.value?.can_accounts ? [{ href: '/admin/accounts', label: 'Accounts' }] : []),
        ...(user.value?.can_audit ? [{ href: '/admin/audit', label: 'Audit history' }] : []),
      ] },
      { label: 'Account', links: [{ href: '/dashboard', label: 'Your account' }, { href: '/listings', label: 'View directory' }] },
    ].filter(group => group.links.length > 0)
  : [{ label: '', links: [
      { href: '/listings', label: 'Explore' },
      { href: '/articles', label: 'Articles' },
      ...(user.value ? [{ href: '/dashboard/listings', label: 'Your listings' }] : []),
      ...(user.value ? [{ href: '/dashboard', label: 'Your account' }] : [{ href: '/login', label: 'Sign in' }]),
      ...(user.value?.can_admin ? [{ href: '/admin', label: 'Administration' }] : []),
    ] }])
const active = (href: string) => page.url.split('?')[0] === href
</script>

<template>
  <div class="shell-navigation" :class="{ 'admin-navigation': admin }">
    <AppearanceToggle />
    <nav class="desktop-navigation" :aria-label="admin ? 'Administration' : 'Main navigation'">
      <div v-for="group in groups" :key="group.label" class="navigation-group" :role="group.label ? 'group' : undefined" :aria-label="group.label || undefined">
        <p v-if="group.label" class="navigation-group-label">{{ group.label }}</p>
        <Link v-for="link in group.links" :key="link.href" :href="link.href" class="nav-link" :aria-current="active(link.href) ? 'page' : undefined">{{ link.label }}</Link>
      </div>
    </nav>
    <div class="desktop-actions">
      <SignOutDialog v-if="user" />
      <Link v-else href="/register" class="button button-primary">Create account</Link>
    </div>
    <Dialog.Root v-model="open">
      <Dialog.Activator class="button button-secondary mobile-menu" aria-label="Open navigation">Menu <span aria-hidden="true">≡</span></Dialog.Activator>
      <Dialog.Content class="app-dialog navigation-dialog">
        <Dialog.Title as="h2">Navigation</Dialog.Title>
        <Dialog.Description>Find your way around the directory.</Dialog.Description>
        <nav aria-label="Mobile navigation">
          <div v-for="group in groups" :key="group.label" class="navigation-group" :role="group.label ? 'group' : undefined" :aria-label="group.label || undefined">
            <p v-if="group.label" class="navigation-group-label">{{ group.label }}</p>
            <Link v-for="link in group.links" :key="link.href" :href="link.href" class="nav-link" :aria-current="active(link.href) ? 'page' : undefined" @click="open = false">{{ link.label }}</Link>
          </div>
          <Link v-if="!user" href="/register" class="nav-link" @click="open = false">Create account</Link>
        </nav>
        <form v-if="user" @submit.prevent="logout.post('/logout')">
          <button type="submit" class="button button-quiet" :disabled="logout.processing">Sign out</button>
        </form>
        <Dialog.Close class="button button-secondary" aria-label="Close navigation">Close navigation</Dialog.Close>
      </Dialog.Content>
    </Dialog.Root>
  </div>
</template>

<style scoped>
.navigation-group { display: contents; }
.navigation-group[role="group"] { display: block; width: 100%; }
.navigation-group-label { margin: var(--space-6) var(--space-4) var(--space-2); color: var(--text-secondary); font-size: var(--text-small); font-weight: 600; }
.navigation-dialog { max-height: calc(100dvh - 32px); overflow-y: auto; }
</style>
