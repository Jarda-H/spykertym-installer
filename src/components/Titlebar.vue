<script setup>
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getVersion } from '@tauri-apps/api/app';
import { message } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import Popup from "./Popup.vue";
</script>
<template>
    <div data-tauri-drag-region class="titlebar">
        <div class="title">
            <img src="../assets/logo.png" alt="Spykertým logo">
            <h2>Spykertym Installer
                <span class="version">{{ version }}
                    <span v-if="API_ENDPOINT.startsWith('https://beta')">-beta</span>
                    <span v-else-if="prod">-prod</span>
                    <span v-else>-dev</span>
                </span>
            </h2>
            <div class="titlebar-btn settings" @click="checkUpdates" title="Aktualizace nainstalovaných češtin">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
                    <path fill="currentColor"
                        d="M12 21q-1.875 0-3.512-.712t-2.85-1.925t-1.925-2.85T3 12t.713-3.512t1.924-2.85t2.85-1.925T12 3q2.05 0 3.888.875T19 6.35V4h2v6h-6V8h2.75q-1.025-1.4-2.525-2.2T12 5Q9.075 5 7.038 7.038T5 12t2.038 4.963T12 19q2.625 0 4.588-1.7T18.9 13h2.05q-.375 3.425-2.937 5.713T12 21m2.8-4.8L11 12.4V7h2v4.6l3.2 3.2z" />
                </svg>
                <h2>Aktualizace češtin</h2>
            </div>
            <div class="titlebar-btn settings" @click="openGH" title="Stránka installeru na GitHubu">
                <img src="../assets/github.svg" alt="GitHub logo">
            </div>
            <div class="titlebar-btn settings" @click="openContact">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
                    <path fill="currentColor"
                        d="M2 22V4q0-.825.588-1.412T4 2h16q.825 0 1.413.588T22 4v12q0 .825-.587 1.413T20 18H6zm10-7q.425 0 .713-.288T13 14t-.288-.712T12 13t-.712.288T11 14t.288.713T12 15m-1-4h2V5h-2z" />
                </svg>
                <h2>Zpětná vazba</h2>
            </div>
            <div v-if="!prod" class="api-toggle">
                <label>Prod</label>
                <div class="toggle-switch" @click="toggleAPI">
                    <div class="toggle-slider" :class="{ active: isUsingBeta }"></div>
                </div>
                <span class="api-label">Beta</span>
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
const appWindow = getCurrentWebviewWindow()
export default {
    name: "Titlebar",
    data() {
        return {
            version: "",
            popup: false,
            checkOut: "",
            isUsingBeta: true
        }
    },
    async mounted() {
        this.version = await getVersion();
        
        // Load API mode preference from localStorage
        const savedAPIMode = localStorage.getItem("api_mode");
        if (savedAPIMode === "prod") {
            this.isUsingBeta = false;
        } else {
            this.isUsingBeta = true;
        }
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
                this.popup = true;
                this.checkOut = "<p>Zatím nebyly nainstalovány žádné češtiny</p>";
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
        },
        async openGH() {
            await openUrl("https://github.com/Jarda-H/spykertym-installer/");
        },
        async openContact() {
            await openUrl("https://spykertym.cz/kontakt");
        },
        toggleAPI() {
            this.isUsingBeta = !this.isUsingBeta;
            
            if (this.isUsingBeta) {
                const newEndpoint = 'https://beta.spykertym.cz/app/';
                window.API_ENDPOINT = newEndpoint;
                localStorage.setItem("api_mode", "beta");
            } else {
                const newEndpoint = 'https://spykertym.cz/app/';
                window.API_ENDPOINT = newEndpoint;
                localStorage.setItem("api_mode", "prod");
            }
            window.location.reload();
        }
    }
};
</script>
<style lang="scss" scoped>
@use "../global" as *;

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
    color: white;

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

        h2 {
            margin-left: 5px;
            font-size: 12px;
        }
    }
}

.version {
    font-size: .7em;
}

.api-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 10px;
    color: white;
    font-size: 12px;

    label {
        margin: 0;
    }
}

.toggle-switch {
    position: relative;
    width: 40px;
    height: 20px;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.3s;

    &:hover {
        background: rgba(0, 0, 0, 0.5);
    }
}

.toggle-slider {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    transition: left 0.3s;

    &.active {
        left: 22px;
    }
}

.api-label {
    min-width: 30px;
}
</style>