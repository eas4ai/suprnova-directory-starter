<script setup lang="ts">
import { useForm } from '@inertiajs/vue3'
import type { RegisterProps } from '../../types/inertia-props'

const props = defineProps<RegisterProps>()

const form = useForm({
  name: '',
  email: '',
  password: '',
  password_confirmation: '',
})

function submit() {
  form.post('/register')
}
</script>

<template>
  <div
    class="auth-container"
  >
    <div class="auth-card">
      <div>
        <h1 class="auth-title">
          Create your account
        </h1>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="submit">
        <div class="space-y-4">
          <div>
            <label for="name" class="form-label">Name</label>
            <input
              id="name"
              autocomplete="name"
              v-model="form.name"
              name="name"
              type="text"
              required
              class="form-input"
            />
            <p v-if="props.errors?.name" class="form-error" role="alert">
              {{ props.errors.name }}
            </p>
          </div>

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
            />
            <p v-if="props.errors?.email" class="form-error" role="alert">
              {{ props.errors.email }}
            </p>
          </div>

          <div>
            <label for="password" class="form-label">Password</label>
            <input
              id="password"
              v-model="form.password"
              name="password"
              type="password"
              autocomplete="new-password"
              minlength="8"
              required
              class="form-input"
            />
            <p v-if="props.errors?.password" class="form-error" role="alert">
              {{ props.errors.password }}
            </p>
          </div>

          <div>
            <label for="password_confirmation" class="form-label"
              >Confirm Password</label
            >
            <input
              id="password_confirmation"
              v-model="form.password_confirmation"
              name="password_confirmation"
              type="password"
              autocomplete="new-password"
              minlength="8"
              required
              class="form-input"
            />
            <p v-if="props.errors?.password_confirmation" class="form-error" role="alert">
              {{ props.errors.password_confirmation }}
            </p>
          </div>
        </div>

        <div>
          <button
            type="submit"
            :disabled="form.processing"
            class="button button-primary"
          >
            {{ form.processing ? 'Creating account...' : 'Register' }}
          </button>
        </div>

        <div class="text-center">
          <a href="/login" class="text-link">
            Already have an account? Sign in
          </a>
        </div>
      </form>
    </div>
  </div>
</template>
