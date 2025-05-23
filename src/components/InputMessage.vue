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
        ref="mdEditor"
        v-model="runParams.prompt"
        theme="dark"
        language="ru-RU"
        :preview="false"
        :footers="fullscreen ? ['markdownTotal', 'scrollSwitch'] : []"
        :show-words-count="false"
        class="custom-md-editor"
        :style="{
          borderRadius: '8px',
          height: '17.5vh',
          width: '100%',
        }"
        :toolbars="editorToolbars"
      />

      <!-- Панель управления -->
      <div class="d-flex justify-space-between align-center">
        <div class="controls">
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
              <v-list density="compact">
                <v-list-item
                  v-for="(func, key) in llmFunctions"
                  :key="key"
                >
                  <template #prepend>
                    <v-switch
                      v-model="func.enabled"
                      density="compact"
                      class="pr-3"
                      hide-details
                      @click.stop
                    />
                  </template>
                  <v-list-item-title>{{ func.name }}</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-card>
          </v-menu>
          <v-btn
            icon="mdi-file-upload-outline"
            size="small"
            variant="text"
            border
            @click="selectFiles"
          />
        </div>
        <div class="controls">
          <!-- Внешняя кнопка переключения pageFullscreen -->
          <v-btn
            icon="mdi-fullscreen"
            size="small"
            variant="text"
            border
            @click="fullscreen = !fullscreen"
          />
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
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { 
  ref, 
  computed, 
  onMounted, 
  watch,
  type ComponentPublicInstance 
} from "vue";
import { MdEditor, type ExposeParam, type ToolbarNames } from "md-editor-v3";
import "md-editor-v3/lib/style.css";
import { useChatStore } from "@/stores/chat";
import { useAppStore } from "@/stores/app";
import { storeToRefs } from "pinia";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import * as pathAPI from "@tauri-apps/api/path";

// Chat Store
const chatStore = useChatStore();
const appStore = useAppStore();

const { currentProvider, currentModel } = storeToRefs(appStore);
const { llmFunctions, runParams } = storeToRefs(chatStore);
chatStore.fetchAvailableFunctions();

// Состояние файлов
interface FileItem {
  name: string;
  type: string;
  preview: string;
  path: string;
}
const files = ref<FileItem[]>([]);
const fileInput = ref<HTMLInputElement | null>(null);
const showMenu = ref(false);

// Ссылка на редактор
const mdEditor = ref<ComponentPublicInstance & ExposeParam | null>(null);
// Режим page-fullscreen
const fullscreen = ref(false);

// Настройка тулбаров
const toolBarConfiguration: ToolbarNames[] = [
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
  "pageFullscreen"
];
const editorToolbars = computed(() => fullscreen.value ? toolBarConfiguration : []);

// Параметры модели
const isLoading = ref(false);
const output = ref("");


// Функция выбора файлов через Tauri-dialog
async function selectFiles() {
  const result = await open({
    multiple: true,
    directory: false,
  });
  if (!result) return;

  const paths = Array.isArray(result) ? result : [result];
  for (const filePath of paths) {
    // получаем имя и расширение
    const name = await pathAPI.basename(filePath);
    const ext = (await pathAPI.extname(filePath)).toLowerCase();

    // читаем файл (plugins-fs автоматически возвращает Uint8Array для бинарников)
    const raw = await readFile(filePath);

    let preview: string;
    let type: string;

    // если это изображение — конвертим в data:URL
    if ([".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp"].includes(ext) && raw instanceof Uint8Array) {
      const b64 = btoa(String.fromCharCode(...raw));
      const mimeExt = ext === ".jpg" ? "jpeg" : ext.slice(1);
      type = `image/${mimeExt}`;
      preview = `data:${type};base64,${b64}`;
    } else {
      // иначе трактуем как текст
      // raw может быть Uint8Array или строкой
      const text = typeof raw === "string" ? raw : new TextDecoder().decode(raw);
      type = "text/plain";
      preview = text;
    }

    files.value.push({ name, type, preview, path: filePath });
  }
}

// Удаление превью
const removeFile = (idx: number) => files.value.splice(idx, 1);

// Отправка модели
async function runModel() {
  runParams.value.model = currentModel.value
  isLoading.value = true;
  output.value = "";
  chatStore.addMessage({ role: "user", content: runParams.value.prompt });
  runParams.value.prompt = "";
  runParams.value.options.functions = llmFunctions.value
    .filter(f => f.enabled)
    .map(f => f.name);
  await chatStore.runModel(
    currentProvider.value,
    runParams.value.model,
    runParams.value.prompt,
    runParams.value.options,
    files.value.map( file => {return file.path})
  );
}

// Инициализация: подписка на fullscreen и Enter
onMounted(() => {
  // 1) Перехват Enter (без Shift) на всем редакторе
  const root = (mdEditor.value as any)?.$el as HTMLElement | undefined;
  if (root) {
    root.addEventListener(
      "keydown",
      (e: KeyboardEvent) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          runModel();
        }
      },
      { capture: true }
    );
  }

  // 2) Подписка на внутреннее событие pageFullscreen
  mdEditor.value?.on("pageFullscreen", (status) => {
    fullscreen.value = status;
  });
});


// При внешнем изменении fullscreen — сразу переключаем редактор
watch(fullscreen, (val) => {
  mdEditor.value?.togglePageFullscreen(val);
})
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
  gap: 8px;
}
</style>
