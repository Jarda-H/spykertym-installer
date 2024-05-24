<template>
    <div class="popup" @click="closeBg">
        <div class="content">
            <div class="header">
                <h2>{{ title }}</h2>
                <img src="../assets/close.svg" alt="close icon" @click="close" v-if="type == 'close'">
                <img src="../assets/loading.svg" alt="retry icon" @click="reload" v-if="type == 'retry'">
            </div>
            <div class="body">
                {{ body }}
            </div>
        </div>
    </div>
</template>
<script>
export default {
    name: "Popup",
    props: {
        active: Boolean,
        title: String,
        body: String,
        type: String
    },
    mounted() {
        if (this.active) {
            this.$el.classList.add("active");
        }
    },
    watch: {
        active() {
            if (this.active) {
                this.$el.classList.add("active");
            } else {
                this.$el.classList.remove("active");
            }
        }
    },
    methods: {
        closeBg(e) {
            if (this.type != 'close') return;
            console.log(e.target.classList)
            if (['header', 'body'].includes(e.target.classList[0])) return;
            this.$emit('close');
        },
        close() {
            this.$emit('close');
        },
        reload() {
            location.reload();
        }
    }
};
</script>
<style lang="scss" scoped>
.popup {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    @include full-height;
    background: rgba(0, 0, 0, .8);
    display: flex;
    justify-content: center;
    align-items: center;
    border-radius: 10px;

    transition: opacity .3s ease-in-out, z-index 0.3s ease-in-out;
    z-index: -1;
    opacity: 0;

    &.active {
        z-index: 1;
        opacity: 1;
    }
}

img {
    width: 2em;
    cursor: pointer;
}

.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1em;
    background: $main3;
    color: white;
}

.body {
    padding: 4em;
    background: $bg;
}
</style>