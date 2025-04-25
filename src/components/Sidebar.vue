<template>
  <v-navigation-drawer
    v-model="isOpen"
    location="left"
    width="300"
  >
    <v-list>
      <!-- Заголовок с кнопками создания и поиска -->
      <v-list-item>
        <v-list-item-title class="text-h6">
          Чаты
        </v-list-item-title>
        <template #append>
          <v-btn
            size="small"
            variant="text"
            icon="mdi-plus"
            @click="createNewChat()"
          />
          <v-btn
            size="small"
            variant="text"
            icon="mdi-magnify"
            @click="showSearchDialog = true"
          >
            <v-icon />
          </v-btn>
        </template>
      </v-list-item>

      <v-divider />

      <!-- Список чатов -->
      <v-list
        nav
        dense
      >
        <v-list-item
          v-for="chat in sortedChatSessions"
          :key="chat.id"
          :active="chat.id === activeChatId"
          @click="selectChat(chat.id)"
        >
          <v-list-item-title>{{ chat.title }}</v-list-item-title>
          <v-list-item-subtitle>
            {{ formatDate(chat.createdAt) }}
          </v-list-item-subtitle>

          <!-- Меню действий: переименовать и удалить -->
          <template #append>
            <v-menu>
              <template #activator="{ props }">
                <v-btn
                  size="small"
                  variant="text"
                  icon="mdi-dots-vertical"
                  v-bind="props"
                />
              </template>
              <v-list>
                <v-list-item @click="openRenameDialog(chat)">
                  <v-list-item-title>Переименовать чат</v-list-item-title>
                </v-list-item>
                <v-list-item @click="deleteChat(chat.id)">
                  <v-list-item-title>Удалить чат</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-menu>
          </template>
        </v-list-item>
      </v-list>
    </v-list>
    <!-- Диалоги -->
    <rename-chat-dialog
      v-model="showRenameDialog"
      :title="renameTarget?.title || ''"
      @save="onRenameSave"
    />
    <search-chat-dialog
      v-model="showSearchDialog"
      @search="onSearch"
    />
  </v-navigation-drawer>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { storeToRefs } from "pinia";
import { useSidebarStore } from "@/stores/sidebar";
import { useChatStore, type ChatSession } from "@/stores/chat";

import RenameChatDialog from './RenameChatDialog.vue'
import SearchChatDialog from './SearchChatDialog.vue'

// Sidebar store
const sidebarStore = useSidebarStore();
const { isOpen } = storeToRefs(sidebarStore);

// Chat store
const chatStore = useChatStore();
const { sortedChatSessions, activeChatId } = storeToRefs(chatStore);
const { selectChat, deleteChat, createNewChat, renameChat } = chatStore;

// Для диалогов
const showRenameDialog = ref(false)
const showSearchDialog = ref(false)
const renameTarget = ref<ChatSession | null>(null)

function openRenameDialog(chat: ChatSession) {
  renameTarget.value = chat
  showRenameDialog.value = true
}
function onRenameSave(newTitle: string) {
  if (renameTarget.value) {
    renameChat(renameTarget.value.id, newTitle)
  }
}

// Поиск
function onSearch(query: string) {
  // TODO: реализовать фильтрацию sortedChatSessions по query
  console.log('Search for:', query)
}

// Форматирование даты
function formatDate(ts: number) {
  const d = new Date(ts);
  return (
    d.toLocaleDateString("ru-RU") +
    " " +
    d.toLocaleTimeString("ru-RU", {
      hour: "2-digit",
      minute: "2-digit",
    })
  );
}
</script>
