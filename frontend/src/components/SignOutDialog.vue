<script setup lang="ts">
import { Dialog } from '@vuetify/v0'
import { useForm } from '@inertiajs/vue3'
const form = useForm({})
</script>
<template>
  <Dialog.Root>
    <Dialog.Activator class="button button-quiet">Sign out</Dialog.Activator>
    <Dialog.Content class="app-dialog">
      <Dialog.Title as="h2">Sign out of your account?</Dialog.Title>
      <Dialog.Description>You can sign in again whenever you’re ready.</Dialog.Description>
      <div class="dialog-actions">
        <Dialog.Close class="button button-secondary" aria-label="Stay signed in">Stay signed in</Dialog.Close>
        <button class="button button-primary" :disabled="form.processing" @click="form.post('/logout')">Sign out now</button>
      </div>
    </Dialog.Content>
  </Dialog.Root>
</template>
