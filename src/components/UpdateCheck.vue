<script setup>
import { getVersion } from '@tauri-apps/api/app';
import Popup from "./Popup.vue";
</script>
<template>
    <Popup :active="updateAvailable" :title="str.update.title"
        :body="`${str.update.body} Aktuální verze: ${currentVersion}, nová verze: ${removeVersion}.`" type="update"
        @close="updateAvailable = false" @update="updateTheApp" @ignoreUpdate="ignoreUpdate" />
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
        //check if the update was ignored
        let ignore = localStorage.getItem("updateIgnore");
        if (ignore) {
            let now = new Date().getTime();
            if (now < ignore) return;
        }
        //get the current version
        this.currentVersion = await getVersion();
        //check for updates
        fetch(this.API_ENDPOINT + "version")
            .then((res) => res.json())
            .then((data) => {
                this.removeVersion = data.version;
                let cmp = this.cmpVersions(this.removeVersion, this.currentVersion);
                if (cmp) {
                    console.log(`${cmp} ${this.removeVersion} > ${this.currentVersion}`);
                    this.updateAvailable = true;
                } else //no update
                    console.log(`${this.removeVersion} <= ${this.currentVersion}`);
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
                    return 0;
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
        },
        ignoreUpdate() {
            //ignore the update for 7 days
            let date = new Date();
            date.setDate(date.getDate() + 7);
            let timestamp = date.getTime();
            localStorage.setItem("updateIgnore", timestamp);

            //close the popup
            this.updateAvailable = false;
        },
    },
};
</script>