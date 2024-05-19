<template>
  <v-card class="elevation-2 mx-6 mt-13">
    <MonitorSummary :monitor="monitor" :is-new="$cookies.isKey(monitor.monitor_id)" />
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
      <v-btn append-icon="mdi-pencil" color="primary" class="ma-3" @click="openEditDialog">
        Edit Monitor
        <v-tooltip activator="parent" location="top">Click to modify this Monitor</v-tooltip>
      </v-btn>
      <v-btn append-icon="mdi-delete" color="primary" class="ma-3" @click="openDeleteDialog">
        Delete Monitor
        <v-tooltip activator="parent" location="top">Click to delete this Monitor</v-tooltip>
      </v-btn>
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
  <SetupMonitorDialog
    :dialogActive="editDialogActive"
    @dialog-complete="editDialogComplete"
    @dialog-aborted="closeEditDialog"
    :monitor="monitor"
  />
  <ConfirmationDialog
    :dialogActive="deleteDialogActive"
    title="Delete this Monitor?"
    icon="mdi-delete"
    question="This cannot be undone and will result in the jobs within this Monitor also being deleted. Are you sure?"
    @dialog-complete="deleteDialogComplete"
  />
</template>

<script setup lang="ts">
import { ref, inject } from 'vue'
import type { VueCookies } from 'vue-cookies'
import { useRoute } from 'vue-router'

import JobInfo from '@/components/JobInfo.vue'
import ConfirmationDialog from '@/components/ConfirmationDialog.vue'
import MonitorSummary from '@/components/MonitorSummary.vue'
import SetupMonitorDialog from '@/components/SetupMonitorDialog.vue'
import type { BasicMonitorInformation, MonitorInformation } from '@/models/monitor'
import { MonitorRepository } from '@/repos/monitor-repo'
import router from '@/router'

const ONE_MINUTE_MS = 60 * 1000

const route = useRoute()
const cookies = inject<VueCookies>('$cookies')

const monitorRepo = new MonitorRepository()
const monitor = ref(await monitorRepo.getMonitor(route.params.id as string))
const editDialogActive = ref(false)
const deleteDialogActive = ref(false)

function copyMonitorId() {
  navigator.clipboard.writeText(monitor.value.monitor_id)
}

async function editDialogComplete(monitorInfo: BasicMonitorInformation) {
  const newMonitor = {
    monitor_id: monitor.value.monitor_id,
    ...monitorInfo
  } as MonitorInformation
  monitor.value = await monitorRepo.updateMonitor(newMonitor)
  cookies?.set(monitor.value.monitor_id, 'new', '5min')
  closeEditDialog()
}

function openEditDialog() {
  editDialogActive.value = true
}

function closeEditDialog() {
  editDialogActive.value = false
}

async function deleteDialogComplete(confirmed: boolean) {
  if (confirmed) {
    await monitorRepo.deleteMonitor(monitor.value)
    router.push('/monitors')
  }
  closeDeleteDialog()

  // We want to close the dialog first before we navigate back to the monitors page,
  // just because it looks slightly better.
  if (confirmed) {
    router.push('/monitors')
  }
}

function openDeleteDialog() {
  deleteDialogActive.value = true
}

function closeDeleteDialog() {
  deleteDialogActive.value = false
}

function resyncMonitor() {
  setTimeout(async () => {
    monitor.value = await monitorRepo.getMonitor(route.params.id as string)
    resyncMonitor()
  }, ONE_MINUTE_MS)
}

resyncMonitor()
</script>
