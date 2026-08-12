<script setup lang="ts">
import { Pencil, Trash2, Wand2 } from 'lucide-vue-next'
import { scenarioLabel, type BuildPreset, type BuildPresetSourceKind } from '@/shared/models/buildPreset'

defineProps<{
  preset: BuildPreset
  applying: boolean
  actionsDisabled: boolean
  autoUseDisabled: boolean
}>()

const emit = defineEmits<{
  apply: [preset: BuildPreset]
  edit: [preset: BuildPreset]
  remove: [preset: BuildPreset]
  setAutoUse: [preset: BuildPreset, enabled: boolean]
}>()

const sourceLabel = (source: BuildPresetSourceKind) =>
  ({
    opgg: 'OP.GG 快照',
    custom: '自定义',
    import: '导入',
    client: '客户端'
  })[source]

const styleName = (styleId: number) =>
  ({
    8000: '精密',
    8100: '主宰',
    8200: '巫术',
    8300: '启迪',
    8400: '坚决'
  })[styleId] ?? `符文系 ${styleId}`
</script>

<template>
  <article class="rounded-xl border border-border/60 bg-background/35 p-3">
    <div class="flex min-w-0 items-start justify-between gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 flex-wrap items-center gap-1.5">
          <h4 class="max-w-full truncate text-sm font-medium">{{ preset.name }}</h4>
          <Badge variant="outline">{{ scenarioLabel(preset.target.scenario) }}</Badge>
        </div>
        <p class="mt-1 truncate text-xs text-muted-foreground">
          {{ styleName(preset.components.runes.primaryStyleId) }} +
          {{ styleName(preset.components.runes.subStyleId) }} · {{ sourceLabel(preset.source.kind) }} ·
          {{ preset.usageCount }} 次
        </p>
      </div>
      <Badge v-if="preset.autoUse">自动使用</Badge>
    </div>

    <div class="mt-3 flex flex-wrap items-center justify-between gap-2">
      <label class="flex cursor-pointer items-center gap-2 text-xs text-muted-foreground">
        <Switch
          :model-value="preset.autoUse"
          :disabled="autoUseDisabled"
          :aria-label="`${preset.name}自动使用`"
          @update:model-value="emit('setAutoUse', preset, $event)"
        />
        自动使用
      </label>
      <div class="flex items-center gap-1">
        <Button size="sm" variant="ghost" :disabled="actionsDisabled" @click="emit('apply', preset)">
          <Spinner v-if="applying" data-icon="inline-start" />
          <Wand2 v-else data-icon="inline-start" />
          立即应用
        </Button>
        <Button size="icon" variant="ghost" :aria-label="`编辑${preset.name}`" @click="emit('edit', preset)">
          <Pencil />
        </Button>
        <Button size="icon" variant="ghost" :aria-label="`删除${preset.name}`" @click="emit('remove', preset)">
          <Trash2 />
        </Button>
      </div>
    </div>
  </article>
</template>
