<template>
<div class="app-shell">
    <!-- Header -->
    <header class="app-header">
        <div class="header-brand">
            <img class="brand-icon" :src="logoUrl" alt="" />
            <div class="brand-text">
                <span class="brand-title">HOSxP Dash</span>
                <span class="brand-sub">โรงพยาบาลสระโบสถ์</span>
            </div>
        </div>

        <nav class="view-switch" aria-label="สลับมุมมองรายงาน">
            <button type="button" class="view-switch-btn" :class="{ active: view === 'overview' }"
                :aria-pressed="view === 'overview'" @click="view = 'overview'">
                <ChartColumnBig :size="14" :stroke-width="2.4" />
                ภาพรวมรายปี
            </button>
            <button type="button" class="view-switch-btn" :class="{ active: view === 'antibiotics' }"
                :aria-pressed="view === 'antibiotics'" @click="view = 'antibiotics'">
                <Syringe :size="14" :stroke-width="2.4" />
                ยาฉีด Antibiotics
            </button>
        </nav>

        <div class="header-controls">
            <YearSelector v-if="view === 'overview'" />
            <DrugSearchBar v-if="view === 'overview'" />
            <button type="button" class="connection-badge" :class="dbStore.connected ? 'badge-ok' : 'badge-err'"
                :title="dbStore.connected ? 'เชื่อมต่อฐานข้อมูลแล้ว — คลิกเพื่อดูการตั้งค่า' : 'ยังไม่ได้เชื่อมต่อ — คลิกเพื่อตั้งค่า'"
                @click="showSettings = true">
                <Wifi v-if="dbStore.connected" :size="13" :stroke-width="2.5" aria-hidden="true" />
                <WifiOff v-else :size="13" :stroke-width="2.5" aria-hidden="true" />
                {{ dbStore.connected ? 'เชื่อมต่อแล้ว' : 'ไม่ได้เชื่อมต่อ' }}
            </button>
            <button type="button" class="settings-btn" aria-label="ตั้งค่าการเชื่อมต่อ" title="ตั้งค่า"
                @click="showSettings = true">
                <Settings :size="17" :stroke-width="2" aria-hidden="true" />
            </button>
        </div>
    </header>

    <!-- Main content -->
    <main class="app-main">
        <template v-if="view === 'overview'">
            <div class="overview">
                <!-- KPI row -->
                <SummaryKpiBar />

                <!-- Panels -->
                <div class="overview-grid">
                    <aside class="panel panel--sidebar">
                        <TopDrugsPanel :drugs="dashStore.topDrugs" :loading="dashStore.loading" />
                    </aside>
                    <section class="panel panel--chart">
                        <DrugTrendChart :data="dashStore.currentChartData" :loading="dashStore.loading" />
                    </section>
                </div>
            </div>
        </template>

        <!-- Injectable-antibiotic dashboard -->
        <AntibioticsDashboard v-else />
    </main>

    <!-- Error banner -->
    <div v-if="dashStore.error" class="error-banner" role="alert" aria-live="assertive">
        <TriangleAlert :size="16" :stroke-width="2.5" class="error-icon" aria-hidden="true" />
        <span class="error-text">{{ dashStore.error }}</span>
        <button type="button" class="error-close" aria-label="ปิดข้อความแจ้งเตือน" @click="dashStore.error = null">
            <X :size="15" :stroke-width="2.5" aria-hidden="true" />
        </button>
    </div>

    <!-- Connection Settings Modal -->
    <ConnectionSettings :show="showSettings" @close="showSettings = false" />
</div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Wifi, WifiOff, Settings, TriangleAlert, X, ChartColumnBig, Syringe } from 'lucide-vue-next'
import { useDbConfigStore } from './stores/dbConfig'
import { useDashboardStore } from './stores/dashboard'
import { useDrugData } from './composables/useDrugData'
import ConnectionSettings from './components/ConnectionSettings.vue'
import YearSelector from './components/YearSelector.vue'
import DrugSearchBar from './components/DrugSearchBar.vue'
import TopDrugsPanel from './components/TopDrugsPanel.vue'
import SummaryKpiBar from './components/SummaryKpiBar.vue'
import DrugTrendChart from './components/DrugTrendChart.vue'
import AntibioticsDashboard from './components/AntibioticsDashboard.vue'
import logoUrl from './assets/logo.svg'

const dbStore = useDbConfigStore()
const dashStore = useDashboardStore()
const { fetchDashboardData, fetchDrugMonthly } = useDrugData()

const showSettings = ref(false)
const view = ref<'overview' | 'antibiotics'>('overview')

async function loadDashboard() {
    if (!dbStore.connected) return
    dashStore.loading = true
    dashStore.error = null
    try {
        // Single IPC call — Rust runs years + top-drugs queries in parallel (tokio::join!)
        const bundle = await fetchDashboardData(dashStore.selectedYear, 10)
        const { years, top_drugs: topDrugs } = bundle
        dashStore.availableYears = years
        if (years.length && !years.includes(dashStore.selectedYear)) {
            dashStore.selectedYear = years[0]
        }
        dashStore.topDrugs = topDrugs
        // Auto-select first drug if none selected
        if (topDrugs.length && !dashStore.selectedIcode) {
            dashStore.selectedIcode = topDrugs[0].icode
        }
    } catch (e) {
        dashStore.error = String(e)
    } finally {
        dashStore.loading = false
    }
}

