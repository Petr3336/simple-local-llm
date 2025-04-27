// ChatMessage.vue
<template>
  <div
    class="d-flex align-center"
    :class="isUser ? 'justify-end' : 'justify-start'"
  >
    <v-sheet
      :class="(isUser ? 'bg-grey-darken-4' : (isTool ? 'bg-indigo-darken-4' : 'bg-grey-darken-3'))+ ' '"
      class="rounded-xl pa-4 mx-8 my-1 my-md-2 d-flex flex-column align-end"
      max-width="70%"
    >
      <md-preview
        :model-value="(isTool ? formatToolCall() : message.content)"
        class="custom-md-view"
        theme="dark"
        language="ru-RU"
        :code-foldable="false"
        no-code-header
      />
      <v-btn
        icon="mdi-close"
        size="x-small"
        variant="text"
        color="error"
        hint="Удалить сообщение"
        @click.stop="$emit('delete-message', originalIndex)"
      />
    </v-sheet>
  </div>
</template>

<script setup lang="ts">
import { defineProps, computed } from 'vue'
import type { ChatMessage } from '@/stores/chat'
import { MdPreview } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import '../plugins/md-editor-config';

const props = defineProps<{
  message: ChatMessage,
  originalIndex: number
}>()

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const emit = defineEmits<{
  (e: 'delete-message', index: number): void
}>()

const isUser = computed(() => props.message.role === 'user')
const isTool = computed(() => props.message.role === 'tool')
function formatToolCall() {
  return 'Произведен запуск функции: ' + props.message.tool_call_id
}
</script>

<style>
  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  .md-editor-code-head {
    display: block !important;
  }
  .md-editor-code-action {
    justify-content: space-between;
  }
  .md-editor-code-lang {
    margin-left: 10px;
  }
  .md-editor-copy-button {
    padding-right: 0px !important;
  }
  .md-editor-code-flag {
    display: none;
  }
  .custom-md-view {
    --md-color: #ffffff;
    --md-bk-color: transparent;
  }
  p:first-of-type {
    margin-top: 0px !important;
  }
</style>
  