<template>
  <v-card variant="elevated" class="mx-6 mt-13">
    <template v-slot:title>
      <div class="title">
        <v-icon>
          <v-img :width="40" :height="40" aspect-ratio="16/9" :src="CronMonIcon"></v-img>
        </v-icon>
        <span class="font-weight-black ml-4">Welcome to CronMon</span>
      </div>
    </template>
    <v-card-text class="text-body-1">
      CronMon is a simple, flexible monitoring tool that allows you to monitor your cron jobs and
      get notified when they don't start when expected, take longer than expected, or fail
      altogether. CronMon also monitors the duration of your cron jobs over time and notifies you if
      they take longer than expected on average. This can help you identify performance issues in
      your cron jobs.

      <br /><br />

      Setting up monitors and using them is quick and easy:

      <v-timeline side="end" class="mt-2">
        <v-timeline-item icon="mdi-cog-outline" dot-color="primary" fill-dot>
          <v-card class="elevation-2">
            <v-card-title class="text-h5">Setup</v-card-title>
            <v-card-text>
              Go to the <v-icon to="/monitors">mdi-monitor-eye</v-icon> in the sidebar to view your
              monitored cron jobs. From here, you can add new monitors for you cron jobs - which is
              a simple as giving the monitor a name, an expected duration and a grace period.
            </v-card-text>
          </v-card>
        </v-timeline-item>

        <v-timeline-item icon="mdi-console" dot-color="primary" fill-dot>
          <v-card class="elevation-2">
            <v-card-title class="text-h5">Integration</v-card-title>
            <v-card-text>
              CronMon provides a simple REST API to integrate with your existing applications, which
              is as simple as two API calls: one to signify that a new job has started and another
              to say that it's finished.
              <v-alert type="info" class="mt-3">
                In the future, CronMon will also provide libraries in various programming languages
                to make integration even simpler.
              </v-alert>
            </v-card-text>
          </v-card>
        </v-timeline-item>

        <v-timeline-item icon="mdi-run" dot-color="primary" fill-dot>
          <v-card class="elevation-2">
            <v-card-title class="text-h5">Go go go!</v-card-title>
            <v-card-text>
              Start running your jobs! CronMon will then take care of the rest.
            </v-card-text>
          </v-card>
        </v-timeline-item>
      </v-timeline>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import CronMonIcon from '@/assets/icon.svg'
</script>

<style scoped>
.title {
  display: flex;
}
</style>
