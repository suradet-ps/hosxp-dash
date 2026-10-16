// Shared ECharts styling — keeps every chart on the same typography,
// palette, tooltip treatment, and grid rhythm (Carbon/Tableau dashboard
// guidance: consistent layout, legends, and colors across all charts).

export const CHART_FONT =
  'Sarabun, IBM Plex Sans Thai, Leelawadee UI, Segoe UI, Tahoma, sans-serif'

export const chartColors = {
  bar: '#4e6e2f',
  barDeep: '#3a5220',
  barBright: '#7aad52',
  line: '#ff8e7e',
  text: '#43301f',
  textMuted: '#75604b',
  axis: '#a08a72',
  split: 'rgba(78,110,47,0.08)',
  tooltipBorder: 'rgba(78,110,47,0.35)',
  gridFill: 'rgba(78,110,47,0.1)',
} as const

export function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

export function baseTooltip() {
  return {
    backgroundColor: '#FFFFFF',
    borderColor: chartColors.tooltipBorder,
    borderWidth: 1,
    padding: [10, 14],
    textStyle: { color: chartColors.text, fontFamily: CHART_FONT, fontSize: 12.5 },
    extraCssText:
      'box-shadow: 0 8px 28px rgba(67,48,31,0.16); border-radius: 10px;',
  }
}

/** Axis labels / splitlines shared by value axes */
export function valueAxis() {
  return {
    type: 'value' as const,
    axisLabel: {
      color: chartColors.axis,
      fontFamily: CHART_FONT,
      fontSize: 11,
      formatter: (v: number) => (v >= 1000 ? (v / 1000).toFixed(1) + 'K' : v.toString()),
    },
    splitLine: {
      lineStyle: { color: chartColors.split, type: 'dashed' as const },
    },
    axisLine: { show: false },
    axisTick: { show: false },
  }
}

/** Horizontal gradient used by bars growing left → right */
export function barGradient(from = 'rgba(78,110,47,0.35)', to = chartColors.bar) {
  return {
    type: 'linear' as const,
    x: 0,
    y: 0,
    x2: 1,
    y2: 0,
    colorStops: [
      { offset: 0, color: from },
      { offset: 1, color: to },
    ],
  }
}
