import { defineStore } from 'pinia'
import { nanoid } from 'nanoid'
import { invoke } from '@tauri-apps/api/core'

export type ChatRole = 'user' | 'assistant' | 'tool'

export interface ChatMessage {
  role: ChatRole
  content: string
  tool_call_id?: string
}

export interface ChatSession {
  id: string
  title: string
  messages: ChatMessage[]
  createdAt: number
}

export interface LLMOptions {
  num_gpu: number
  num_ctx: number
  functions: string[]
  stream: boolean
}

export const useChatStore = defineStore('chat', {
  state: () => {
    // Создаем первый чат сразу при инициализации хранилища
    const firstChat: ChatSession = {
      id: nanoid(),
      title: 'Первый чат',
      messages: [],
      createdAt: Date.now()
    }
    return {
      chatSessions: [firstChat] as ChatSession[],
      activeChatId: firstChat.id as string | null,
    }
  },
  getters: {
    sortedChatSessions(state): ChatSession[] {
      return [...state.chatSessions].sort((a, b) => b.createdAt - a.createdAt)
    },
    activeChat(state): ChatSession | null {
      return state.chatSessions.find(c => c.id === state.activeChatId) || null
    },
    messages(): ChatMessage[] {
      return this.activeChat ? this.activeChat.messages : []
    }
  },
  actions: {
    createNewChat(title = 'Новый чат') {
      const id = nanoid()
      const chat: ChatSession = {
        id,
        title,
        createdAt: Date.now(),
        messages: []
      }
      this.chatSessions.push(chat)
      this.activeChatId = id
    },
    deleteChat(id: string) {
      this.chatSessions = this.chatSessions.filter(c => c.id !== id)
      if (this.activeChatId === id) {
        const last = this.chatSessions[this.chatSessions.length - 1]
        this.activeChatId = last ? last.id : null
      }
    },
    selectChat(id: string) {
      this.activeChatId = id
    },
    renameChat(id: string, newTitle: string) {
      const chat = this.chatSessions.find(c => c.id === id)
      if (chat) {
        chat.title = newTitle
      }
    },
    addMessage(message: ChatMessage) {
      if (this.activeChat) {
        this.activeChat.messages.push(message)
      }
    },
    clearActiveChatMessages() {
      if (this.activeChat) {
        this.activeChat.messages = []
      }
    },
    async runModel(
      providerName: string,
      model: string,
      prompt: string,
      options: LLMOptions
    ) {
      // Клонируем options, чтобы не изменять оригинальный объект
      const clonedOptions = { ...options };
    
      // Если провайдер — ollama и есть доступные для вызова функции, отключаем стриминг
      if (providerName === "ollama" && options.functions.length > 0) {
        clonedOptions.stream = false;
      }
    
      // Формируем массив сообщений для отправки (сохраняем только role и content)
      const msgs = this.activeChat
        ? this.activeChat.messages.map(m => ({ role: m.role, content: m.content }))
        : [];
    
      try {
        await invoke("run_model", {
          providerName,
          model,
          messages: msgs,
          options: clonedOptions,
          chatId: this.activeChatId
        });
      } catch (error) {
        console.error("Ошибка при запуске модели:", error);
      }
    },
    appendMessageByRole(chatId: string, role: ChatRole, content: string, tool_call_id?: string) {
      const session = this.chatSessions.find(c => c.id === chatId);
      if (!session) return;
    
      // Специальная логика для tool-ответов
      if (role === 'tool' && tool_call_id) {
        // Ищем последнее сообщение tool с тем же ID
        const lastToolMsg = [...session.messages]
          .reverse()
          .find(m => m.role === 'tool' && m.tool_call_id === tool_call_id);
        if (lastToolMsg) {
          lastToolMsg.content = content;
          return;
        }
        // Иначе добавляем новое tool-сообщение
        session.messages.push({ role, content, tool_call_id });
        return;
      }
    
      // Обычная логика для assistant
      if (role === 'assistant') {
        const lastMsg = session.messages[session.messages.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.content += content;
        } else {
          session.messages.push({ role, content });
        }
        return;
      }
    
      // На случай других ролей — просто добавим новое сообщение
      session.messages.push({ role, content });
    },    
    async stopModel(
      providerName: string,
      model: string,
      prompt: string,
      options: LLMOptions
    ) {
      try {
        await invoke("stop_model", {
          providerName,
          model,
          prompt,
          options,
        })
      } catch (error) {
        console.error("Ошибка при остановке модели:", error)
      }
    },
  }
})
