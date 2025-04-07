<script setup>
import Popup from "./Popup.vue";
</script>

<template>
  <nav ref="menu">
    <div class="filters">
      <button ref="select-all" class="selected" @click="selectOneFilter($event), searchGame()">Vše</button>
      <button @click="filterByDone">Dokončeno</button>
      <button @click="filterByPatch">Dostupný patch</button>
    </div>
    <div class="search">
      <input type="text" placeholder="Hledat hru..." ref="search" @input="searchGame">
    </div>
    <div class="list">
      <template v-for="(game, i) in jsonDisplay">
        <div class="game" @click="selectGame" :data-id="game.id" v-bind:class="{ 'selected': selectedGame == game.id }">
          <div class="img">
            <img :src="!isNaN(Number(game.icon)) ?
              'https://cdn.cloudflare.steamstatic.com/steam/apps/' + game.icon + '/capsule_231x87.jpg'
              : game.icon
              " :alt="`icon ${game.hra}`">
          </div>
          <div class="txt">
            <h2>{{ game.hra }}</h2>
            <p v-if="game.hotovo !== 100">Dokončeno {{ game.hotovo }}%</p>
          </div>
        </div>
      </template>
      <template v-if="JSON.stringify(jsonDisplay) == '[]'">
        <h3 class="ml-1">Zadanému hledání nebo filtru neodpovádá žádná hra</h3>
      </template>
    </div>
    <div class="resize-handle" @mousedown="startResize"></div>
  </nav>
  <Popup :active="fetchError" title="Chyba" :body="`Nastala chyba při načítání dat. ${fetchErrorText}`" type="retry" />
</template>
<script>
import { currentGame } from "./store/CurrentGame.js";
import { menuWidth } from "./store/MenuWidth.js";
export default {
  name: "Menu",
  data() {
    return {
      json: {},
      jsonDisplay: {},
      selectedGame: 0,
      filters: {
        done: false,
        patch: false,
      },
      fetchError: false,
      fetchErrorText: "",
    };
  },
  async mounted() {
    let json = {};
    await fetch(this.API_ENDPOINT + "get/games")
      .then((res) => res.json())
      .then((data) => {
        json = data;
      }).catch((err) => {
        this.fetchErrorText = err;
        this.fetchError = true;
      });
    if (
      (json.error || !json.games) &&
      !this.fetchError
    ) {
      this.fetchError = true;
      this.fetchErrorText = this.str.server_error;
    }
    if (this.fetchError) return;
    //sort json.dokonceno by json.hra desc
    json.games.sort((a, b) => a.hra.localeCompare(b.hra));
    this.json = json.games;
    this.displayGames();
    //pick a game from the list
    let last = localStorage.getItem("last")
    if (last) {
      this.selectedGame = last;
      const activeGame = currentGame();
      activeGame.set(last);
    }
    let savedWidth = localStorage.getItem('nav-width');
    if (savedWidth) {
      this.$refs.menu.style.width = savedWidth;
    }
  },
  methods: {
    selectOneFilter(e) {
      // if ref is select-all
      if (e.target == this.$refs["select-all"]) {
        this.filters.done = false;
        this.filters.patch = false;
      }
      let btn = e.target;
      let filters = document.querySelectorAll(".filters button");
      filters.forEach((filter) => {
        filter.classList.remove("selected");
      });
      btn.classList.add("selected");
    },
    filterByDone(e) {
      this.filters.done = true;
      this.filters.patch = false;
      this.searchGame();
      this.selectOneFilter(e);
    },
    filterByPatch(e) {
      this.filters.done = false;
      this.filters.patch = true;

      this.searchGame();
      this.selectOneFilter(e);
    },
    displayGames(q = "") {
      if (!this.json) return;
      let json = this.json;
      // if filter is on
      if (this.filters.done) {
        json = json.filter((preklad) => {
          return preklad.hotovo == 100;
        });
      }
      if (this.filters.patch) {
        json = json.filter((preklad) => {
          return preklad.patch;
        });
      }
      if (q != "") {
        // filter out games
        json = json.filter((preklad) => {
          let game = preklad.hra.toLowerCase();
          return game.includes(q);
        });
      }
      this.jsonDisplay = json;
    },
    searchGame() {
      let q = this.$refs.search.value;
      if (!q) {
        this.displayGames();
        return;
      }
      q = q.toLowerCase();
      this.displayGames(q);
    },
    selectGame(e) {
      let game = e.target;
      let id = e.target.dataset.id;
      if (!id) {
        game = e.target.closest(".game");
        id = game.dataset.id;
      }
      this.selectedGame = id;
      const activeGame = currentGame();
      activeGame.set(id);
    },
    startResize(e) {
      this.initialX = e.clientX;
      this.initialWidth = this.$refs.menu.offsetWidth;
      // listen for mouse events
      document.addEventListener('mousemove', this.resize);
      document.addEventListener('mouseup', this.stopResize);
    },
    resize(e) {
      // calc new width
      let dx = e.clientX - this.initialX;
      let newWidth = this.initialWidth + dx;

      let minWidth = 400;
      let maxWidth = window.innerWidth * 0.6; // 60% of the screen width

      if (newWidth >= minWidth && newWidth <= maxWidth) {
        this.$refs.menu.style.width = newWidth + 'px';
        menuWidth().set(this.$refs.menu.style.width);
      }
    },

    stopResize() {
      document.removeEventListener('mousemove', this.resize);
      document.removeEventListener('mouseup', this.stopResize);
      localStorage.setItem('nav-width', this.$refs.menu.style.width);
    }
  },
};
</script>
<style lang="scss" scoped>
@use "../global" as *;

