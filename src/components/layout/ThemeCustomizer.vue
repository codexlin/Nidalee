<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0 space-y-0.5">
        <h2 class="text-lg font-medium leading-tight">主题外观</h2>
        <p class="mt-0.5 text-xs text-muted-foreground">主色、圆角与深浅模式</p>
      </div>
      <Button
        size="sm"
        variant="outline"
        class="h-8 shrink-0 transition hover:bg-destructive/10 hover:text-destructive"
        title="重置主题"
        @click="settingsStore.resetTheme"
      >
        <RotateCcw class="size-3.5" />
        重置
      </Button>
    </div>

    <div>
      <Label class="mb-2 block text-sm font-medium">主题主色</Label>
      <div class="grid grid-cols-3 gap-3">
        <button
          v-for="color in settingsStore.colors"
          :key="color.name"
          :class="
            cn(
              'flex h-10 w-full items-center justify-center rounded-lg border-2 text-xs font-medium transition-all duration-150',
              settingsStore.selectedColor === color.name
                ? 'border-primary ring-2 ring-primary/60 shadow-lg scale-105'
                : 'border-muted hover:border-primary/40 hover:scale-105'
            )
          "
          @click="() => settingsStore.setColor(color.name)"
          type="button"
        >
          <div
            :class="
              cn(
                'h-5 w-5 rounded-full border',
                color.bgClass,
                settingsStore.selectedColor === color.name ? 'border-primary' : 'border-muted'
              )
            "
          />
          <span class="ml-2 text-foreground">{{ color.label }}</span>
        </button>
      </div>
    </div>

    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <div class="min-w-0 space-y-2">
        <Label class="block text-sm font-medium">圆角风格</Label>
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="radius in settingsStore.radiusOptions"
            :key="radius.value"
            type="button"
            :class="
              cn(
                'flex h-9 w-11 items-center justify-center rounded-lg border text-xs font-medium transition-all duration-150',
                settingsStore.selectedRadius === radius.value
                  ? 'border-primary bg-accent shadow'
                  : 'border-muted hover:border-primary/40'
              )
            "
            @click="settingsStore.setRadius(radius.value)"
          >
            {{ radius.label }}
          </button>
        </div>
      </div>

      <div class="min-w-0 space-y-2">
        <Label class="block text-sm font-medium">主题模式</Label>
        <div class="flex flex-wrap gap-1.5">
          <button
            type="button"
            :class="
              cn(
                'inline-flex h-9 items-center gap-1.5 rounded-lg border px-3 text-sm font-medium transition-colors',
                !settingsStore.isDark ? 'border-primary bg-accent shadow' : 'border-muted hover:border-primary/40'
              )
            "
            @click="() => settingsStore.toggleTheme(false)"
          >
            <Sun class="size-3.5" />
            Light
          </button>
          <button
            type="button"
            :class="
              cn(
                'inline-flex h-9 items-center gap-1.5 rounded-lg border px-3 text-sm font-medium transition-colors',
                settingsStore.isDark ? 'border-primary bg-accent shadow' : 'border-muted hover:border-primary/40'
              )
            "
            @click="() => settingsStore.toggleTheme(true)"
          >
            <Moon class="size-3.5" />
            Dark
          </button>
        </div>
      </div>

      <div class="min-w-0 space-y-2 sm:col-span-2 lg:col-span-1">
        <Label class="block text-sm font-medium">风格</Label>
        <div class="grid grid-cols-2 gap-1.5">
          <button
            v-for="style in settingsStore.styles"
            :key="style.name"
            type="button"
            :class="
              cn(
                'flex h-9 items-center justify-center rounded-lg border text-xs font-medium transition-all duration-150',
                settingsStore.selectedStyle === style.name
                  ? 'border-primary bg-accent shadow'
                  : 'border-muted hover:border-primary/40'
              )
            "
            @click="settingsStore.setStyle(style.name)"
          >
            {{ style.label }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { cn } from '@/lib/utils'
import { RotateCcw, Sun, Moon } from 'lucide-vue-next'

const settingsStore = useSettingsStore()

onMounted(() => {
  settingsStore.initTheme()
})
</script>
