<script setup>
import { currentGame } from "./store/CurrentGame.js";
//tauri utils
import { open as openPath } from "@tauri-apps/api/shell";
import { open as openExplorer } from '@tauri-apps/api/dialog';
import { desktopDir } from '@tauri-apps/api/path';
import { invoke } from "@tauri-apps/api/tauri";
import { message } from '@tauri-apps/api/dialog';

import * as VDF from "vdf-parser";
import Popup from "./Popup.vue";
</script>

<template>
    <div class="game-view">
        <div v-if="game">
            <div class="loading" v-if="loading">
                <img src="../assets/loading.svg" alt="loading icon">
            </div>
            <div class="header">
                <h1>{{ game.hra }}</h1>
            </div>
            <div class="content">
                <div class="tab actions">
                    <a class="btn" @click="openGame" v-if='isInstalled()'>
                        <img src="../assets/run.svg" alt="Spustit hru">
                        Spustit hru
                    </a>
                    <a class="btn" @click="openLink(game.steam)" v-else-if='isSteamLink(game.steam)'>
                        <img src="../assets/steam.svg" alt="Steam logo">
                        Steam
                    </a>
                    <a class="btn" @click="openLink(game.steam)" v-else>
                        <img src="../assets/cart.svg" alt="Cart ico">
                        Obchod
                    </a>
                    <a class="btn" @click="openInstallPopup"
                        v-if="game.patches[0] && game_version == 'patched' && game_patch_offset != 0">
                        <img src="../assets/redownload.svg" alt="Redownload icon">
                        Aktualizovat
                    </a>
                    <a class="btn" @click="openInstallPopup" v-else-if="game.patches[0] && game_version == 'patched'">
                        <img src="../assets/redownload.svg" alt="Redownload icon">
                        Přeinstalovat
                    </a>
                    <a class="btn" @click="openInstallPopup" v-else-if="game.patches[0]">
                        <img src="../assets/install.svg" alt="Install icon">
                        Nainstalovat
                    </a>
                    <a class="btn disabled" v-else>
                        <img src="../assets/install.svg" alt="Install icon">
                        Čeština není dostupná
                    </a>
                    <a class="btn danger" @click="openUninstallPopup()"
                        v-if="game_version == 'patched' || game_version == 'backup'">
                        <img src="../assets/uninstall.svg" alt="Uninstall icon">
                        Odinstalovat češtinu
                    </a>
                </div>
                <div class="tab" v-if="game_version == 'patched'">
                    <div class="alert alert-warning" v-if="game_patch_offset != 0">
                        <img src="../assets/alert/warning.svg" alt="warn ico">
                        <span>Byla vydána nová verze češtiny <b>{{ game.patches[0].version }}</b> dne:
                            <b>{{ game.patches[0].release }}</b>. Vy máte nainstalovanou verzi <b>{{
            game.patches[game_patch_offset].version }}</b></span>
                    </div>
                    <div class="alert alert-info" v-else>
                        <img src="../assets/alert/info.svg" alt="info ico">
                        <span>Akutálně máte nainstalovanou češtinu <b>{{ game.patches[game_patch_offset].version }}</b>
                            vydanou dne: <b>{{ game.patches[game_patch_offset].release }}</b></span>
                    </div>
                </div>
                <div class="tab">
                    <h2>Stav:</h2>
                    <div class="state-of-the-translation">
                        <div v-for="state in game.procenta" class="progress-row">
                            <h3 v-if="state.title">{{ state.title }}</h3>
                            <div class="category">
                                <div class="row" v-for="item in state.procenta">
                                    <p>{{ item.nazev }}</p>
                                    <div class="progress-bar" ref="procenta">
                                        <span :data-percentage="item.procenta">0%</span>
                                    </div>
                                    <p>{{ item.info }}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="tab" v-if="game.popup">
                    <h2>Překladatelé:</h2>
                    <div class="translators" v-html="newLinesToPara(game.popup)"></div>
                </div>
                <div class="tab pegi">
                    <h2>PEGI:</h2>
                    <div class="pegi-row">
                        <img :src="`https://spykertym.cz/uploads/img/PEGI_${game.pegi}.svg`" alt="pegi img">
                        <p v-html="game.pegi_info"></p>
                    </div>
                </div>
            </div>
        </div>
        <div class="select-a-game" v-else>
            <h1>{{ selectAGame }}</h1>
        </div>
        <div class="install" ref="install" v-bind:class="{ 'hidden': !popupOpen }">
            <div class="step" v-if="installStep == installPages.path">
                <h2 class="title">Instalace překladu</h2>
                <div v-if="game && game.patches.length" class="about-patch">
                    <div class="info">
                        <img src="../assets/install/version.svg" alt="">
                        {{ game.patches[0].version }}
                    </div>
                    <div class="info">
                        <img src="../assets/install/date.svg" alt="">
                        {{ game.patches[0].release }}
                    </div>
                </div>
                <h2>Cesta ke hře:</h2>
                <input type="text" v-model="steamPath" v-on:input="onPathInput">
                <div class="path-status" v-if="!pathLoading">
                    <p v-if="!steamPath">Vyberte složku s hrou</p>
                    <p v-if="isFolderOk && steamPath">Byla nalezena složka s hrou</p>
                    <p v-if="steamPath && !isFolderOk">
                        <template v-if="game_version == 'patched'">Hra má již nainstalovaný patch</template>
                        <template v-else-if="game_version == 'backup'">Hra má zálohu</template>
                        <template v-else-if="game_version == 'unknown'">Vyberte složku s hrou</template>
                        <template v-else>Vyberte složku s hrou. Tato složka neobsahuje hru</template>
                    </p>
                </div>
                <div class="path-status" v-else>
                    <p>Hledání...</p>
                </div>
                <div class="actions">
                    <a class="btn" @click="browseFiles">
                        <img src="../assets/folder.svg" alt="Folder icon">
                        Vybrat
                    </a>
                    <a class="btn" @click="install" v-if="game_version == 'patched'">
                        Přeinstalovat
                        <img src="../assets/next.svg" alt="next step">
                    </a>
                    <a class="btn" v-bind:class="{ 'disabled': !isFolderOk }" @click="install" v-else>
                        Instalovat
                        <img src="../assets/next.svg" alt="next step">
                    </a>
                </div>
            </div>
            <div class="step" v-else-if="installStep == installPages.install">
                <h2 class="title">Instalace</h2>
                <div ref="install-log" class="log" v-html="installLog"></div>
            </div>
            <div class="step" v-else-if="installStep == installPages.done">
                <h2 class="title">Hotovo</h2>
                <div ref="install-log-after" class="small" v-html="installLog"></div>
                <p>Instalace byla dokončena</p>
                <div class="actions">
                    <a class="btn" @click="closeInstallPopup">
                        Zavřít
                    </a>
                </div>
            </div>
            <div class="step" v-else-if="installStep == installPages.error">
                <h2 class="title">Chyba</h2>
                <div ref="install-log-after-err" class="small" v-html="installLog"></div>
                <p>Při instalaci nastala chyba!</p>
                <p>Pokud se chyba opakuje, kontaktujte nás.</p>
                <div class="actions">
                    <a class="btn" @click="closeInstallPopup">
                        Zavřít
                    </a>
                </div>
            </div>
            <div class="step" v-else>
                <h2 class="title">Neznamy krok.. zkuste restartovat program</h2>
            </div>
            <div class="close" @click="closeInstallPopup">
                <img src="../assets/close.svg" alt="close icon">
            </div>
        </div>
        <div class="uninstall" ref="uninstall" v-bind:class="{ 'hidden': !popupUninstallOpen }">
            <div class="step" v-if="uninstallStep == 1">
                <h2 class="title">Opravdu chcete vrátit hru do původního stavu?</h2>
                <div class="actions">
                    <a class="btn" @click="closeUninstallPopup">
                        <img src="../assets/close.svg" alt="Close icon">
                        Zrušit
                    </a>
                    <a class="btn danger" @click="uninstall">
                        Odinstalovat
                        <img src="../assets/uninstall.svg" alt="next step">
                    </a>
                </div>
            </div>
            <div class="step" v-else-if="uninstallStep == 2">
                <h2 class="title">Odinstalace</h2>
                <div class="log" v-html="uninstallLog" ref="uninstall-log"></div>
                <p>{{ lastSentenceFromUninstallLog }}</p>
                <div class="actions">
                    <a class="btn" @click="closeUninstallPopup">
                        Zavřít
                    </a>
                </div>
            </div>
            <div class="step" v-else>
                <h2 class="title">Neznamy krok.. zkuste restartovat program</h2>
            </div>
        </div>
    </div>
    <Popup :active="fetchError" title="Chyba" :body="`Nastala chyba při načítání dat. ${fetchErrorText}`" type="close"
        @close="fetchError = false" />
