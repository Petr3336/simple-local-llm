/**
 * plugins/index.ts
 *
 * Automatically included in `./src/main.ts`
 */

// Plugins
import vuetify from './vuetify'
import { createPinia } from 'pinia'
import router from '../router'
import { markRaw } from 'vue'
import type { Router } from 'vue-router';
import { createPlugin } from '@tauri-store/pinia';

// Types
import type { App } from 'vue'

declare module 'pinia' {
  export interface PiniaCustomProperties {
    router: Router;
  }
}

const pinia = createPinia();

pinia.use(({ store }) => {
  store.router = markRaw(router)
})
pinia.use(createPlugin());

export function registerPlugins (app: App) {
  app
    .use(vuetify)
    .use(router)
    .use(pinia)
}
