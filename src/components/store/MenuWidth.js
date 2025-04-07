import { defineStore } from 'pinia'
export const menuWidth = defineStore('menuWidth', {
    state: () => ({ CurrentWidth: 0 }),
    getters: {
      width: (state) => state.CurrentWidth,
    },
    actions: {
      set(width) {
        this.CurrentWidth = width
      },
    },
  })