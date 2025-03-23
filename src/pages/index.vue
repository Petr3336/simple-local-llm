<template>
  <v-container>
    <v-row class="mt-2">
      <v-col cols="12">
        <v-card class="pa-4">
          <v-card-title>Чат с моделью</v-card-title>
          <v-card-text>
            <v-text-field
              v-model="modelParams.model"
              label="Введите название модели"
            />
            <v-text-field
              v-model="modelParams.prompt"
              label="Введите сообщение"
            />
            <v-btn
              color="primary"
              @click="startOllama"
            >
              Отправить
            </v-btn>
          </v-card-text>
          <pre>{{ output }}</pre>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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
  model: '  ',
  prompt: 'Say "hello world"!',
  options: {
    num_gpu: 100,
    num_ctx: 2048,
  },
});

const output = ref('');

// Функция для вызова Tauri-команды с параметрами
function startOllama() {
  invoke('run_ollama', {
    model: modelParams.value.model,
    prompt: modelParams.value.prompt,
    options: modelParams.value.options,
  }).catch((e: unknown) => console.error(e));
}

// Подписка на событие для получения потоковых данных

onMounted(() => {
  listen('ollama-output', (event) => {
    output.value += JSON.parse(event.payload as string).response;
  });
});
</script>
