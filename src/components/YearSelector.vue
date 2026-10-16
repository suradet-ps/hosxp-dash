<template>
<div class="year-selector">
    <label for="year-select" class="year-label">ปี พ.ศ.</label>
    <div class="year-select-wrap">
        <select id="year-select" v-model="selectedCeYear" class="year-select" @change="onYearChange">
            <option v-for="year in displayYears" :key="year" :value="year">
                {{ year + 543 }} ({{ year }})
            </option>
        </select>
        <ChevronDown :size="14" :stroke-width="2.5" class="year-chevron" aria-hidden="true" />
    </div>
</div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import { useDashboardStore } from '../stores/dashboard'

const dashStore = useDashboardStore()

const selectedCeYear = ref(dashStore.selectedYear)

const displayYears = computed(() => {
    return dashStore.availableYears.length
        ? dashStore.availableYears
        : [new Date().getFullYear()]
})

watch(() => dashStore.selectedYear, (val) => {
    selectedCeYear.value = val
})

watch(() => dashStore.availableYears, (years) => {
    if (years.length && !years.includes(selectedCeYear.value)) {
        selectedCeYear.value = years[0]
    }
})

function onYearChange() {
    dashStore.setYear(selectedCeYear.value)
}
</script>

<style scoped>
.year-selector {
    display: flex;
    align-items: center;
    gap: var(--space-2);
}

.year-label {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
}

.year-select-wrap {
    position: relative;
    display: flex;
    align-items: center;
}

.year-select {
    background: var(--bg-inset);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 7px 30px 7px 12px;
    color: var(--text-primary);
    font-family: inherit;
    font-size: var(--text-md);
    font-weight: 700;
    cursor: pointer;
    outline: none;
    appearance: none;
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease),
        background var(--dur) var(--ease);
    font-variant-numeric: tabular-nums;
}

.year-select:hover {
    background: #eaeade;
}

.year-select:focus-visible {
    background: #ffffff;
    border-color: var(--basil-400);
    box-shadow: var(--focus-ring);
    outline: none;
}

.year-chevron {
    position: absolute;
    right: 9px;
    color: var(--basil-500);
    pointer-events: none;
}
</style>
