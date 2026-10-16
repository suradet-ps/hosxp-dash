<template>
<div class="abx-view fade-slide-up">
    <!-- ── Filter bar ── -->
    <div class="filter-card">
        <div class="filter-icon" aria-hidden="true">
            <CalendarRange :size="19" :stroke-width="2.2" />
        </div>

        <div class="filter-fields">
            <label class="date-field">
                <span>จากวันที่</span>
                <input v-model="startDate" type="date" :max="endDate" aria-label="วันที่เริ่มต้น" />
            </label>
            <span class="date-sep" aria-hidden="true">→</span>
            <label class="date-field">
                <span>ถึงวันที่</span>
                <input v-model="endDate" type="date" :min="startDate" aria-label="วันที่สิ้นสุด" />
            </label>
        </div>

        <div class="presets" role="group" aria-label="ช่วงเวลาสำเร็จรูป">
            <button v-for="p in PRESETS" :key="p.key" type="button" class="preset-btn"
                :class="{ active: activePreset === p.key }" :aria-pressed="activePreset === p.key"
                @click="applyPreset(p.key)">
                {{ p.label }}
            </button>
        </div>

        <div class="filter-right">
            <span v-if="rangeCaption" class="range-caption">{{ rangeCaption }}</span>
            <button type="button" class="refresh-btn" :disabled="loading" aria-label="ค้นหาข้อมูลใหม่" title="ค้นหาใหม่"
                @click="load()">
                <RefreshCw :size="15" :stroke-width="2.4" :class="{ spin: loading }" aria-hidden="true" />
            </button>
            <button type="button" class="export-btn" :disabled="loading || exporting || !items.length"
                @click="exportReport">
                <Loader2 v-if="exporting" :size="15" :stroke-width="2.4" class="spin" aria-hidden="true" />
                <FileDown v-else :size="15" :stroke-width="2.4" aria-hidden="true" />
                {{ exporting ? 'กำลังสร้าง PDF...' : 'ส่งออก PDF' }}
            </button>
        </div>
    </div>

    <!-- ── Export result ── -->
    <div v-if="exportedPath" class="export-notice" role="status">
        <FileCheck2 :size="15" :stroke-width="2.4" aria-hidden="true" />
        <span class="export-path">บันทึก PDF แล้ว: {{ exportedPath }}</span>
        <button type="button" class="notice-btn" @click="openExported">
            <FolderOpen :size="13" :stroke-width="2.4" aria-hidden="true" /> เปิดไฟล์
        </button>
        <button type="button" class="notice-close" aria-label="ปิดการแจ้งเตือน" @click="exportedPath = null">
            <X :size="13" :stroke-width="2.5" aria-hidden="true" />
        </button>
    </div>

    <div v-if="exportError" class="abx-error" role="alert">
        <TriangleAlert :size="15" :stroke-width="2.4" aria-hidden="true" />
        <span>สร้างไฟล์ PDF ไม่สำเร็จ: {{ exportError }}</span>
    </div>

    <!-- ── Error ── -->
    <div v-if="error" class="abx-error" role="alert">
        <TriangleAlert :size="15" :stroke-width="2.4" aria-hidden="true" />
        <span>{{ error }}</span>
    </div>

    <!-- ── KPI row ── -->
    <div class="kpi-row">
        <StatCard label="ปริมาณการใช้รวม" :value="formatQty(totalQty)" unit="หน่วยนับรวมทุกรายการ" tone="basil"
            delay="0s">
            <template #icon><Syringe :size="20" :stroke-width="2" /></template>
        </StatCard>

        <StatCard label="รายการยาฉีด" :value="items.length.toLocaleString()" unit="รายการที่มีการใช้"
            tone="shrimp" delay="0.06s">
            <template #icon><Package :size="20" :stroke-width="2" /></template>
        </StatCard>

        <StatCard label="ใช้มากที่สุด" :value="topItem?.drug_name ?? '—'"
            :unit="topItem ? `${formatQty(topItem.total_qty)} ${topItem.units || 'หน่วย'}` : 'ยังไม่มีข้อมูล'"
            :title="topItem?.drug_name ?? ''" tone="neutral" compact delay="0.12s">
            <template #icon><Trophy :size="20" :stroke-width="2" /></template>
        </StatCard>
    </div>

    <!-- ── Chart ── -->
    <div class="chart-card">
        <AntibioticBarChart :data="items" :loading="loading" />
    </div>

    <!-- ── Ranking table ── -->
    <div class="table-card">
        <div class="table-head">
            <div class="table-title">
                <ClipboardList :size="16" :stroke-width="2.4" />
                <span>ตารางจัดอันดับปริมาณการใช้</span>
            </div>
            <div class="table-sub">เรียงจากมากไปน้อย · {{ items.length.toLocaleString() }} รายการ</div>
        </div>

        <div class="table-scroll">
            <table v-if="items.length" aria-label="ตารางจัดอันดับปริมาณการใช้ยาฉีด Antibiotics">
                <thead>
                    <tr>
                        <th scope="col" class="col-rank">อันดับ</th>
                        <th scope="col">รายการยา</th>
                        <th scope="col" class="col-unit">หน่วย</th>
                        <th scope="col" class="col-qty">จำนวนที่ใช้</th>
                        <th scope="col" class="col-visit">ครั้งที่จ่าย</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(item, idx) in items" :key="item.icode">
                        <td class="col-rank">
                            <span class="rank-badge" :class="rankClass(idx)">{{ idx + 1 }}</span>
                        </td>
                        <td>
                            <div class="drug-cell">
                                <span class="drug-name">{{ item.drug_name }}</span>
                                <span class="drug-code">{{ item.icode }}</span>
                            </div>
                        </td>
                        <td class="col-unit">{{ item.units || '—' }}</td>
                        <td class="col-qty">
                            <div class="qty-wrap">
                                <div class="qty-bar">
                                    <div class="qty-fill" :style="{ width: barWidth(item) }" />
                                </div>
                                <span class="qty-value tnum">{{ formatQty(item.total_qty) }}</span>
                            </div>
                        </td>
                        <td class="col-visit tnum">{{ item.visit_count.toLocaleString() }}</td>
                    </tr>
                </tbody>
            </table>

            <div v-if="loading && !items.length" class="table-state" role="status">
                <Loader2 :size="28" :stroke-width="2" class="spin" />
                <p>กำลังโหลดข้อมูล...</p>
            </div>

            <div v-else-if="!items.length" class="table-state">
                <Syringe :size="38" :stroke-width="1.2" class="state-bloom" />
                <p>ไม่พบข้อมูลในช่วงวันที่เลือก</p>
            </div>
        </div>
    </div>

    <!-- ── Hidden A4 report captured for PDF export ── -->
    <div ref="reportRef" class="pdf-report-host" aria-hidden="true">
        <div class="pdf-report">
            <div class="pr-band">
                <img :src="logoUrl" class="pr-logo" alt="">
                <div class="pr-band-text">
                    <div class="pr-hospital">โรงพยาบาลสระโบสถ์</div>
                    <div class="pr-app">HOSxP Dash · ระบบติดตามการใช้ยา</div>
                </div>
                <div class="pr-printed">{{ printedAtLabel }}</div>
            </div>

            <div class="pr-title-wrap">
                <h1 class="pr-title">รายงานปริมาณการใช้ยาฉีด Antibiotics</h1>
                <div class="pr-subtitle">15 อันดับที่มีปริมาณการใช้สูงสุด · เรียงจากมากไปน้อย</div>
                <div class="pr-range">
                    ช่วงข้อมูล {{ rangeCaption || '—' }}
                    <template v-if="items.length > REPORT_TOP_N">
                        · แสดง {{ REPORT_TOP_N }} จากทั้งหมด {{ items.length.toLocaleString() }} รายการ
                    </template>
                </div>
            </div>

            <div class="pr-kpis">
                <div class="pr-kpi">
                    <div class="pr-kpi-label">ปริมาณการใช้รวม</div>
                    <div class="pr-kpi-value">{{ formatQty(totalQty) }}</div>
                    <div class="pr-kpi-unit">หน่วยนับรวมทุกรายการ</div>
                </div>
                <div class="pr-kpi">
                    <div class="pr-kpi-label">รายการยาฉีดทั้งหมด</div>
                    <div class="pr-kpi-value">{{ items.length.toLocaleString() }}</div>
                    <div class="pr-kpi-unit">รายการที่มีการใช้ในช่วงนี้</div>
                </div>
                <div class="pr-kpi">
                    <div class="pr-kpi-label">ใช้มากที่สุด</div>
                    <div class="pr-kpi-value pr-kpi-value--name">{{ topItem?.drug_name ?? '—' }}</div>
                    <div class="pr-kpi-unit">
                        {{ topItem ? `${formatQty(topItem.total_qty)} ${topItem.units || 'หน่วย'}` : '—' }}
                    </div>
                </div>
            </div>

            <table class="pr-table">
                <thead>
                    <tr>
                        <th class="pr-col-rank">อันดับ</th>
                        <th>รายการยา</th>
                        <th class="pr-col-unit">หน่วย</th>
                        <th class="pr-col-qty">จำนวนที่ใช้</th>
                        <th class="pr-col-visit">ครั้งที่จ่าย</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(item, idx) in reportItems" :key="item.icode" :class="{ 'pr-row-alt': idx % 2 === 1 }">
                        <td class="pr-col-rank">
                            <span class="pr-rank" :class="{ 'pr-rank-top': idx < 3 }">{{ idx + 1 }}</span>
                        </td>
                        <td>
                            <div class="pr-name">{{ item.drug_name }}</div>
                            <div class="pr-code">{{ item.icode }}</div>
                        </td>
                        <td class="pr-col-unit">{{ item.units || '—' }}</td>
                        <td class="pr-col-qty">{{ formatQty(item.total_qty) }}</td>
                        <td class="pr-col-visit">{{ item.visit_count.toLocaleString() }}</td>
                    </tr>
                </tbody>
            </table>

            <div class="pr-foot">
                เอกสารสร้างจากฐานข้อมูล HOSxP · จัดทำโดย HOSxP Dash · ใช้สำหรับงานเภสัชกรรมภายในโรงพยาบาล
            </div>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import { open } from '@tauri-apps/plugin-shell'
