<template>
  <v-container>
    <v-row class="mt-2">
      <v-col cols="12">
        <v-card class="pa-4">
          <v-card-title>Чат с моделью</v-card-title>
          <v-card-text>
            <!-- Выбор провайдера моделей -->
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
              <!-- Выпадающий список для выбора модели -->
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
            </v-form>
          </v-card-text>
          <div
            v-if="isLoading"
            class="d-flex justify-center my-4 mb-14"
          >
            <v-progress-circular
              indeterminate
              color="primary"
            />
          </div>
          <md-preview
            v-else
            v-model="output"
            theme="dark"
            class="px-16 md-preview mt-2"
            style="padding-bottom: 19.2px;"
            language="ru-RU"
            :code-foldable="false"
            no-code-header
          />
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


// Интерфейс для дополнительных параметров модели
interface LLMOptions {
  num_gpu: number;
  num_ctx: number;
  // Можно добавить дополнительные параметры по необходимости
}

// Интерфейс для объединения параметров модели
interface ModelParameters {
  model: string;
  prompt: string;
  options: LLMOptions;
}

// Инициализируем параметры модели
const modelParams = ref<ModelParameters>({
  model: "", // значение будет установлено после загрузки списка моделей
  prompt: 'Say "hello world"!',
  options: {
    num_gpu: 100,
    num_ctx: 2048,
  },
});

const output = ref("");
const modelsList = ref<string[]>([]);
const providersList = ref<string[]>([]);
const modelProvider = ref("");
const isLoading = ref(false);

// Функция для вызова Tauri-команды с параметрами
function runModel() {
  isLoading.value = true; // включаем индикатор загрузки
  output.value = "";
  invoke("run_model", {
    providerName: modelProvider.value,
    model: modelParams.value.model,
    prompt: modelParams.value.prompt,
    options: modelParams.value.options,
  }).catch((e: unknown) => console.error(e));
}

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
      getModels(); // обновим список после загрузки
    })
    .catch((error: unknown) => {
      console.error("Ошибка загрузки модели:", error);
    })
    .finally(() => {
      isLoading.value = false;
    });
}

// Удаление выбранной модели
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
      getModels(); // обновим список после удаления
    })
    .catch((error: unknown) => {
      console.error("Ошибка удаления модели:", error);
    })
    .finally(() => {
      isLoading.value = false;
    });
}

// Остановка выбранной модели
function stopModel() {
  invoke("stop_model", {
    providerName: modelProvider.value,
    model: modelParams.value.model,
    prompt: modelParams.value.prompt,
    options: modelParams.value.options,
  }).catch((e: unknown) => console.error(e));
}

// Подписка на событие для получения потоковых данных
onMounted(() => {
  listen("model-output", (event) => {
    console.log(event)
    // Как только приходит первый ответ — скрываем индикатор загрузки
    if (isLoading.value) {
      isLoading.value = false;
    }
    try {
      // Ожидается, что event.payload является JSON-строкой с полем response
      const parsed = JSON.parse(event.payload as string);
      output.value += parsed.response;
    } catch {
      // Если не удалось распарсить JSON, просто добавляем как текст
      output.value += event.payload as string;
    }
  });
  listen("stop-model", (event) => {
    console.log(event)
  });
  // Получение списка провайдеров моделей
  invoke<string[]>("get_available_providers").then((providers) => {
    console.log(providers)
    providersList.value = providers;
    if (providersList.value.length > 0) {
      modelProvider.value = providers[0]
      getModels();
    }
  })
  // Получение списка установленных моделей
});
</script>

<style>
pre {
  white-space: pre-wrap;
  /* Перенос строк при необходимости */
  word-wrap: break-word;
  /* Разрешить перенос длинных слов */
}

.md-editor {
  background-color: #00000000;
}

.md-editor-code-head {
  display: block !important;
}

.md-editor-code-action {
  justify-content: space-between
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