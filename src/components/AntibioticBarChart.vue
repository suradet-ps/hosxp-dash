<template>
<div class="abx-chart">
    <div class="chart-heading">
        <div class="heading-left">
            <Activity :size="17" :stroke-width="2.4" />
            <span>ปริมาณการใช้สูงสุด {{ topCount }} อันดับ</span>
        </div>
        <div v-if="data.length" class="heading-right">
            จากทั้งหมด {{ data.length.toLocaleString() }} รายการ
        </div>
    </div>

    <div v-if="loading" class="chart-state">
        <Loader2 :size="34" :stroke-width="2" class="spin" />
        <p>กำลังโหลดข้อมูล...</p>
    </div>

    <div v-else-if="!data.length" class="chart-state">
        <Syringe :size="46" :stroke-width="1.2" class="state-bloom" />
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
            trigger: 'axis',
            axisPointer: { type: 'shadow' },
            backgroundColor: '#FFFFFF',
            borderColor: 'rgba(78,110,47,0.4)',
            borderWidth: 1,
            textStyle: { color: '#4B3621', fontFamily: 'Tahoma' },
            formatter: (params: any[]) => {
                const p = params[0]
                if (!p) return ''
                const item = items[p.dataIndex]
                if (!item) return ''
                return `
          <div style="font-weight:600;margin-bottom:6px;">${item.drug_name}</div>
          <div>${item.icode}</div>
          <div style="margin-top:4px;">ปริมาณ: <b style="color:#4E6E2F">${formatQty(item.total_qty)}</b> ${item.units || 'หน่วย'}</div>
          <div>จ่ายใน <b style="color:#FF8E7E">${item.visit_count.toLocaleString()}</b> ครั้ง</div>
        `
            },
        },
        xAxis: {
            type: 'value',
            splitLine: {
                lineStyle: { color: 'rgba(78,110,47,0.08)', type: 'dashed' },
            },
            axisLabel: {
                color: '#B09880',
                fontFamily: 'Tahoma',
                fontSize: 11,
                formatter: (v: number) =>
                    v >= 1000 ? (v / 1000).toFixed(1) + 'K' : v.toString(),
            },
        },
        yAxis: {
            type: 'category',
            inverse: true,
            data: names,
            axisLabel: {
                color: '#4B3621',
                fontFamily: 'Tahoma',
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
                barMaxWidth: 22,
                itemStyle: {
                    borderRadius: [0, 8, 8, 0],
                    color: {
                        type: 'linear',
                        x: 0,
                        y: 0,
                        x2: 1,
                        y2: 0,
                        colorStops: [
                            { offset: 0, color: 'rgba(78,110,47,0.35)' },
                            { offset: 1, color: '#4E6E2F' },
                        ],
                    },
                },
                emphasis: {
                    itemStyle: {
                        color: {
                            type: 'linear',
                            x: 0,
                            y: 0,
                            x2: 1,
                            y2: 0,
                            colorStops: [
                                { offset: 0, color: '#7AAD52' },
                                { offset: 1, color: '#3A5220' },
                            ],
                        },
                    },
                },
                label: {
                    show: true,
                    position: 'right',
                    distance: 6,
                    color: '#7A6248',
                    fontFamily: 'Tahoma',
                    fontSize: 11,
                    fontWeight: 'bold',
                    formatter: (p: any) => `${formatQty(p.value)} ${units[p.dataIndex] ?? ''}`,
                },
                animationDelay: (idx: number) => idx * 45,
            },
        ],
        animationEasing: 'cubicOut' as const,
        animationDuration: 700,
    }
})
</script>

<style scoped>
.abx-chart {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 18px 22px 14px;
    gap: 10px;
}

.chart-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
}

.heading-left {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
}

.heading-left svg {
    color: var(--basil-400);
}

.heading-right {
    font-size: 12px;
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
    gap: 10px;
    color: var(--text-muted);
    font-size: 14px;
}

.state-bloom {
    opacity: 0.35;
}

.state-sub {
    font-size: 12.5px;
    color: var(--text-muted);
}
</style>