import {
    CalendarRange,
    ClipboardList,
    FileCheck2,
    FileDown,
    FolderOpen,
    Loader2,
    Package,
    RefreshCw,
    Syringe,
    TriangleAlert,
    Trophy,
    X,
} from 'lucide-vue-next'
import AntibioticBarChart from './AntibioticBarChart.vue'
import StatCard from './StatCard.vue'
import { useDbConfigStore } from '../stores/dbConfig'
import { useDrugData } from '../composables/useDrugData'
import type { AntibioticUsage } from '../composables/useDrugData'
import { formatQty, THAI_MONTHS_SHORT } from '../utils/ceDate'
import { exportElementToPdf } from '../utils/reportPdf'
import logoUrl from '../assets/logo.svg'

const REPORT_TOP_N = 15

const dbStore = useDbConfigStore()
const { loading, error, fetchAntibioticUsage } = useDrugData()

const items = ref<AntibioticUsage[]>([])

// ── Date range ──
const today = dayjs()
const startDate = ref(today.startOf('year').format('YYYY-MM-DD'))
const endDate = ref(today.format('YYYY-MM-DD'))

const PRESETS = [
    { key: '7d', label: '7 วัน' },
    { key: '30d', label: '30 วัน' },
    { key: '90d', label: '90 วัน' },
    { key: 'ytd', label: 'ปีนี้' },
    { key: 'lastYear', label: 'ปีที่แล้ว' },
] as const

