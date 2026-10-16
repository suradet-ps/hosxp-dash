<template>
<div class="top-panel">
    <div class="panel-header">
        <div class="panel-title">
            <Trophy :size="15" :stroke-width="2.4" class="header-icon" aria-hidden="true" />
            <span>Top {{ drugs.length || 10 }} ยาจ่ายสูงสุด</span>
        </div>
        <span class="year-badge">พ.ศ. {{ selectedYear + 543 }}</span>
    </div>

    <div v-if="loading" class="skeleton-list" aria-hidden="true">
        <div v-for="i in 9" :key="i" class="skeleton-row" :style="`animation-delay: ${i * 0.05}s`" />
    </div>

    <div v-else-if="!drugs.length" class="empty-state">
        <div class="empty-icon"><DatabaseZap :size="38" :stroke-width="1.5" /></div>
        <p>ยังไม่มีข้อมูล<br />กรุณาเชื่อมต่อฐานข้อมูล</p>
    </div>

    <div v-else class="drug-list">
        <button v-for="(drug, idx) in drugs" :key="drug.icode" type="button" class="drug-row"
            :class="{ active: dashStore.selectedIcode === drug.icode }"
            :aria-current="dashStore.selectedIcode === drug.icode ? 'true' : undefined"
            :title="`${drug.drug_name} (${drug.icode})`" @click="selectDrug(drug.icode)">
            <span class="drug-rank" :class="rankClass(idx)">{{ idx + 1 }}</span>
            <div class="drug-info">
                <div class="drug-name">{{ drug.drug_name }}</div>
                <div class="drug-code">{{ drug.icode }}</div>
                <div class="mini-bar-track">
                    <div class="mini-bar-fill" :style="`width: ${maxQty > 0 ? (drug.total_qty / maxQty) * 100 : 0}%`" />
                </div>
            </div>
            <div class="drug-qty tnum">{{ formatQty(drug.total_qty) }}</div>
        </button>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Trophy, DatabaseZap } from 'lucide-vue-next'
import { useDashboardStore } from '../stores/dashboard'
import type { DrugSummary } from '../stores/dashboard'
import { formatQty } from '../utils/ceDate'

const props = defineProps<{
    drugs: DrugSummary[]
    loading: boolean
}>()

const dashStore = useDashboardStore()

const selectedYear = computed(() => dashStore.selectedYear)

const maxQty = computed(() => {
    if (!props.drugs.length) return 1
    return Math.max(...props.drugs.map((d) => d.total_qty))
})

function selectDrug(icode: string) {
    dashStore.selectDrug(icode)
}

function rankClass(idx: number): string {
    return idx < 3 ? `rank-top rank-${idx + 1}` : ''
}
</script>

<style scoped>
.top-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}

.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: 14px var(--space-4);
    border-bottom: 1px solid var(--border-card);
    flex-shrink: 0;
    background: var(--bg-surface);
}

.panel-title {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.header-icon {
    color: var(--basil-500);
    flex-shrink: 0;
}

.year-badge {
    font-size: var(--text-xs);
    background: var(--bg-elevated);
    color: var(--basil-600);
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-subtle);
    font-weight: 700;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
}

.drug-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-2) var(--space-3) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.drug-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 9px 10px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    background: none;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    transition: background var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease),
        box-shadow var(--dur-fast) var(--ease);
    width: 100%;
}

.drug-row:hover {
    background: var(--basil-50);
    border-color: var(--border-subtle);
}

.drug-row:active {
    background: #e7eeda;
}

.drug-row.active {
    background: var(--basil-50);
    border-color: rgba(78, 110, 47, 0.38);
    box-shadow: inset 3px 0 0 var(--basil-500), var(--shadow-xs);
}

.drug-rank {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--text-muted);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
}

.drug-rank.rank-top {
    color: #ffffff;
    font-size: var(--text-xs);
}

.drug-rank.rank-1 {
    background: linear-gradient(135deg, #d9ad3f, #b8860b);
    box-shadow: 0 2px 6px rgba(184, 134, 11, 0.3);
}

.drug-rank.rank-2 {
    background: linear-gradient(135deg, #a8b0b8, #7f8a94);
    box-shadow: 0 2px 6px rgba(127, 138, 148, 0.3);
}

.drug-rank.rank-3 {
    background: linear-gradient(135deg, #c98d5e, #a0522d);
    box-shadow: 0 2px 6px rgba(160, 82, 45, 0.3);
}

.drug-info {
    flex: 1;
    min-width: 0;
}

.drug-name {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: var(--leading-tight);
}

.drug-code {
    font-size: var(--text-2xs);
    color: var(--text-muted);
    margin-top: 1px;
    margin-bottom: 5px;
    letter-spacing: 0.02em;
}

.mini-bar-track {
    height: 3px;
    background: rgba(78, 110, 47, 0.1);
    border-radius: 2px;
    overflow: hidden;
}

.mini-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--basil-300), var(--basil-500));
    border-radius: 2px;
    transition: width 0.6s var(--ease);
}

.drug-qty {
    font-size: var(--text-sm);
    color: var(--basil-600);
    white-space: nowrap;
    flex-shrink: 0;
    font-weight: 700;
}

/* Skeleton */
.skeleton-list {
    padding: var(--space-2) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
}

.skeleton-row {
    height: 58px;
    border-radius: var(--radius-md);
    background: linear-gradient(90deg,
            var(--bg-surface) 25%,
            var(--basil-50) 50%,
            var(--bg-surface) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
}

.empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-muted);
    text-align: center;
    padding: var(--space-6);
    font-size: var(--text-md);
    line-height: 1.6;
}

.empty-icon {
    color: var(--text-muted);
    opacity: 0.4;
}
</style>
