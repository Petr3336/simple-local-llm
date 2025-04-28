/**
 * router/index.ts
 *
 * Automatic routes for `./src/pages/*.vue`
 */

// Composables
import { createRouter, createWebHistory } from "vue-router/auto";
import { setupLayouts } from "virtual:generated-layouts";
import { routes } from "vue-router/auto-routes";
import { useChatStore } from "@/stores/chat";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: setupLayouts(routes),
});

// Workaround for https://github.com/vitejs/vite/issues/11804
router.onError((err, to) => {
  if (err?.message?.includes?.("Failed to fetch dynamically imported module")) {
    if (!localStorage.getItem("vuetify:dynamic-reload")) {
      console.log("Reloading page to fix dynamic import error");
      localStorage.setItem("vuetify:dynamic-reload", "true");
      location.assign(to.fullPath);
    } else {
      console.error("Dynamic import error, reloading page did not fix it", err);
    }
  } else {
    console.error(err);
  }
});

router.beforeEach((to, from, next) => {
  const hasCompletedSetup = localStorage.getItem('hasCompletedSetup')

  if (!hasCompletedSetup && to.name !== '/initial-setup') {
    // Если нет флага и текущий маршрут не 'initial-setup', перенаправляем на '/initial-setup'
    next({ name: '/initial-setup' })
  }
  if (to.name == "/[id]") {
    const chatStore = useChatStore();
    const chatIdFromRoute = to.params.id as string;

    // Проверяем, если id в URL изменился, обновляем activeChatId в хранилище
    if (chatIdFromRoute && chatIdFromRoute !== chatStore.activeChatId) {
      chatStore.activeChatId = chatIdFromRoute;
    }
  }
  next();
});

router.isReady().then(() => {
  localStorage.removeItem("vuetify:dynamic-reload");
});

export default router;