type PresetKey = (typeof PRESETS)[number]['key']

function presetRange(key: PresetKey): [string, string] {
    const now = dayjs()
    switch (key) {
        case '7d':
            return [now.subtract(6, 'day').format('YYYY-MM-DD'), now.format('YYYY-MM-DD')]
        case '30d':
            return [now.subtract(29, 'day').format('YYYY-MM-DD'), now.format('YYYY-MM-DD')]
        case '90d':
            return [now.subtract(89, 'day').format('YYYY-MM-DD'), now.format('YYYY-MM-DD')]
        case 'ytd':
            return [now.startOf('year').format('YYYY-MM-DD'), now.format('YYYY-MM-DD')]
        case 'lastYear': {
            const prev = now.subtract(1, 'year')
            return [
                prev.startOf('year').format('YYYY-MM-DD'),
                prev.endOf('year').format('YYYY-MM-DD'),
            ]
        }
    }
}

const activePreset = computed<PresetKey | null>(() => {
    for (const p of PRESETS) {
        const [s, e] = presetRange(p.key)
        if (s === startDate.value && e === endDate.value) return p.key
    }
    return null
})

function applyPreset(key: PresetKey) {
    const [s, e] = presetRange(key)
    startDate.value = s
    endDate.value = e
}

const rangeCaption = computed(() => {
    if (!startDate.value || !endDate.value || startDate.value > endDate.value) return ''
    return `${thaiDateLabel(startDate.value)} – ${thaiDateLabel(endDate.value)}`
})

