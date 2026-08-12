<template>
  <div class="flex items-center gap-0.5">
    <button
      type="button"
      :disabled="launching"
      class="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs font-medium text-muted-foreground outline-none transition-colors hover:bg-muted/60 hover:text-foreground focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50"
      :title="launching ? '启动中…' : '启动纯净客户端'"
      @click="launchGame"
    >
      <Loader2 v-if="launching" class="size-3.5 animate-spin" />
      <Gamepad2 v-else class="size-3.5" />
      <span>{{ launching ? '启动中' : '启动纯净客户端' }}</span>
    </button>

    <button
      type="button"
      class="inline-flex size-7 items-center justify-center rounded-lg text-muted-foreground outline-none transition-colors hover:bg-muted/50 hover:text-foreground focus-visible:ring-ring/50 focus-visible:ring-[3px]"
      aria-label="配置游戏路径"
      title="配置游戏路径"
      @click="showPathConfig = true"
    >
      <Settings class="size-3.5" />
    </button>

    <Dialog v-model:open="showPathConfig">
      <DialogContent class="max-w-md bg-background text-foreground">
        <DialogHeader>
          <div class="flex items-center gap-2">
            <Settings class="size-5" />
            <span class="text-lg font-semibold">游戏路径配置</span>
          </div>
          <div class="mt-1 text-sm text-muted-foreground">设置英雄联盟游戏的安装路径，留空将自动检测</div>
        </DialogHeader>
        <div class="mt-4 space-y-4">
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
                <FolderOpen class="size-4" />
              </Button>
            </div>
          </div>
          <div class="flex gap-2">
            <Button @click="saveGamePath" class="flex-1" size="sm">
              <Save class="mr-1 size-4" />
              保存
            </Button>
            <Button @click="autoDetectPath" variant="outline" class="flex-1" size="sm">
              <Search class="mr-1 size-4" />
              自动检测
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { FolderOpen, Gamepad2, Loader2, Save, Search, Settings } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core'

const launching = ref(false)
const showPathConfig = ref(false)
const gamePath = ref('')

const launchGame = async () => {
  if (launching.value) return

  launching.value = true

  try {
    const result = await invoke('launch_game', {
      customPath: gamePath.value || null
    })

    if (result) {
      toast.success('游戏启动成功！', {
        description: '正在等待客户端连接，请稍候...'
      })
    } else {
      toast.error('启动失败', {
        description: '无法启动游戏，请检查安装路径是否正确'
      })
    }
  } catch (error) {
    console.error('启动游戏失败:', error)
    toast.error('启动失败', {
      description: String(error)
    })
  } finally {
    launching.value = false
  }
}

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

const autoDetectPath = async () => {
  try {
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
  }
}

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

onMounted(() => {
  void loadGamePath()
})
</script>
