<template>
  <v-theme-provider class="app-container" :theme="appliedTheme.name" with-background>
    <v-app>
      <v-navigation-drawer :rail="rail">
        <v-list-item class="logo pa-0" :height="80">
          <v-img
            :width="rail ? 40 : 300"
            :height="rail ? 40 : 220"
            cover
            aspect-ratio="16/9"
            :src="rail ? CronMonIcon : CronMonLogo"
          ></v-img>
        </v-list-item>
        <v-divider></v-divider>

        <v-list density="compact" nav>
          <v-list-item link prepend-icon="mdi-home" title="Home" to="/"></v-list-item>
          <v-list-item
            link
            prepend-icon="mdi-monitor-eye"
            title="Monitors"
            to="/monitors"
          ></v-list-item>
          <v-list-item link prepend-icon="mdi-bookshelf" title="Docs" to="/docs"></v-list-item>
        </v-list>
      </v-navigation-drawer>

      <v-main>
        <v-toolbar density="compact">
          <v-btn density="comfortable" @click="rail = !rail" icon="mdi-dots-vertical"></v-btn>
          <v-spacer></v-spacer>
          <v-btn density="comfortable" @click="toggleTheme" :icon="appliedTheme.icon"></v-btn>
        </v-toolbar>
        <RouterView class="mb-3" />
        <v-footer app absolute class="text-center d-flex flex-column">
          <a href="https://github.com/howamith/cron-mon" target="_blank" rel="noopener noreferrer">
            <v-btn flat density="comfortable" icon>
              <template v-slot:default>
                <GitHubIcon :dark="appliedTheme.name === ThemeName.Dark" />
              </template>
            </v-btn>
          </a>
          <div>&copy; {{ new Date().getFullYear() }} — <strong>CronMon</strong></div>
        </v-footer>
      </v-main>
    </v-app>
  </v-theme-provider>
</template>

<script setup lang="ts">
import CronMonLogo from '@/assets/logo.svg'
import CronMonIcon from '@/assets/icon.svg'
import GitHubIcon from '@/components/icons/GitHub.vue'
import { ref, computed } from 'vue'
import { THEMES, getThemeName, setThemeName, ThemeName } from './utils/theme'

const rail = ref(false)
const theme = ref(THEMES)
const themeName = ref(getThemeName())
const appliedTheme = computed(() => theme.value[themeName.value])

function toggleTheme() {
  themeName.value = themeName.value === ThemeName.Dark ? ThemeName.Light : ThemeName.Dark
  setThemeName(themeName.value)
}
</script>

<style scoped>
.logo {
  display: flex;
  justify-content: center;
}
</style>
