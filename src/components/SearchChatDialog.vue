<!-- SearchChatDialog.vue -->
<template>
  <v-dialog
    v-model="dialog"
    max-width="500"
  >
    <v-card>
      <v-card-title>Поиск по чатам</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="internalQuery"
          label="Введите запрос"
          autofocus
          @input="onInput"
        />

        <v-list
          v-if="results.length"
          nav
          dense
        >
          <v-list-item
            v-for="item in results"
            :key="item.chatId + '-' + item.msgIndex"
            @click="selectResult(item.chatId)"
          >
            <v-list-item-content>
              <v-list-item-title class="font-weight-medium">
                {{ item.chatTitle }}
              </v-list-item-title>
              <v-list-item-subtitle>
                <span v-html="item.highlightedMessage" />
              </v-list-item-subtitle>
            </v-list-item-content>
          </v-list-item>
        </v-list>

        <div v-else-if="internalQuery">
          Ничего не найдено
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn
          text
          @click="onCancel"
        >
          Отмена
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { defineProps, defineEmits, ref, watch, computed } from "vue"
import { useChatStore } from "@/stores/chat"

const props = defineProps<{ modelValue: boolean }>()
const emits = defineEmits<{
    (e: "update:modelValue", val: boolean): void
}>()

// локальный ref для v-model
const dialog = ref(props.modelValue)
watch(() => props.modelValue, v => (dialog.value = v))
watch(dialog, v => emits("update:modelValue", v))

// строка запроса
const internalQuery = ref("")

// чат-хранилище
const chatStore = useChatStore()

// результаты поиска
interface SearchItem {
    chatId: string
    chatTitle: string
    msgIndex: number
    highlightedMessage: string
}
const results = computed<SearchItem[]>(() => {
    const q = internalQuery.value.trim().toLowerCase()
    if (!q) return []

    const out: SearchItem[] = []
    for (const chat of chatStore.chatSessions) {
        for (let i = 0; i < chat.messages.length; i++) {
            const message = chat.messages[i]
            // пропускаем инструментальные вызовы
            if (message.role === "tool") continue

            const msg = message.content
            const idx = msg.toLowerCase().indexOf(q)
            if (idx !== -1) {
                // подготавливаем HTML с подсветкой
                const before = escapeHtml(msg.slice(0, idx))
                const match = escapeHtml(msg.slice(idx, idx + q.length))
                const after = escapeHtml(msg.slice(idx + q.length))
                out.push({
                    chatId: chat.id,
                    chatTitle: chat.title,
                    msgIndex: i,
                    highlightedMessage: `${before}<mark>${match}</mark>${after}`,
                })
            }
        }
    }
    return out
})

// эскейпинг спецсимволов для безопасного v-html
function escapeHtml(s: string) {
    return s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;")
}

// при вводе — просто обновляем computed
function onInput() {
    /* ничего не нужно */
}

// при клике на результат — переключаем чат и закрываем диалог
function selectResult(chatId: string) {
    chatStore.selectChat(chatId)
    dialog.value = false
}

function onCancel() {
    dialog.value = false
}
</script>

<style scoped>
/* если хотите чуть ярче подсветить */
mark {
    background-color: yellow;
    color: inherit;
}
</style>