</template>
<script>
const installPages = {
    path: 1,
    install: 2,
    done: 3,
    error: 4
}

export default {
    name: "Game",
    data() {
        return {
            selectAGame: "Vyber hru",
            game: "",
            loading: false,
            // install popup
            installStep: 1,
            popupOpen: false,
            // install log
            installLog: "",
            // uninstall popup
            uninstallStep: 1,
            popupUninstallOpen: false,
            // uninstall log
            uninstallLog: "",
            // game params
            steamPath: "",
            isFolderOk: false,
            pathLoading: false,
            is_backup: false,
            game_version: "unknown",
            game_patch_offset: 0,
            //errors
            fetchError: false,
            fetchErrorText: "",
        };
    },
    mounted() {
        let store = currentGame();
        store.$subscribe((mutation, state) => {
            this.storeLastGame(state.game_id);
            this.updateGame(state.game_id);
        })
    },
    methods: {
        storeLastGame(id) {
            localStorage.setItem("last", id);
        },
        getActiveGameID() {
            let store = currentGame();
            return store.game_id;
        },
        getIDfromURL(url) {
            const regex = /https?:\/\/store\.steampowered\.com\/app\/(\d+)/;
            const match = url.match(regex);
            return match ? match[1] : null;
        },
        async doMD5Check() {
            let patches = this.game.patches;
            let gamePath = this.steamPath;
            let game_version = "unknown";
            let patch_offset = -1;
            if (gamePath && patches.length) {
                await Promise.all(patches.map(async patch => {
                    //if patched, end the whole loop
                    if (game_version == "patched" || game_version == "original") {
                        return;
                    }
                    patch_offset++;
                    let zipFiles = patch.files;
                    let countPatched = 0;
                    for (let i = 0; i < zipFiles.length; i++) {
                        let currentPatch = zipFiles[i];
                        let fileToCheck = gamePath + currentPatch.path;
                        if (patch.unzip_path) {
                            fileToCheck = gamePath + `\\${patch.unzip_path}\\` + currentPatch.path;
                        }
                        let md5 = await this.getMD5(fileToCheck);
                        // check for backups
                        if (!this.is_backup) {
                            let md5_backup = await this.getMD5(fileToCheck + ".backup");
                            if (md5_backup == currentPatch.old) {
                                this.is_backup = true;
                            }
                        }
                        switch (md5) {
                            case currentPatch.new:
                                countPatched++;
                                break;
                            case currentPatch.old:
                                game_version = "original";
                                break;
                            default:
                                game_version = "unknown";
                                // not a patch, just zip
                                if (!currentPatch.old) {
                                    game_version = "original";
                                }
                                break;
                        }
                    }
                    if (countPatched == zipFiles.length) {
                        this.game_patch_offset = patches.indexOf(patch);
                        game_version = "patched";
                    }
                }));
            }
            // it can be in backups
            if (this.is_backup && !game_version == "patched") {
                return "backup";
            }
            return {
                version: game_version,
                patch: patch_offset
            };
        },
        async updateGame(id) {
            this.popupOpen = false; // close popup if is opened
            // default values
            this.steamPath = "";
            this.game_version = "unknown";
            this.isFolderOk = false;
            this.is_backup = false;
            this.pathLoading = false;
            this.installStep = installPages.path;
            this.uninstallStep = 1;
            this.installLog = "";
            this.uninstallLog = "";
            this.game_patch_offset = 0;
            //
            this.loading = true;
            // reset the progress to 0
            if (this.$refs.procenta) {
                for (let i = 0; i < this.$refs.procenta.length; i++) {
                    let ref = this.$refs.procenta[i];
                    let text = ref.querySelector("span");
                    text.textContent = "0%";
                    ref.style.background = `rgba(0, 0, 0, .1)`
                }
            }

            let data = {};
            await fetch(this.API_ENDPOINT + `get/game?id=${id}`)
                .then((res) => res.json())
                .then((d) => {
                    data = d;
                }).catch((err) => {
                    this.fetchError = true;
                    this.fetchErrorText = err;
                    this.loading = false;
                });
            if (data.error && !this.fetchError) {
                this.fetchError = true;
                this.fetchErrorText = this.str.server_error;
                this.loading = false;
                return;
            }
            if (!data.game) {
                this.selectAGame = "Hra nebyla nalezena";
                this.loading = false;
                return;
            }
            this.game = data.game;
            //try to get the path from the local storage
            let patches = localStorage.getItem("installed_patches");
            if (patches) {
                //find the game
                patches = JSON.parse(patches);
                let index = patches.findIndex((p) => p.game == id);
                if (index != -1) {
                    this.steamPath = patches[index].path;
                }
            }
            // no luck with local storage, try to get the path from steam
            if (!this.steamPath) {
                // update install path
                let gamePath = "";
                try {
                    gamePath = await this.getSteamGamePath(data.game.steam);
                } catch (err) {
                    //TODO handle error better
                }
                this.steamPath = gamePath;
            }
            // try hashes
            if (this.steamPath) await this.checkFolder();
            this.loading = false;
            // set the progress
            for (let i = 0; i < this.$refs.procenta.length; i++) {
                let ref = this.$refs.procenta[i];
                let text = ref.querySelector("span");

                let now = 0;
                let end = parseInt(text.dataset.percentage);
                let progress = setInterval(() => {
                    now++;
                    text.textContent = `${now}%`
                    ref.style.background = `conic-gradient(#20a566 ${now * 3.6}deg, rgba(0, 0, 0, .1) 0deg)`
                    if (now == end) clearInterval(progress);
                }, 5);
            }
        },
        async openLink(link) {
            if (!link) return;
            let steamInstalled = false;
            await invoke('steam_is_installed')
                .then(async (path) => {
                    steamInstalled = true;
                })
            if (steamInstalled) {
                //if is stream link
                let id = this.getIDfromURL(link);
                if (id) {
                    link = `steam://advertise/${id}`;
                }
            }
            await openPath(link).catch(() => {
                this.fetchError = true;
                this.fetchErrorText = this.str.steam_open_error;
            });
        },
        steamGameByID(json, id) {
            let path;
            // loop using foreach
            Object.keys(json.libraryfolders).forEach((key) => {
                if (json.libraryfolders[key].apps[id]) {
                    path = json.libraryfolders[key].path;
                    return path;
                }
            });
            if (path) return path.replaceAll("\\\\", "\\");
            return false;
        },
        async getSteamGamePath(link) {
            return new Promise(async (resolve, reject) => {
                if (!link) reject("no link");

                const regex = /https:\/\/store\.steampowered\.com\/app\/(\d+)/;
                const match = link.match(regex);
                const id = match ? match[1] : null;
                if (id) {
                    await invoke('get_steam_vdf')
                        .then((data) => {
                            let parsed = VDF.parse(data);
                            let path = this.steamGameByID(parsed, id);
                            if (path && this.game.patches.length) resolve(
                                `${path}\\steamapps\\common\\${this.game.patches[0].folder}`);
                        })
                }
                reject("game not found in steam vdf");
            })
        },
        async browseFiles() {
            let path = this.steamPath;
            if (!path) {
                path = await desktopDir();
            }

            let selected = await openExplorer({
                directory: true,
                multiple: false,
                defaultPath: path,
                title: "Vyberte složku s hrou"
            });

            if (selected) {
                this.steamPath = selected;
                this.checkFolder();
            }
        },
        openInstallPopup() {
            this.popupOpen = true;
        },
        closeInstallPopup() {
            this.installStep = installPages.path;
            this.popupOpen = false;
        },
        openUninstallPopup() {
            this.popupUninstallOpen = true;
        },
        closeUninstallPopup() {
            this.uninstallStep = 1;
            this.popupUninstallOpen = false;
        },
        newLinesToPara(text) {
            return "<p>" + text.replaceAll("\n", "</p><p>") + "</p>";
        },
        async onPathInput() {
            this.pathLoading = true;
            await this.checkFolder();
            this.pathLoading = false;
        },
        async checkFolder() {
            let path = this.steamPath;
            if (!path) {
                this.isFolderOk = false;
                return;
            }
            // Check if the `$APPDATA/avatar.png` file exists
            if (!this.game.patches[0] ||
                !this.game.patches[0].hasOwnProperty('exe')) {
                // vyber cesty installeru
                if (this.installStep == installPages.path && this.popupOpen) {
                    this.fetchError = true;
                    this.fetchErrorText = "Patch nemá exe soubor nebo neexistuje.";
                }
                return;
            }
            path = path + "\\" + this.game.patches[0].exe;
            let check = await invoke('file_exists', {
                path
            });
            if (check == "false") {
                this.isFolderOk = false;
                this.game_version = "unknown";
                return;
            }
            let checkFiles = await this.doMD5Check();
            this.game_version = checkFiles.version;
            if (this.game_version == "unknown") {
                this.isFolderOk = false;
                return;
            }
            this.isFolderOk = true;
            if (this.game_version == "patched") {
                await this.saveInstalledPatch(checkFiles.patch);
            }
        },
        async getMD5(path) {
            let hash = "";
            await invoke('get_md5', {
                path
            }).then((md5) => {
                hash = md5;
            }).catch(() => {
                if (this.installStep == installPages.install) {
                    this.fetchError = true;
                    this.fetchErrorText = `Nepodařilo se získat hash ${path}`;
                }
            })
            return hash;
        },
        async install() {
            await this.checkFolder();
            if (!this.isFolderOk) {
                await message(
                    "Zvolená složka neobsahuje hru.",
                    "Neplatná složka",
                );
                return;
            }
            if (this.game_version == "unknown") return;
            //install
            this.installStep = installPages.install;
            let patch = this.game.patches[0];
            let isPatch = patch.files[0].hasOwnProperty('old');
            let toDownload = patch.zip;
            let filename = toDownload.split("/").pop();
            let zipFolder = filename.slice(0, -4);
            let jsonLog = []; //for server logging
            // download this zip
            this.installLog = `Stahuji zip s češtinou<br>`;
            if (isPatch) {
                jsonLog.push("[init] Stahování zipu s patch soubory");
            } else {
                jsonLog.push("[init] Stahování zipu s češtinou");
            }
            let error = false;
            await invoke('download', {
                url: "https://spykertym.cz" + toDownload,
                filename: filename
            }).then(() => {
                this.installLog += `Zip byl stažen<br>`;
                jsonLog.push("[OK] Zip byl stažen");
            }).catch((err) => {
                this.installLog += `Chyba při stahování zipu - ${err}<br>`;
                jsonLog.push(`[ERROR] Chyba při stahování zipu - ${err}`);
                error = true;
            });
            // extract zip
            let patchFiles = []
            await invoke('unzip_file', {
                path: filename
            }).then((files) => {
                patchFiles = files;
                this.installLog += `Zip byl extrahován<br>`;
                jsonLog.push("[OK] Zip byl extrahován");
            }).catch((err) => {
                this.installLog += `Chyba při extrahování zipu - ${err}<br>`;
                jsonLog.push(`[ERROR] Chyba při extrahování zipu - ${err}`);
                error = true;
            });
            // foreach file run patch
            if (isPatch) {
                await Promise.all(patchFiles.map(async patchFilePath => {
                    //'C:\\Users\\JardaH\\AppData\\Local\\Temp\\3d8cd654a32ce942f0f94c4c8564e535\\Assembly-CSharp.dll.patch'
                    let patchName = patchFilePath.split("\\").pop();
                    // and replace .patch at the end
                    if (!patchName.endsWith('.patch')) {
                        this.installLog += `Soubor ${patchName} nekončí .patch<br>`;
                        jsonLog.push(`[ERROR] Soubor ${patchName} nekončí .patch`);
                        error = true;
                        return;
                    }
                    patchName = patchName.slice(0, -6);
                    // find filename in json.patch.files
                    let file = patch.files.find((f) => f.patch == patchName);
                    if (!file) {
                        this.installLog += `Soubor ${patchName} nebyl nalezen v seznamu souborů<br>`;
                        jsonLog.push(`[ERROR] Soubor ${patchName} nebyl nalezen v seznamu souborů`);
                        error = true;
                        return;
                    }
                    let fileToPatch = this.steamPath + file.path;
                    //if backup exists
                    if (this.is_backup && this.game_version == "patched") {
                        fileToPatch += ".backup";
                    }
                    await invoke('patch_file', {
                        path: fileToPatch,
                        patch: patchFilePath
                    }).then(() => {
                        this.installLog += `Soubor ${file.patch} byl upraven<br>`;
                        jsonLog.push(`[OK] Soubor ${file.patch} byl upraven`);
                    }).catch((err) => {
                        if (err.startsWith("xdelta3: target window checksum mismatch: XD3_INVALID_INPUT")) {
                            this.installLog += `U souboru ${file.patch} - <b>nesedí hash</b>, zkontrolujte jestli už nemáte patch nainstalovaný, popř. ověřte integritu hry<br>`;
                        } else {
                            this.installLog += `Chyba při úpravě souboru ${file.patch} - ${err}<br>`;
                        }
                        jsonLog.push(`[ERROR] xdelta3 ${err}`);
                        error = true;
                    });
                    this.scrollLog();
                }));
            } else {
                // just copy files
                let tmp = await invoke('get_temp_dir');
                await Promise.all(patchFiles.map(async newFile => {
                    let newFilename = newFile.split("\\").pop();
                    let post = newFile.split(tmp + zipFolder).pop();
                    //upzip path
                    if (patch.unzip_path) {
                        post = `\\${patch.unzip_path}` + post;
                    }
                    let dest = this.steamPath + post;
                    console.log(newFile, dest)
                    await invoke('copy_and_replace', {
                        from: newFile,
                        to: dest
                    }).then(() => {
                        this.installLog += `Soubor ${newFilename} byl zkopírován<br>`;
                        jsonLog.push(`[OK] Soubor ${newFilename} byl zkopírován`);
                    }).catch((err) => {
                        this.installLog += `Chyba při kopírování souboru ${newFilename} - ${err}<br>`;
                        jsonLog.push(`[ERROR] Chyba při kopírování souboru ${newFilename} - ${err}`);
                        jsonLog.push(`[ERROR] from: ${newFile} to: ${dest}`)
                        error = true;
                    });
                    this.scrollLog();
                }));
            }
            // delete patch files
            if (patchFiles.length) {
                await invoke('delete_temps', {
                    delete: patchFiles,
                    folder: zipFolder
                }).then(() => {
                    this.installLog += `Patch soubory byly smazány<br>`;
                    jsonLog.push("[OK] Patch soubory byly smazány");
                }).catch((err) => {
                    this.installLog += `Chyba při mazání - ${err}<br>`;
                    jsonLog.push(`[ERROR] Chyba při mazání - ${err}`);
                    error = true;
                });
                this.scrollLog();
            }
            if (error) {
                this.installLog += `Instalace skončila s chybou<br>`;
                jsonLog.push("[ERROR] Instalace skončila s chybou");
                if (this.game_version != "patched") {
                    try {
                        await this.sendLogToServer(jsonLog, false);
                        this.installLog += `Chyba byla odeslána<br>`;
                    } catch (err) {
                        this.installLog += `Chyba při odesílání chyby - ${err}<br>`;
                    }
                }
                this.installStep = installPages.error;
                this.scrollLog();
                return;
            }
            jsonLog.push("[OK] Instalace byla dokončena");
            if (this.game_version != "patched") await this.sendLogToServer(jsonLog, true)

            this.is_backup = true;
            this.game_version = "patched";
            await this.saveInstalledPatch();
            this.installStep = installPages.done;
            this.scrollLog();
        },
        async uninstall() {
            this.uninstallLog = "";
            if (!this.is_backup && this.game.patches[0].files[0].hasOwnProperty('old')) {
                this.uninstallLog += `Hra nemá zálohu<br>Nelze odinstalovat`;
                this.uninstallStep++;
                return;
            }
            let currentGame = this.getActiveGameID();
            //search for installed patch
            let patch = this.game.patches.find((p) =>
                p.version == this.getInstalledPatchVersion(currentGame)
            );
            if (!patch) {
                patch = this.game.patches[0];
            }
            //for every patch file
            let error = false;
            await Promise.all(patch.files.map(async file => {
                if (!file.old) {
                    //delete the file
                    let fileToDelete = this.steamPath + file.path;
                    if (patch.unzip_path) {
                        fileToDelete = this.steamPath + `\\${patch.unzip_path}\\` + file.path;
                    }
                    let filename = fileToDelete.split("\\").pop();
                    await invoke('delete_file', {
                        path: fileToDelete
                    }).then(() => {
                        this.uninstallLog += `Soubor ${filename} byl smazán<br>`;
                    }).catch((err) => {
                        this.uninstallLog += `Soubor ${filename} nelze smazat - ${err}<br>`;
                        error = true;
                    });
                    this.scrollLog();
                    return;
                }
                let fileToRenew = this.steamPath + file.path + ".backup";

                await invoke('backup_renew', {
                    path: fileToRenew
                }).then(() => {
                    this.uninstallLog += `Soubor ${file.patch} byl obnoven<br>`;
                }).catch((err) => {
                    this.uninstallLog += `Soubor ${file.patch} nelze obnovit - ${err}<br>`;
                    error = true;
                });
                this.scrollLog();
            }));
            if (error) {
                this.uninstallLog += `Odinstalace skončila s chybou<br>`;
                this.uninstallStep++;
                this.scrollLog();
                return;
            }
            this.uninstallLog += `Odinstalace byla dokončena`;
            this.is_backup = false;
            this.game_version = "original";
            this.uninstallStep++;
            this.scrollLog();
            this.removeGameID(currentGame);
        },
        async sendLogToServer(log, success) {
            return new Promise(async (ok, err) => {
                let data = {
                    game: this.getActiveGameID(),
                    version: this.game_version,
                    path: this.steamPath,
                    log,
                    soubor: this.game.patches[0].version
                }
                if (this.game_patch_offset != 0) {
                    data.old_patch = this.game.patches[this.game_patch_offset].version;
                }
                //create a hash
                let datetime = new Date().getTime().toString();
                let hash;
                await invoke('create_sha256_hash_from_timestamp_with_salt', { 'timestamp': datetime }).then((h) => {
                    hash = h;
                })
                //return a promise, fetch the server
                let endpoint = "post/log";
                if (!success) {
                    endpoint = "post/error";
                }
                return await fetch(this.API_ENDPOINT + endpoint, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        timestamp: datetime,
                        auth: hash,
                        data: data,
                        success
                    })
                }).then((response) => {
                    return response.json();
                }).then((data) => {
                    if (data.error) throw new Error("Při odesílání došlo k chybě");
                    ok(data);
                }).catch((error) => {
                    err(error);
                });
            });
        },
        async saveInstalledPatch(patchIndex = 0) {
            let gamePath = this.steamPath;
            let patch = this.game.patches[patchIndex];
            let gameID = this.getActiveGameID();
            let data = {
                game: gameID,
                version: patch.version,
                path: gamePath
            }
            // if already exists
            let patches = localStorage.getItem("installed_patches")
            if (patches) {
                patches = JSON.parse(patches);
                let index = patches.findIndex((p) => p.game == gameID);
                if (index != -1) {
                    patches[index] = data;
                } else {
                    patches.push(data);
                }
            } else {
                patches = [data];
            }
            localStorage.setItem("installed_patches", JSON.stringify(patches));
        },
        isSteamLink(link) {
            return /^(https?:)?\/\/store\.steampowered\.com/.test(link);
        },
        getInstalledPatchVersion(gameID) {
            let patches = localStorage.getItem("installed_patches");
            if (patches) {
                patches = JSON.parse(patches);
                let index = patches.findIndex((p) => p.game == gameID);
                if (index != -1) {
                    return patches[index].version;
                }
            }
            return false;
        },
        removeGameID(gameID) {
            let patches = localStorage.getItem("installed_patches");
            if (patches) {
                patches = JSON.parse(patches);
                let index = patches.findIndex((p) => p.game == gameID);
                if (index != -1) {
                    patches.splice(index, 1);
                    localStorage.setItem("installed_patches", JSON.stringify(patches));
                }
            }
        },
        isInstalled() {
            let patches = localStorage.getItem("installed_patches");
            let id = this.getActiveGameID();
            if (patches) {
                patches = JSON.parse(patches);
                let index = patches.findIndex((p) => p.game == id);
                if (index != -1) {
                    return true;
                }
            }
            return false;
        },
        async openGame() {
            let path = this.steamPath + "\\" + this.game.patches[0].exe;
            await openPath(path).catch(() => {
                this.fetchError = true;
                this.fetchErrorText = "Hru nelze spustit";
            });
        },
        scrollLog() {
            let log = this.$refs["install-log"];
            // after install
            if (this.installStep == installPages.done) {
                log = this.$refs["install-log-after"];
            }
            if (this.installStep == installPages.error) {
                log = this.$refs["install-log-after-err"];
            }
            //uninstall
            if (this.uninstallStep == 2) {
                log = this.$refs["uninstall-log"];
            }
            if (log) log.scrollTop = log.scrollHeight;
        }
    },
    computed: {
        lastSentenceFromUninstallLog() {
            let sentences = this.uninstallLog.split('<br>');
            let lastSentence = sentences[sentences.length - 1];
            return lastSentence.trim();
        }
    }
};
</script>
<style lang="scss" scoped>
:deep(a) {
    color: $alt;
}

