// [id].vue
<template>
  <v-virtual-scroll
    ref="virtualScroll"
    class="chat-container"
    :items="displayMessages"
    item-height="auto"
    height="100%"
  >
    <template #default="{ item }">
      <ChatMessage
        :message="item.message"
        :original-index="item.originalIndex"
        @delete-message="handleDeleteMessage"
      />
    </template>
  </v-virtual-scroll>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useChatStore, type ChatMessage as ChatMessageType } from '@/stores/chat'
import ChatMessage from '../components/ChatMessage.vue'
import { storeToRefs } from 'pinia'


const chatStore = useChatStore()

const { activeChatId } = storeToRefs(chatStore)

const messages = computed(() => {
  const session = chatStore.chatSessions.find(c => c.id === activeChatId.value)
  return session ? session.messages : []
})

const displayMessages = computed(() => {
  const result: { message: ChatMessageType; originalIndex: number }[] = []
  messages.value.forEach((msg, idx) => {
      result.push({ message: msg, originalIndex: idx })
  })
  return result
})

function handleDeleteMessage(index: number) {
  const session = chatStore.chatSessions.find(c => c.id === activeChatId.value)
  if (!session) return
    // Просто удаляем одно сообщение
    session.messages.splice(index, 1)
}


const virtualScroll = ref<InstanceType<typeof import('vuetify/components').VVirtualScroll>>()

watch(messages, () => {
  nextTick(() => {
    virtualScroll.value?.scrollToIndex(messages.value.length - 1)
  })
})
</script>

<style scoped>
.chat-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
