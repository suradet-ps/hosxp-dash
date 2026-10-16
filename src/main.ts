import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
// Sarabun (national Thai documentary font) — bundled via @fontsource so the
// app renders consistently on offline hospital machines.
import '@fontsource/sarabun/400.css'
import '@fontsource/sarabun/500.css'
import '@fontsource/sarabun/600.css'
import '@fontsource/sarabun/700.css'
import './styles/peach-theme.css'

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
