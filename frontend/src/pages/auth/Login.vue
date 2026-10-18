<script setup lang="ts">
import { useForm } from '@inertiajs/vue3'


const form = useForm({
  email: '',
  password: '',
  remember: false,
})

function submit() {
  form.post('/login')
}
</script>

<template>
  <div
    class="auth-container"
  >
    <div class="auth-card">
      <div>
        <h1 class="auth-title">
          Sign in to your account
        </h1>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="submit">
        <div class="space-y-4">
          <div>
            <label for="email" class="form-label">Email address</label>
            <input
              id="email"
              v-model="form.email"
              name="email"
              type="email"
              autocomplete="email"
              required
              class="form-input"
              placeholder="Email address"
            />
          </div>
          <div>
            <label for="password" class="form-label">Password</label>
            <input
              id="password"
              v-model="form.password"
              name="password"
              type="password"
              autocomplete="current-password"
              required
              class="form-input"
              placeholder="Password"
            />
          </div>
        </div>

        <div v-if="form.errors.email" class="form-error" role="alert">
          {{ form.errors.email }}
        </div>

        <div v-if="form.errors.password" class="form-error" role="alert">
          {{ form.errors.password }}
        </div>

        <div class="flex items-center">
          <input
            id="remember"
            v-model="form.remember"
            name="remember"
            type="checkbox"
            class="form-checkbox"
          />
          <label for="remember" class="ml-2">
            Remember me
          </label>
        </div>

        <div>
          <button
            type="submit"
            :disabled="form.processing"
            class="button button-primary"
          >
            {{ form.processing ? 'Signing in...' : 'Sign in' }}
          </button>
        </div>

        <div class="flex flex-col items-center gap-2 text-center">
          <a href="/forgot-password" class="text-link">Forgot your password?</a>
          <a href="/register" class="text-link">
            Don't have an account? Register
          </a>
        </div>
      </form>
    </div>
  </div>
</template>
