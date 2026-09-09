<script setup lang="ts">
import { Link, useForm } from '@inertiajs/vue3'
const props = defineProps<{ token: string }>()
const form = useForm({ token: props.token, password: '', password_confirmation: '' })
</script>

<template>
  <section class="auth-card">
    <h1 class="auth-title">Choose a new password</h1>
    <form class="space-y-4" @submit.prevent="form.post('/reset-password', { onFinish: () => form.reset('password', 'password_confirmation') })">
      <label for="password" class="block">New password</label>
      <input id="password" v-model="form.password" type="password" autocomplete="new-password" minlength="8" required class="form-input" />
      <label for="confirmation" class="block">Confirm new password</label>
      <input id="confirmation" v-model="form.password_confirmation" type="password" autocomplete="new-password" minlength="8" required class="form-input" />
      <p v-for="(error, field) in form.errors" :key="field" role="alert">{{ error }}</p>
      <button type="submit" :disabled="form.processing" class="button button-primary">Save new password</button>
    </form>
    <Link href="/forgot-password">Request another reset link</Link>
  </section>
</template>
