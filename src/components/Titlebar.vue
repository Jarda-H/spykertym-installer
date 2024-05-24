<script setup>
import { appWindow } from '@tauri-apps/api/window'
</script>
<template>
    <div data-tauri-drag-region class="titlebar">
        <div class="title">
            <img src="../assets/logo.png" alt="Spykertým logo">
            <h2>Spykertym Installer</h2>
        </div>
        <div class="btns">
            <div class="titlebar-btn min" @click="min">
                <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                    <path fill="white"
                        d="M7 21q-.425 0-.712-.288T6 20t.288-.712T7 19h10q.425 0 .713.288T18 20t-.288.713T17 21z" />
                </svg>
            </div>
            <div class="titlebar-btn max" @click="max">
                <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                    <path fill="white"
                        d="M5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h14q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21zm0-2h14V5H5z" />
                </svg>
            </div>
            <div class="titlebar-btn close" @click="close">
                <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                    <path fill="white"
                        d="m12 13.4l-4.9 4.9q-.275.275-.7.275t-.7-.275t-.275-.7t.275-.7l4.9-4.9l-4.9-4.9q-.275-.275-.275-.7t.275-.7t.7-.275t.7.275l4.9 4.9l4.9-4.9q.275-.275.7-.275t.7.275t.275.7t-.275.7L13.4 12l4.9 4.9q.275.275.275.7t-.275.7t-.7.275t-.7-.275z" />
                </svg>
            </div>
        </div>
    </div>
</template>
<script>
export default {
    name: "Titlebar",
    methods: {
        min() {
            appWindow.minimize()
        },
        max() {
            appWindow.toggleMaximize()
        },
        close() {
            appWindow.close()
        }
    }
};
</script>
<style lang="scss" scoped>
.titlebar {
    height: 30px;
    background: $main3;
    user-select: none;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;

    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-left: 10px;
}

.title {
    display: flex;
    align-items: center;
    gap: .5em;

    img {
        height: 20px;
        width: 20px;
    }

    h2 {
        color: white;
        font-size: 14px;
    }
}

.btns {
    display: flex;
}

.titlebar-btn {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    width: 30px;
    height: 30px;

    &:hover {
        background: rgba(0, 0, 0, 0.2);
    }
}
</style>