// Utilities
import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', {
  state: () => ({
    currentProvider: '',
    currentModel: '',
    sideBarState: false,
  }),
  actions: {
    sideBarToggle() {
      this.sideBarState = !this.sideBarState
    },
  },
})