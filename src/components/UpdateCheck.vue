<script setup>
import { getVersion } from '@tauri-apps/api/app';
import Popup from "./Popup.vue";
</script>
<template>
    <Popup :active="updateAvailable" title="Je dostupná aktualizace"
        :body="`Nová verze aplikace. Aktuální verze: ${currentVersion}, nová verze: ${removeVersion}.`" type="update"
        @close="updateAvailable = false" @update="updateTheApp" />
</template>
<script>
export default {
    name: "UpdateCheck",
    data() {
        return {
            updateAvailable: false,
            currentVersion: "",
            removeVersion: "",
        };
    },
    async mounted() {
        this.currentVersion = await getVersion();
        //check for updates
        fetch(this.API_ENDPOINT + "version")
            .then((res) => res.json())
            .then((data) => {
                this.removeVersion = data.version;
                let cmp = this.cmpVersions(this.removeVersion, this.currentVersion);
                console.log(cmp, this.removeVersion, this.currentVersion);
                if (cmp) {
                    this.updateAvailable = true;
                }
            });
    },
    methods: {
        cmpVersions(v1, v2) {
            var v1parts = v1.split('.');
            var v2parts = v2.split('.');
            var maxLen = Math.max(v1parts.length, v2parts.length);
            for (var i = 0; i < maxLen; i++) {
                var v1part = parseInt(v1parts[i], 10);
                var v2part = parseInt(v2parts[i], 10);
                if (v1part < v2part) {
                    return -1;
                }
                if (v1part > v2part) {
                    return 1;
                }
            }
            return 0;
        },
        updateTheApp() {
            //TODO donwload the update

            //close the popup
            this.updateAvailable = false;
        }
    },
};
</script>