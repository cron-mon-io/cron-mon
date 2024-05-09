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

      <!-- Try this as a timeline as well - see what looks better! -->
      <v-stepper class="my-5 mx-10" :items="['Step 1', 'Step 2', 'Step 3']">
        <!-- eslint-disable-next-line vue/valid-v-slot -->
        <template v-slot:item.1>
          <v-card title="Setup">
            Go to the <v-icon to="/monitors">mdi-monitor-eye</v-icon> in the sidebar to view your
            monitored cron jobs. From here, you can add new monitors for you cron jobs - which is a
            simple as giving the monitor a name, an expected duration and a grace period.
          </v-card>
        </template>

        <!-- eslint-disable-next-line vue/valid-v-slot -->
        <template v-slot:item.2>
          <v-card title="Integration">
            CronMon provides a simple REST API to integrate with your existing applications, which
            is as simple as two API calls: one to signify that a new job has started and another to
            say that it's finished.
            <v-alert type="info" class="mt-3">
              In the future, CronMon will also provide libraries in various programming languages to
              make integration even simpler.
            </v-alert>
          </v-card>
        </template>

        <!-- eslint-disable-next-line vue/valid-v-slot -->
        <template v-slot:item.3>
          <v-card title="Go go go!">
            Start running your jobs! CronMon will then take care of the rest.
          </v-card>
        </template>
      </v-stepper>

      To get started, click on the <v-icon>mdi-monitor-eye</v-icon> icon in the sidebar to view your
      monitored cron jobs.
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
