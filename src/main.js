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
}

// app.config.globalProperties.API_ENDPOINT = 'https://spykertym.cz/app/';
// app.config.globalProperties.prod = true;

app.config.globalProperties.API_ENDPOINT = 'https://beta.spykertym.cz/app/';
app.config.globalProperties.prod = false;

app.use(pinia)
app.mount('#app')
