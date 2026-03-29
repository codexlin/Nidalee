<template>
  <div class="p-6 max-w-4xl mx-auto space-y-6">
    <!-- 🔍 LCU 认证验证工具 -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          LCU 认证验证
        </CardTitle>
        <CardDescription>对比 lockfile 和进程命令行获取的认证信息</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="flex gap-2">
          <Button @click="handleVerify" :disabled="isVerifying">
            <svg v-if="!isVerifying" class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <svg v-else class="w-4 h-4 mr-1 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isVerifying ? '验证中...' : '开始验证' }}
          </Button>
          <Button variant="outline" @click="handleClearLogs" v-if="verifyLogs.length > 0"> 清空日志 </Button>
        </div>

        <!-- 验证日志 -->
        <div v-if="verifyLogs.length > 0" class="p-3 rounded-lg bg-muted max-h-60 overflow-y-auto text-xs font-mono space-y-1">
          <div v-for="(log, index) in verifyLogs" :key="index" :class="getLogClass(log)">
            {{ log }}
          </div>
        </div>
        <div v-else class="p-3 rounded-lg bg-muted/50 text-center text-sm text-muted-foreground">
          点击"开始验证"按钮，对比两种方式获取的 LCU 认证信息
        </div>
      </CardContent>
    </Card>

    <!-- 房间聊天测试 -->
    <Card>
      <CardHeader>
        <CardTitle>房间聊天测试</CardTitle>
        <CardDescription>测试 Lobby 聊天功能</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- 房间信息 -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-medium">房间信息</h3>
            <Button size="sm" variant="outline" @click="handleRefreshLobby">
              <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              刷新
            </Button>
          </div>

          <div v-if="currentLobby" class="p-4 rounded-lg bg-muted space-y-2 text-sm">
            <div class="flex items-center gap-2">
              <Badge :variant="canSendMessage ? 'default' : 'secondary'">
                {{ canSendMessage ? '✅ 可发送消息' : '❌ 不在房间中' }}
              </Badge>
              <Badge variant="outline">队伍类型: {{ currentLobby.partyType }}</Badge>
            </div>
            <div>
              <span class="text-muted-foreground">聊天室ID:</span>
              <code class="ml-2 text-xs bg-background px-2 py-1 rounded">{{ chatId || '无' }}</code>
            </div>
            <div>
              <span class="text-muted-foreground">成员数量:</span>
              <span class="ml-2">{{ currentLobby.members?.length || 0 }} 人</span>
            </div>
          </div>
          <div v-else class="p-4 rounded-lg bg-muted/50 text-center text-sm text-muted-foreground">
            未获取到房间信息，请先创建或加入房间
          </div>
        </div>

        <!-- 发送消息区域 -->
        <div class="space-y-2">
          <Label>发送消息</Label>
          <div class="flex gap-2">
            <Input v-model="messageInput" placeholder="输入消息内容..." @keyup.enter="handleSendMessage" />
            <Button :disabled="!canSendMessage || !messageInput.trim()" @click="handleSendMessage"> 发送 </Button>
          </div>
        </div>

        <!-- 快捷消息 -->
        <div class="space-y-2">
          <Label>快捷消息</Label>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="msg in quickMessages"
              :key="msg"
              size="sm"
              variant="outline"
              :disabled="!canSendMessage"
              @click="handleQuickMessage(msg)"
            >
              {{ msg }}
            </Button>
          </div>
        </div>

        <!-- 英雄推荐测试 -->
        <div class="space-y-2">
          <Label>英雄推荐测试</Label>
          <div class="flex gap-2">
            <Input v-model="championName" placeholder="英雄名称（如：亚索）" />
            <Input v-model="recommendReason" placeholder="推荐理由" />
            <Button :disabled="!canSendMessage || !championName" @click="handleSendRecommendation"> 发送推荐 </Button>
          </div>
        </div>

        <!-- 发送日志 -->
        <div class="space-y-2">
          <Label>发送日志</Label>
          <div class="p-3 rounded-lg bg-muted max-h-40 overflow-y-auto text-xs font-mono space-y-1">
            <div v-for="(log, index) in logs" :key="index" class="text-muted-foreground">
              {{ log }}
            </div>
            <div v-if="logs.length === 0" class="text-center text-muted-foreground/50">暂无日志</div>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useLobbyChat } from '@/composables/lobby'

