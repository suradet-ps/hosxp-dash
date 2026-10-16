<template>
<div class="chart-container">
    <div class="chart-header">
        <div v-if="data" class="chart-title">
            <div class="drug-name-title">{{ data.drug_name }}</div>
            <div class="drug-code-title">รหัสยา {{ data.icode }}</div>
        </div>
        <div v-else class="chart-title-empty">เลือกยาจากรายการเพื่อดูกราฟแนวโน้ม</div>

        <div v-if="data" class="chart-meta">
            <span v-if="peak" class="meta-chip meta-chip--peak">
                <TrendingUp :size="13" :stroke-width="2.4" aria-hidden="true" />
                เดือนสูงสุด: {{ THAI_MONTHS_FULL[peak.idx] }} · {{ formatQty(peak.value) }}
            </span>
            <div class="chart-total">
                <span class="total-label">รวมทั้งปี</span>
                <span class="total-value tnum">{{ formatQty(data.total_qty) }}</span>
                <span class="total-unit">หน่วย</span>
            </div>
        </div>
    </div>

    <div v-if="!data && !loading" class="empty-chart">
        <div class="empty-bloom"><BarChart3 :size="52" :stroke-width="1.2" /></div>
        <p class="empty-title">ยังไม่มีข้อมูลกราฟ</p>
        <p class="empty-sub">เลือกจาก 10 อันดับด้านซ้าย หรือค้นหาชื่อยาเพื่อแสดงแนวโน้มการจ่าย</p>
    </div>

    <div v-else-if="loading" class="loading-chart" role="status">
        <div class="loading-bloom"><Loader2 :size="34" :stroke-width="2" class="spin" /></div>
        <p>กำลังโหลดข้อมูล...</p>
    </div>

    <v-chart v-else ref="chartRef" class="echart" :option="chartOption" :autoresize="true" />
</div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { BarChart3, Loader2, TrendingUp } from 'lucide-vue-next'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart, LineChart } from 'echarts/charts'
import {
    TitleComponent,
    TooltipComponent,
    GridComponent,
    LegendComponent,
} from 'echarts/components'
import type { DrugMonthlyData } from '../stores/dashboard'
import { formatQty, THAI_MONTHS_SHORT, THAI_MONTHS_FULL } from '../utils/ceDate'
import { CHART_FONT, chartColors, baseTooltip, valueAxis, escapeHtml } from '../utils/chartTheme'

use([
    CanvasRenderer,
    BarChart,
    LineChart,
    TitleComponent,
    TooltipComponent,
    GridComponent,
    LegendComponent,
])

const props = defineProps<{
    data: DrugMonthlyData | null
    loading: boolean
}>()

const chartRef = ref()

/** Peak month (client-side, mirrors the SQL peak for the annotation) */
const peak = computed<{ idx: number; value: number } | null>(() => {
    const monthly = props.data?.monthly_qty ?? []
    if (!monthly.length) return null
    let idx = 0
    monthly.forEach((v, i) => {
        if (v > monthly[idx]) idx = i
    })
    return monthly[idx] > 0 ? { idx, value: monthly[idx] } : null
})

