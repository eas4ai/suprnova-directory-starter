<script setup lang="ts">
import { Link, useForm } from '@inertiajs/vue3'
import { ref } from 'vue'
const form = useForm({ email: '' })
const sent = ref(false)
</script>

<template>
  <main class="mx-auto max-w-lg p-8 space-y-6">
    <h1 class="text-3xl font-semibold">Reset your password</h1>
    <p>Enter the email address you verified for your account.</p>
    <p v-if="sent" role="status">If this address belongs to a verified account, a reset link has been sent.</p>
    <form class="space-y-4" @submit.prevent="form.post('/forgot-password', { onSuccess: () => sent = true })">
      <label for="email" class="block">Email address</label>
      <input id="email" v-model="form.email" type="email" autocomplete="email" required class="w-full rounded border p-2" />
      <p v-if="form.errors.email" role="alert">{{ form.errors.email }}</p>
      <button type="submit" :disabled="form.processing" class="rounded border px-4 py-2">Send reset link</button>
    </form>
    <Link href="/login">Back to sign in</Link>
  </main>
</template>
