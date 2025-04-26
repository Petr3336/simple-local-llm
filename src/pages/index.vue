<template>
  <v-container>
    <v-row class="mt-2">
      <v-col cols="12">
        <v-card class="pa-4">
          <v-card-title>Чат с моделью</v-card-title>
          <v-card-text>
            <!-- Форма для отправки сообщения -->
            <v-form @submit.prevent="runModel">
              <v-select
                id="model-provider"
                v-model="modelProvider"
                :items="providersList"
                label="Выберите провайдера модели"
                dense
                @update:model-value="getModels"
              >
                <template #no-data>
                  <p class="px-4">
                    Данный провайдер моделей не найден
                  </p>
                </template>
              </v-select>
              <v-combobox
                id="model-name"
                v-model="modelParams.model"
                :items="modelsList"
                label="Выберите модель"
                dense
              />
              <v-text-field
                id="model-prompt"
                v-model="modelParams.prompt"
                label="Введите сообщение"
                dense
              />
              <v-btn
                id="generate"
                color="primary"
                type="submit"
                class="mr-4 mt-4"
              >
                Отправить
              </v-btn>
              <v-btn
                id="download-model"
                color="secondary"
                class="mr-4 mt-4"
                @click="downloadSelectedModel"
              >
                Загрузить модель
              </v-btn>
              <v-btn
                id="delete-model"
                color="error"
                class="mr-4 mt-4"
                @click="deleteSelectedModel"
              >
                Удалить выбранную модель
              </v-btn>
              <v-btn
                id="stop-model"
                color="error"
                class="mr-4 mt-4"
                @click="stopModel"
              >
                Остановить модель
              </v-btn>
              <v-switch
                v-model="modelParams.options.stream"
                color="primary"
              >
                Stream
              </v-switch>
            </v-form>
          </v-card-text>
          <!-- <div
              v-if="isLoading"
              class="d-flex justify-center my-4 mb-14"
            >
              <v-progress-circular
                :indeterminate="(downloadProgress === null)"
                :model-value="downloadProgress ?? 0"
                color="primary"
              />
            </div> -->
          <div v-if="messages && messages.length">
            <div
              v-for="(msg, index) in messages"
              :key="index"
              class="chat-message"
            >
              <md-preview
                v-if="msg.role == 'assistant'"
                v-model="msg.content"
                theme="dark"
                class="px-16 md-preview mt-2"
                style="padding-bottom: 19.2px;"
                language="ru-RU"
                :code-foldable="false"
                no-code-header
              />
              <p v-if="msg.role == 'user'">
                {{ msg.content }}
              </p>
            </div>
          </div>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>
  
  <script lang="ts" setup>
  import { ref, onMounted } from "vue";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { MdPreview } from 'md-editor-v3';
  import 'md-editor-v3/lib/style.css';
  import '../plugins/md-editor-config';
  
  // Импортируем хранилище и storeToRefs для реактивного доступа
  import { useChatStore, type LLMOptions, type ChatRole } from "@/stores/chat";
  import { storeToRefs } from "pinia";
  
  // Интерфейс для объединения параметров модели
  interface ModelParameters {
    model: string;
    prompt: string;
    options: LLMOptions;
  }
  
  const chatStore = useChatStore();
  const { messages } = storeToRefs(chatStore);
  
  const modelParams = ref<ModelParameters>({
    model: "", // значение будет установлено после загрузки списка моделей
    prompt: 'Say "hello world"!',
    options: {
      num_gpu: 100,
      num_ctx: 4096,
      functions: ["get_unix_time"],
      stream: true,
    },
  });
  
  const output = ref("");
  const modelsList = ref<string[]>([]);
  const providersList = ref<string[]>([]);
  const modelProvider = ref("");
  const isLoading = ref(false);
  const downloadProgress = ref<number | null>(null);
  const streamBuffer = ref(""); // Буфер для накопления частей JSON
  
  /**
   * Отправка запроса к модели через хранилище.
   * При вызове chatStore.runModel автоматически добавится сообщение пользователя в историю.
   */
  async function runModel() {
    isLoading.value = true;
    output.value = "";
    chatStore.addMessage({ role: 'user', content: modelParams.value.prompt })
    await chatStore.runModel(
      modelProvider.value,
      modelParams.value.model,
      modelParams.value.prompt,
      modelParams.value.options
    );
  }
  
  /**
   * Получение списка установленных моделей для выбранного провайдера.
   */
  function getModels() {
    invoke<string[]>("get_installed_models", { providerName: modelProvider.value })
      .then((models) => {
        modelsList.value = models;
        if (models.length > 0) {
          modelParams.value.model = models[0];
        }
      })
      .catch((error: unknown) => {
        console.error("Ошибка получения моделей:", error);
      });
  }
  
  /**
   * Загрузка выбранной модели.
   */
  function downloadSelectedModel() {
    if (!modelParams.value || !modelProvider.value) {
      console.warn("Не указан провайдер или модель для загрузки");
      return;
    }
    isLoading.value = true;
    invoke("download_model", {
      providerName: modelProvider.value,
      model: modelParams.value.model,
    })
      .then(() => {
        console.log("Модель успешно загружена");
        getModels();
      })
      .catch((error: unknown) => {
        console.error("Ошибка загрузки модели:", error);
      })
      .finally(() => {
        isLoading.value = false;
        downloadProgress.value = null;
      });
  }
  
  /**
   * Удаление выбранной модели.
   */
  function deleteSelectedModel() {
    if (!modelParams.value.model || !modelProvider.value) {
      console.warn("Не указан провайдер или модель для удаления");
      return;
    }
    isLoading.value = true;
    invoke("delete_model", {
      providerName: modelProvider.value,
      model: modelParams.value.model,
    })
      .then(() => {
        console.log("Модель успешно удалена");
        modelParams.value.model = "";
        getModels();
      })
      .catch((error: unknown) => {
        console.error("Ошибка удаления модели:", error);
      })
      .finally(() => {
        isLoading.value = false;
      });
  }
  
  /**
   * Остановка модели через хранилище.
   */
  async function stopModel() {
    await chatStore.stopModel(
      modelProvider.value,
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
              modelProvider.value,
              modelParams.value.model,
              "На основе предыдущего ответа, в котором была использована функция " + tool_call_id + ", выполни следующую задачу: " + modelParams.value.prompt,
              modelParams.value.options
            );
          }
      } catch (e) {
        console.error("Ошибка обработки model-output", e);
      }
    });
  
  
    // Слушатель для прогресса загрузки модели
    listen("model-download-progress", (event) => {
      const raw = parseFloat(event.payload as string);
      const progress = Math.min(Math.max(raw * 100, 0), 100);
      if (!isNaN(progress)) {
        downloadProgress.value = progress;
      }
    });
  
    listen("stop-model", (event) => {
      console.log(event);
    });
  
    // Получение списка провайдеров моделей
    invoke<string[]>("get_available_providers").then((providers) => {
      console.log(providers);
      providersList.value = providers;
      if (providersList.value.length > 0) {
        modelProvider.value = providers[0];
        getModels();
      }
    });
  });
  </script>
  
  <style>
  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  .md-editor {
    background-color: #00000000;
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
  </style>
  