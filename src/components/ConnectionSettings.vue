<template>
<div class="settings-overlay" v-if="show" @click.self="$emit('close')" @keydown.esc="$emit('close')">
    <div ref="modalRef" class="settings-modal" role="dialog" aria-modal="true"
        aria-labelledby="connection-settings-title" @keydown.tab="trapFocus">
        <div class="modal-header">
            <h2 id="connection-settings-title">
                <span class="modal-title-icon"><DatabaseIcon :size="18" :stroke-width="2" /></span>
                ตั้งค่าการเชื่อมต่อฐานข้อมูล
            </h2>
            <button type="button" class="close-btn" aria-label="ปิดหน้าต่างตั้งค่า" @click="$emit('close')">
                <XIcon :size="18" :stroke-width="2" />
            </button>
        </div>

        <div class="connection-status" :class="dbStore.connected ? 'status-ok' : 'status-err'">
            <CircleCheck v-if="dbStore.connected" :size="15" :stroke-width="2.5" />
            <CircleX v-else :size="15" :stroke-width="2.5" />
            <span>{{ dbStore.connected ? 'เชื่อมต่อสำเร็จ' : 'ยังไม่ได้เชื่อมต่อ' }}</span>
        </div>

        <form @submit.prevent="handleConnect" class="settings-form">
            <div class="form-grid">
                <div class="form-row form-row--host">
                    <label for="db-host">Host</label>
                    <input id="db-host" ref="firstFieldRef" v-model="form.host" type="text"
                        placeholder="localhost" spellcheck="false" :disabled="dbStore.connecting" />
                </div>
                <div class="form-row form-row--port">
                    <label for="db-port">Port</label>
                    <input id="db-port" v-model.number="form.port" type="number" inputmode="numeric"
                        placeholder="3306" :disabled="dbStore.connecting" />
                </div>
            </div>

            <div class="form-row">
                <label for="db-user">User</label>
                <input id="db-user" v-model="form.user" type="text" placeholder="hosxp_user"
                    autocomplete="username" spellcheck="false" :disabled="dbStore.connecting" />
            </div>

            <div class="form-row">
                <label for="db-password">Password</label>
                <div class="password-wrap">
                    <input id="db-password" v-model="form.password" :type="showPassword ? 'text' : 'password'"
                        placeholder="••••••••" autocomplete="current-password" :disabled="dbStore.connecting" />
                    <button type="button" class="password-toggle"
                        :aria-label="showPassword ? 'ซ่อนรหัสผ่าน' : 'แสดงรหัสผ่าน'"
                        @click="showPassword = !showPassword">
                        <EyeOff v-if="showPassword" :size="16" :stroke-width="2" />
                        <Eye v-else :size="16" :stroke-width="2" />
                    </button>
                </div>
            </div>

            <div class="form-row">
                <label for="db-name">Database</label>
                <input id="db-name" v-model="form.database" type="text" placeholder="hospdb"
                    spellcheck="false" :disabled="dbStore.connecting" />
            </div>

            <p class="security-note">
                <ShieldAlert :size="14" :stroke-width="2" class="note-icon" />
                รหัสผ่านถูกเก็บเฉพาะบนเครื่องนี้ และเชื่อมต่อแบบอ่านข้อมูลเท่านั้น
            </p>

            <div class="form-actions">
                <button type="submit" class="btn-connect" :disabled="dbStore.connecting">
                    <Loader2 v-if="dbStore.connecting" :size="16" :stroke-width="2.5" class="spin" />
                    <Plug v-else :size="16" :stroke-width="2.5" />
                    {{ dbStore.connecting ? 'กำลังเชื่อมต่อ...' : 'ทดสอบการเชื่อมต่อ' }}
                </button>
            </div>

            <div v-if="dbStore.connectionError" class="error-msg" role="alert">
                <TriangleAlert :size="14" :stroke-width="2.4" class="msg-icon" />
                <span>{{ dbStore.connectionError }}</span>
            </div>
            <div v-if="successMsg" class="success-msg" role="status">
                <CircleCheck :size="15" :stroke-width="2.4" class="msg-icon" />
                <span>{{ successMsg }}</span>
            </div>
        </form>
    </div>
</div>
</template>

<script setup lang="ts">
import { reactive, ref, watch, nextTick } from 'vue'
import {
    Database as DatabaseIcon,
    X as XIcon,
    CircleCheck,
    CircleX,
    ShieldAlert,
    Plug,
    Loader2,
    Eye,
    EyeOff,
    TriangleAlert,
} from 'lucide-vue-next'
import { useAuthStore } from '../stores/auth'
import { useDbConfigStore } from '../stores/dbConfig'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ close: [] }>()

const authStore = useAuthStore()
const dbStore = useDbConfigStore()
const successMsg = ref('')
const showPassword = ref(false)

const modalRef = ref<HTMLElement | null>(null)
const firstFieldRef = ref<HTMLInputElement | null>(null)

const form = reactive({
    host: dbStore.host,
    port: dbStore.port,
    user: dbStore.user,
    password: dbStore.password,
    database: dbStore.database,
})

watch(() => props.show, async (val) => {
    clearTimeout(autoCloseTimer)
    if (val) {
        form.host = dbStore.host
        form.port = dbStore.port
        form.user = dbStore.user
        form.password = dbStore.password
        form.database = dbStore.database
        successMsg.value = ''
        showPassword.value = false
        await nextTick()
        firstFieldRef.value?.focus()
    }
})

