<template>
  <Popover>
    <PopoverTrigger as-child>
      <FloatIconButton class="p-2" aria-label="查看活动通知" title="查看活动通知">
        <Bell :size="16" />
        <span
          v-if="unreadCount > 0"
          class="absolute -top-1 -right-1.5 flex h-4 min-w-4 items-center justify-center rounded-full border border-background bg-destructive px-1 text-[10px] font-bold text-white"
        >
          {{ unreadCount > 9 ? '9+' : unreadCount }}
        </span>
      </FloatIconButton>
    </PopoverTrigger>

    <PopoverContent :side="side" :align="align" class="surface-overlay w-[320px] !border-none p-0">
      <div class="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/5 to-primary/10 rounded-t-2xl">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <div class="p-1.5 rounded-lg bg-primary/10">
              <Bell :size="15" class="text-primary" />
            </div>
            <h3 class="text-base font-semibold text-foreground">{{ title }}</h3>
          </div>
          <div class="flex items-center space-x-2">
            <div v-if="errorCount > 0" class="flex items-center space-x-1">
              <div class="h-2 w-2 bg-red-500 rounded-full"></div>
              <span class="text-xs text-red-600 font-medium">{{ errorCount }}</span>
            </div>
            <div v-if="warningCount > 0" class="flex items-center space-x-1">
              <div class="h-2 w-2 bg-yellow-500 rounded-full"></div>
              <span class="text-xs text-yellow-600 font-medium">{{ warningCount }}</span>
            </div>
            <Badge v-if="unreadCount > 0" variant="secondary" class="text-xs bg-red-500/10 text-red-600 border-red-200">
              {{ unreadCount }} 条未读
            </Badge>
          </div>
        </div>
      </div>

      <ScrollArea>
        <div class="px-3 py-2 max-h-[260px]">
          <div class="space-y-2">
            <div
              v-for="activity in (activities || []).slice(0, 6)"
              :key="activity.id"
              class="flex items-center gap-3 p-3 rounded-xl border border-border/30 bg-card/70 hover:bg-card transition-all duration-200 shadow-sm"
            >
              <span
                class="flex items-center justify-center w-7 h-7 rounded-full"
                :class="[getActivityTypeConfig(activity.type).bg, getActivityTypeConfig(activity.type).text]"
              >
                <span class="text-base">{{ getActivityTypeConfig(activity.type).icon }}</span>
              </span>
              <div class="flex-1 min-w-0">
                <div class="flex items-center">
                  <span class="text-sm font-medium text-foreground truncate flex-1">{{ activity.message }}</span>
                  <div class="flex items-center min-w-12 justify-end ml-2 gap-1">
                    <span class="text-xs text-muted-foreground">{{ formatRelativeTime(activity.timestamp) }}</span>
                    <span v-if="!activity.read" class="h-2 w-2 bg-blue-500 rounded-full animate-pulse"></span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="!activities || activities.length === 0" class="text-center py-8">
              <div class="p-3 rounded-full bg-muted/50 w-12 h-12 mx-auto mb-2 flex items-center justify-center">
                <Bell class="h-5 w-5 text-muted-foreground" />
              </div>
              <p class="text-sm text-muted-foreground">暂无活动记录</p>
              <p class="text-xs text-muted-foreground mt-1">所有活动将在这里显示</p>
            </div>
          </div>
        </div>
      </ScrollArea>

      <div class="px-4 py-2 border-t border-border/30 bg-muted/20 rounded-b-2xl">
        <div class="flex items-center justify-between">
          <button
            v-if="unreadCount > 0"
            class="inline-flex items-center px-2 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors rounded-md hover:bg-muted/40"
            @click="handleMarkAllRead"
          >
            <CheckCheck :size="13" class="mr-1" />
            全部标记为已读
          </button>
          <div v-else></div>

          <button
            class="inline-flex items-center px-2 py-1.5 text-xs text-primary hover:text-primary/80 font-medium rounded-md hover:bg-primary/10 transition-all duration-200"
            @click="handleClearAll"
          >
            清理全部活动
            <ArrowRight :size="13" class="ml-1" />
          </button>
        </div>
      </div>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { ArrowRight, Bell, CheckCheck } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'

withDefaults(
  defineProps<{
    title?: string
    side?: 'top' | 'right' | 'bottom' | 'left'
    align?: 'start' | 'center' | 'end'
  }>(),
  {
    title: '最近活动',
    side: 'bottom',
    align: 'end'
  }
)

const activityStore = useActivityStore()
const { formatRelativeTime } = useFormatters()

const activities = computed(() => activityStore.recentActivities)

const errorCount = computed(() => activityStore.errorCount)
const warningCount = computed(() => activityStore.warningCount)

const unreadCount = computed(() => {
  return activities.value?.filter((activity) => !activity.read)?.length || 0
})

const handleMarkAllRead = () => {
  activityStore.markAllAsRead()
}

const handleClearAll = () => {
  activityStore.clearActivities()
}

// 获取活动类型的颜色和图标
const getActivityTypeConfig = (type: string) => {
  const configs = {
    success: { bg: 'bg-green-100', text: 'text-green-600', icon: '✔️' },
    info: { bg: 'bg-blue-100', text: 'text-blue-600', icon: 'ℹ️' },
    warning: { bg: 'bg-yellow-100', text: 'text-yellow-600', icon: '⚠️' },
    error: { bg: 'bg-red-100', text: 'text-red-600', icon: '✖️' }
  }
  return configs[type as keyof typeof configs] || configs.info
}
</script>

<style scoped>
.line-clamp-1 {
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
