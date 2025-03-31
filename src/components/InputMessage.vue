<template>
  <v-container>
    <div class="chat-container">
      <!-- Загруженные файлы (превью) -->
      <div v-if="files.length > 0" class="file-previews">
        <div v-for="(file, index) in files" :key="index" class="file-preview">

          <img v-if="file.type.startsWith('image/')" :src="file.preview" alt="Preview" class="preview-image" />
          <div v-else class="file-document">
            <span class="file-icon">📄</span>
            <span class="file-name">{{ file.name }}</span>
          </div>

        </div>
        <v-btn @click="removeFile(index)" icon="mdi-close-thick" size="x-small" variant="text" border
          class="file-close"> </v-btn>

      </div>

      <!-- Основное поле ввода -->
      <div class="input-wrapper">

        <MdEditor v-model="text" theme="dark" :toolbars-exclude="['words-count']" :language='en - US' :preview="false"
          :toolbars="toolbars" :no-footer="true" :show-words-count="false" :style="{
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
const textareaRef = ref(null);
const fileInput = ref(null);
const showMenu = ref(false);
const systemSearchEnabled = ref(false);

const toggleMenu = () => {
  showMenu.value = !showMenu.value;
};

const toolbars = [
  'bold', 'italic', 'underline', 'strikeThrough',
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

.file-icon {
  position: absolute;
  right: 10px;
  top: 5px;
  font-size: 24px;
  margin-bottom: 4px;
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

.menu-wrapper {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  background-color: #2d2d2d;
  border: 1px solid #424242;
  border-radius: 4px;
  padding: 8px;
  z-index: 1001;
  min-width: 180px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
}

.menu-label {
  color: #e0e0e0;
  font-size: 14px;
}

.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 20px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #424242;
  transition: .4s;
  border-radius: 20px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked+.slider {
  background-color: #1976d2;
}

input:checked+.slider:before {
  transform: translateX(20px);
}
</style>
