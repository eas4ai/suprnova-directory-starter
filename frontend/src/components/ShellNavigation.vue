<script setup lang="ts">
import { computed, ref } from 'vue'
import { Link, useForm, usePage } from '@inertiajs/vue3'
import { Dialog } from '@vuetify/v0'
import type { SharedProps } from '../types/shared'
import SignOutDialog from './SignOutDialog.vue'

const props = defineProps<{ admin?: boolean }>()
const page = usePage<SharedProps>()
const user = computed(() => page.props.auth.user)
const open = ref(false)
const logout = useForm({})
const links = computed(() => props.admin
  ? [{ href: '/admin', label: 'Overview' }, ...(user.value?.can_billing ? [{ href: '/admin/billing', label: 'Payment providers' }] : []), { href: '/dashboard', label: 'Your account' }, { href: '/', label: 'View directory' }]
  : [
      { href: '/', label: 'Explore' },
      ...(user.value ? [{ href: '/dashboard', label: 'Your account' }] : [{ href: '/login', label: 'Sign in' }]),
      ...(user.value?.can_admin ? [{ href: '/admin', label: 'Administration' }] : []),
    ])
const active = (href: string) => page.url.split('?')[0] === href
</script>

<template>
  <div class="shell-navigation" :class="{ 'admin-navigation': admin }">
    <nav class="desktop-navigation" :aria-label="admin ? 'Administration' : 'Main navigation'">
      <Link v-for="link in links" :key="link.href" :href="link.href" class="nav-link" :aria-current="active(link.href) ? 'page' : undefined">{{ link.label }}</Link>
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
          <Link v-for="link in links" :key="link.href" :href="link.href" class="nav-link" :aria-current="active(link.href) ? 'page' : undefined" @click="open = false">{{ link.label }}</Link>
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
