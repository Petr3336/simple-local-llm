<template>
  <Sidebar />
  <AppBar />
  <v-main>
    <router-view />
  </v-main>
  <v-footer
    app
    class="input-message-footer"
  >
    <InputMessage />
  </v-footer>
</template>

<script lang="ts" setup>
  //
</script>

<style scoped>
.input-message-footer {
  background: transparent !important;
  transform: none !important;
}
.input-message-footer:has(.md-editor-fullscreen) {
  z-index: 1010 !important;
}
</style>
