<template>
<div class="search-bar" ref="containerRef">
    <div class="search-input-wrap">
        <Search :size="15" :stroke-width="2.5" class="search-icon" aria-hidden="true" />
        <input ref="inputRef" v-model="query" type="text" role="combobox" placeholder="ค้นหายา (รหัส / ชื่อยา)..."
            aria-label="ค้นหารายการยา" aria-autocomplete="list" aria-controls="drug-search-listbox"
            :aria-expanded="open" :aria-activedescendant="open && results.length ? `drug-option-${highlightIdx}` : undefined"
            autocomplete="off" spellcheck="false" @input="onInput" @focus="onFocus"
            @keydown.esc="close" @keydown.down.prevent="highlightNext" @keydown.up.prevent="highlightPrev"
            @keydown.enter.prevent="selectHighlighted" />
        <Loader2 v-if="loading" :size="15" :stroke-width="2.5" class="loading-dot spin" aria-hidden="true" />
        <button v-else-if="query" type="button" class="clear-btn" aria-label="ล้างคำค้นหา" @click="clearSearch">
            <X :size="14" :stroke-width="2.5" />
        </button>
    </div>

    <div v-if="open" id="drug-search-listbox" class="dropdown" role="listbox" aria-label="ผลการค้นหารายการยา">
        <button v-for="(drug, i) in results" :id="`drug-option-${i}`" :key="drug.icode" type="button"
            class="dropdown-item" role="option" :aria-selected="i === highlightIdx"
            :class="{ highlighted: i === highlightIdx }" @click="selectDrug(drug)" @mouseenter="highlightIdx = i">
            <span class="drug-code">{{ drug.icode }}</span>
            <span class="drug-name">{{ drug.name }}</span>
        </button>

        <div v-if="!results.length && !loading" class="dropdown-empty">
            <SearchX :size="18" :stroke-width="1.8" aria-hidden="true" />
            <span>ไม่พบรายการยาที่ตรงกับ “{{ query }}”</span>
        </div>
    </div>
</div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Search, SearchX, Loader2, X } from 'lucide-vue-next'
import { useDrugData } from '../composables/useDrugData'
import { useDashboardStore } from '../stores/dashboard'
import type { DrugItem } from '../stores/dashboard'

const { searchDrugs, loading } = useDrugData()
const dashStore = useDashboardStore()

const query = ref('')
const results = ref<DrugItem[]>([])
const open = ref(false)
const highlightIdx = ref(0)
const containerRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLInputElement | null>(null)

let debounceTimer: ReturnType<typeof setTimeout>

function handleOutsideClick(event: MouseEvent) {
    if (containerRef.value && !containerRef.value.contains(event.target as Node)) {
        open.value = false
    }
}

onMounted(() => {
    document.addEventListener('mousedown', handleOutsideClick)
})

onUnmounted(() => {
    document.removeEventListener('mousedown', handleOutsideClick)
})

function onInput() {
    clearTimeout(debounceTimer)
    open.value = false
    if (!query.value.trim()) {
        results.value = []
        return
    }
    debounceTimer = setTimeout(async () => {
        results.value = await searchDrugs(query.value)
        highlightIdx.value = 0
        open.value = true
    }, 300)
}

function onFocus() {
    if (query.value.trim() && results.value.length) open.value = true
}

function selectDrug(drug: DrugItem) {
    dashStore.selectDrug(drug.icode)
    query.value = `${drug.icode} — ${drug.name}`
    open.value = false
    inputRef.value?.blur()
}

function clearSearch() {
    query.value = ''
    results.value = []
    open.value = false
    inputRef.value?.focus()
}

function close() {
    // APG combobox pattern: Escape closes the popup, focus stays on the input
    open.value = false
}

function highlightNext() {
    if (open.value && highlightIdx.value < results.value.length - 1) highlightIdx.value++
}

function highlightPrev() {
    if (open.value && highlightIdx.value > 0) highlightIdx.value--
}

function selectHighlighted() {
    if (open.value && results.value[highlightIdx.value]) {
        selectDrug(results.value[highlightIdx.value])
    }
}
</script>

<style scoped>
.search-bar {
    position: relative;
    flex: 1;
    max-width: 420px;
    min-width: 200px;
}

.search-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--bg-inset);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 7px 12px;
    transition: border-color var(--dur) var(--ease), box-shadow var(--dur) var(--ease),
        background var(--dur) var(--ease);
}

.search-input-wrap:hover {
    background: #eaeade;
}

.search-input-wrap:focus-within {
    background: #ffffff;
    border-color: var(--basil-400);
    box-shadow: var(--focus-ring);
}

.search-icon {
    flex-shrink: 0;
    color: var(--text-muted);
}

.search-input-wrap input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-family: inherit;
    font-size: var(--text-md);
}

.search-input-wrap input::placeholder {
    color: var(--text-muted);
    opacity: 0.8;
}

.loading-dot {
    flex-shrink: 0;
    color: var(--basil-300);
}

.clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 3px;
    border-radius: var(--radius-xs);
    line-height: 0;
    flex-shrink: 0;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}

.clear-btn:hover {
    color: var(--text-primary);
    background: var(--bg-inset);
}

.dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-md);
    overflow: hidden;
    overflow-y: auto;
    z-index: 200;
    box-shadow: var(--shadow-lg);
    max-height: 280px;
    padding: 4px;
    animation: fadeSlideUp var(--dur-fast) var(--ease);
}

.dropdown-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 9px 12px;
    background: none;
    border: none;
    border-radius: var(--radius-xs);
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    transition: background var(--dur-fast) var(--ease);
}

.dropdown-item:hover,
.dropdown-item.highlighted {
    background: var(--basil-50);
}

.drug-code {
    font-size: var(--text-sm);
    color: var(--basil-400);
    white-space: nowrap;
    min-width: 84px;
    font-weight: 600;
}

.drug-name {
    font-size: var(--text-md);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.dropdown-empty {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 14px;
    color: var(--text-muted);
    font-size: var(--text-md);
}

.dropdown-empty svg {
    flex-shrink: 0;
    opacity: 0.6;
}

.dropdown-empty span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
