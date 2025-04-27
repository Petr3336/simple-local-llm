<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import { attachConsole } from '@tauri-apps/plugin-log';
import { useChatStore } from "@/stores/chat";
import { useAppStore } from "@/stores/app";

onMounted(() => {
  attachConsole();
})

const chatStore = useChatStore();
const appStore = useAppStore();
chatStore.$tauri.start();
appStore.$tauri.start();
</script>

<style>
html { overflow-y: auto !important }
</style>
