import { defineStore } from 'pinia'
export const currentGame = defineStore('currentGame', {
    state: () => ({ game_id: 0 }),
    getters: {
      id: (state) => state.game_id,
    },
    actions: {
      set(id) {
        this.game_id = id
      },
    },
  })