<template>
<div class="app-shell">
    <!-- Header -->
    <header class="app-header">
        <div class="header-brand">
            <img class="brand-icon" :src="logoUrl" alt="HOSxp Dash Logo" />
            <div class="brand-text">
                <span class="brand-title">HOSxp Dash</span>
                <span class="brand-sub">โรงพยาบาลสระโบสถ์</span>
            </div>
        </div>

        <div class="header-controls">
            <YearSelector />
            <DrugSearchBar />
            <div class="connection-badge" :class="dbStore.connected ? 'badge-ok' : 'badge-err'"
                @click="showSettings = true" title="คลิกเพื่อตั้งค่าการเชื่อมต่อ">
                <Wifi v-if="dbStore.connected" :size="13" :stroke-width="2.5" />
                <WifiOff v-else :size="13" :stroke-width="2.5" />
                {{ dbStore.connected ? 'เชื่อมต่อแล้ว' : 'ไม่ได้เชื่อมต่อ' }}
            </div>
            <button class="settings-btn" @click="showSettings = true" title="ตั้งค่า">
                <Settings :size="18" :stroke-width="2" />
            </button>
        </div>
    </header>

    <!-- Main content -->
    <main class="app-main">
        <!-- Sidebar: Top drugs -->
        <aside class="sidebar">
            <TopDrugsPanel :drugs="dashStore.topDrugs" :loading="dashStore.loading" />
        </aside>

        <!-- Right panel -->
        <div class="content-area">
            <!-- KPI Bar -->
            <SummaryKpiBar />

            <!-- Chart area -->
            <div class="chart-card">
                <DrugTrendChart :data="dashStore.currentChartData" :loading="dashStore.loading" />
            </div>
        </div>
    </main>

    <!-- Error banner -->
    <div v-if="dashStore.error" class="error-banner">
        <TriangleAlert :size="16" :stroke-width="2.5" class="error-icon" />
        <span>{{ dashStore.error }}</span>
        <button @click="dashStore.error = null"><X :size="15" :stroke-width="2.5" /></button>
    </div>

    <!-- Connection Settings Modal -->
    <ConnectionSettings :show="showSettings" @close="showSettings = false" />
</div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Wifi, WifiOff, Settings, TriangleAlert, X } from 'lucide-vue-next'
import { useDbConfigStore } from './stores/dbConfig'
import { useDashboardStore } from './stores/dashboard'
import { useDrugData } from './composables/useDrugData'
import ConnectionSettings from './components/ConnectionSettings.vue'
import YearSelector from './components/YearSelector.vue'
import DrugSearchBar from './components/DrugSearchBar.vue'
import TopDrugsPanel from './components/TopDrugsPanel.vue'
import SummaryKpiBar from './components/SummaryKpiBar.vue'
import DrugTrendChart from './components/DrugTrendChart.vue'
import logoUrl from './assets/logo.svg'

const dbStore = useDbConfigStore()
const dashStore = useDashboardStore()
const { fetchDashboardData, fetchDrugMonthly } = useDrugData()

const showSettings = ref(false)

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
    padding: 12px 24px;
    background: var(--basil-400);
    border-bottom: 2px solid var(--basil-500);
    flex-shrink: 0;
    gap: 20px;
    z-index: 20;
    box-shadow: 0 2px 12px rgba(58, 82, 32, 0.25);
    position: relative;
}

.header-brand {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
}

.brand-icon {
    width: 38px;
    height: 38px;
    object-fit: contain;
    filter: drop-shadow(0 1px 6px rgba(0, 0, 0, 0.35));
    flex-shrink: 0;
}

.brand-text {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
}

.brand-title {
    font-family: 'Tahoma', sans-serif;
    font-size: 18px;
    font-weight: 700;
    color: #ffffff;
}

.brand-sub {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.92);
    margin-top: 1px;
}

.header-controls {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    justify-content: flex-end;
}

.connection-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 6px 14px;
    border-radius: 20px;
    cursor: pointer;
    white-space: nowrap;
    border: 1px solid transparent;
    transition: opacity 0.2s, transform 0.1s;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 6px;
}

.connection-badge:hover {
    opacity: 0.85;
    transform: translateY(-1px);
}

.badge-ok {
    background: rgba(255, 255, 255, 0.92);
    color: var(--basil-500);
    border-color: rgba(255, 255, 255, 0.6);
}

.badge-err {
    background: rgba(192, 57, 43, 0.85);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.3);
    animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {

    0%,
    100% {
        opacity: 1;
    }

    50% {
        opacity: 0.65;
    }
}

.settings-btn {
    background: rgba(255, 255, 255, 0.18);
    border: 1px solid rgba(255, 255, 255, 0.35);
    border-radius: 8px;
    padding: 8px;
    color: #ffffff;
    cursor: pointer;
    line-height: 1;
    transition: background 0.2s, border-color 0.2s, transform 0.1s;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
}

.settings-btn:hover {
    background: rgba(255, 255, 255, 0.28);
    border-color: rgba(255, 255, 255, 0.6);
    transform: rotate(30deg);
}

/* ── Main layout ── */
.app-main {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
}

/* ── Sidebar ── */
.sidebar {
    width: 300px;
    flex-shrink: 0;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-card);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

/* ── Content area ── */
.content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 18px;
    gap: 14px;
    min-width: 0;
    background: var(--bg-base);
}

/* ── Chart card ── */
.chart-card {
    flex: 1;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: 16px;
    overflow: hidden;
    transition: box-shadow 0.3s ease;
    min-height: 0;
    box-shadow: 0 2px 8px rgba(75, 54, 33, 0.06);
}

.chart-card:hover {
    box-shadow: 0 4px 24px var(--basil-glow-strong);
}

/* ── Error banner ── */
.error-banner {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(255, 255, 255, 0.96);
    border: 1px solid rgba(192, 57, 43, 0.4);
    color: var(--chili-400);
    padding: 10px 20px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 14px;
    font-size: 13px;
    font-family: 'Tahoma', sans-serif;
    font-weight: 600;
    backdrop-filter: blur(10px);
    z-index: 999;
    max-width: 620px;
    box-shadow: 0 4px 24px rgba(192, 57, 43, 0.15);
    animation: fadeSlideUp 0.3s ease;
}

.error-icon {
    flex-shrink: 0;
    color: var(--chili-300);
}

.error-banner span {
    flex: 1;
    line-height: 1.5;
}

.error-banner button {
    background: none;
    border: none;
    color: var(--chili-400);
    cursor: pointer;
    font-size: 15px;
    padding: 2px 6px;
    border-radius: 4px;
    line-height: 1;
    flex-shrink: 0;
    opacity: 0.7;
    transition: opacity 0.2s;
}

.error-banner button:hover {
    opacity: 1;
}
</style>
