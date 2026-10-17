<script setup lang="ts">
import { Link, useForm } from '@inertiajs/vue3'
import { ref } from 'vue'
defineProps<{ email: string }>()
const form = useForm({})
const sent = ref(false)
</script>

<template>
  <section class="auth-card">
    <h1 class="auth-title">Verify your email</h1>
    <p>Open the verification link sent to {{ email }} while signed in to this account.</p>
    <p v-if="sent" role="status">A new link has been sent.</p>
    <form @submit.prevent="form.post('/email/verification-notification', { onSuccess: () => sent = true })">
      <button type="submit" :disabled="form.processing" class="button button-primary">Resend verification link</button>
    </form>
    <Link href="/dashboard">Back to your account</Link>
  </section>
</template>
