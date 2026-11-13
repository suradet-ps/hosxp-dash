<template>
<div class="login-shell">
    <aside class="login-brand">
        <div class="login-brand__top">
            <img :src="logoUrl" class="login-brand__logo" alt="" />
            <div class="login-brand__titles">
                <span class="login-brand__name">HOSxP Dash</span>
                <span class="login-brand__sub">โรงพยาบาลสระโบสถ์</span>
            </div>
        </div>

        <div class="login-brand__stage">
            <span class="login-brand__eyebrow">PHARMACY ANALYTICS</span>
            <h1 class="login-brand__headline">
                เห็นภาพการใช้ยา<br />จากข้อมูล HOSxP
            </h1>
            <p class="login-brand__lead">
                แดชบอร์ดสำหรับงานเภสัชกรรม เปิดดูแนวโน้มการใช้ยา
                ค้นหาช่วงพีค และติดตามยาฉีด Antibiotics ได้ในที่เดียว
            </p>

            <ul class="login-brand__points">
                <li>
                    <CircleCheck :size="16" :stroke-width="2.4" aria-hidden="true" />
                    ภาพรวมรายปีและแนวโน้มรายเดือน
                </li>
                <li>
                    <CircleCheck :size="16" :stroke-width="2.4" aria-hidden="true" />
                    จัดอันดับรายการยาที่ใช้มากที่สุด
                </li>
                <li>
                    <CircleCheck :size="16" :stroke-width="2.4" aria-hidden="true" />
                    วิเคราะห์ยาฉีด Antibiotics ตามช่วงวันที่
                </li>
            </ul>
        </div>

        <div class="login-brand__foot">
            <ShieldCheck :size="15" :stroke-width="2.2" aria-hidden="true" />
            อ่านข้อมูลเท่านั้น · ไม่มีการเขียนกลับสู่ฐานข้อมูล HOSxP
        </div>
    </aside>

    <main class="login-main">
        <div class="login-toolbar">
            <button type="button" class="login-settings" @click="emit('open-settings')">
                <Settings :size="16" :stroke-width="2.2" aria-hidden="true" />
                ตั้งค่าการเชื่อมต่อ
            </button>
        </div>

        <div class="login-card">
            <header class="login-card__head">
                <span class="login-card__eyebrow">HOSxP OFFICER</span>
                <h2 class="login-card__title">เข้าสู่ระบบ</h2>
                <p class="login-card__sub">ใช้บัญชีเจ้าหน้าที่จากตาราง officer ใน HOSxP</p>
            </header>

            <div v-if="!dbStore.connected" class="login-notice" role="status">
                <WifiOff :size="18" :stroke-width="2.2" class="login-notice__icon" aria-hidden="true" />
                <div class="login-notice__body">
                    <b>ยังไม่ได้เชื่อมต่อฐานข้อมูล HOSxP</b>
                    <small>เปิดการตั้งค่าเพื่อเชื่อมต่อก่อนเข้าสู่ระบบ</small>
                    <small v-if="dbStore.connectionError" class="login-notice__error">
                        {{ dbStore.connectionError }}
                    </small>
                </div>
                <button type="button" class="login-notice__action" @click="emit('open-settings')">
                    ตั้งค่าการเชื่อมต่อ
                </button>
            </div>

            <div v-else class="login-status" role="status">
                <CircleCheck :size="15" :stroke-width="2.4" aria-hidden="true" />
                เชื่อมต่อฐานข้อมูลแล้ว · {{ dbStore.database }}
            </div>

            <form class="login-form" @submit.prevent="submit">
                <label class="login-field" for="login-username">
                    <span class="login-field__label">ชื่อผู้ใช้</span>
                    <span class="login-field__control">
                        <UserRound :size="17" :stroke-width="2" class="login-field__icon" aria-hidden="true" />
                        <input id="login-username" ref="usernameRef" v-model="username" type="text"
                            autocomplete="username" spellcheck="false" placeholder="ชื่อผู้ใช้ HOSxP"
                            :disabled="authStore.busy" />
                    </span>
                </label>

                <label class="login-field" for="login-password">
                    <span class="login-field__label">รหัสผ่าน</span>
                    <span class="login-field__control">
                        <LockKeyhole :size="17" :stroke-width="2" class="login-field__icon" aria-hidden="true" />
                        <input id="login-password" v-model="password" :type="showPassword ? 'text' : 'password'"
                            autocomplete="current-password" placeholder="••••••••" :disabled="authStore.busy" />
                        <button type="button" class="login-field__toggle"
                            :aria-label="showPassword ? 'ซ่อนรหัสผ่าน' : 'แสดงรหัสผ่าน'"
                            @click="showPassword = !showPassword">
                            <EyeOff v-if="showPassword" :size="16" :stroke-width="2" aria-hidden="true" />
                            <Eye v-else :size="16" :stroke-width="2" aria-hidden="true" />
                        </button>
                    </span>
                </label>

                <p v-if="errorMessage" class="login-error" role="alert">
                    <TriangleAlert :size="15" :stroke-width="2.4" aria-hidden="true" />
                    <span>{{ errorMessage }}</span>
                </p>

                <button type="submit" class="login-submit"
                    :disabled="!dbStore.connected || authStore.busy">
                    <Loader2 v-if="authStore.busy" :size="17" :stroke-width="2.5" class="spin" aria-hidden="true" />
                    <ArrowRight v-else :size="17" :stroke-width="2.5" aria-hidden="true" />
                    {{ authStore.busy ? 'กำลังเข้าสู่ระบบ...' : 'เข้าสู่ระบบ' }}
                </button>
            </form>

            <p class="login-card__foot">
                <ShieldCheck :size="14" :stroke-width="2.2" aria-hidden="true" />
                รหัสผ่านตรวจสอบในเครื่องนี้และไม่ถูกจัดเก็บ
            </p>
        </div>
    </main>
