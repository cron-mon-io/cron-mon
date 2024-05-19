<template>
  <v-card class="elevation-2 ma-3 w-50">
    <MonitorSummary :monitor="monitor" :is-new="isNew" />
    <v-card-actions>
      <v-chip class="ma-2 font-weight-bold" :color="lastFinish.colour" variant="outlined">
        <v-icon :icon="lastFinish.icon" start></v-icon>
        {{ lastFinish.text }}
      </v-chip>
      <v-chip v-if="lastJobWasLate" class="ma-2 font-weight-bold" color="error" variant="outlined">
        <v-icon icon="mdi-timer-alert" start></v-icon>
        Late
      </v-chip>
      <v-chip v-if="jobInProgress" class="ma-2 font-weight-bold" color="info" variant="outlined">
        <v-icon icon="mdi-information" start></v-icon>
        In progress
      </v-chip>
      <v-spacer></v-spacer>
      <v-btn
        color="primary"
        variant="elevated"
        append-icon="mdi-open-in-app"
        :to="`/monitors/${monitor.monitor_id}`"
      >
        View
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import type { MonitorInformation } from '@/models/monitor'
import MonitorSummary from '@/components/MonitorSummary.vue'

const props = defineProps<{
  monitor: MonitorInformation
  isNew: boolean
}>()

const lastFinishedJob = props.monitor.last_finished_job
const lastStartedJob = props.monitor.last_started_job
const lastFinish = ref({
  colour:
    lastFinishedJob === null ? 'info' : lastFinishedJob.succeeded! === true ? 'success' : 'error',
  icon:
    lastFinishedJob === null
      ? 'mdi-help-circle'
      : lastFinishedJob.succeeded === true
        ? 'mdi-check-circle'
        : 'mdi-close-circle',
  text:
    lastFinishedJob === null
      ? 'No finished jobs'
      : lastFinishedJob.succeeded === true
        ? 'Succeeded'
        : 'Failed'
})
const lastJobWasLate = ref(lastFinishedJob === null ? false : lastFinishedJob.late)
const jobInProgress = ref(lastStartedJob === null ? false : lastFinishedJob?.in_progress)
</script>
