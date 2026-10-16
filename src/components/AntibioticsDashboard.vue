<template>
<div class="abx-view fade-slide-up">
    <!-- ── Filter bar ── -->
    <div class="filter-card">
        <div class="filter-icon">
            <CalendarRange :size="20" :stroke-width="2.2" />
        </div>

        <div class="filter-fields">
            <label class="date-field">
                <span>จากวันที่</span>
                <input v-model="startDate" type="date" :max="endDate" />
            </label>
            <span class="date-sep">→</span>
            <label class="date-field">
                <span>ถึงวันที่</span>
                <input v-model="endDate" type="date" :min="startDate" />
            </label>
        </div>

        <div class="presets">
            <button v-for="p in PRESETS" :key="p.key" class="preset-btn"
                :class="{ active: activePreset === p.key }" @click="applyPreset(p.key)">
                {{ p.label }}
            </button>
        </div>

        <div class="filter-right">
            <span v-if="rangeCaption" class="range-caption">{{ rangeCaption }}</span>
            <button class="refresh-btn" :disabled="loading" title="ค้นหาใหม่" @click="load()">
                <RefreshCw :size="15" :stroke-width="2.4" :class="{ spin: loading }" />
            </button>
        </div>
    </div>

    <!-- ── Error ── -->
    <div v-if="error" class="abx-error">
        <TriangleAlert :size="15" :stroke-width="2.4" />
        <span>{{ error }}</span>
    </div>

    <!-- ── KPI row ── -->
    <div class="kpi-row">
        <div class="kpi-card" style="animation-delay: 0s">
            <div class="kpi-icon kpi-icon--syringe">
                <Syringe :size="22" :stroke-width="2" />
            </div>
            <div class="kpi-body">
                <div class="kpi-label">ปริมาณการใช้รวม</div>
                <div class="kpi-value">{{ formatQty(totalQty) }}</div>
                <div class="kpi-unit">หน่วยนับรวมทุกรายการ</div>
            </div>
        </div>

        <div class="kpi-card" style="animation-delay: 0.08s">
            <div class="kpi-icon kpi-icon--box">
                <Package :size="22" :stroke-width="2" />
            </div>
            <div class="kpi-body">
                <div class="kpi-label">รายการยาฉีด</div>
                <div class="kpi-value">{{ items.length.toLocaleString() }}</div>
                <div class="kpi-unit">รายการที่มีการใช้</div>
            </div>
        </div>

        <div class="kpi-card" style="animation-delay: 0.16s">
            <div class="kpi-icon kpi-icon--trophy">
                <Trophy :size="22" :stroke-width="2" />
            </div>
            <div class="kpi-body">
                <div class="kpi-label">ใช้มากที่สุด</div>
                <div class="kpi-value kpi-value--name" :title="topItem?.drug_name ?? ''">
                    {{ topItem?.drug_name ?? '—' }}
                </div>
                <div class="kpi-unit">
                    {{ topItem ? `${formatQty(topItem.total_qty)} ${topItem.units || 'หน่วย'}` : 'ยังไม่มีข้อมูล' }}
                </div>
            </div>
        </div>
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
            <table v-if="items.length">
                <thead>
                    <tr>
                        <th class="col-rank">อันดับ</th>
                        <th>รายการยา</th>
                        <th class="col-unit">หน่วย</th>
                        <th class="col-qty">จำนวนที่ใช้</th>
                        <th class="col-visit">ครั้งที่จ่าย</th>
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
                                <span class="qty-value">{{ formatQty(item.total_qty) }}</span>
                            </div>
                        </td>
                        <td class="col-visit">{{ item.visit_count.toLocaleString() }}</td>
                    </tr>
                </tbody>
            </table>

            <div v-if="loading && !items.length" class="table-state">
                <Loader2 :size="30" :stroke-width="2" class="spin" />
                <p>กำลังโหลดข้อมูล...</p>
            </div>

            <div v-else-if="!items.length" class="table-state">
                <Syringe :size="40" :stroke-width="1.2" class="state-bloom" />
                <p>ไม่พบข้อมูลในช่วงวันที่เลือก</p>
            </div>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import {
    CalendarRange,
    ClipboardList,
    Loader2,
    Package,
    RefreshCw,
    Syringe,
    TriangleAlert,
    Trophy,
} from 'lucide-vue-next'
import AntibioticBarChart from './AntibioticBarChart.vue'
import { useDbConfigStore } from '../stores/dbConfig'
import { useDrugData } from '../composables/useDrugData'
import type { AntibioticUsage } from '../composables/useDrugData'
import { formatQty, THAI_MONTHS_SHORT } from '../utils/ceDate'

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

async function load() {
    if (!dbStore.connected) return
    if (!startDate.value || !endDate.value) return
    if (startDate.value > endDate.value) return
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
    gap: 14px;
    padding: 18px;
    overflow-y: auto;
    background: var(--bg-base);
}

/* ── Filter bar ── */
.filter-card {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: 14px;
    padding: 12px 18px;
    box-shadow: 0 2px 8px rgba(75, 54, 33, 0.06);
    flex-shrink: 0;
}

