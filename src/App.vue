<template>
  <v-app>
    <router-view />
  </v-app>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import { attachConsole } from '@tauri-apps/plugin-log';
import { useChatStore, type ChatRole } from "@/stores/chat";
import { useAppStore } from "@/stores/app";
import { listen } from "@tauri-apps/api/event";
import { storeToRefs } from "pinia";
import router from "./router";

onMounted(() => {
  attachConsole();
})

const chatStore = useChatStore();
const appStore = useAppStore();
chatStore.$tauri.start();
appStore.$tauri.start().then(() => {
  if (appStore.$state.initialSetup) {
    router.push('/initial-setup')
    initialSetup.value = false
  }
});

const { currentProvider, currentModel, initialSetup } = storeToRefs(appStore)
const { runParams } = storeToRefs(chatStore)
const streamBuffer = ref("");
const isLoading = ref(false);

onMounted(() => {
  // Обработчик для стримингового вывода
  listen("model-stream-output", (event) => {
    console.log("model-stream-output", event);
    const chatStore = useChatStore();
    if (isLoading.value) {
      isLoading.value = false;
    }
    try {
      const eventData = JSON.parse(event.payload as string);
      const chatId = eventData.chat_id;
      const output = eventData.output;

      // Добавляем новую часть в буфер
      streamBuffer.value += output;

      // Пробуем найти полные JSON-сообщения
      const messages = streamBuffer.value.split('\n').filter(msg => msg.trim());

      for (const message of messages) {
        try {
          const parsedMessage = JSON.parse(message);
          if (parsedMessage.message) {
            const msg = parsedMessage.message;
            const role = msg.role || "assistant";
            const content = msg.content || "";
            const tool_call_id = msg.tool_call_id;

            chatStore.appendMessageByRole(chatId, role, content, tool_call_id);
          }
        } catch {
          // Если не удалось распарсить JSON, оставляем в буфере
          continue;
        }
      }

      // Очищаем буфер от обработанных сообщений
      streamBuffer.value = streamBuffer.value.split('\n').slice(-1)[0];
    } catch (e) {
      console.error("Ошибка обработки model-stream-output", e);
    }
  });

  listen("model-output", (event) => {
    console.log("model-output", event);
    const chatStore = useChatStore();
    if (isLoading.value) {
      isLoading.value = false;
    }
    try {
      const eventData = JSON.parse(event.payload as string);
      const chatId = eventData.chat_id;
      const output = eventData.output;

      let role: ChatRole = "assistant";
      let content = "";
      let tool_call_id: string | undefined;

      if (typeof output === "object" && output.role && output.content) {
        role = output.role as ChatRole;
        content = output.content;
        tool_call_id = output.tool_call_id;

      } else if (typeof output === "object" && output.message && output.message.content != null) {
        const msg = output.message;
        role = msg.role || "assistant";
        content = msg.content;
        tool_call_id = msg.tool_call_id;
      } else {
        content = JSON.stringify(output);
      }

      chatStore.appendMessageByRole(chatId, role, content, tool_call_id);

      // Если это ответ от функции (tool), запускаем модель снова
      if (role === "tool" && tool_call_id) {
        // Отключаем функции для следующего запроса
        runParams.value.options.functions = [];
        // Запускаем модель снова
        chatStore.runModel(
          currentProvider.value,
          currentModel.value,
          "На основе предыдущего ответа, в котором была использована функция " + tool_call_id + ", выполни следующую задачу: " + runParams.value.prompt,
          runParams.value.options
        );
      }
    } catch (e) {
      console.error("Ошибка обработки model-output", e);
    }
  });
});
</script>

<style>
html { overflow-y: auto !important }
</style>
