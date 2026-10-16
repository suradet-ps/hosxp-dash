<template>
<div class="kpi-row">
    <StatCard label="ยอดจ่ายยา (Top 10)" :value="formatQty(totalQty)" unit="หน่วยนับรวมของ 10 อันดับแรก"
        tone="basil" delay="0s">
        <template #icon><Pill :size="20" :stroke-width="2" /></template>
    </StatCard>

    <StatCard label="รายการยา (Top 10)" :value="activeDrugs.toLocaleString()" unit="รายการที่จ่ายสูงสุด"
        tone="shrimp" delay="0.06s">
        <template #icon><FlaskConical :size="20" :stroke-width="2" /></template>
    </StatCard>

    <StatCard label="เฉลี่ยต่อรายการ" :value="avgPerDrug" unit="หน่วยต่อรายการ (Top 10)"
        tone="neutral" delay="0.12s">
        <template #icon><Sigma :size="20" :stroke-width="2" /></template>
    </StatCard>

    <StatCard label="เดือนที่จ่ายสูงสุด" :value="peakMonthName"
        :unit="`จากยาทั้ง 10 อันดับ · ปี ${selectedYear + 543}`" tone="basil" compact delay="0.18s">
        <template #icon><CalendarRange :size="20" :stroke-width="2" /></template>
    </StatCard>
</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Pill, FlaskConical, Sigma, CalendarRange } from 'lucide-vue-next'
import StatCard from './StatCard.vue'
import { useDashboardStore } from '../stores/dashboard'
import { formatQty, THAI_MONTHS_FULL } from '../utils/ceDate'

const dashStore = useDashboardStore()

const selectedYear = computed(() => dashStore.selectedYear)

const totalQty = computed(() => {
    return dashStore.topDrugs.reduce((sum, d) => sum + d.total_qty, 0)
})

const activeDrugs = computed(() => dashStore.topDrugs.length)

const avgPerDrug = computed(() => {
    if (!activeDrugs.value) return '—'
    return formatQty(totalQty.value / activeDrugs.value)
})

const peakMonthName = computed(() => {
    if (!dashStore.topDrugs.length) return '—'

    // Aggregate monthly qty across all top drugs
    const monthly = Array(12).fill(0)
    dashStore.topDrugs.forEach(d => {
        if (d.peak_month >= 1 && d.peak_month <= 12) {
            // Use peak_month as a vote
            monthly[d.peak_month - 1]++
        }
    })
    const peakIdx = monthly.indexOf(Math.max(...monthly))
    return THAI_MONTHS_FULL[peakIdx] ?? '—'
})
</script>

<style scoped>
.kpi-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-4);
    flex-shrink: 0;
}
</style>
