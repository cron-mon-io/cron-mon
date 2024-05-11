<template>
  <v-card variant="elevated" class="ma-3">
    <v-card-title class="text-h5"
      ><code>{{ job.job_id }}</code></v-card-title
    >
    <v-divider></v-divider>
    <v-card-text class="d-flex flex-column justify-space-between">
      <span>
        <v-chip class="ma-2 font-weight-bold" :color="succeeded.colour" variant="outlined">
          <v-icon :icon="succeeded.icon" start></v-icon>
          {{ succeeded.text }}
        </v-chip>
        <v-chip v-if="job.late" class="ma-2 font-weight-bold" color="error" variant="outlined">
          <v-icon icon="mdi-timer-alert" start></v-icon>
          Late
        </v-chip>
      </span>
      <div class="mt-3 d-flex flex-column text-body-1">
        <span>Started at: {{ job.start_time }}</span>
        <span v-if="job.end_time !== null">Ended at: {{ job.end_time }}</span>
        <span v-if="job.duration !== null">Duration: {{ formatDuration(job.duration) }}</span>
        <span v-if="job.output !== null">
          Output: <code>{{ job.output }}</code>
        </span>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Job } from '@/models/job'
import { formatDuration } from '@/utils/time'
const props = defineProps<{
  job: Job
}>()

const job = props.job
const succeeded = ref({
  colour: job.succeeded === true ? 'success' : job.succeeded === false ? 'error' : 'info',
  icon:
    job.succeeded === true
      ? 'mdi-check-circle'
      : job.succeeded === false
        ? 'mdi-close-circle'
        : 'mdi-information',
  text: job.succeeded === true ? 'Succeeded' : job.succeeded === false ? 'Failed' : 'In progress'
})
</script>
