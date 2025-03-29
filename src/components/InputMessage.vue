<template>
  <v-container>
    <div class="chat-container" :class="{ 'fullscreen-mode': isFullscreen }">
      <!-- Загруженные файлы (превью) -->
      <div v-if="files.length > 0" class="file-previews">
        <div v-for="(file, index) in files" :key="index" class="file-preview">
          <img 
            v-if="file.type.startsWith('image/')" 
            :src="file.preview" 
            alt="Preview"
            class="preview-image"
          />
          <div v-else class="file-document">
            <span class="file-icon">📄</span>
            <span class="file-name">{{ file.name }}</span>
          </div>
          <button @click="removeFile(index)" class="remove-file-btn">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <path d="M18 6L6 18M6 6L18 18" stroke="currentColor" stroke-width="2"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Основное поле ввода -->
      <div class="input-wrapper">
        <textarea
          ref="textareaRef"
          v-model="message"
          placeholder="Введите ваше сообщение..."
          @keydown.enter.exact.prevent="sendMessage"
          class="message-input"
        ></textarea>

        <!-- Панель управления -->
        <div class="controls">
          <!-- Левая группа кнопок -->
          <div class="left-controls">
            <!-- Кнопка меню -->
            <div class="menu-wrapper">
              <button @click="toggleMenu" class="round-btn">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                  <path d="M4 6H20M4 12H20M4 18H20" stroke="currentColor" stroke-width="2"/>
                </svg>
              </button>
              
              <!-- Выпадающее меню -->
              <div v-if="showMenu" class="dropdown-menu">
                <div class="menu-item">
                  <label class="switch">
                    <input type="checkbox" v-model="systemSearchEnabled">
                    <span class="slider round"></span>
                  </label>
                  <span class="menu-label">Поиск по системе</span>
                </div>
              </div>
            </div>

            <!-- Кнопка раскрытия на весь экран -->
            <button @click="toggleFullscreen" class="round-btn">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                <path v-if="!isFullscreen" d="M7 7H4V4M17 7h3V4M7 17H4v3M17 17h3v3" stroke="currentColor" stroke-width="2"/>
                <path v-else d="M7 17V20H4M17 17V20H20M7 7V4H4M17 7V4H20" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
            
            <!-- Кнопка прикрепления файлов -->
            <button @click="triggerFileInput" class="round-btn">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
                <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
          </div>
          
          <!-- Кнопка отправки (стрелка) -->
          <button @click="sendMessage" class="send-btn round-btn">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
              <path d="M5 12H19M12 5L19 12L12 19" stroke="currentColor" stroke-width="2"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Скрытый input для загрузки файлов -->
      <input
        type="file"
        ref="fileInput"
        @change="handleFileUpload"
        multiple
        style="display: none;"
      />
    </div>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';

const message = ref('');
const files = ref([]);
const isFullscreen = ref(false);
const textareaRef = ref(null);
const fileInput = ref(null);
const showMenu = ref(false);
const systemSearchEnabled = ref(false);

const toggleMenu = () => {
  showMenu.value = !showMenu.value;
};

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

const toggleFullscreen = () => {
  isFullscreen.value = !isFullscreen.value;
  if (textareaRef.value) {
    textareaRef.value.focus();
  }
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

.fullscreen-mode {
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  border-radius: 0;
  border: none;
  padding: 16px;
  background-color: #121212;
}

.file-previews {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.file-preview {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: 4px;
  overflow: hidden;
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
  height: 100%;
  color: #b0b0b0;
  font-size: 12px;
}

.file-icon {
  font-size: 24px;
  margin-bottom: 4px;
}

.remove-file-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  border: none;
  border-radius: 50%;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}

.input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.message-input {
  width: 100%;
  min-height: 100px;
  padding: 12px;
  border: 1px solid #424242;
  border-radius: 4px;
  background-color: #1c1c1c;
  color: #e0e0e0;
  resize: none;
  font-family: inherit;
  font-size: 14px;
}

.message-input:focus {
  outline: none;
  border-color: #1976d2;
}

.fullscreen-mode .message-input {
  min-height: calc(100vh - 180px);
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

.round-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: 1px solid #424242;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}

.round-btn:hover {
  background-color: #3d3d3d;
}

.send-btn {
  background-color: #1976d2;
  color: white;
}

.send-btn:hover {
  background-color: #1565c0;
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
  box-shadow: 0 2px 10px rgba(0,0,0,0.2);
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

input:checked + .slider {
  background-color: #1976d2;
}

input:checked + .slider:before {
  transform: translateX(20px);
}
</style>