.filter-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: 11px;
    background: linear-gradient(135deg, rgba(78, 110, 47, 0.12), rgba(78, 110, 47, 0.04));
    color: var(--basil-400);
    flex-shrink: 0;
}

.filter-fields {
    display: flex;
    align-items: flex-end;
    gap: 8px;
}

.date-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.date-field span {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 0.04em;
}

.date-field input {
    font-family: 'Tahoma', sans-serif;
    font-size: 13px;
    color: var(--text-primary);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 6px 10px;
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.date-field input:focus {
    border-color: var(--border-active);
    box-shadow: 0 0 0 3px var(--basil-glow);
}

.date-sep {
    color: var(--text-muted);
    font-size: 14px;
    padding-bottom: 8px;
}

.presets {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
}

.preset-btn {
    font-family: 'Tahoma', sans-serif;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: 20px;
    padding: 5px 13px;
    cursor: pointer;
    transition: all 0.18s ease;
    white-space: nowrap;
}

.preset-btn:hover {
    border-color: var(--border-active);
    color: var(--basil-400);
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
    gap: 12px;
}

.range-caption {
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 600;
    white-space: nowrap;
}

.refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 9px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--basil-400);
    cursor: pointer;
    transition: all 0.18s ease;
}

.refresh-btn:hover:not(:disabled) {
    border-color: var(--border-active);
    background: var(--basil-50);
}

.refresh-btn:disabled {
    opacity: 0.55;
    cursor: default;
}

/* ── Error ── */
.abx-error {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.96);
    border: 1px solid rgba(192, 57, 43, 0.35);
    color: var(--chili-400);
    border-radius: 12px;
    padding: 10px 16px;
    font-size: 13px;
    font-weight: 600;
    flex-shrink: 0;
}

/* ── KPI row ── */
.kpi-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
    flex-shrink: 0;
}

.kpi-card {
    display: flex;
    align-items: center;
    gap: 15px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: 14px;
    padding: 16px 20px;
    box-shadow: 0 2px 8px rgba(75, 54, 33, 0.06);
    animation: fadeSlideUp 0.4s ease forwards;
    opacity: 0;
    transition: box-shadow 0.25s ease;
    min-width: 0;
}

.kpi-card:hover {
    box-shadow: 0 4px 20px var(--basil-glow);
}

.kpi-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 42px;
    height: 42px;
    border-radius: 12px;
    flex-shrink: 0;
    transition: transform 0.2s ease;
}

.kpi-card:hover .kpi-icon {
    transform: scale(1.08);
}

.kpi-icon--syringe {
    background: linear-gradient(135deg, rgba(78, 110, 47, 0.14), rgba(78, 110, 47, 0.04));
    color: var(--basil-400);
}

.kpi-icon--box {
    background: linear-gradient(135deg, rgba(255, 142, 126, 0.16), rgba(255, 142, 126, 0.05));
    color: var(--shrimp-500);
}

.kpi-icon--trophy {
    background: linear-gradient(135deg, rgba(122, 98, 72, 0.14), rgba(122, 98, 72, 0.04));
    color: var(--text-secondary);
}

.kpi-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.kpi-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
}

.kpi-value {
    font-family: 'Tahoma', sans-serif;
    font-size: 26px;
    font-weight: 700;
    color: var(--basil-400);
    line-height: 1.15;
}

.kpi-value--name {
    font-size: 17px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
}

.kpi-unit {
    font-size: 12px;
    color: var(--text-secondary);
}

/* ── Chart card ── */
.chart-card {
    height: 400px;
    flex-shrink: 0;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: 16px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(75, 54, 33, 0.06);
    transition: box-shadow 0.3s ease;
}

.chart-card:hover {
    box-shadow: 0 4px 24px var(--basil-glow-strong);
}

/* ── Table card ── */
.table-card {
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: 16px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(75, 54, 33, 0.06);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
}

.table-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border-card);
    flex-shrink: 0;
}

.table-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
}

.table-title svg {
    color: var(--basil-400);
}

.table-sub {
    font-size: 12px;
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
    font-size: 13px;
}

thead th {
    position: sticky;
    top: 0;
    z-index: 2;
    background: var(--bg-surface);
    text-align: left;
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border-card);
    white-space: nowrap;
}

tbody td {
    padding: 9px 16px;
    border-bottom: 1px solid rgba(75, 54, 33, 0.05);
    vertical-align: middle;
}

tbody tr {
    transition: background 0.15s ease;
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
    border-radius: 8px;
    font-size: 12px;
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
    font-size: 11px;
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

.qty-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
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
    background: linear-gradient(90deg, rgba(78, 110, 47, 0.45), #4E6E2F);
    transition: width 0.5s ease;
}

.qty-value {
    font-weight: 700;
    color: var(--basil-400);
    min-width: 70px;
    text-align: right;
    font-variant-numeric: tabular-nums;
}

.col-visit {
    width: 110px;
    color: var(--text-secondary);
    text-align: right;
    font-variant-numeric: tabular-nums;
}

.table-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 44px 0;
    color: var(--text-muted);
    font-size: 13.5px;
}

.state-bloom {
    opacity: 0.35;
}
</style>
