<template>
  <v-app-bar
    app
    flat
    color="transparent"
    class="px-4"
  >
    <!-- Кнопка меню -->
    <v-btn
      icon
      @click="appStore.sideBarToggle"
    >
      <v-icon>mdi-menu</v-icon>
    </v-btn>
    <v-spacer />
    <v-menu :close-on-content-click="false">
      <template #activator="{ props }">
        <v-btn v-bind="props">
          {{ currentProvider + '/' + currentModel || "Выберите провайдера" }}
          <v-icon right>
            mdi-chevron-down
          </v-icon>
        </v-btn>
      </template>

      <v-card
        class="pa-2"
        style="background: transparent; box-shadow: none;"
      >
        <div class="d-flex align-center">
          <v-select
            v-model="currentProvider"
            :items="providersList"
            label="Провайдер"
            density="compact"
            variant="outlined"
            class="mr-2"
            style="min-width: 120px; max-width: 180px;"
            hide-details
            @update:model-value="getModels"
          />
          <v-combobox
            v-model="currentModel"
            :items="modelsList"
            label="Модель"
            density="compact"
            variant="outlined"
            style="min-width: 150px; max-width: 180px;"
            hide-details
          />
        </div>
      </v-card>
    </v-menu>
  </v-app-bar>
</template>

<script lang="ts" setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from '@/stores/app'
import { storeToRefs } from "pinia";

const modelsList = ref<string[]>([]);
const providersList = ref<string[]>([]);
const appStore = useAppStore()
const { currentProvider, currentModel } = storeToRefs(appStore)

onMounted(() => {
  invoke<string[]>("get_available_providers").then((providers) => {
    providersList.value = providers;
    if (providersList.value.length > 0 && currentModel == null) {
      currentProvider.value = providers[0];
      getModels();
    }
  });
});

function getModels() {
  if (!currentProvider.value) return;

  invoke<string[]>("get_installed_models", { providerName: currentProvider.value })
    .then((models) => {
      modelsList.value = models;
      if (models.length > 0 && !currentModel.value) {
        currentModel.value = models[0];
      }
    })
    .catch((error: unknown) => {
      console.error("Ошибка получения моделей:", error);
    });
}
</script>

<style scoped>
v-app-bar {
  backdrop-filter: blur(5px);
}

.v-btn--variant-outlined {
  border: thin solid #ffffff1f;
}
</style>
