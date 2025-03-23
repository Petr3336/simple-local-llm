<template>
  <v-container>
    <v-row class="mt-2">
      <v-col cols="12">
        <v-card class="pa-4">
          <v-card-title>Чат с моделью</v-card-title>
          <v-card-text>
            <!-- Выпадающий список для выбора модели -->
            <v-autocomplete
              v-model="modelParams.model"
              :items="modelsList"
              label="Выберите модель"
              dense
            >
              <template #no-data>
                <p class="px-4">
                  Данная модель не обнаружена среди установленных, при попытке её использования произойдет попытка её загрузки с серверов ollama
                </p>
              </template>
            </v-autocomplete>
            <v-text-field
              v-model="modelParams.prompt"
              label="Введите сообщение"
              dense
            />
            <v-btn color="primary" @click="startOllama">
              Отправить
            </v-btn>
            <!-- Индикатор загрузки, отображается до первого ответа -->
            <div v-if="isLoading" class="d-flex justify-center my-2">
              <v-progress-circular indeterminate color="primary" />
            </div>
          </v-card-text>
          <pre>{{ output }}</pre>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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
const isLoading = ref(false);

// Функция для вызова Tauri-команды с параметрами
function startOllama() {
  isLoading.value = true; // включаем индикатор загрузки
  output.value = "";
  invoke("run_ollama", {
    model: modelParams.value.model,
    prompt: modelParams.value.prompt,
    options: modelParams.value.options,
  }).catch((e: unknown) => console.error(e));
}

// Подписка на событие для получения потоковых данных
onMounted(() => {
  listen("ollama-output", (event) => {
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

  // Получение списка установленных моделей
  invoke<string[]>("get_installed_models")
    .then((models) => {
      modelsList.value = models;
      if (models.length > 0 && !modelParams.value.model) {
        modelParams.value.model = models[0];
      }
    })
    .catch((error: unknown) => {
      console.error("Ошибка получения моделей:", error);
    });
});
</script>