async function loadDrugChart() {
    if (!dbStore.connected || !dashStore.selectedIcode) return
    dashStore.loading = true
    try {
        const data = await fetchDrugMonthly(dashStore.selectedYear, dashStore.selectedIcode)
        dashStore.currentChartData = data
    } catch (e) {
        dashStore.error = String(e)
    } finally {
        dashStore.loading = false
    }
}

// Watch year change → reload dashboard + chart
watch(
    () => dashStore.selectedYear,
    async () => {
        await loadDashboard()
        if (dashStore.selectedIcode) await loadDrugChart()
    }
)

// Watch selected drug change → reload chart only
watch(
    () => dashStore.selectedIcode,
    async (icode) => {
        if (icode) await loadDrugChart()
    }
)

// Watch connection state → trigger initial data load
watch(
    () => dbStore.connected,
    async (connected) => {
        if (connected) {
            await loadDashboard()
            if (dashStore.selectedIcode) await loadDrugChart()
        }
    }
)

onMounted(async () => {
    await dbStore.initFromDisk()
    if (dbStore.connected) {
        await loadDashboard()
        if (dashStore.selectedIcode) await loadDrugChart()
    } else {
        // Show settings modal on first launch if not connected
        showSettings.value = true
    }
})
</script>

<style scoped>
.app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    position: relative;
}

/* ── Header ── */
.app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px var(--space-5);
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-card);
    box-shadow: var(--shadow-xs);
    flex-shrink: 0;
    gap: var(--space-4);
    z-index: 20;
    position: relative;
}

/* brand accent strip */
.app-header::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--basil-500), var(--basil-300) 60%, var(--shrimp-400));
}

.header-brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-shrink: 0;
}

.brand-icon {
    width: 36px;
    height: 36px;
    object-fit: contain;
    flex-shrink: 0;
}

.brand-text {
    display: flex;
    flex-direction: column;
    line-height: var(--leading-tight);
}

.brand-title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 0.005em;
}

.brand-sub {
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-muted);
    margin-top: 1px;
}

/* ── View switch (segmented control) ── */
.view-switch {
    display: flex;
    gap: 2px;
    padding: 3px;
    background: var(--bg-inset);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-md);
    flex-shrink: 0;
}

.view-switch-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 7px;
    padding: 6px 14px;
    min-height: 30px;
    cursor: pointer;
    white-space: nowrap;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease),
        box-shadow var(--dur) var(--ease);
}

.view-switch-btn:hover:not(.active) {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.75);
}

.view-switch-btn.active {
    background: #ffffff;
    color: var(--basil-600);
    border-color: rgba(75, 54, 33, 0.08);
    box-shadow: var(--shadow-xs);
}

.header-controls {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    justify-content: flex-end;
    min-width: 0;
}

/* ── Connection badge ── */
.connection-badge {
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 600;
    padding: 6px 14px;
    border-radius: var(--radius-pill);
    cursor: pointer;
    white-space: nowrap;
    border: 1px solid transparent;
    transition: box-shadow var(--dur) var(--ease), background var(--dur) var(--ease);
    user-select: none;
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 32px;
}

.connection-badge:hover {
    box-shadow: var(--shadow-sm);
}

.connection-badge:active {
    transform: translateY(1px);
}

.badge-ok {
    background: var(--basil-50);
    color: var(--basil-600);
    border-color: var(--border-subtle);
}

.badge-err {
    background: rgba(192, 57, 43, 0.07);
    color: var(--chili-500);
    border-color: rgba(192, 57, 43, 0.22);
    animation: pulseSoft 2.4s ease-in-out infinite;
}

.settings-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 7px;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1;
    transition: background var(--dur) var(--ease), color var(--dur) var(--ease),
        border-color var(--dur) var(--ease);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    min-height: 32px;
}

.settings-btn:hover {
    background: var(--bg-inset);
    color: var(--text-primary);
    border-color: var(--border-card);
}

.settings-btn:active {
    background: #e8e8dd;
}

/* ── Main layout ── */
.app-main {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-base);
}

/* ── Overview view ── */
.overview {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    overflow-y: auto;
}

.overview-grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 320px minmax(0, 1fr);
    gap: var(--space-4);
}

.panel {
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 460px;
    min-width: 0;
    transition: box-shadow var(--dur-slow) var(--ease);
}

.panel:hover {
    box-shadow: var(--shadow-md);
}

/* ── Error banner ── */
.error-banner {
    position: fixed;
    bottom: var(--space-5);
    left: 50%;
    transform: translateX(-50%);
    background: rgba(255, 255, 255, 0.97);
    border: 1px solid rgba(169, 50, 38, 0.4);
    border-left: 4px solid var(--chili-400);
    color: var(--chili-500);
    padding: 12px 14px 12px 18px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-md);
    font-weight: 600;
    backdrop-filter: blur(10px);
    z-index: 999;
    max-width: min(680px, calc(100vw - 48px));
    box-shadow: var(--shadow-lg);
    animation: fadeSlideUp var(--dur-slow) var(--ease);
}

.error-icon {
    flex-shrink: 0;
    color: var(--chili-400);
}

.error-text {
    flex: 1;
    line-height: 1.45;
    word-break: break-word;
}

.error-close {
    background: none;
    border: none;
    color: var(--chili-400);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-xs);
    line-height: 0;
    flex-shrink: 0;
    opacity: 0.75;
    transition: opacity var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.error-close:hover {
    opacity: 1;
    background: rgba(192, 57, 43, 0.1);
}

@media (max-width: 1180px) {
    .brand-sub {
        display: none;
    }

    .overview-grid {
        grid-template-columns: 280px minmax(0, 1fr);
    }
}
</style>
