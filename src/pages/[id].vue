// [id].vue
<template>
  <v-virtual-scroll
    ref="virtualScroll"
    class="chat-container"
    :items="messages"
    item-height="auto"
    height="100%"
  >
    <template #default="{ item }">
      <ChatMessage :message="item" />
    </template>
  </v-virtual-scroll>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useChatStore } from '@/stores/chat'
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
