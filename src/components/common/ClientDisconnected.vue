<template>
  <div class="relative flex min-h-[calc(100vh-40px)] w-full flex-col items-center justify-center bg-background">
    <MorphingText class="lg:text-[4rem] text-muted-foreground" :texts="texts" />
    <ParticlesBg
      class="absolute inset-0"
      :quantity="100"
      :ease="100"
      :color="useSettingsStore().isDark ? '#FFF' : '#000'"
      :staticity="10"
      refresh
    />
  </div>
</template>

<script setup lang="ts">
import { useSettingsStore } from '@/shared/stores/ui/settingsStore'
const texts = [
  'Nidalee - By CodexLin',
  '等待连接英雄联盟客户端...',
  '智能分析敌我玩家',
  '检测游戏进程中...',
  '符文出装推荐，一键导入',
  '等待客户端启动中...',
  '实时对局数据分析',
  '检查网络连接状态...',
  '自动选择最优符文配置',
  '等待召唤师上线...',
  'OP.GG 配置一键同步',
  '检测游戏窗口...',
  '智能英雄选择建议',
  '准备连接服务器...',
  '个性化游戏体验',
  '等待客户端响应...',
  '让游戏更简单',
  '让胜利更轻松',
  '检测游戏状态...',
  '实时对局统计信息',
  '等待连接建立...',
  'Nidalee - By CodexLin'
]
</script>
