<template>
<div class="stat-card" :style="delay ? { animationDelay: delay } : undefined">
    <div class="stat-icon" :class="`tone-${tone}`" aria-hidden="true">
        <slot name="icon" />
    </div>
    <div class="stat-body">
        <div class="stat-label">{{ label }}</div>
        <div class="stat-value tnum" :class="{ 'stat-value--compact': compact }">{{ value }}</div>
        <div v-if="unit" class="stat-unit">{{ unit }}</div>
    </div>
</div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
    label: string
    value: string
    unit?: string
    tone?: 'basil' | 'shrimp' | 'neutral'
    compact?: boolean
    delay?: string
}>(), {
    unit: '',
    tone: 'basil',
    compact: false,
    delay: '',
})
</script>

<style scoped>
.stat-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border-card);
    border-radius: var(--radius-lg);
    padding: 15px 18px;
    box-shadow: var(--shadow-sm);
    transition: box-shadow var(--dur) var(--ease), border-color var(--dur) var(--ease),
        transform var(--dur) var(--ease);
    min-width: 0;
    animation: fadeSlideUp var(--dur-slow) var(--ease) forwards;
    opacity: 0;
}

.stat-card:hover {
    box-shadow: var(--shadow-md);
    border-color: rgba(78, 110, 47, 0.22);
    transform: translateY(-1px);
}

.stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    flex-shrink: 0;
}

.tone-basil {
    background: linear-gradient(135deg, rgba(78, 110, 47, 0.14), rgba(78, 110, 47, 0.05));
    color: var(--basil-500);
}

.tone-shrimp {
    background: linear-gradient(135deg, rgba(255, 142, 126, 0.18), rgba(255, 142, 126, 0.06));
    color: var(--shrimp-600);
}

.tone-neutral {
    background: linear-gradient(135deg, rgba(109, 87, 65, 0.14), rgba(109, 87, 65, 0.05));
    color: var(--text-secondary);
}

.stat-body {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
}

.stat-label {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.stat-value {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--basil-600);
    line-height: 1.2;
}

.stat-value--compact {
    font-size: var(--text-xl);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
}

.stat-unit {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
</style>