function thaiDateLabel(iso: string): string {
    const d = dayjs(iso)
    return `${d.date()} ${THAI_MONTHS_SHORT[d.month()]} ${d.year() + 543}`
}

// ── Data ──
const totalQty = computed(() => items.value.reduce((sum, i) => sum + i.total_qty, 0))
const topItem = computed(() => items.value[0] ?? null)
const maxQty = computed(() => items.value[0]?.total_qty ?? 0)

function barWidth(item: AntibioticUsage): string {
    if (maxQty.value <= 0) return '0%'
    return `${Math.max((item.total_qty / maxQty.value) * 100, 2).toFixed(1)}%`
}

function rankClass(idx: number): string {
    if (idx === 0) return 'rank-1'
    if (idx === 1) return 'rank-2'
    if (idx === 2) return 'rank-3'
    return ''
}

// ── PDF export ──
const reportRef = ref<HTMLElement | null>(null)
const exporting = ref(false)
const exportedPath = ref<string | null>(null)
const exportError = ref<string | null>(null)

const reportItems = computed(() => items.value.slice(0, REPORT_TOP_N))

const printedAtLabel = computed(() => {
    const d = dayjs()
    return `พิมพ์เมื่อ ${d.date()} ${THAI_MONTHS_SHORT[d.month()]} ${d.year() + 543} เวลา ${d.format('HH:mm')} น.`
})

async function exportReport() {
    if (!reportRef.value || !items.value.length || exporting.value) return
    exporting.value = true
    exportedPath.value = null
    exportError.value = null
    try {
        const d = dayjs()
        const fileName = `antibiotics-usage-${d.year() + 543}-${d.format('MM-DD')}.pdf`
        exportedPath.value = await exportElementToPdf(reportRef.value, fileName)
    } catch (e) {
        exportError.value = String(e)
    } finally {
        exporting.value = false
    }
}

async function openExported() {
    if (exportedPath.value) await open(exportedPath.value)
}

async function load() {
    if (!dbStore.connected) return
    if (!startDate.value || !endDate.value) return
    if (startDate.value > endDate.value) return
    exportedPath.value = null
    exportError.value = null
    items.value = await fetchAntibioticUsage(startDate.value, endDate.value)
}

