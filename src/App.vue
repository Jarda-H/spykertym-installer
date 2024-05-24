<script setup>
import Game from "./components/Game.vue";
import Menu from "./components/Menu.vue";
import Titlebar from "./components/Titlebar.vue";
</script>

<template>
  <Titlebar />
  <div class="view" v-if="internet">
    <Menu />
    <Game />
  </div>
  <div class="view" v-else>
    <h1>Pro fungování se musíte připojit k internetu.</h1>
  </div>
</template>
<script>
export default {
  name: "App",
  data() {
    return {
      internet: true,
    };
  },
  mounted() {
    // disable browser features
    //disable right click
    document.addEventListener('contextmenu', e => e.preventDefault());
    //and reload/print etc options
    document.addEventListener('keydown', function (e) {
      if (e.key === 'F5' ||
        (e.ctrlKey && e.key === 'r') || //reload
        (e.ctrlKey && e.shiftKey && e.key === 'r') || //reload with shift 
        (e.ctrlKey && e.key === 'p') || //print
        (e.ctrlKey && e.shiftKey && e.key === 's') || //print win32
        (e.ctrlKey && e.key === 'u') || //view source
        (e.ctrlKey && e.key === 'j') || //downloads
        (e.ctrlKey && e.key === 'g') || //search
        (e.ctrlKey && e.shiftKey && e.key === 'g') || //search
        (e.ctrlKey && e.key === 'f') //search
      ) {
        e.preventDefault();
      }
    });
    //no internet
    if (!navigator.onLine) {
      this.internet = false;
    }
    //add event listener
    window.addEventListener("online", () => {
      this.internet = true;
    });
    window.addEventListener("offline", () => {
      this.internet = false;
    });
  },
};
</script>
<style lang="scss" scoped>
.view {
  color: #f6f6f6;
  background-color: $mainbg;
  height: calc(100vh - $topbar);
  margin-top: $topbar;
  display: flex;
}
</style>
