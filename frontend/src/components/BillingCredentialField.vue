<script setup lang="ts">
import { ref } from 'vue'
defineProps<{ id: string; label: string; error?: string; secret?: boolean; hint?: string; disabled?: boolean }>()
const value = defineModel<string>({ required: true })
const visible = ref(false)
</script>

<template>
  <div class="billing-field">
    <label :for="id" class="form-label">{{ label }}</label>
    <div class="credential-input">
      <input :id="id" v-model="value" class="form-input" :type="secret && !visible ? 'password' : 'text'"
        :disabled="disabled" :aria-invalid="!!error" :aria-describedby="`${id}-help${error ? ` ${id}-error` : ''}`"
        :autocomplete="secret ? 'new-password' : 'off'" autocapitalize="none" :spellcheck="false" maxlength="512" />
      <button v-if="secret" type="button" class="button button-quiet" :disabled="disabled"
        :aria-label="`${visible ? 'Hide' : 'Show'} ${label}`" :aria-pressed="visible" @click="visible = !visible">{{ visible ? 'Hide' : 'Show' }}</button>
    </div>
    <p :id="`${id}-help`" class="field-help">{{ hint || 'Copy this value from the provider account for the selected mode.' }}</p>
    <p v-if="error" :id="`${id}-error`" class="form-error">{{ error }}</p>
  </div>
</template>