watch([startDate, endDate], () => {
    load()
})

watch(
    () => dbStore.connected,
    (connected) => {
        if (connected) load()
    }
)

onMounted(() => load())
</script>

<style scoped>
.abx-view {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    overflow-y: auto;
    background: var(--bg-base);
}

/* ── Filter bar ── */
.filter-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex-wrap: wrap;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-lg);
    padding: var(--space-3) var(--space-5);
    box-shadow: var(--shadow-sm);
    flex-shrink: 0;
}

.filter-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: var(--radius-sm);
    background: linear-gradient(135deg, rgba(78, 110, 47, 0.12), rgba(78, 110, 47, 0.04));
    color: var(--basil-400);
    flex-shrink: 0;
}

.filter-fields {
    display: flex;
    align-items: flex-end;
    gap: var(--space-2);
}

.date-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.date-field span {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 0.04em;
}

.date-field input {
    font-family: inherit;
    font-size: var(--text-md);
    color: var(--text-primary);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    outline: none;
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease);
    font-variant-numeric: tabular-nums;
}

.date-field input:hover {
    border-color: var(--border-active);
}

.date-field input:focus {
    border-color: var(--basil-400);
    box-shadow: var(--focus-ring);
}

.date-sep {
    color: var(--text-muted);
    font-size: var(--text-md);
    padding-bottom: 7px;
}

.presets {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
}

.preset-btn {
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-pill);
    padding: 5px 13px;
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease);
    white-space: nowrap;
}

.preset-btn:hover {
    border-color: var(--border-active);
    color: var(--basil-500);
}

.preset-btn:active {
    transform: translateY(1px);
}

.preset-btn.active {
    background: var(--basil-400);
    border-color: var(--basil-400);
    color: #ffffff;
    box-shadow: 0 2px 8px var(--basil-glow-strong);
}

.filter-right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-3);
}

.range-caption {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: 600;
    white-space: nowrap;
}

.refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--basil-400);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease);
}

.refresh-btn:hover:not(:disabled) {
    border-color: var(--border-active);
    background: var(--basil-50);
}

.refresh-btn:active:not(:disabled) {
    transform: translateY(1px);
}

.refresh-btn:disabled {
    opacity: 0.55;
    cursor: default;
}

/* ── Error ── */
.abx-error {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: rgba(255, 255, 255, 0.97);
    border: 1px solid rgba(169, 50, 38, 0.35);
    border-left: 4px solid var(--chili-400);
    color: var(--chili-500);
    border-radius: var(--radius-md);
    padding: 10px 16px;
    font-size: var(--text-md);
    font-weight: 600;
    flex-shrink: 0;
    box-shadow: var(--shadow-xs);
}

/* ── KPI row ── */
.kpi-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-4);
    flex-shrink: 0;
}

/* ── Chart card ── */
.chart-card {
    height: 400px;
    flex-shrink: 0;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
    transition: box-shadow var(--dur-slow) var(--ease);
}

.chart-card:hover {
    box-shadow: var(--shadow-md);
}

/* ── Table card ── */
.table-card {
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
}

.table-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-card);
    flex-shrink: 0;
    background: var(--bg-surface);
}

.table-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
}

.table-title svg {
    color: var(--basil-400);
}

.table-sub {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: 600;
}

.table-scroll {
    max-height: 420px;
    overflow-y: auto;
}

table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-md);
}

thead th {
    position: sticky;
    top: 0;
    z-index: 2;
    background: var(--bg-surface);
    text-align: left;
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border-card);
    white-space: nowrap;
}

tbody td {
    padding: 9px 16px;
    border-bottom: 1px solid rgba(75, 54, 33, 0.06);
    vertical-align: middle;
}

tbody tr {
    transition: background var(--dur-fast) var(--ease);
}

tbody tr:hover {
    background: var(--basil-50);
}

tbody tr:last-child td {
    border-bottom: none;
}

