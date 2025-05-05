<template>
  <v-navigation-drawer
    v-model="sideBarState"
    app
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
          />
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
          :to="chat.id"
          @click="selectChat(chat.id)"
          @contextmenu.prevent="openContextMenu($event, chat)"
        >
          <v-list-item-title>{{ chat.title }}</v-list-item-title>
          <v-list-item-subtitle>
            {{ formatDate(chat.createdAt) }}
          </v-list-item-subtitle>

          <!-- Меню действий через кнопку -->
          <template #append>
            <v-menu
              :model-value="openMenu === `item-${chat.id}`"
              @update:modelValue="val => openMenu = val ? `item-${chat.id}` : null"
              location-strategy="connected"
              offset-y
              max-width="200"
            >
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
    />

    <!-- Контекстное меню по правому клику -->
    <v-menu
      :model-value="openMenu === 'context'"
      @update:modelValue="val => openMenu = val ? 'context' : null"
      :target="[context.x, context.y]"
      location-strategy="connected"
      max-width="200"
    >
      <v-list>
        <v-list-item @click="onContextRename">
          <v-list-item-title>Переименовать чат</v-list-item-title>
        </v-list-item>
        <v-list-item @click="onContextDelete">
          <v-list-item-title>Удалить чат</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>
  </v-navigation-drawer>
</template>

<script lang="ts" setup>
import { ref } from "vue"
import { storeToRefs } from "pinia"
import { useAppStore } from "@/stores/app"
import { useChatStore, type ChatSession } from "@/stores/chat"

import RenameChatDialog from "./RenameChatDialog.vue"
import SearchChatDialog from "./SearchChatDialog.vue"

// Sidebar
const AppStore = useAppStore()
const { sideBarState } = storeToRefs(AppStore)

// Chat store
const chatStore = useChatStore()
const { sortedChatSessions, activeChatId } = storeToRefs(chatStore)
const { selectChat, deleteChat, createNewChat, renameChat } = chatStore

// Переименование
const showRenameDialog = ref(false)
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
const showSearchDialog = ref(false)

// Форматирование даты
function formatDate(ts: number) {
  const d = new Date(ts)
  return (
    d.toLocaleDateString("ru-RU") +
    " " +
    d.toLocaleTimeString("ru-RU", {
      hour: "2-digit",
      minute: "2-digit",
    })
  )
}

// Контекстное меню
const context = ref<{ x: number; y: number; chat: ChatSession | null }>({
  x: 0, y: 0, chat: null
})
function openContextMenu(event: MouseEvent, chat: ChatSession) {
  context.value = { x: event.clientX, y: event.clientY, chat }
  openMenu.value = "context"
}

// Общее состояние открытого меню
const openMenu = ref<string | null>(null)

// Обработчики контекстного меню
function onContextRename() {
  if (context.value.chat) openRenameDialog(context.value.chat)
  openMenu.value = null
}
function onContextDelete() {
  if (context.value.chat) deleteChat(context.value.chat.id)
  openMenu.value = null
}
</script>
