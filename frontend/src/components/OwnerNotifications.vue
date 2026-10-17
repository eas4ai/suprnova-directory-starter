<script setup lang="ts">
import type { OwnerNotice } from '../types/notifications'
defineProps<{ notifications: OwnerNotice[] }>()
const date = (timestamp: number) => new Date(timestamp * 1000).toISOString().replace('T', ' ').slice(0, 16) + ' UTC'
</script>
<template>
  <section aria-labelledby="owner-notifications-title" class="owner-notifications">
    <h2 id="owner-notifications-title">Updates for this listing</h2>
    <p class="field-help">Your latest 20 moderation and payment updates. Email delivery may arrive later.</p>
    <p v-if="!notifications.length">No updates yet.</p>
    <ol v-else class="owner-notification-list">
      <li v-for="notice in notifications" :key="notice.id">
        <h3>{{ notice.title }}</h3>
        <time :datetime="new Date(notice.created_at * 1000).toISOString()">{{ date(notice.created_at) }}</time>
        <p>{{ notice.body }}</p>
      </li>
    </ol>
  </section>
</template>
<style scoped>
.owner-notifications { margin-block-start: 2rem; border-top: 1px solid var(--border-default); padding-block-start: 1.5rem; }
.owner-notification-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 1.25rem; }
.owner-notification-list h3 { margin-bottom: .25rem; }
.owner-notification-list time { font-size: .875rem; }
.owner-notification-list p { white-space: pre-line; overflow-wrap: anywhere; }
</style>
