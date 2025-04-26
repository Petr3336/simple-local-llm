// ChatMessage.vue
<template>
  <div
    class="d-flex"
    :class="isUser ? 'justify-end' : 'justify-start'"
  >
    <v-sheet
      :class="(isUser ? 'bg-grey-darken-4' : 'bg-grey-darken-3')+ ' '"
      class="rounded-xl pa-4 mx-8 my-1 my-md-2"
      max-width="70%"
    >
      <md-preview
        :model-value="message.content"
        class="custom-md-view"
        theme="dark"
        language="ru-RU"
        :code-foldable="false"
        no-code-header
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

const props = defineProps<{ message: ChatMessage }>()
const isUser = computed(() => props.message.role === 'user')
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
  