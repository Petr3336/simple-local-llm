<template>
  <v-container>
    <div class="chat-container">
      <!-- Загруженные файлы (превью) -->
      <div v-if="files.length > 0" class="file-previews">
        <div v-for="(file, index) in files" :key="index" class="file-preview">
          <img v-if="file.type.startsWith('image/')" :src="file.preview" alt="Preview" class="preview-image" />
          <div v-else class="file-document">
            <span class="file-name">{{ file.name }}</span>
          </div>

        </div>
        <v-btn @click="removeFile(index)" icon="mdi-close-thick" size="x-small" variant="text"
          class="file-close"> </v-btn>
      </div>

      <!-- Основное поле ввода -->
      <div class="input-wrapper">
        <MdEditor v-model="text" theme="dark" :language='en - US' :preview="false"
          :toolbars="toolbars" :no-footer="true" :show-words-count="false" :style="{
            borderRadius: '8px',
            height: '200px',
            width: '100%'
          }" class="custom-md-editor" />

        <!-- Панель управления -->
        <div class="controls">
          <div class="left-controls">
            <v-menu v-model="showMenu" :close-on-content-click="false">
              <template v-slot:activator="{ props }">
                <v-btn v-bind="props" icon="mdi-menu" size="small" variant="text" border></v-btn>
              </template>
              <v-card class="menu-card" width="200">
                <v-list>
                  <v-list-item>
                    <template v-slot:prepend>
                      <v-switch v-model="systemSearchEnabled" @click.stop hide-details></v-switch>
                    </template>
                    <v-list-item-title>Поиск по системе</v-list-item-title>
                  </v-list-item>
                </v-list>
              </v-card>
            </v-menu>
            <v-btn @click="triggerFileInput" icon="mdi-file-upload-outline" size="small" variant="text" border></v-btn>
          </div>

          <!-- Кнопка отправки (стрелка) -->
          <v-btn @click="sendMessage" icon="mdi-send-variant" size="small" color="#1976d2" variant="flat"><template
              v-slot:default><v-icon color="white"></v-icon></template></v-btn>
        </div>
      </div>

      <!-- Скрытый input для загрузки файлов -->
      <input type="file" ref="fileInput" @change="handleFileUpload" multiple style="display: none;" />
    </div>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const message = ref('');
const files = ref([]);
const fileInput = ref(null);
const systemSearchEnabled = ref(false);

const toggleMenu = () => {
  showMenu.value = !showMenu.value;
};

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
  if (!message.value.trim() && files.value.length === 0) return;

  const payload = {
    text: message.value,
    files: files.value.map(f => f.file),
    systemSearch: systemSearchEnabled.value
  };

  console.log('Отправлено:', payload);
  message.value = '';
  files.value = [];
};

const triggerFileInput = () => {
  fileInput.value.click();
};

onMounted(() => {
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.menu-wrapper')) {
      showMenu.value = false;
    }
  });
});

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

.file-previews {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.file-preview {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  overflow: hidden;
  width: 80px;
  height: 80px;
  border-radius: 4px;
  background-color: #2d2d2d;
}

.preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.file-document {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #b0b0b0;
  font-size: 12px;
}

.file-close {
  display: flex;
  color: #b90000;
  font-size: 12px;
  right: 24px;
  top: -14px;

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
