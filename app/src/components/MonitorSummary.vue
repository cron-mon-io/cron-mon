<template>
  <v-card-title class="text-h5" color="primary">{{ monitor.name }}</v-card-title>
  <v-card-subtitle class="d-flex flex-column">
    <span>Expected duration: {{ formatDuration(monitor.expected_duration) }}</span>
    <span>Grace duration: {{ formatDuration(monitor.grace_duration) }}</span>
  </v-card-subtitle>
</template>

<script setup lang="ts">
import type { MonitorInformation } from '@/models/monitor'
import { formatDuration } from '@/utils/time'

defineProps<{
  monitor: MonitorInformation
}>()
</script>