.loading {
    display: flex;
    justify-content: center;
    align-items: center;
    position: absolute;
    top: 0;
    width: 70%;
    @include full-height;

    background: rgb(0 0 0 / 50%);
    z-index: 1;

    img {
        width: 100px;
        //set spin animation
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }

        100% {
            transform: rotate(360deg);
        }
    }
}

.game-view {
    height: 100vh;
    width: 70%;
    overflow-y: auto;

    &::-webkit-scrollbar {
        width: 20px;
    }

    &::-webkit-scrollbar-track {
        background-color: #e4e4e417;
        border-radius: 100px;
    }

    &::-webkit-scrollbar-thumb {
        border-radius: 100px;
        border: 5px solid transparent;
        background-clip: content-box;
        background-color: $alt;
    }
}

.select-a-game {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    width: 100%;
}

.header {
    display: flex;
    justify-content: center;
    align-items: center;
    flex-direction: column;
    // margin-bottom: 1em;
    position: relative;

    img {
        width: 100%;
        max-height: 30vh;
        object-fit: cover;
        position: relative;

        border-bottom-left-radius: 1em;
        border-bottom-right-radius: 1em;
    }

    .dimm-img {
        position: absolute;
        width: 100%;
        height: 30vh;
        background-color: rgba(0, 0, 0, 0.8);
        border-bottom-left-radius: 1em;
        border-bottom-right-radius: 1em;
    }

    h1 {
        margin-top: .5em;
        //  position: absolute;
    }
}

