<template>
  <v-card class="elevation-2 mx-6 mt-13">
    <MonitorSummary :monitor="monitor" :is-new="false" />
    <v-card-text>
      <v-chip
        append-icon="mdi-content-copy"
        color="teal-accent-4"
        @click="copyMonitorId"
        class="text-body-1 font-weight-bold ma-2"
        variant="tonal"
        label
      >
        Monitor ID: <code>{{ monitor.monitor_id }}</code>
        <v-tooltip activator="parent" location="top">
          You'll need this to use the monitor in your cron job, see the docs for more.
        </v-tooltip>
      </v-chip>
      <!--
        When the key changes Vue will re-render the component so using `late` and `succeeded`
        in the key means that as jobs become late or are finished we'll trigger a re-render.
        This is a bit of a hack but it works for now. TODO: Find a better way to do this.
      -->
      <JobInfo
        v-for="job in monitor.jobs"
        :key="job.job_id + job.late + job.succeeded"
        :job="job"
      />
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { ref } from 'vue'

import { MonitorRepository } from '@/repos/monitor-repo'
import JobInfo from '@/components/JobInfo.vue'
import MonitorSummary from '@/components/MonitorSummary.vue'

const ONE_MINUTE_MS = 60 * 1000

const route = useRoute()

const monitorRepo = new MonitorRepository()
const monitor = ref(await monitorRepo.getMonitor(route.params.id as string))

function copyMonitorId() {
  navigator.clipboard.writeText(monitor.value.monitor_id)
}

function resyncMonitor() {
  setTimeout(async () => {
    monitor.value = await monitorRepo.getMonitor(route.params.id as string)
    resyncMonitor()
  }, ONE_MINUTE_MS)
}

resyncMonitor()
</script>
