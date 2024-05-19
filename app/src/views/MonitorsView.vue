<template>
  <v-btn append-icon="mdi-plus" color="primary" class="ma-4" @click="openDialog">
    Add Monitor
    <v-tooltip activator="parent" location="top">Click to add a new monitor</v-tooltip>
  </v-btn>
  <div class="d-flex flex-column align-center">
    <MonitorInfo
      v-for="monitor in monitors"
      :key="monitor.monitor_id"
      :monitor="monitor"
      :isNew="$cookies.isKey(monitor.monitor_id)"
    />
  </div>
  <SetupMonitorDialog
    :dialogActive="dialogActive"
    @dialog-complete="dialogComplete"
    @dialog-aborted="closeDialog"
  />
</template>

<script setup lang="ts">
import { ref, inject } from 'vue'
import type { VueCookies } from 'vue-cookies'

import MonitorInfo from '@/components/MonitorInfo.vue'
import SetupMonitorDialog from '@/components/SetupMonitorDialog.vue'
import { MonitorRepository } from '@/repos/monitor-repo'
import type { BasicMonitorInformation } from '@/models/monitor'

const FIVE_MINUTES_MS = 5 * 60 * 1000

const cookies = inject<VueCookies>('$cookies')
const monitorRepo = new MonitorRepository()
const monitors = ref(await monitorRepo.getMonitorInfos())
const dialogActive = ref(false)

async function dialogComplete(monitorInfo: BasicMonitorInformation) {
  const monitor = await monitorRepo.addMonitor(monitorInfo)
  cookies?.set(monitor.monitor_id, 'new', '5min')

  // We get the list of monitors again here, rather than just inserting the new monitor,
  // so that the list is sorted by the API.
  monitors.value = await monitorRepo.getMonitorInfos()
  closeDialog()
}

function openDialog() {
  dialogActive.value = true
}

function closeDialog() {
  dialogActive.value = false
}

function resyncMonitors() {
  setTimeout(async () => {
    monitors.value = await monitorRepo.getMonitorInfos()
    resyncMonitors()
  }, FIVE_MINUTES_MS)
}

resyncMonitors()
</script>
