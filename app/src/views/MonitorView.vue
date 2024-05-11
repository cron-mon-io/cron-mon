<template>
  <v-card class="elevation-2 mx-6 mt-13">
    <MonitorSummary :monitor="monitor" />
    <v-card-text>
      <span class="text-h6">
        Monitor ID: <code>{{ monitor.monitor_id }}</code>
        <v-tooltip activator="parent" location="top">
          You'll need this when to use the monitor in your cron job, see the docs for more
        </v-tooltip>
      </span>
      <JobInfo v-for="job in monitor.jobs" :key="job.job_id" :job="job" />
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { ref } from 'vue'

import { MonitorRepository } from '@/repos/monitor-repo'
import JobInfo from '@/components/JobInfo.vue'
import MonitorSummary from '@/components/MonitorSummary.vue'

const route = useRoute()

const monitorRepo = new MonitorRepository()
const monitor = ref(await monitorRepo.getMonitor(route.params.id as string))
</script>