</div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
    ArrowRight,
    CircleCheck,
    Eye,
    EyeOff,
    Loader2,
    LockKeyhole,
    Settings,
    ShieldCheck,
    TriangleAlert,
    UserRound,
    WifiOff,
} from 'lucide-vue-next'
import logoUrl from '../assets/logo.svg'
import { useAuthStore } from '../stores/auth'
import { useDbConfigStore } from '../stores/dbConfig'

const emit = defineEmits<{ 'open-settings': [] }>()

const authStore = useAuthStore()
const dbStore = useDbConfigStore()

const username = ref('')
const password = ref('')
const showPassword = ref(false)
const localError = ref<string | null>(null)
const usernameRef = ref<HTMLInputElement | null>(null)

const errorMessage = computed(() => localError.value ?? authStore.error)

watch([username, password], () => {
    localError.value = null
    authStore.error = null
})

onMounted(() => {
    usernameRef.value?.focus()
})

async function submit() {
    localError.value = null
    if (!dbStore.connected) {
        localError.value = 'กรุณาเชื่อมต่อฐานข้อมูลก่อนเข้าสู่ระบบ'
        return
    }
    const loginName = username.value.trim()
    if (!loginName) {
        localError.value = 'กรุณากรอกชื่อผู้ใช้'
        return
    }
    if (!password.value) {
        localError.value = 'กรุณากรอกรหัสผ่าน'
        return
    }

    const ok = await authStore.login(loginName, password.value)
    if (ok) {
        password.value = ''
        showPassword.value = false
    }
}
</script>

<style scoped>
.login-shell {
    display: grid;
    grid-template-columns: minmax(0, 1.08fr) minmax(430px, 0.92fr);
    height: 100vh;
    overflow: hidden;
    background: var(--bg-base);
}