let autoCloseTimer: ReturnType<typeof setTimeout>

/** Keep Tab / Shift+Tab cycling inside the dialog while it is open */
function trapFocus(event: KeyboardEvent) {
    const modal = modalRef.value
    if (!modal) return
    const focusables = Array.from(
        modal.querySelectorAll<HTMLElement>(
            'button:not(:disabled), input:not(:disabled), select, [tabindex]:not([tabindex="-1"])'
        )
    )
    if (!focusables.length) return
    const first = focusables[0]
    const last = focusables[focusables.length - 1]
    const active = document.activeElement as HTMLElement | null

    if (event.shiftKey && active === first) {
        event.preventDefault()
        last.focus()
    } else if (!event.shiftKey && active === last) {
        event.preventDefault()
        first.focus()
    }
}

async function handleConnect() {
    successMsg.value = ''
    try {
        await dbStore.connect({ ...form })
        // A changed connection clears the backend session; mirror that so the
        // login gate returns instead of leaving stale dashboard access.
        await authStore.restoreSession()
        successMsg.value = 'เชื่อมต่อสำเร็จ'
        // The dashboard loads behind the dialog — step out of the way.
        autoCloseTimer = setTimeout(() => emit('close'), 900)
    } catch {
        // error shown via dbStore.connectionError
    }
}
</script>

<style scoped>
.settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(58, 40, 24, 0.48);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
    animation: fadeIn var(--dur) var(--ease);
}

.settings-modal {
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-xl);
    padding: var(--space-6) var(--space-8) var(--space-8);
    width: 460px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
    animation: fadeSlideUp var(--dur-slow) var(--ease);
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-4);
    gap: var(--space-3);
}

.modal-header h2 {
    font-size: var(--text-lg);
    font-weight: 700;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    line-height: var(--leading-tight);
}

.modal-title-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    background: var(--basil-50);
    color: var(--basil-400);
    flex-shrink: 0;
}

.close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 7px;
    border-radius: var(--radius-xs);
    line-height: 0;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.close-btn:hover {
    color: var(--text-primary);
    background: var(--basil-50);
}

.connection-status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 8px 14px;
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-5);
    font-size: var(--text-md);
    font-weight: 600;
}

.status-ok {
    background: var(--basil-50);
    border: 1px solid var(--border-subtle);
    color: var(--basil-500);
}

.status-err {
    background: rgba(192, 57, 43, 0.07);
    border: 1px solid rgba(192, 57, 43, 0.22);
    color: var(--chili-400);
}

.settings-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
}

.form-grid {
    display: grid;
    grid-template-columns: 1fr 116px;
    gap: var(--space-3);
}

.form-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.form-row label {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
}

.form-row input {
    background: var(--bg-surface);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-sm);
    padding: 9px 13px;
    color: var(--text-primary);
    font-family: inherit;
    font-size: var(--text-base);
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
    outline: none;
    width: 100%;
}

.form-row input:focus {
    border-color: var(--basil-400);
    box-shadow: var(--focus-ring);
}

.form-row input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.form-row input::placeholder {
    color: var(--text-muted);
    opacity: 0.75;
}

.password-wrap {
    position: relative;
}

.password-wrap input {
    padding-right: 42px;
}

.password-toggle {
    position: absolute;
    top: 50%;
    right: 6px;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    background: none;
    border: none;
    border-radius: var(--radius-xs);
    color: var(--text-muted);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.password-toggle:hover {
    color: var(--basil-400);
    background: var(--basil-50);
}

.security-note {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    padding: 10px 13px;
    background: var(--bg-inset);
    border-radius: var(--radius-sm);
    border-left: 3px solid var(--basil-300);
    line-height: 1.5;
}

.note-icon {
    flex-shrink: 0;
    margin-top: 2px;
    color: var(--basil-400);
}

.form-actions {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-1);
}

.btn-connect {
    flex: 1;
    background: linear-gradient(135deg, var(--basil-400), var(--basil-500));
    border: none;
    border-radius: var(--radius-md);
    padding: 12px;
    color: #fff;
    font-family: inherit;
    font-size: var(--text-lg);
    font-weight: 700;
    cursor: pointer;
    transition: opacity var(--dur) var(--ease), transform var(--dur-fast) var(--ease),
        box-shadow var(--dur) var(--ease);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    box-shadow: 0 2px 8px var(--basil-glow-strong);
}

.btn-connect:hover:not(:disabled) {
    box-shadow: 0 4px 16px var(--basil-glow-strong);
}

.btn-connect:active:not(:disabled) {
    transform: translateY(1px);
    box-shadow: var(--shadow-xs);
}

.btn-connect:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    box-shadow: none;
}

.error-msg,
.success-msg {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    font-size: var(--text-md);
    line-height: 1.5;
    word-break: break-word;
}

.msg-icon {
    flex-shrink: 0;
    margin-top: 2px;
}

.error-msg {
    background: rgba(192, 57, 43, 0.07);
    border: 1px solid rgba(192, 57, 43, 0.22);
    color: var(--chili-500);
}

.success-msg {
    background: var(--basil-50);
    border: 1px solid var(--border-subtle);
    color: var(--basil-500);
    font-weight: 600;
}
</style>