const chartOption = computed(() => {
    const monthly = props.data?.monthly_qty ?? Array(12).fill(0)
    const total = props.data?.total_qty ?? 0
    const peakIdx = peak.value?.idx ?? -1

    return {
        backgroundColor: 'transparent',
        grid: {
            left: '2%',
            right: '2%',
            top: 44,
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
                const bar = params.find((p: any) => p.seriesName === 'จำนวนจ่าย')
                if (!bar) return ''
                const pct = total > 0 ? ((bar.value / total) * 100).toFixed(1) : '0'
                const isPeak = bar.dataIndex === peakIdx
                return `
          <div style="font-weight:600;margin-bottom:5px;">${escapeHtml(bar.name)}</div>
          <div>จำนวนจ่าย: <b style="color:${chartColors.bar}">${formatQty(bar.value)}</b> หน่วย</div>
          <div>สัดส่วน: <b style="color:${chartColors.barDeep}">${pct}%</b> ของทั้งปี</div>
          ${isPeak ? `<div style="margin-top:4px;color:${chartColors.barDeep};font-weight:600;">● เดือนที่จ่ายสูงสุดของปี</div>` : ''}
        `
            },
        },
        legend: {
            data: ['จำนวนจ่าย', 'แนวโน้ม'],
            top: 0,
            right: 0,
            itemWidth: 14,
            itemHeight: 8,
            textStyle: { color: chartColors.textMuted, fontFamily: CHART_FONT, fontSize: 12 },
            icon: 'roundRect',
        },
        xAxis: {
            type: 'category',
            data: THAI_MONTHS_SHORT,
            axisLabel: {
                color: chartColors.textMuted,
                fontFamily: CHART_FONT,
                fontSize: 12,
            },
            axisLine: { lineStyle: { color: 'rgba(78,110,47,0.15)' } },
            axisTick: { show: false },
        },
        yAxis: valueAxis(),
        series: [
            {
                name: 'จำนวนจ่าย',
                type: 'bar',
                data: monthly.map((v: number, i: number) =>
                    i === peakIdx
                        ? {
                            value: v,
                            itemStyle: {
                                borderRadius: [6, 6, 0, 0],
                                color: {
                                    type: 'linear',
                                    x: 0, y: 0, x2: 0, y2: 1,
                                    colorStops: [
                                        { offset: 0, color: chartColors.line },
                                        { offset: 1, color: 'rgba(255,142,126,0.35)' },
                                    ],
                                },
                            },
                        }
                        : v
                ),
                barMaxWidth: 40,
                itemStyle: {
                    color: {
                        type: 'linear',
                        x: 0, y: 0, x2: 0, y2: 1,
                        colorStops: [
                            { offset: 0, color: chartColors.bar },
                            { offset: 1, color: 'rgba(78,110,47,0.25)' },
                        ],
                    },
                    borderRadius: [6, 6, 0, 0],
                },
                emphasis: {
                    itemStyle: {
                        color: {
                            type: 'linear',
                            x: 0, y: 0, x2: 0, y2: 1,
                            colorStops: [
                                { offset: 0, color: chartColors.barBright },
                                { offset: 1, color: 'rgba(78,110,47,0.4)' },
                            ],
                        },
                    },
                },
                animationDelay: (idx: number) => idx * 50,
            },
            {
                name: 'แนวโน้ม',
                type: 'line',
                data: monthly,
                smooth: true,
                symbol: 'circle',
                symbolSize: 7,
                lineStyle: { color: chartColors.line, width: 2.5 },
                itemStyle: {
                    color: chartColors.line,
                    borderColor: '#FFFFFF',
                    borderWidth: 2,
                },
                areaStyle: {
                    color: {
                        type: 'linear',
                        x: 0, y: 0, x2: 0, y2: 1,
                        colorStops: [
                            { offset: 0, color: 'rgba(255,142,126,0.16)' },
                            { offset: 1, color: 'rgba(255,142,126,0)' },
                        ],
                    },
                },
                animationDelay: (idx: number) => idx * 50 + 350,
            },
        ],
        animationEasing: 'cubicOut' as const,
        animationDuration: 700,
    }
})
</script>

<style scoped>
.chart-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-5) var(--space-6) var(--space-4);
    gap: var(--space-4);
}

.chart-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    flex-shrink: 0;
    flex-wrap: wrap;
}

.chart-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.drug-name-title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
    line-height: var(--leading-tight);
}

.drug-code-title {
    font-size: var(--text-sm);
    color: var(--basil-500);
    font-weight: 600;
}

.chart-title-empty {
    font-size: var(--text-lg);
    color: var(--text-muted);
}

.chart-meta {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
    justify-content: flex-end;
}

.meta-chip {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-sm);
    font-weight: 600;
    padding: 7px 13px;
    border-radius: var(--radius-pill);
    white-space: nowrap;
}

.meta-chip--peak {
    background: var(--shrimp-100);
    border: 1px solid rgba(224, 79, 60, 0.25);
    color: #b23b28;
}

.chart-total {
    display: flex;
    align-items: baseline;
    gap: 6px;
    background: var(--basil-50);
    padding: 6px 14px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
}

.total-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: 600;
}

.total-value {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--basil-400);
    line-height: 1.2;
}

.total-unit {
    font-size: var(--text-sm);
    color: var(--text-secondary);
}

.echart {
    flex: 1;
    min-height: 0;
}

.empty-chart,
.loading-chart {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-muted);
}

.empty-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--text-secondary);
}

.empty-bloom,
.loading-bloom {
    color: var(--text-muted);
    opacity: 0.35;
}

.empty-sub {
    font-size: var(--text-md);
    color: var(--text-muted);
    text-align: center;
    line-height: 1.6;
    max-width: 340px;
}
</style>