.col-rank {
    width: 64px;
    text-align: center;
}

.rank-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-xs);
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--text-muted);
    background: rgba(75, 54, 33, 0.06);
}

.rank-badge.rank-1 {
    background: var(--basil-400);
    color: #ffffff;
    box-shadow: 0 2px 8px var(--basil-glow-strong);
}

.rank-badge.rank-2 {
    background: var(--basil-200);
    color: var(--basil-500);
}

.rank-badge.rank-3 {
    background: var(--basil-100);
    color: var(--basil-500);
}

.drug-cell {
    display: flex;
    flex-direction: column;
    gap: 1px;
}

.drug-name {
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.35;
}

.drug-code {
    font-size: var(--text-xs);
    color: var(--text-muted);
}

.col-unit {
    width: 90px;
    color: var(--text-secondary);
    white-space: nowrap;
}

.col-qty {
    width: 240px;
}

/* The plain `thead th` rule would otherwise left-align this header while
   the numbers below are right-aligned. */
thead th.col-qty {
    text-align: right;
}

.qty-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-3);
}

.qty-bar {
    flex: 1;
    height: 7px;
    border-radius: 4px;
    background: rgba(78, 110, 47, 0.1);
    overflow: hidden;
}

.qty-fill {
    height: 100%;
    border-radius: 4px;
    background: linear-gradient(90deg, rgba(78, 110, 47, 0.45), var(--basil-400));
    transition: width 0.5s var(--ease);
}

.qty-value {
    font-weight: 700;
    color: var(--basil-400);
    min-width: 70px;
    text-align: right;
}

.col-visit {
    width: 110px;
    color: var(--text-secondary);
    text-align: right;
}

.table-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    padding: 44px 0;
    color: var(--text-muted);
    font-size: var(--text-md);
}

.state-bloom {
    opacity: 0.35;
}

/* ── Export button ── */
.export-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 700;
    color: #ffffff;
    background: linear-gradient(135deg, var(--basil-400), var(--basil-500));
    border: none;
    border-radius: var(--radius-sm);
    padding: 8px 15px;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity var(--dur) var(--ease), transform var(--dur-fast) var(--ease),
        box-shadow var(--dur) var(--ease);
    box-shadow: 0 2px 8px var(--basil-glow-strong);
}

.export-btn:hover:not(:disabled) {
    box-shadow: 0 4px 14px var(--basil-glow-strong);
}

.export-btn:active:not(:disabled) {
    transform: translateY(1px);
    box-shadow: var(--shadow-xs);
}

.export-btn:disabled {
    opacity: 0.5;
    cursor: default;
    box-shadow: none;
}

/* ── Export notice ── */
.export-notice {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--basil-50);
    border: 1px solid var(--border-subtle);
    color: var(--basil-500);
    border-radius: var(--radius-md);
    padding: 9px 16px;
    font-size: var(--text-sm);
    font-weight: 600;
    flex-shrink: 0;
    box-shadow: var(--shadow-xs);
}

.export-notice > svg {
    color: var(--basil-400);
    flex-shrink: 0;
}

.export-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
}

.notice-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: inherit;
    font-size: var(--text-sm);
    font-weight: 700;
    color: #ffffff;
    background: var(--basil-400);
    border: none;
    border-radius: var(--radius-xs);
    padding: 5px 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: background var(--dur-fast) var(--ease);
}

.notice-btn:hover {
    background: var(--basil-500);
}

.notice-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--basil-500);
    opacity: 0.6;
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-xs);
    line-height: 0;
    transition: opacity var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.notice-close:hover {
    opacity: 1;
    background: rgba(78, 110, 47, 0.1);
}

/* ── Hidden PDF report (A4 at 96dpi = 794px wide) ── */
.pdf-report-host {
    position: fixed;
    left: -10000px;
    top: 0;
    z-index: -1;
    pointer-events: none;
}