/* ── Brand panel ── */
.login-brand {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: var(--space-8);
    padding: clamp(30px, 5vw, 64px);
    overflow: hidden;
    background:
        radial-gradient(circle at 14% 8%, rgba(255, 142, 126, 0.26), transparent 42%),
        radial-gradient(circle at 88% 92%, rgba(78, 110, 47, 0.22), transparent 46%),
        linear-gradient(155deg, #f5f8ed 0%, #e8f0dc 52%, #fdf1ed 100%);
}

.login-brand::before,
.login-brand::after {
    content: "";
    position: absolute;
    border-radius: 50%;
    border: 1px solid rgba(78, 110, 47, 0.16);
    pointer-events: none;
}

.login-brand::before {
    width: 420px;
    height: 420px;
    top: -160px;
    right: -140px;
}

.login-brand::after {
    width: 300px;
    height: 300px;
    bottom: -120px;
    left: -90px;
    border-color: rgba(255, 142, 126, 0.24);
}

.login-brand__top {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: var(--space-3);
}

.login-brand__logo {
    width: 48px;
    height: 48px;
    object-fit: contain;
    filter: drop-shadow(0 4px 10px rgba(67, 48, 31, 0.12));
}

.login-brand__titles {
    display: flex;
    flex-direction: column;
    line-height: var(--leading-tight);
}

.login-brand__name {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
}

.login-brand__sub {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
}

.login-brand__stage {
    position: relative;
    z-index: 1;
    max-width: 520px;
}

.login-brand__eyebrow {
    display: inline-block;
    margin-bottom: var(--space-4);
    padding: 4px 10px;
    border: 1px solid rgba(78, 110, 47, 0.22);
    border-radius: var(--radius-pill);
    background: rgba(255, 255, 255, 0.6);
    color: var(--basil-500);
    font-size: var(--text-2xs);
    font-weight: 700;
    letter-spacing: 0.12em;
}

.login-brand__headline {
    font-size: clamp(30px, 3.3vw, 42px);
    font-weight: 700;
    line-height: 1.24;
    letter-spacing: -0.015em;
    color: var(--text-primary);
}

.login-brand__lead {
    margin-top: var(--space-4);
    max-width: 440px;
    font-size: var(--text-base);
    line-height: 1.65;
    color: var(--text-secondary);
}

.login-brand__points {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: var(--space-6);
    list-style: none;
}

.login-brand__points li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-md);
    font-weight: 500;
    color: var(--text-secondary);
}

.login-brand__points svg {
    flex-shrink: 0;
    color: var(--basil-400);
}

.login-brand__foot {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-muted);
}

.login-brand__foot svg {
    color: var(--basil-400);
    flex-shrink: 0;
}

/* ── Login panel ── */
.login-main {
    display: flex;
    flex-direction: column;
    padding: var(--space-5) clamp(24px, 4vw, 56px) var(--space-8);
    overflow-y: auto;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-card);
}

.login-toolbar {
    display: flex;
    justify-content: flex-end;
    flex-shrink: 0;
}

.login-settings {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 7px 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 600;
    cursor: pointer;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease),
        border-color var(--dur) var(--ease);
}

.login-settings:hover {
    background: var(--bg-inset);
    border-color: var(--border-card);
    color: var(--text-primary);
}

.login-card {
    width: 100%;
    max-width: 400px;
    margin: auto;
    padding: var(--space-8);
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-md);
    animation: fadeSlideUp var(--dur-slow) var(--ease);
}

.login-card__eyebrow {
    display: block;
    margin-bottom: var(--space-2);
    color: var(--basil-400);
    font-size: var(--text-2xs);
    font-weight: 700;
    letter-spacing: 0.14em;
}

.login-card__title {
    font-size: var(--text-2xl);
    font-weight: 700;
    line-height: var(--leading-tight);
    color: var(--text-primary);
}

.login-card__sub {
    margin-top: 6px;
    font-size: var(--text-md);
    line-height: 1.5;
    color: var(--text-muted);
}

.login-notice {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    margin-top: var(--space-5);
    padding: 12px 14px;
    border: 1px solid rgba(180, 83, 9, 0.28);
    border-radius: var(--radius-md);
    background: rgba(255, 248, 225, 0.72);
    color: #8a6420;
}

