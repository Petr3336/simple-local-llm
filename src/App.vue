<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import { attachConsole } from '@tauri-apps/plugin-log';
onMounted(() => {
  attachConsole();
})
</script>
