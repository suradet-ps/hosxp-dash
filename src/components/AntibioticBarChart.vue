<template>
<div class="abx-chart">
    <div class="chart-heading">
        <div class="heading-left">
            <Activity :size="16" :stroke-width="2.4" aria-hidden="true" />
            <span>ปริมาณการใช้สูงสุด {{ topCount }} อันดับ</span>
        </div>
        <div v-if="data.length" class="heading-right">
            จากทั้งหมด {{ data.length.toLocaleString() }} รายการ
        </div>
    </div>

    <div v-if="loading" class="chart-state" role="status">
        <Loader2 :size="32" :stroke-width="2" class="spin" />
        <p>กำลังโหลดข้อมูล...</p>
    </div>

    <div v-else-if="!data.length" class="chart-state">
        <Syringe :size="44" :stroke-width="1.2" class="state-bloom" />
        <p>ไม่พบข้อมูลยาฉีด Antibiotics</p>
        <p class="state-sub">ลองปรับช่วงวันที่แล้วค้นหาอีกครั้ง</p>
    </div>

    <v-chart v-else class="echart" :option="chartOption" :autoresize="true" />
</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Activity, Loader2, Syringe } from 'lucide-vue-next'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart } from 'echarts/charts'
import { TooltipComponent, GridComponent } from 'echarts/components'
import type { AntibioticUsage } from '../composables/useDrugData'
import { formatQty } from '../utils/ceDate'
import { CHART_FONT, chartColors, baseTooltip, valueAxis, barGradient, escapeHtml } from '../utils/chartTheme'

use([CanvasRenderer, BarChart, TooltipComponent, GridComponent])

const TOP_N = 15

const props = defineProps<{
    data: AntibioticUsage[]
    loading: boolean
}>()

const topCount = TOP_N

const topItems = computed(() => props.data.slice(0, TOP_N))

function truncate(name: string, max = 30): string {
    return name.length > max ? name.slice(0, max - 1) + '…' : name
}

const chartOption = computed(() => {
    const items = topItems.value
    const names = items.map(i => truncate(i.drug_name))
    const values = items.map(i => i.total_qty)
    const units = items.map(i => i.units || 'หน่วย')

    return {
        backgroundColor: 'transparent',
        grid: {
            left: 8,
            right: 72,
            top: 8,
            bottom: 8,
            containLabel: true,
        },
        tooltip: {
            ...baseTooltip(),
            trigger: 'axis',
            axisPointer: {
                type: 'shadow',
                shadowStyle: { color: 'rgba(78,110,47,0.06)' },
            },
            formatter: (params: any[]) => {
                const p = params[0]
                if (!p) return ''
                const item = items[p.dataIndex]
                if (!item) return ''
                return `
          <div style="font-weight:600;margin-bottom:5px;">${escapeHtml(item.drug_name)}</div>
          <div style="color:${chartColors.textMuted};">${escapeHtml(item.icode)}</div>
          <div style="margin-top:4px;">ปริมาณ: <b style="color:${chartColors.bar}">${formatQty(item.total_qty)}</b> ${escapeHtml(item.units || 'หน่วย')}</div>
          <div>จ่ายใน <b style="color:${chartColors.barDeep}">${item.visit_count.toLocaleString()}</b> ครั้ง</div>
        `
            },
        },
        xAxis: valueAxis(),
        yAxis: {
            type: 'category',
            inverse: true,
            data: names,
            axisLabel: {
                color: chartColors.text,
                fontFamily: CHART_FONT,
                fontSize: 12,
            },
            axisLine: { show: false },
            axisTick: { show: false },
        },
        series: [
            {
                name: 'ปริมาณ',
                type: 'bar',
                data: values,
                barMaxWidth: 20,
                itemStyle: {
                    borderRadius: [0, 8, 8, 0],
                    color: barGradient(),
                },
                emphasis: {
                    itemStyle: {
                        color: barGradient(chartColors.barBright, chartColors.barDeep),
                    },
                },
                label: {
                    show: true,
                    position: 'right',
                    distance: 6,
                    color: chartColors.textMuted,
                    fontFamily: CHART_FONT,
                    fontSize: 11,
                    fontWeight: 'bold',
                    formatter: (p: any) => `${formatQty(p.value)} ${units[p.dataIndex] ?? ''}`,
                },
                animationDelay: (idx: number) => idx * 40,
            },
        ],
        animationEasing: 'cubicOut' as const,
        animationDuration: 650,
    }
})
</script>

<style scoped>
.abx-chart {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-5) var(--space-6) var(--space-4);
    gap: var(--space-3);
}

.chart-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    flex-shrink: 0;
}

.heading-left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-lg);
    font-weight: 700;
    color: var(--text-primary);
}

.heading-left svg {
    color: var(--basil-400);
}

.heading-right {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: 600;
}

.echart {
    flex: 1;
    min-height: 0;
}

.chart-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-muted);
    font-size: var(--text-md);
}

.state-bloom {
    opacity: 0.35;
}

.state-sub {
    font-size: var(--text-sm);
    color: var(--text-muted);
}
</style>