.content {
    padding: 1em;
    margin-bottom: 5em;
}

.tab {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1em;
    margin-bottom: .5em;

    &.actions {
        flex-direction: row;
    }
}

.state-of-the-translation {
    display: flex;
    justify-content: center;
    flex-direction: column;

    .category {
        display: flex;
        justify-content: space-evenly;
    }

    .row {
        display: flex;
        flex-direction: column;
        gap: 1em;
        justify-content: center;
        align-items: center;
    }


    progress {
        width: 100%;
    }
}

.pegi {
    .pegi-row {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 1em;
    }

    img {
        width: 100px;
    }
}

.install,
.uninstall {
    &.hidden {
        opacity: 0;
        z-index: -1;
    }

    .close {
        position: absolute;
        top: 20px;
        right: 20px;
        cursor: pointer;

        img {
            width: 2em;
        }
    }

    opacity: 1;
    z-index: 100;
    transition: opacity 0.3s ease-in-out,
    z-index 0.3s ease-in-out;

    display: flex;
    justify-content: center;
    align-items: center;
    position: absolute;
    width: 70%;
    @include full-height;
    top: 0;
    background-color: rgba(0, 0, 0, 0.7);

    .step {
        display: flex;
        flex-direction: column;
        gap: 1em;
        padding: 1em;
        background-color: $bg;

        border-radius: 10px;
        min-width: 80%;
        max-width: 90%;
        max-height: 60%;

        .log {
            overflow-y: auto;
        }

        .btn {
            border: 1px solid $alt;
        }

        input {
            background-color: #1a1a1a;
            border-color: $alt;
        }

        h2.title {
            //title
            text-align: center
        }

        .about-patch {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1em;

            .info {
                display: flex;
                align-items: center;
                gap: .5em;
            }
        }

        // loading blur
        .path-status {
            transition: all 0.3s;

            &.blur {
                filter: blur(5px);
            }
        }

        .path-wrapper {
            position: relative;
        }

        .path-wrapper .loader-txt {
            display: none;

            &.show {
                display: block;
                position: absolute;
                top: 0;
                background-color: #0000008e;
                padding: 2px 10px;
                border-radius: 10px;
            }
        }
    }
}

.small {
    max-height: 60%;
    overflow-y: auto;
    font-size: 0.6em;
}

// alerts
.alert {
    position: relative;
    padding: 1rem 1rem;
    margin-bottom: 1rem;
    border: 1px solid rgba(0, 0, 0, 0);
    display: flex !important;
    border-radius: 10px;
    align-items: center;
    gap: .5em;

    img {
        width: 30px;
    }
}

.alert-info {
    color: white;
    border-color: $main;
    background: rgba(93, 182, 205, .25);
}

.alert-warning {
    color: white;
    background-color: rgb(255, 243, 205, .25);
    border-color: #ffeeba;
}

// progress bar
$res: 10em;

.progress-row {
    margin-bottom: 1em;

    &:last-child {
        margin-bottom: 0;
    }
}

.progress-bar {
    position: relative;
    height: $res;
    width: $res;
    border-radius: 50%;
    background: rgba(0, 0, 0, .1);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 3s ease-in-out;

    &::before {
        content: "";
        position: absolute;
        height: calc($res - 40px);
        width: calc($res - 40px);
        border-radius: 50%;
        background-color: $mainbg;
    }

    span {
        position: relative;
        color: $alt;
        font-size: 2em;
    }
}
</style>