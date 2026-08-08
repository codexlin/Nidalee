<template>
  <div class="flex items-center gap-2">
    <Button
      @click="launchGame"
      :disabled="launching"
      :class="[
        'group relative overflow-hidden text-white font-semibold py-3 px-8 rounded-xl shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105',
        isDark
          ? 'bg-gradient-to-r from-blue-900 to-indigo-800 hover:from-blue-800 hover:to-indigo-700'
          : 'bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700'
      ]"
    >
      <div class="flex items-center gap-2">
        <Gamepad2 v-if="!launching" class="w-5 h-5" />
        <div v-else class="w-5 h-5">
          <div class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"></div>
        </div>
        <span>{{ launching ? '启动中...' : '一键启动游戏' }}</span>
      </div>
      <div
        :class="[
          'absolute inset-0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-1000',
          isDark
            ? 'bg-gradient-to-r from-white/0 via-white/10 to-white/0'
            : 'bg-gradient-to-r from-white/0 via-white/20 to-white/0'
        ]"
      ></div>
    </Button>
    <Button
      @click="showPathConfig = true"
      variant="ghost"
      size="icon"
      class="text-muted-foreground hover:text-foreground"
      :aria-label="'配置游戏路径'"
    >
      <Settings class="w-5 h-5" />
    </Button>

    <!-- 状态提示 -->
    <div v-if="statusMessage" class="text-sm text-muted-foreground text-center ml-2">
      {{ statusMessage }}
    </div>

    <!-- 配置弹窗 -->
    <Dialog v-model:open="showPathConfig">
      <DialogContent class="max-w-md bg-background text-foreground">
        <DialogHeader>
          <div class="flex items-center gap-2">
            <Settings class="w-5 h-5" />
            <span class="text-lg font-semibold">游戏路径配置</span>
          </div>
          <div class="text-muted-foreground text-sm mt-1">设置英雄联盟游戏的安装路径，留空将自动检测</div>
        </DialogHeader>
        <div class="space-y-4 mt-4">
          <div class="space-y-2">
            <Label htmlFor="game-path" class="text-foreground">游戏安装路径</Label>
            <div class="flex gap-2">
              <Input
                id="game-path"
                v-model="gamePath"
                placeholder="eg: D:\WeGameApps\英雄联盟（含经典模式）\Launcher\Client.exe"
                class="flex-1 bg-background text-foreground placeholder:text-muted-foreground"
              />
              <Button @click="selectGamePath" variant="outline" size="sm" class="px-3">
                <FolderOpen class="w-4 h-4" />
              </Button>
            </div>
          </div>
          <div class="flex gap-2">
            <Button @click="saveGamePath" class="flex-1" size="sm">
              <Save class="w-4 h-4 mr-1" />
              保存
            </Button>
            <Button @click="autoDetectPath" variant="outline" class="flex-1" size="sm">
              <Search class="w-4 h-4 mr-1" />
              自动检测
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { Gamepad2, Settings, FolderOpen, Save, Search } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core'
import { appContextKey, type AppContext } from '@/types'

// 状态
const launching = ref(false)
const showPathConfig = ref(false)
const gamePath = ref('')
const statusMessage = ref('')

// 主题 - 使用 inject 获取 App.vue 提供的状态
const appContext = inject<AppContext>(appContextKey)!
const { isDark } = appContext

// 启动游戏
const launchGame = async () => {
  if (launching.value) return

  launching.value = true
  statusMessage.value = '正在启动游戏...'

  try {
    // 调用 Rust 后端启动游戏
    const result = await invoke('launch_game', {
      customPath: gamePath.value || null
    })

    if (result) {
      statusMessage.value = '游戏启动成功！等待客户端连接...'
      toast.success('游戏启动成功！', {
        description: '正在等待客户端连接，请稍候...'
      })
    } else {
      statusMessage.value = '启动失败，请检查游戏路径'
      toast.error('启动失败', {
        description: '无法启动游戏，请检查安装路径是否正确'
      })
    }
  } catch (error) {
    console.error('启动游戏失败:', error)
    statusMessage.value = '启动失败，请检查游戏是否已安装'
    toast.error('启动失败', {
      description: String(error)
    })
  } finally {
    launching.value = false
    // 3秒后清除状态
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
  }
}

// 选择路径
const selectGamePath = async () => {
  try {
    const selected = await invoke('select_game_path')

    if (selected) {
      gamePath.value = selected as string
      toast.success('选择成功！', {
        description: '已选择游戏安装路径'
      })
    }
  } catch (error) {
    console.error('选择文件失败:', error)
    toast.error('选择文件失败')
  }
}

// 自动检测路径
const autoDetectPath = async () => {
  try {
    statusMessage.value = '正在检测游戏路径...'
    const detected = await invoke('detect_game_path')

    if (detected) {
      gamePath.value = detected as string
      toast.success('检测成功！', {
        description: '已自动检测到游戏安装路径'
      })
    } else {
      toast.warning('未检测到游戏', {
        description: '请手动选择游戏安装路径'
      })
    }
  } catch (error) {
    console.error('自动检测失败:', error)
    toast.error('检测失败', {
      description: '自动检测游戏路径失败'
    })
  } finally {
    statusMessage.value = ''
  }
}

// 保存路径
const saveGamePath = async () => {
  try {
    await invoke('save_game_path', { path: gamePath.value })
    toast.success('保存成功！', {
      description: '游戏路径已保存到配置中'
    })
    showPathConfig.value = false
  } catch (error) {
    console.error('保存失败:', error)
    toast.error('保存失败')
  }
}

// 加载保存路径
const loadGamePath = async () => {
  try {
    const saved = await invoke('get_saved_game_path')
    if (saved) {
      gamePath.value = saved as string
    }
  } catch (error) {
    console.error('加载游戏路径失败:', error)
  }
}

// 挂载时加载
onMounted(() => {
  loadGamePath()
})
</script>

<style scoped>
/* 自定义样式 */
</style>