.pdf-report {
    width: 794px;
    background: #ffffff;
    color: #4B3621;
    font-family: var(--font-sans);
}

.pr-band {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 18px 36px;
    background: linear-gradient(120deg, #3A5220, #4E6E2F);
}

.pr-logo {
    width: 44px;
    height: 44px;
    object-fit: contain;
}

.pr-band-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
}

.pr-hospital {
    font-size: 17px;
    font-weight: 700;
    color: #ffffff;
}

.pr-app {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.85);
}

.pr-printed {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.9);
    text-align: right;
    line-height: 1.5;
    white-space: nowrap;
}

.pr-title-wrap {
    padding: 22px 36px 0;
}

.pr-title {
    font-size: 20px;
    font-weight: 700;
    color: #3A5220;
}

.pr-subtitle {
    font-size: 12.5px;
    color: #7A6248;
    margin-top: 3px;
    font-weight: 600;
}

.pr-range {
    font-size: 12px;
    color: #806550;
    margin-top: 6px;
}

.pr-kpis {
    display: flex;
    gap: 12px;
    padding: 14px 36px 0;
}

.pr-kpi {
    flex: 1;
    background: #F7F9F1;
    border: 1px solid rgba(78, 110, 47, 0.22);
    border-left: 4px solid #4E6E2F;
    border-radius: 8px;
    padding: 10px 14px;
}

.pr-kpi-label {
    font-size: 10.5px;
    font-weight: 700;
    color: #806550;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.pr-kpi-value {
    font-size: 20px;
    font-weight: 700;
    color: #4E6E2F;
    margin-top: 2px;
}

.pr-kpi-value--name {
    font-size: 13px;
}

.pr-kpi-unit {
    font-size: 11px;
    color: #7A6248;
    margin-top: 1px;
}

.pr-table {
    width: calc(100% - 72px);
    margin: 14px 36px 0;
    border-collapse: collapse;
}

.pr-table thead th {
    background: #3A5220;
    color: #ffffff;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-align: left;
    padding: 8px 12px;
}

.pr-table thead th:first-child {
    border-radius: 6px 0 0 0;
}

.pr-table thead th:last-child {
    border-radius: 0 6px 0 0;
}

/* `thead th { text-align: left }` above has higher specificity than the
   `.pr-col-*` rules, so the header alignment for the rank and numeric
   columns must be set with matching specificity. */
.pr-table thead th.pr-col-rank {
    text-align: center;
}

.pr-table thead th.pr-col-qty,
.pr-table thead th.pr-col-visit {
    text-align: right;
}

.pr-table tbody td {
    padding: 7px 12px;
    border-bottom: 1px solid rgba(75, 54, 33, 0.1);
    vertical-align: middle;
}

.pr-row-alt td {
    background: #FAFBF5;
}

.pr-col-rank {
    width: 56px;
    text-align: center;
}

.pr-rank {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: rgba(75, 54, 33, 0.07);
    color: #806550;
    font-size: 11px;
    font-weight: 700;
}

.pr-rank-top {
    background: #4E6E2F;
    color: #ffffff;
}

.pr-name {
    font-size: 12.5px;
    font-weight: 600;
    color: #4B3621;
    line-height: 1.35;
}

.pr-code {
    font-size: 10.5px;
    color: #806550;
}

.pr-col-unit {
    width: 130px;
    font-size: 12px;
    color: #7A6248;
}

.pr-col-qty {
    width: 110px;
    text-align: right;
    font-size: 12.5px;
    font-weight: 700;
    color: #4E6E2F;
    font-variant-numeric: tabular-nums;
}

.pr-col-visit {
    width: 90px;
    text-align: right;
    font-size: 12px;
    color: #7A6248;
}

.pr-foot {
    margin-top: 14px;
    padding: 10px 36px 16px;
    font-size: 10.5px;
    color: #806550;
    border-top: 1px solid rgba(75, 54, 33, 0.12);
}
</style>
