<!-- RenameChatDialog.vue -->
<template>
  <v-dialog
    v-model="dialog"
    max-width="400"
  >
    <v-card>
      <v-card-title>Переименовать чат</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="internalTitle"
          label="Новое название"
          autofocus
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn
          text
          color="error"
          @click="onCancel"
        >
          Отмена
        </v-btn>
        <v-btn
          text
          color="primary"
          @click="onSave"
        >
          Сохранить
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { defineProps, defineEmits, ref, watch } from 'vue'

const props = defineProps<{
  modelValue: boolean
  title: string
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', val: boolean): void
  (e: 'save', newTitle: string): void
}>()

// внутренний ref для управления состоянием диалога
const dialog = ref(props.modelValue)
watch(() => props.modelValue, v => (dialog.value = v))
watch(dialog, v => emits('update:modelValue', v))

// отдельный ref для поля ввода
const internalTitle = ref(props.title)
watch(() => props.title, v => (internalTitle.value = v))

function onCancel() {
  dialog.value = false
}

function onSave() {
  emits('save', internalTitle.value.trim())
  dialog.value = false
}
</script>
