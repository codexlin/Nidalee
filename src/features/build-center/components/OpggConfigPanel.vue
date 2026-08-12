<template>
  <div class="flex flex-wrap items-center gap-2">
    <template v-if="!compact">
      <Select :model-value="config.region" @update:model-value="patch('region', String($event))">
        <SelectTrigger class="h-9 w-[8rem] text-sm">
          <SelectValue placeholder="区域" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="region in regions" :key="region.value" :value="region.value">
            {{ region.label }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select :model-value="config.mode" @update:model-value="patch('mode', String($event))">
        <SelectTrigger class="h-9 w-[7.5rem] text-sm">
          <SelectValue placeholder="模式" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="mode in modes" :key="mode.value" :value="mode.value">
            {{ mode.label }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select
        v-if="config.mode === 'ranked'"
        :model-value="config.tier"
        @update:model-value="patch('tier', String($event))"
      >
        <SelectTrigger class="h-9 w-[8rem] text-sm">
          <SelectValue placeholder="段位" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="tier in tiers" :key="tier.value" :value="tier.value">
            {{ tier.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </template>
    <span v-else class="text-sm text-muted-foreground">国服 · 海克斯增强</span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  config: {
    region: string
    mode: string
    tier: string
    position: string
    championId: number
  }
  regions: Array<{ value: string; label: string }>
  modes: Array<{ value: string; label: string }>
  tiers: Array<{ value: string; label: string }>
  positions: Array<{ value: string; label: string }>
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), { compact: false })
const emit = defineEmits<{
  'update:config': [value: Props['config']]
}>()

const patch = <K extends keyof Props['config']>(key: K, value: Props['config'][K]) => {
  emit('update:config', { ...props.config, [key]: value })
}
</script>
