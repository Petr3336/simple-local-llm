<template>
  <v-navigation-drawer
    v-model="isOpen"
    location="left"
    width="300"
  >
    <v-list>
      <v-list-item title="Sidebar Content"></v-list-item>
      <!-- Здесь будет содержимое sidebar -->
    </v-list>
  </v-navigation-drawer>
</template>

<script lang="ts" setup>
import { useSidebarStore } from '@/stores/sidebar'
import { storeToRefs } from 'pinia'

const sidebarStore = useSidebarStore()
const { isOpen } = storeToRefs(sidebarStore)
</script>