// LCU 验证相关
const isVerifying = ref(false)
const verifyLogs = ref<string[]>([])

const getLogClass = (log: string) => {
  if (log.includes('✅') || log.includes('一致')) return 'text-green-500'
  if (log.includes('❌') || log.includes('不一致') || log.includes('失败')) return 'text-red-500'
  if (log.includes('⚠️')) return 'text-yellow-500'
  return 'text-muted-foreground'
}

const handleVerify = async () => {
  isVerifying.value = true
  verifyLogs.value = []

  try {
    await invoke('verify_lockfile_vs_cmdline')
    // 延迟一下让后端日志输出完成
    await new Promise(resolve => setTimeout(resolve, 500))

    // 提示用户查看后端日志
    verifyLogs.value = [
      '========== 验证完成 ==========',
      '✅ 请查看应用后端日志查看详细对比结果',
      '📍 日志位置：终端输出或日志文件',
      '============================'
    ]
  } catch (error) {
    verifyLogs.value = [
      '❌ 验证失败：' + (error instanceof Error ? error.message : '未知错误')
    ]
  } finally {
    isVerifying.value = false
  }
}

const handleClearLogs = () => {
  verifyLogs.value = []
}

const {
  currentLobby,
  chatId,
  canSendMessage,
  refreshLobby,
  sendMessage,
  sendNidaleeMessage,
  sendChampionRecommendation,
  sendTacticTip
} = useLobbyChat()

// 消息输入
const messageInput = ref('')
const championName = ref('')
const recommendReason = ref('当前版本强势')

// 快捷消息
const quickMessages = ['大家好！', '我先拿打野', '推荐ban亚索', '准备开始吧', '祝游戏愉快！']

// 日志
const logs = ref<string[]>([])
const addLog = (message: string) => {
  const timestamp = new Date().toLocaleTimeString()
  logs.value.unshift(`[${timestamp}] ${message}`)
  if (logs.value.length > 20) {
    logs.value = logs.value.slice(0, 20)
  }
}

// 刷新房间
const handleRefreshLobby = async () => {
  addLog('正在刷新房间信息...')
  const result = await refreshLobby()
  if (result) {
    addLog(`✅ 房间信息已更新，聊天室ID: ${chatId.value}`)
  } else {
    addLog('❌ 获取房间信息失败')
  }
}

// 发送消息
const handleSendMessage = async () => {
  if (!messageInput.value.trim()) return

  const message = messageInput.value
  addLog(`发送消息: ${message}`)

  const success = await sendMessage(message)
  if (success) {
    addLog('✅ 消息发送成功')
    messageInput.value = ''
  } else {
    addLog('❌ 消息发送失败')
  }
}

// 快捷消息
const handleQuickMessage = async (message: string) => {
  addLog(`发送快捷消息: ${message}`)
  const success = await sendMessage(message)
  if (success) {
    addLog('✅ 快捷消息发送成功')
  } else {
    addLog('❌ 快捷消息发送失败')
  }
}

// 发送英雄推荐
const handleSendRecommendation = async () => {
  if (!championName.value.trim()) return

  addLog(`发送英雄推荐: ${championName.value}`)
  const success = await sendChampionRecommendation(championName.value, recommendReason.value)
  if (success) {
    addLog('✅ 英雄推荐发送成功')
  } else {
    addLog('❌ 英雄推荐发送失败')
  }
}

// 组件挂载时刷新房间信息
onMounted(() => {
  handleRefreshLobby()
})
</script>
