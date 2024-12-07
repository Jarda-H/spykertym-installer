import { createApp } from "vue";
import { createPinia } from 'pinia';
import "./styles.css";
import App from "./App.vue";

const pinia = createPinia()
const app = createApp(App)


// global strings
app.config.globalProperties.str = {
    server_error: "Chyba na straně serveru, zkuste to prosím později.",
    steam_open_error: "Nepodařilo se otevřít Steam.",
    no_internet: "Pro fungování se musíte připojit k internetu.",

    update: {
        title: "Aktualizace",
        body: "Je dostupná nová verze aplikace.",
        ignore: "Ignorovat po dobu 7 dní",
        do: "Aktualizovat",
    }
}

app.config.globalProperties.API_ENDPOINT = 'https://beta.spykertym.cz/app/';
app.config.globalProperties.prod = false;

if (process.env.NODE_ENV === 'production') {
    app.config.globalProperties.API_ENDPOINT = 'https://spykertym.cz/app/';
    app.config.globalProperties.prod = true;
}
function debugPrint(...args) {
    if (!app.config.globalProperties.prod) {
        console.log(...args);
    }
}
window.debugPrint = debugPrint;

app.use(pinia)
app.mount('#app')
