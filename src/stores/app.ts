// Utilities
import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

const LLAMA_MODELS = [
  'google_gemma-3-1b-it-Q4_0.gguf',
  'google_gemma-3-4b-it-Q4_0.gguf',
  'google_gemma-3-12b-it-Q4_0.gguf',
  'google_gemma-3-27b-it-Q4_0.gguf',
];

const LLAMA_MODEL_MAPPING: Record<string, { repo: string; url: string }> = {
  "google_gemma-3-1b-it-Q4_0.gguf": {
    repo: "bartowski/google_gemma-3-1b-it-GGUF",
    url: "https://huggingface.co/bartowski/google_gemma-3-1b-it-GGUF/resolve/main/google_gemma-3-1b-it-Q4_0.gguf?download=true",
  },
  "google_gemma-3-4b-it-Q4_0.gguf": {
    repo: "bartowski/google_gemma-3-4b-it-GGUF",
    url: "https://huggingface.co/bartowski/google_gemma-3-4b-it-GGUF/resolve/main/google_gemma-3-4b-it-Q4_0.gguf?download=true",
  },
  "google_gemma-3-12b-it-Q4_0.gguf": {
    repo: "bartowski/google_gemma-3-12b-it-GGUF",
    url: "https://huggingface.co/bartowski/google_gemma-3-12b-it-GGUF/resolve/main/google_gemma-3-12b-it-Q4_0.gguf?download=true",
  },
  "google_gemma-3-27b-it-Q4_0.gguf": {
    repo: "bartowski/google_gemma-3-27b-it-GGUF",
    url: "https://huggingface.co/bartowski/google_gemma-3-27b-it-GGUF/resolve/main/google_gemma-3-27b-it-Q4_0.gguf?download=true",
  },
};

export const useAppStore = defineStore("app", {
  state: () => ({
    initialSetup: ref<boolean | undefined>(true),
    sideBarState: false,
    currentProvider: "",
    currentModel: "",
    availableModels: [] as string[],
    installedModels: [] as string[],
    providersList: [] as string[],
  }),
  actions: {
    sideBarToggle() {
      this.sideBarState = !this.sideBarState;
    },
    async getModels() {
      if (!this.currentProvider) return;
      // жёсткий список для llama.cpp
      if (this.currentProvider === 'llama.cpp') {
        // 1) получаем список уже скачанных моделей
        try {
          const downloaded = await invoke<string[]>("get_installed_models", { providerName: 'llama.cpp' })
          this.installedModels = downloaded
        } catch (e) {
          console.error("Не удалось получить список скачанных моделей:", e)
          this.installedModels = []
        }
        // 2) всегда показываем полный жесткий список
        this.availableModels = Array.from(new Set([...LLAMA_MODELS, ...this.installedModels]));
        // 3) если ещё нет выбора — ставим первую
        if (!this.currentModel && LLAMA_MODELS.length) {
          this.currentModel = LLAMA_MODELS[0]
        }
        return
      }

      try {
        const models = await invoke<string[]>("get_installed_models", { providerName: this.currentProvider })
        this.availableModels = models
        if (!this.currentModel && models.length) {
          this.currentModel = models[0]
        }
      } catch (e) {
        console.error("Ошибка get_installed_models:", e)
      }
    },
    async downloadSelectedModel(provider: string, model: string) {
      if (provider !== 'llama.cpp') {
        // просто шлём на бек
        await invoke("download_model", { providerName: provider, model })
        this.getModels()
        return
      }

      const info = LLAMA_MODEL_MAPPING[model]
      const modelParam = info ? `${info.repo}:${model}` : model
      const downloadUrl = info?.url

      try {
        await invoke("download_model", {
          providerName: 'llama.cpp',
          model: modelParam,
          ...(downloadUrl ? { downloadUrl } : {})
        })
        // после успешного скачивания — помечаем эту модель как скачанную
        if (!this.installedModels.includes(model)) {
          this.installedModels.push(model)
        }
      } catch (e) {
        console.error("Ошибка скачивания llama.cpp-модели:", e)
      }
    }
  },
});
