<!-- eslint-disable vue/valid-v-slot -->
// index.vue
<template>
  <v-container>
    <v-stepper
      v-model="page"
      :items="['Настрой провайдера модели', 'Выбери модель', 'Скачай embedding модель']"
      next-text="Далее"
      prev-text="Назад"
    >
      <template #item.1>
        <v-card
          title="Шаг 1: Выбери провайдера модели"
          flat
        >
          <v-select
            v-model="currentProvider"
            :items="providersList"
            label="Провайдер"
            variant="outlined"
            class="mx-4"
            hide-details
            @update:model-value="{appStore.getModels(); currentModel=''}"
          />
        </v-card>
      </template>

      <template #item.2>
        <v-card
          title="Шаг 2: Выбери модель"
          flat
        >
          <v-select
            v-if="currentProvider=='llama.cpp'"
            v-model="currentModel"
            :items="availableModels"
            label="Провайдер"
            variant="outlined"
            class="mx-4"
            hide-details
          >
            <template #append-inner>
              <v-btn
                v-if="!installedModels.includes(currentModel)"
                icon="mdi-download"
                variant="text"
                @click="appStore.downloadSelectedModel(currentProvider, currentModel)"
              />
            </template>
          </v-select>
          <v-combobox
            v-else
            v-model="currentModel"
            :items="availableModels"
            label="Модель"
            variant="outlined"
            hide-details
          >
            <template #append-inner>
              <v-btn
                v-if="!availableModels.includes(currentModel)"
                icon="mdi-download"
                variant="text"
                @click="appStore.downloadSelectedModel(currentProvider, currentModel)"
              />
            </template>
          </v-combobox>
        </v-card>
      </template>
      <template #item.3>
        <v-card
          title="Шаг 3: Скачай embedding модель"
          flat
        >
          <div class="d-flex justify-center align-center">
            <v-progress-linear
              :model-value="embeddingDownload"
              height="12"
              class="mx-4"
            />
            <v-btn
              color="primary mx-4"
              @click="startDownloadModel('gpustack\/bge-m3-GGUF:bge-m3-Q4_0.gguf')"
            >
              Начать загрузку
            </v-btn>
          </div>
        </v-card>
      </template>
      <template #actions="{ prev, next}">
        <v-stepper-actions
          @click:prev="prev"
          @click:next="next"
        >
          <template #next="{ props }">
            <v-btn
              :disabled="(page == 3 && embeddingDownload != 100)"
              @click="nextPage(page, props)"
            />
          </template>
        </v-stepper-actions>
      </template>
    </v-stepper>
  </v-container>
</template>

<script setup lang="ts">
import router from '@/router';
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { storeToRefs,  } from 'pinia';

definePage({
  meta: {
    layout: 'initial-setup',  // будет искать /src/layouts/custom-layout.vue
  },
})

const appStore = useAppStore()
const { currentProvider, currentModel, providersList, availableModels, installedModels } = storeToRefs(appStore)

const page = ref(1)

const embeddingDownload = ref<number>(0);

// Следим за прогрессом загрузки
const handleDownloadProgress = (progress: number) => {
  embeddingDownload.value = progress * 100; // Переводим в проценты
};

// Обработчик для начала загрузки модели
const startDownloadModel = async (model: string) => {
  try {
    await invoke('download_embedding_model', { model }); // Вызов команды Tauri
  } catch (error) {
    console.error('Ошибка при загрузке модели:', error);
  }
};

onMounted(() => {
  listen('model-download-progress', (event) => {
    console.log(event)
    // Проверяем и преобразуем payload в number
    const progress = event.payload as number;

    // Если прогресс — это число, обновляем состояние
    if (typeof progress === 'number') {
      handleDownloadProgress(progress); // Обновляем прогресс
    } else {
      console.error('Received non-number progress value:', progress);
    } // Обновляем прогресс
  });
})

function nextPage(page: number, props: any ){
  if (page == 3) {
    localStorage.setItem('hasCompletedSetup', 'true')
    router.push('/')
  } else {
    props.onClick()
  }
}

onMounted(() => {
  invoke<string[]>("get_available_providers").then((providers) => {
    providersList.value = providers;
    if (providersList.value.length > 0 && currentModel == null) {
      currentProvider.value = providers[0];
      appStore.getModels();
    }
  });
});
</script>

<style scoped>
</style>
