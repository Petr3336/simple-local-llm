<template>
  <v-container>
    <div class="chat-container">
      <!-- Компонент превью файлов -->
      <FilePreviews :files="files" @remove-file="removeFile" />

      <!-- Основное поле ввода -->
      <div class="input-wrapper">
        <MdEditor v-model="text" theme="dark" :language="'en-US'" :preview="false" :toolbars="toolbars"
          :no-footer="true" :show-words-count="false" class="custom-md-editor" :style="{
            borderRadius: '8px',
            height: '200px',
            width: '100%'
          }" />

        <!-- Панель управления -->
        <div class="controls">
          <div class="left-controls">
            <v-menu v-model="showMenu" :close-on-content-click="false">
              <template v-slot:activator="{ props }">
                <v-btn v-bind="props" icon="mdi-menu" size="small" variant="text" border></v-btn>
              </template>
              <v-card class="menu-card">
                <v-list>
                  <v-list-item>
                    <template v-slot:prepend>
                      <v-switch v-model="systemSearchEnabled" class="pr-3" @click.stop hide-details></v-switch>
                    </template>
                    <v-list-item-title>Поиск по системе</v-list-item-title>
                  </v-list-item>
                </v-list>
              </v-card>
            </v-menu>
            <v-btn @click="triggerFileInput" icon="mdi-file-upload-outline" size="small" variant="text" border />
          </div>

          <!-- Кнопка отправки -->
          <v-btn @click="sendMessage" icon="mdi-send-variant" size="small" color="#1976d2" variant="flat" />
        </div>
      </div>

      <!-- Скрытый input для загрузки файлов -->
      <input type="file" ref="fileInput" @change="handleFileUpload" multiple style="display: none;" />
    </div>
  </v-container>
</template>

<script setup>
import { ref } from 'vue';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import FilePreviews from './FilePreviews.vue';

const text = ref('');
const files = ref([]);
const fileInput = ref(null);
const systemSearchEnabled = ref(false);
const showMenu = ref(false);

const toolbars = [
  'bold', 'underline', 'orderedList',
  'code', 'link', 'fullscreen'
];

const handleFileUpload = (e) => {
  const uploadedFiles = Array.from(e.target.files);
  uploadedFiles.forEach(file => {
    const reader = new FileReader();
    reader.onload = (event) => {
      files.value.push({
        name: file.name,
        type: file.type,
        preview: event.target.result,
        file
      });
    };
    if (file.type.startsWith('image/')) {
      reader.readAsDataURL(file);
    } else {
      reader.readAsText(file);
    }
  });
};

const removeFile = (index) => {
  files.value.splice(index, 1);
};

const sendMessage = () => {
  if (!text.value.trim() && files.value.length === 0) return;

  const payload = {
    text: text.value,
    files: files.value.map(f => f.file),
    systemSearch: systemSearchEnabled.value
  };

  console.log('Отправлено:', payload);
  text.value = '';
  files.value = [];
};

const triggerFileInput = () => {
  fileInput.value.click();
};
</script>

<style scoped>
.chat-container {
  border: 1px solid #424242;
  border-radius: 8px;
  padding: 12px;
  transition: all 0.3s ease;
}

.custom-md-editor {
  --md-bk-color: #0000002b;
  --md-color: #ffffff;
  --md-border-color: #424242;
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
</style>