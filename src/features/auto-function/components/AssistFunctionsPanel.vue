<template>
  <Card class="gap-0 overflow-hidden py-0">
    <!-- 辅助接受：整行 -->
    <section class="border-b border-border/50 px-4 py-4 sm:px-5">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 space-y-0.5">
          <h3 class="text-lg font-medium leading-tight">辅助接受</h3>
          <p class="text-xs text-muted-foreground">匹配到对局后代为接受，避免错过</p>
        </div>
        <Switch
          :model-value="accept.enabled"
          class="mt-0.5 shrink-0"
          @update:model-value="(v) => (accept.enabled = v)"
        />
      </div>
      <DelayControls v-if="accept.enabled" class="mt-3" v-model:delay="accept.delay" />
    </section>

    <!-- 辅助选人 | 辅助禁人 -->
    <div class="grid items-start gap-0 lg:grid-cols-2 lg:divide-x lg:divide-border/50">
      <section class="min-w-0 px-4 py-4 sm:px-5">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0 space-y-0.5">
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-medium leading-tight">辅助选人</h3>
              <Tooltip :delay-duration="100">
                <TooltipTrigger>
                  <AlertTriangle class="size-4 shrink-0 text-amber-500" />
                </TooltipTrigger>
                <TooltipContent side="top" class="max-w-xs">
                  <p class="text-sm">请设置适当延迟，降低异常风险</p>
                </TooltipContent>
              </Tooltip>
            </div>
            <p class="text-xs text-muted-foreground">选人阶段按顺序锁定英雄</p>
          </div>
          <Switch
            :model-value="select.enabled"
            class="mt-0.5 shrink-0"
            @update:model-value="(v) => (select.enabled = v)"
          />
        </div>
        <div v-if="select.enabled" class="mt-3 space-y-3">
          <ChampionPickRow
            :champions="select.championList"
            @add="emit('select-add', $event)"
            @remove="emit('select-remove', $event)"
            @clear="emit('select-clear')"
            @reorder="(from, to) => emit('select-reorder', from, to)"
          />
          <DelayControls v-model:delay="select.delay" />
        </div>
      </section>

      <section class="min-w-0 border-t border-border/50 px-4 py-4 sm:px-5 lg:border-t-0">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0 space-y-0.5">
            <div class="flex items-center gap-2">
              <h3 class="text-lg font-medium leading-tight">辅助禁人</h3>
              <Tooltip :delay-duration="100">
                <TooltipTrigger>
                  <AlertTriangle class="size-4 shrink-0 text-amber-500" />
                </TooltipTrigger>
                <TooltipContent side="top" class="max-w-xs">
                  <p class="text-sm">请设置适当延迟，降低异常风险</p>
                </TooltipContent>
              </Tooltip>
            </div>
            <p class="text-xs text-muted-foreground">禁用阶段按顺序禁英雄</p>
          </div>
          <Switch :model-value="ban.enabled" class="mt-0.5 shrink-0" @update:model-value="(v) => (ban.enabled = v)" />
        </div>
        <div v-if="ban.enabled" class="mt-3 space-y-3">
          <ChampionPickRow
            :champions="ban.championList"
            @add="emit('ban-add', $event)"
            @remove="emit('ban-remove', $event)"
            @clear="emit('ban-clear')"
            @reorder="(from, to) => emit('ban-reorder', from, to)"
          />
          <DelayControls v-model:delay="ban.delay" />
        </div>
      </section>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next'
import DelayControls from './DelayControls.vue'
import ChampionPickRow from './ChampionPickRow.vue'

defineProps<{
  accept: { enabled: boolean; delay: number }
  select: { enabled: boolean; delay: number; championList: ChampionInfo[] }
  ban: { enabled: boolean; delay: number; championList: ChampionInfo[] }
}>()

const emit = defineEmits<{
  'select-add': [champion: ChampionInfo]
  'select-remove': [championId: number]
  'select-clear': []
  'select-reorder': [from: number, to: number]
  'ban-add': [champion: ChampionInfo]
  'ban-remove': [championId: number]
  'ban-clear': []
  'ban-reorder': [from: number, to: number]
}>()
</script>
