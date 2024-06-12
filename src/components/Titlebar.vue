<script setup>
import { appWindow } from '@tauri-apps/api/window';
import { getVersion } from '@tauri-apps/api/app';
import { message } from '@tauri-apps/api/dialog';
import Popup from "./Popup.vue";
</script>
<template>
    <div data-tauri-drag-region class="titlebar">
        <div class="title">
            <img src="../assets/logo.png" alt="Spykertým logo">
            <h2>Spykertym Installer
                <span class="version">{{ version }}
                    <span v-if="prod">-prod</span>
                    <span v-else>-dev</span>
                </span>
            </h2>
            <div class="titlebar-btn settings" @click="checkUpdates">
                <h2>Zkontrolovat aktualizace češtin</h2>
            </div>
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
    <Popup :active="popup" title="Kontrola aktualizací češtin" :body="checkOut" type="close" @close="popup = false" />
</template>
<script>
export default {
    name: "Titlebar",
    data() {
        return {
            version: "",
            popup: false,
            checkOut: ""
        }
    },
    async mounted() {
        this.version = await getVersion();
    },
    methods: {
        min() {
            appWindow.minimize()
        },
        max() {
            appWindow.toggleMaximize()
        },
        close() {
            appWindow.close()
        },
        async checkUpdates() {
            if (this.popup) return;
            if (!localStorage.getItem("installed_patches") ||
                localStorage.getItem("installed_patches") == "[]") {
                await message(
                    "První nainstalujte nějaký překlad a poté zkuste zkontrolovat aktualizace",
                    {
                        title: "Neméte nainstalované češtiny",
                        type: "info"
                    }
                );
                return;
            }
            await fetch(this.API_ENDPOINT + "get/latest-versions")
                .then(res => res.json())
                .then(server => {
                    if (server.error) {
                        message(server.message || "Neznámá chyba.", {
                            title: "Chyba",
                            type: "error"
                        });
                        return;
                    }
                    // get installed from ls
                    let local = localStorage.getItem("installed_patches");
                    local = JSON.parse(local);

                    let ok = "", update = "";
                    local.forEach(local => {
                        let serverPatch = server.patches
                            .find(patch => patch.game_id === Number(local.game));
                        if (!serverPatch) {
                            update += `<p>
                            Hra s ID ${local.game} není na serveru dostupná
                            </p>`;
                            return;
                        }
                        if (serverPatch.version === local.version) {
                            ok += `<p>
                                ${serverPatch.name || local.game} - Verze ${local.version} z dne ${serverPatch.release} je aktuální
                                </p>`;
                        } else {
                            update += `<p>
                            Hra ${serverPatch.name || local.game} - Verze ${local.version} není aktuální, aktuální verze je ${serverPatch.version}
                            </p>`;
                        }
                    });
                    if (!update) {
                        this.checkOut = "<p>Všechny hry mají nejnovější češtinu</p>";
                        this.popup = true;
                        return;
                    }
                    this.checkOut = `
                        <h2>Je dostupná nová verze češtin</h2>
                        ${update || "<p>Všechny hry mají nejnovější češtinu</p>"}
                        ${ok ? `<h2>Aktuální verze</h2>${ok}` : ""}
                    `;
                    this.popup = true;
                }).catch(err => {
                    message("Nepovedlo se získat data, zkuste to později.", {
                        title: "Chyba",
                        type: "error"
                    });
                });
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
    transition: .2s background;

    &:hover {
        background: rgba(0, 0, 0, 0.2);
    }

    &.settings {
        background: rgba(0, 0, 0, 0.2);
        padding: 0 10px;
        width: fit-content;

        &:hover {
            background: rgba(0, 0, 0, 0.6);
        }
    }
}

.version {
    font-size: .7em;
}
</style>