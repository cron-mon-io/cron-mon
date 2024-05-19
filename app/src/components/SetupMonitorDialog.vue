<template>
  <v-dialog v-model="active" width="auto" @keyup.esc="exit" @keyup.enter="saveAndExit">
    <v-card min-width="500">
      <v-card-title prepend-icon="mdi-update">Create new Monitor</v-card-title>
      <v-card-text>
        <v-form v-model="formValid">
          <v-text-field
            class="mt-5"
            hint="Give your monitor a name"
            persistent-hint
            label="Name"
            v-model="name"
            clearable
            variant="outlined"
          ></v-text-field>
          <v-text-field
            class="mt-5"
            hint="How long do you expect the jobs being monitored to run for?"
            persistent-hint
            label="Expected duration"
            v-model="expectedDuration"
            clearable
            variant="outlined"
            :rules="[validateDuration(expectedDuration)]"
          ></v-text-field>
          <v-text-field
            class="mt-5"
            hint="How much longer after that should we wait before considering the job late?"
            persistent-hint
            label="Grace duration"
            v-model="graceDuration"
            clearable
            variant="outlined"
            :rules="[validateDuration(graceDuration)]"
          ></v-text-field>
        </v-form>
      </v-card-text>
      <v-card-actions>
        <v-btn
          text="Cancel"
          @click="exit"
          color="orange"
          variant="tonal"
          append-icon="mdi-close-circle"
        ></v-btn>
        <v-btn
          text="Save"
          @click="saveAndExit"
          color="primary"
          variant="elevated"
          append-icon="mdi-check-circle"
          :disabled="!canSave"
          :loading="loading"
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import type { BasicMonitorInformation, Monitor } from '@/models/monitor'
import { durationFromString, formatDuration } from '@/utils/time'

const props = defineProps<{
  dialogActive: boolean
  monitor?: Monitor | null
}>()
const emit = defineEmits<{
  (e: 'dialog-complete', monitorInfo: BasicMonitorInformation): void
  (e: 'dialog-aborted'): void
}>()

const name = ref(props.monitor ? props.monitor.name : '')
const expectedDuration = ref(props.monitor ? formatDuration(props.monitor.expected_duration) : '')
const graceDuration = ref(props.monitor ? formatDuration(props.monitor.grace_duration) : '')

const formValid = ref(false)
const loading = ref(false)
const active = computed(() => props.dialogActive)
const canSave = computed(
  () =>
    formValid.value &&
    name.value.length > 0 &&
    expectedDuration.value.length > 0 &&
    graceDuration.value.length > 0
)
const monitorInfo = computed(
  () =>
    ({
      name: name.value,
      expected_duration: durationFromString(expectedDuration.value),
      grace_duration: durationFromString(graceDuration.value)
    }) as BasicMonitorInformation
)

// When the parent component closes the dialog, we want to set loading back to false.
watch(active, (newActive, oldActive) => {
  if (oldActive && !newActive) {
    loading.value = false
  }
})

function exit() {
  emit('dialog-aborted')
}

async function saveAndExit() {
  loading.value = true
  emit('dialog-complete', monitorInfo.value)
}

function validateDuration(duration: string): boolean | string {
  try {
    durationFromString(duration)
    return true
  } catch (e: unknown) {
    return (e as Error).message
  }
}
</script>
