<template>
  <v-container>
    <v-app-bar app flat color="transparent" class="px-4">
      <!-- Кнопка меню -->
      <v-btn icon @click="toggleSidebar">
        <v-icon>mdi-menu</v-icon>
      </v-btn>
      <v-spacer></v-spacer>
      <v-menu :close-on-content-click="false">
        <template v-slot:activator="{ props }" >
          <v-btn v-bind="props">
            {{ modelProvider || "Выберите провайдера" }}
            <v-icon right>mdi-chevron-down</v-icon>
          </v-btn>
        </template>

        <v-card class="pa-2" style="background: transparent; box-shadow: none;">
          <div class="d-flex align-center">
            <v-select v-model="modelProvider" :items="providersList" label="Провайдер" density="compact"
              variant="outlined" class="mr-2" style="min-width: 120px; max-width: 180px;" hide-details
              @update:model-value="getModels"></v-select>
            <v-combobox v-model="selectedModel" :items="modelsList" label="Модель" density="compact" variant="outlined"
              style="min-width: 150px; max-width: 180px;" hide-details></v-combobox>
          </div>
        </v-card>
      </v-menu>
    </v-app-bar>
  </v-container>
</template>

<script lang="ts" setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSidebarStore } from '@/stores/sidebar'

const modelProvider = ref("");
const selectedModel = ref("");
const modelsList = ref<string[]>([]);
const providersList = ref<string[]>([]);
const sidebarStore = useSidebarStore()

function toggleSidebar() {
  sidebarStore.toggle()
}

onMounted(() => {
  invoke<string[]>("get_available_providers").then((providers) => {
    providersList.value = providers;
    if (providersList.value.length > 0) {
      modelProvider.value = providers[0];
      getModels();
    }
  });
});

function getModels() {
  if (!modelProvider.value) return;

  invoke<string[]>("get_installed_models", { providerName: modelProvider.value })
    .then((models) => {
      modelsList.value = models;
      if (models.length > 0 && !selectedModel.value) {
        selectedModel.value = models[0];
      }
    })
    .catch((error: unknown) => {
      console.error("Ошибка получения моделей:", error);
    });
}

function getModelDisplayName(modelName: string): string {
  if (modelName.length > 15) {
    return modelName.substring(0, 15) + '...';
  }
  return modelName;
}

defineExpose({
  modelProvider,
  selectedModel
});
</script>

<style scoped>
.v-app-bar {
  backdrop-filter: blur(5px);
}

.v-btn--variant-outlined {
  border: thin solid #ffffff1f;
}
</style>
