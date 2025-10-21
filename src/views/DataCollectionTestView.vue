<template>
  <div class="p-6 space-y-6">
    <div class="bg-card rounded-lg p-6">
      <h2 class="text-2xl font-bold mb-4">🔬 数据收集测试工具</h2>
      <p class="text-muted-foreground mb-6">
        用于生成分析数据文件，帮助优化算法。生成的数据文件可以用于算法改进和阈值优化。
      </p>

      <!-- 数据收集配置 -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div>
          <label class="block text-sm font-medium mb-2">游戏场次</label>
          <Input
            v-model="gameCount"
            type="number"
            placeholder="20"
            min="1"
            max="100"
          />
        </div>
        <div>
          <label class="block text-sm font-medium mb-2">队列ID</label>
          <Select v-model="selectedQueue">
            <SelectTrigger>
              <SelectValue placeholder="选择队列" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">所有队列</SelectItem>
              <SelectItem value="420">单双排 (420)</SelectItem>
              <SelectItem value="440">灵活组排 (440)</SelectItem>
              <SelectItem value="450">大乱斗 (450)</SelectItem>
              <SelectItem value="700">排位 (700)</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div class="flex items-end">
          <Button @click="generateData" :disabled="isGenerating" class="w-full">
            <Loader2 v-if="isGenerating" class="w-4 h-4 mr-2 animate-spin" />
            {{ isGenerating ? '生成中...' : '生成数据文件' }}
          </Button>
        </div>
      </div>

      <!-- 生成结果 -->
      <div v-if="generationResult" class="mb-6">
        <Alert>
          <CheckCircle class="h-4 w-4" />
          <AlertTitle>生成成功</AlertTitle>
          <AlertDescription>
            {{ generationResult }}
          </AlertDescription>
        </Alert>
      </div>

      <!-- 错误信息 -->
      <div v-if="errorMessage" class="mb-6">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>生成失败</AlertTitle>
          <AlertDescription>
            {{ errorMessage }}
          </AlertDescription>
        </Alert>
      </div>
    </div>

    <!-- 数据分析 -->
    <div class="bg-card rounded-lg p-6">
      <h3 class="text-xl font-bold mb-4">📊 数据分析</h3>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <div>
          <label class="block text-sm font-medium mb-2">数据文件路径</label>
          <Input
            v-model="dataFilePath"
            placeholder="analysis_data_20241201_120000.json"
          />
        </div>
        <div class="flex items-end">
          <Button @click="analyzeData" :disabled="isAnalyzing" class="w-full">
            <Loader2 v-if="isAnalyzing" class="w-4 h-4 mr-2 animate-spin" />
            {{ isAnalyzing ? '分析中...' : '分析数据' }}
          </Button>
        </div>
      </div>

      <!-- 分析结果 -->
      <div v-if="analysisResult" class="mt-4">
        <Alert>
          <BarChart3 class="h-4 w-4" />
          <AlertTitle>分析结果</AlertTitle>
          <AlertDescription>
            <pre class="mt-2 text-sm whitespace-pre-wrap">{{ analysisResult }}</pre>
          </AlertDescription>
        </Alert>
      </div>

      <!-- 分析错误 -->
      <div v-if="analysisError" class="mt-4">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>分析失败</AlertTitle>
          <AlertDescription>
            {{ analysisError }}
          </AlertDescription>
        </Alert>
      </div>
    </div>

    <!-- 使用说明 -->
    <div class="bg-card rounded-lg p-6">
      <h3 class="text-xl font-bold mb-4">📖 使用说明</h3>
      <div class="space-y-4 text-sm">
        <div>
          <h4 class="font-semibold text-primary">1. 生成数据文件</h4>
          <p class="text-muted-foreground">
            选择游戏场次和队列类型，点击"生成数据文件"按钮。系统会收集你的游戏数据并生成 JSON 文件。
          </p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">2. 分析数据</h4>
          <p class="text-muted-foreground">
            输入生成的数据文件名，点击"分析数据"按钮查看统计摘要。
          </p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">3. 分享数据</h4>
          <p class="text-muted-foreground">
            将生成的数据文件分享给开发者，用于优化算法和调整阈值。
          </p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">4. 队列说明</h4>
          <ul class="text-muted-foreground list-disc list-inside space-y-1">
            <li>单双排 (420): 排位赛单排/双排</li>
            <li>灵活组排 (440): 排位赛灵活组排</li>
            <li>大乱斗 (450): 极地大乱斗</li>
            <li>排位 (700): 其他排位模式</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/shared/components/ui/button'
import { Input } from '@/shared/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/components/ui/select'
import { Alert, AlertDescription, AlertTitle } from '@/shared/components/ui/alert'
import { CheckCircle, AlertCircle, Loader2, BarChart3 } from 'lucide-vue-next'

// 响应式数据
const gameCount = ref(20)
const selectedQueue = ref('all')
const isGenerating = ref(false)
const isAnalyzing = ref(false)
const generationResult = ref('')
const errorMessage = ref('')
const dataFilePath = ref('')
const analysisResult = ref('')
const analysisError = ref('')

// 生成数据文件
const generateData = async () => {
  isGenerating.value = true
  generationResult.value = ''
  errorMessage.value = ''

  try {
    const queueId = selectedQueue.value === 'all' ? null : parseInt(selectedQueue.value)

    const result = await invoke<string>('generate_test_data_file', {
      count: gameCount.value,
      queueId: queueId,
      includeSummary: true
    })

    generationResult.value = result
    console.log('数据生成成功:', result)
  } catch (error: any) {
    errorMessage.value = error.message || '生成数据文件失败'
    console.error('数据生成失败:', error)
  } finally {
    isGenerating.value = false
  }
}

// 分析数据文件
const analyzeData = async () => {
  if (!dataFilePath.value.trim()) {
    analysisError.value = '请输入数据文件路径'
    return
  }

  isAnalyzing.value = true
  analysisResult.value = ''
  analysisError.value = ''

  try {
    const result = await invoke<string>('analyze_data_file', {
      filePath: dataFilePath.value.trim()
    })

    analysisResult.value = result
    console.log('数据分析成功:', result)
  } catch (error: any) {
    analysisError.value = error.message || '分析数据文件失败'
    console.error('数据分析失败:', error)
  } finally {
    isAnalyzing.value = false
  }
}
</script>
