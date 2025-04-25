<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import { attachConsole } from '@tauri-apps/plugin-log';
import { useChatStore } from "@/stores/chat";

onMounted(() => {
  attachConsole();
})

const chatStore = useChatStore();
chatStore.$tauri.start();
</script>