nav {
  // right side nav
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding-top: 1em;

  width: 30%;
  background-color: $menu_bg;
  border: 1px solid #ffffff1d;
  border-top-right-radius: 1em;
  border-bottom-right-radius: 1em;

  position: relative;

  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 5px;
    height: 100%;
    cursor: col-resize;
    background-color: transparent;

    &:hover {
      background-color: $alt;
    }

    &:active {
      background-color: $alt;
    }
  }
}

.list {
  overflow: auto;
  width: 100%;

  &::-webkit-scrollbar {
    width: 20px;
  }

  &::-webkit-scrollbar-track {
    background-color: $menu_bg;
    border-radius: 100px;
  }

  &::-webkit-scrollbar-thumb {
    border-radius: 100px;
    border: 5px solid transparent;
    background-clip: content-box;
    background-color: $alt;
  }
}

.search {
  width: 100%;

  display: flex;
  justify-content: center;
  align-items: center;

  input {
    width: 80%;
    padding: 0.5em;
    margin-bottom: 1em;
    background-color: $bg;
    color: white;
    border: 0;
    border-radius: 10px;
    outline: none;
    border: 1px solid transparent;

    &:focus {
      border: 1px solid $alt;
    }
  }
}

.game {
  width: 100%;
  background-color: $bg;
  padding: 1em;
  margin-bottom: 1em;
  display: flex;
  justify-content: flex-start;
  align-items: center;

  border-top-right-radius: 1em;
  border-bottom-right-radius: 1em;

  &.selected {
    background-color: $alt;
  }

  @media (max-width: $mobile) {
    margin-bottom: .5em;
  }

  .img {
    display: flex;
    justify-content: center;
    align-items: center;

    img {
      max-width: 150px;
      height: 50px;

      @media (max-width: $mobile) {
        display: none;
      }
    }
  }

  .txt {
    margin-left: 1em;

    h2 {
      font-size: 1.5em;
      margin: 0;

      @media (max-width: $mobile) {
        font-size: 1em;
      }
    }

    p {
      margin: 0;

      @media (max-width: $mobile) {
        display: none;
      }
    }
  }
}

.filters {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;

  button {
    padding: 0.5em;
    margin: 0.5em;
    background-color: $bg;
    color: white;
    border: 0;
    border-radius: 10px;
    outline: none;
    border: 1px solid transparent;

    &:focus {
      border: 1px solid $alt;
    }

    &.selected {
      background-color: $alt;
    }

    @media (max-width: $mobile) {
      font-size: 0.7em;
    }
  }
}
</style>