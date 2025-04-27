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
import { useRoute } from 'vue-router'
import { useChatStore, type ChatMessage as ChatMessageType } from '@/stores/chat'
import ChatMessage from '../components/ChatMessage.vue'

const chatStore = useChatStore()
const route = useRoute()
const chatId = computed(() => chatStore.activeChatId ?? (route.params.id as string))

onMounted(() => {
  const idParam = route.params.id as string | undefined
  if (idParam && idParam !== chatStore.activeChatId) {
    chatStore.selectChat(idParam)
  }
})

const messages = computed(() => {
  const session = chatStore.chatSessions.find(c => c.id === chatId.value)
  return session ? session.messages : []
})

const displayMessages = computed(() => {
  const result: { message: ChatMessageType; originalIndex: number }[] = []
  messages.value.forEach((msg, idx) => {
    if (msg.role === 'tool' && result.length > 0) {
      // Заменяем последнее сообщение на tool, но оставляем индекс того сообщения
      result[result.length - 1] = { message: msg, originalIndex: result[result.length - 1].originalIndex }
    } else {
      result.push({ message: msg, originalIndex: idx })
    }
  })
  return result
})

function handleDeleteMessage(index: number) {
  const session = chatStore.chatSessions.find(c => c.id === chatId.value)
  if (!session) return

  const msg = session.messages[index]

  if (msg.role === 'tool') {
    // Удаляем tool и предыдущее сообщение
    session.messages.splice(index - 1, 2)
  } else {
    // Просто удаляем одно сообщение
    session.messages.splice(index, 1)
  }
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