.login-notice__icon {
    flex-shrink: 0;
    margin-top: 2px;
}

.login-notice__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
}

.login-notice__body b {
    font-size: var(--text-md);
}

.login-notice__body small {
    font-size: var(--text-sm);
    line-height: 1.45;
}

.login-notice__error {
    margin-top: 4px;
    color: var(--chili-500);
    word-break: break-word;
}

.login-notice__action {
    align-self: center;
    flex-shrink: 0;
    padding: 6px 10px;
    border: 1px solid rgba(180, 83, 9, 0.3);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.82);
    color: #8a6420;
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
    transition: background var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
}

.login-notice__action:hover {
    background: #ffffff;
    border-color: rgba(180, 83, 9, 0.5);
}

.login-status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-5);
    padding: 8px 12px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--basil-50);
    color: var(--basil-500);
    font-size: var(--text-sm);
    font-weight: 600;
}

.login-status svg {
    flex-shrink: 0;
}

.login-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-top: var(--space-5);
}

.login-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.login-field__label {
    font-size: var(--text-xs);
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
}

.login-field__control {
    position: relative;
    display: flex;
    align-items: center;
}

.login-field__icon {
    position: absolute;
    left: 12px;
    color: var(--text-muted);
    pointer-events: none;
}

.login-field__control input {
    width: 100%;
    padding: 11px 40px 11px 38px;
    border: 1px solid var(--border-card);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-family: inherit;
    font-size: var(--text-base);
    outline: none;
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease),
        background var(--dur) var(--ease);
}

.login-field__control input:hover:not(:disabled) {
    border-color: var(--border-strong);
}

.login-field__control input:focus {
    border-color: var(--basil-400);
    background: var(--bg-elevated);
    box-shadow: var(--focus-ring);
}

.login-field__control input::placeholder {
    color: var(--text-muted);
    opacity: 0.7;
}

.login-field__control input:disabled {
    opacity: 0.65;
    cursor: not-allowed;
}

.login-field__toggle {
    position: absolute;
    right: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.login-field__toggle:hover {
    color: var(--basil-400);
    background: var(--basil-50);
}

.login-error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: 9px 12px;
    border: 1px solid rgba(192, 57, 43, 0.24);
    border-radius: var(--radius-sm);
    background: rgba(192, 57, 43, 0.06);
    color: var(--chili-500);
    font-size: var(--text-md);
    font-weight: 600;
    line-height: 1.45;
}

.login-error svg {
    flex-shrink: 0;
    margin-top: 1px;
}

.login-submit {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    margin-top: var(--space-1);
    padding: 12px;
    border: none;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, var(--basil-400), var(--basil-500));
    color: #ffffff;
    font-family: inherit;
    font-size: var(--text-lg);
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 8px var(--basil-glow-strong);
    transition: box-shadow var(--dur) var(--ease), transform var(--dur-fast) var(--ease),
        opacity var(--dur) var(--ease);
}

.login-submit:hover:not(:disabled) {
    box-shadow: 0 4px 16px var(--basil-glow-strong);
}

.login-submit:active:not(:disabled) {
    transform: translateY(1px);
    box-shadow: var(--shadow-xs);
}

.login-submit:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    box-shadow: none;
}

.login-card__foot {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    margin-top: var(--space-5);
    color: var(--text-muted);
    font-size: var(--text-sm);
}

.login-card__foot svg {
    flex-shrink: 0;
    color: var(--basil-400);
}

@media (max-width: 980px) {
    .login-shell {
        grid-template-columns: 1fr;
        overflow-y: auto;
    }

    .login-brand {
        padding: var(--space-6);
        gap: var(--space-6);
    }

    .login-brand__points,
    .login-brand__foot {
        display: none;
    }

    .login-brand__headline {
        font-size: var(--text-3xl);
    }

    .login-main {
        border-left: none;
        overflow: visible;
    }

    .login-card {
        margin: var(--space-4) auto var(--space-8);
    }
}
</style>
