// index.vue
<template>
  <v-empty-state
    headline="Чат не выбран"
    title="Пока-что тут пусто...."
    text="Но вы можете это исправить выбрав чат справа \(￣︶￣)"
  />
</template>

<script setup lang="ts">
definePage({
  meta: {
    layout: 'index',  // будет искать /src/layouts/custom-layout.vue
  },
})
</script>

<style scoped>
</style>
