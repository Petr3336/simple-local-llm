<template>
  <v-app>
    <v-sidebar>
      <Sidebar />
    </v-sidebar>
    <v-appbar>
      <AppBar />
    </v-appbar>
    <v-main>
      <router-view />
    </v-main>
    <v-footer class="exact-fit-footer">
      <InputMessage />
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
  //
</script>

<style scoped>
html { overflow-y: auto }
.exact-fit-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: transparent !important;
  justify-content: center; 
}
</style>
