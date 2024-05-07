<template>
  <v-theme-provider class="app-container" :theme="appliedTheme.name" with-background>
    <v-app>
      <v-navigation-drawer>
        <v-list-item :height="80">
          <v-img :width="300" :height="220" cover aspect-ratio="16/9" :src="CronMonLogo"></v-img>
        </v-list-item>
        <v-divider></v-divider>
        <v-list-item link title="Home" to="/"></v-list-item>
        <v-list-item link title="App" to="/app"></v-list-item>
        <v-list-item link title="About" to="/about"></v-list-item>
      </v-navigation-drawer>

      <v-main>
        <v-toolbar density="compact">
          <v-spacer></v-spacer>
          <v-btn @click="toggleTheme">
            <v-icon :icon="appliedTheme.icon"></v-icon>
          </v-btn>
        </v-toolbar>
        <RouterView />
      </v-main>
    </v-app>
  </v-theme-provider>
</template>

<script setup lang="ts">
import CronMonLogo from '@/assets/logo.svg'
import { ref, computed } from 'vue'
import { THEMES, getThemeName, setThemeName, ThemeName } from './utils/theme'

const theme = ref(THEMES)
const themeName = ref(getThemeName())
const appliedTheme = computed(() => {
  console.log('themeName.value', themeName.value)
  return theme.value[themeName.value]
})

function toggleTheme() {
  themeName.value = themeName.value === ThemeName.Dark ? ThemeName.Light : ThemeName.Dark
  setThemeName(themeName.value)
}
</script>
