<template>
  <v-dialog v-model="active" width="auto" @keyup.esc="abort" @keyup.enter="confirm">
    <v-card max-width="500">
      <v-card-title :prepend-icon="icon">{{ title }}</v-card-title>
      <v-card-text>{{ question }}</v-card-text>
      <v-card-actions>
        <v-btn
          text="No"
          @click="abort"
          color="orange"
          variant="tonal"
          append-icon="mdi-close-circle"
        ></v-btn>
        <v-btn
          text="Yes"
          @click="confirm"
          color="primary"
          variant="elevated"
          append-icon="mdi-check-circle"
          :loading="loading"
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = defineProps<{
  dialogActive: boolean
  title: string
  icon: string
  question: string
}>()
const emit = defineEmits<{
  (e: 'dialog-complete', confirmed: boolean): void
}>()

const loading = ref(false)
const active = computed(() => props.dialogActive)

// When the parent component closes the dialog, we want to set loading back to false.
watch(active, (newActive, oldActive) => {
  if (oldActive && !newActive) {
    loading.value = false
  }
})

function abort() {
  emit('dialog-complete', false)
}

async function confirm() {
  loading.value = true
  emit('dialog-complete', true)
}
</script>
