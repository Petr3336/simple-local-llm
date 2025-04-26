<template>
  <v-container class="chat-container">
    <!-- Компонент превью файлов -->
    <FilePreviews
      :files="files"
      @remove-file="removeFile"
    />

    <!-- Основное поле ввода -->
    <div class="input-wrapper">
      <MdEditor
        v-model="modelParams.prompt"
        theme="dark"
        :language="'en-US'"
        :preview="false"
        :toolbars="toolbars"
        :no-footer="true"
        :show-words-count="false"
        class="custom-md-editor"
        :style="{
          borderRadius: '8px',
          height: '200px',
          width: '100%',
        }"
      />

      <!-- Панель управления -->
      <div class="controls">
        <div class="left-controls">
          <v-menu
            v-model="showMenu"
            :close-on-content-click="false"
          >
            <template #activator="{ props }">
              <v-btn
                v-bind="props"
                icon="mdi-menu"
                size="small"
                variant="text"
                border
              />
            </template>
            <v-card class="menu-card">
              <v-list>
                <v-list-item>
                  <template #prepend>
                    <v-switch
                      v-model="systemSearchEnabled"
                      class="pr-3"
                      hide-details
                      @click.stop
                    />
                  </template>
                  <v-list-item-title>Поиск по системе</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-card>
          </v-menu>
          <v-btn
            icon="mdi-file-upload-outline"
            size="small"
            variant="text"
            border
            @click="triggerFileInput"
          />
        </div>

        <!-- Кнопка отправки -->
        <v-btn
          icon="mdi-send-variant"
          size="small"
          color="#1976d2"
          variant="flat"
          @click="runModel"
        />
      </div>
    </div>

    <!-- Скрытый input для загрузки файлов -->
    <input
      ref="fileInput"
      type="file"
      multiple
      style="display: none"
      @change="handleFileUpload"
    >
  </v-container>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { MdEditor } from "md-editor-v3";
import "md-editor-v3/lib/style.css";
import { useChatStore, type LLMOptions, type ChatRole } from '@/stores/chat'
import { useAppStore } from '@/stores/app'
import { listen } from "@tauri-apps/api/event";
import { storeToRefs } from "pinia";

// Chat Store
const chatStore = useChatStore()
const appStore = useAppStore()

const { currentProvider, currentModel } = storeToRefs(appStore)

const files = ref<
  Array<{ name: string; type: string; preview: string; file: File }>
>([]);
const fileInput = ref<HTMLInputElement | null>(null);
const systemSearchEnabled = ref(false);
const showMenu = ref(false);
const toolbars = [
  "bold",
  "-",
  "title",
  "unorderedList",
  "orderedList",
  "-",
  "codeRow",
  "code",
  "link",
  "table",
  "mermaid",
  "katex",
  "-",
  "revoke",
  "next",
  "=",
  "preview",
  "fullscreen",
];

const isLoading = ref(false);
const streamBuffer = ref("");
const output = ref("");

interface ModelParameters {
    model: string;
    prompt: string;
    options: LLMOptions;
  }

const modelParams = ref<ModelParameters>({
  model: currentModel.value, // значение будет установлено после загрузки списка моделей
  prompt: "",
  options: {
    num_gpu: 100,
    num_ctx: 4096,
    functions: [],
    stream: true,
  },
});

// Выбор файла
const triggerFileInput = () => fileInput.value?.click()

// Загрузка файлов
const handleFileUpload = (e: Event) => {
  const input = e.target as HTMLInputElement
  if (!input.files) return
  Array.from(input.files).forEach(file => {
    const reader = new FileReader()
    reader.onload = () => {
      files.value.push({ name: file.name, type: file.type, preview: reader.result as string, file })
    }
    file.type.startsWith('image/') ? reader.readAsDataURL(file) : reader.readAsText(file)
  })
}

// Удаление превью
const removeFile = (idx: number) => files.value.splice(idx, 1)

/**
   * Отправка запроса к модели через хранилище.
   * При вызове chatStore.runModel автоматически добавится сообщение пользователя в историю.
   */
async function runModel() {
  isLoading.value = true;
  output.value = "";
  chatStore.addMessage({ role: 'user', content: modelParams.value.prompt })
  await chatStore.runModel(
    currentProvider.value,
    modelParams.value.model,
    modelParams.value.prompt,
    modelParams.value.options
  );
}

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
        modelParams.value.options.functions = [];
        // Запускаем модель снова
        chatStore.runModel(
          currentProvider.value,
          modelParams.value.model,
          "На основе предыдущего ответа, в котором была использована функция " + tool_call_id + ", выполни следующую задачу: " + modelParams.value.prompt,
          modelParams.value.options
        );
      }
    } catch (e) {
      console.error("Ошибка обработки model-output", e);
    }
  });
});
</script>

<style scoped>
.chat-container {
  border: 1px solid rgba(255, 255, 255, 0.12);
  background-color: rgb(33, 33, 33);
  border-radius: 8px;
  padding: 12px;
  transition: all 0.3s ease;
}

.custom-md-editor {
  --md-bk-color: rgb(25, 25, 25);
  --md-color: #ffffff;
  --md-border-color: rgba(255, 255, 255, 0.12);
}

.input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.left-controls {
  display: flex;
  gap: 8px;
}

.md-editor-fullscreen {
  position: fixed !important;
  top: 0 !important;
  left: 0 !important;
  width: 100vw !important;
  height: 100vh !important;
  z-index: 9999 !important;
}
</style>
