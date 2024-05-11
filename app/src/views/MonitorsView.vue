<template>
  <div class="d-flex flex-column align-center">
    <MonitorInfo v-for="monitor in monitors" :key="monitor.monitor_id" :monitor="monitor" />
  </div>
</template>

<script setup lang="ts">
import MonitorInfo from '@/components/MonitorInfo.vue'
import { MonitorRepository } from '@/repos/monitor-repo'
import { ref } from 'vue'

const monitorRepo = new MonitorRepository()
const monitors = ref(await monitorRepo.getMonitorInfos())
</script>
