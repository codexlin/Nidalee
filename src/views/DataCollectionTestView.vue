<template>
  <div class="p-6 space-y-6">
    <div class="bg-card rounded-lg p-6">
      <h2 class="text-2xl font-bold mb-4">🔬 数据收集测试工具</h2>
      <p class="text-muted-foreground mb-6">
        用于收集原始LCU数据和分析数据文件，帮助优化算法。支持时间线分析和队列差异化分析。
      </p>

      <!-- 原始数据收集 -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold mb-3">📊 原始LCU数据收集</h3>
        <div class="grid grid-cols-1 md:grid-cols-5 gap-4 mb-4">
          <div>
            <label class="block text-sm font-medium mb-2">游戏场次</label>
            <Input v-model="rawGameCount" type="number" placeholder="50" min="1" max="100" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-2">队列ID</label>
            <Select v-model="rawSelectedQueue">
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
            <Button @click="collectRawData" :disabled="isCollectingRaw" class="w-full">
              <Loader2 v-if="isCollectingRaw" class="w-4 h-4 mr-2 animate-spin" />
              {{ isCollectingRaw ? '收集中...' : '收集原始数据' }}
            </Button>
          </div>
          <div class="flex items-end">
            <Button @click="analyzeRawData" :disabled="isAnalyzingRaw" class="w-full" variant="outline">
              <Loader2 v-if="isAnalyzingRaw" class="w-4 h-4 mr-2 animate-spin" />
              {{ isAnalyzingRaw ? '分析中...' : '分析时间线' }}
            </Button>
          </div>
          <div class="flex items-end">
            <Button @click="analyzeThresholds" :disabled="isAnalyzingThresholds" class="w-full" variant="secondary">
              <Loader2 v-if="isAnalyzingThresholds" class="w-4 h-4 mr-2 animate-spin" />
              {{ isAnalyzingThresholds ? '分析中...' : '阈值分析' }}
            </Button>
          </div>
        </div>
        <div class="mb-4">
          <label class="block text-sm font-medium mb-2">原始数据文件名</label>
          <Input v-model="rawDataFilePath" placeholder="raw_match_data_20251021_135716.json" />
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          <div>
            <label class="block text-sm font-medium mb-2">对局索引 (查看JSON结构)</label>
            <Input v-model="jsonStructureIndex" type="number" placeholder="0" min="0" />
          </div>
          <div class="flex items-end">
            <Button @click="showJsonStructure" :disabled="isShowingJson" class="w-full" variant="secondary">
              <Loader2 v-if="isShowingJson" class="w-4 h-4 mr-2 animate-spin" />
              {{ isShowingJson ? '加载中...' : '查看JSON结构' }}
            </Button>
          </div>
        </div>
      </div>

      <!-- 分析数据生成 -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold mb-3">📈 分析数据生成</h3>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          <div>
            <label class="block text-sm font-medium mb-2">游戏场次</label>
            <Input v-model="gameCount" type="number" placeholder="20" min="1" max="100" />
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
              {{ isGenerating ? '生成中...' : '生成分析数据' }}
            </Button>
          </div>
        </div>
      </div>

      <!-- 原始数据收集结果 -->
      <div v-if="rawCollectionResult" class="mb-6">
        <Alert>
          <CheckCircle class="h-4 w-4" />
          <AlertTitle>原始数据收集成功</AlertTitle>
          <AlertDescription>
            {{ rawCollectionResult }}
          </AlertDescription>
        </Alert>
      </div>

      <!-- 原始数据收集错误 -->
      <div v-if="rawCollectionError" class="mb-6">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>原始数据收集失败</AlertTitle>
          <AlertDescription>
            {{ rawCollectionError }}
          </AlertDescription>
        </Alert>
      </div>

      <!-- 原始数据分析结果 -->
      <div v-if="rawAnalysisResult" class="mb-6">
        <Alert>
          <BarChart3 class="h-4 w-4" />
          <AlertTitle>原始数据分析结果</AlertTitle>
          <AlertDescription>
            <pre class="mt-2 text-sm whitespace-pre-wrap">{{ rawAnalysisResult }}</pre>
          </AlertDescription>
        </Alert>
      </div>

      <!-- 原始数据分析错误 -->
      <div v-if="rawAnalysisError" class="mb-6">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>原始数据分析失败</AlertTitle>
          <AlertDescription>
            {{ rawAnalysisError }}
          </AlertDescription>
        </Alert>
      </div>

      <!-- JSON结构查看结果 -->
      <div v-if="jsonStructureResult" class="mb-6">
        <Alert>
          <BarChart3 class="h-4 w-4" />
          <AlertTitle>JSON结构分析结果</AlertTitle>
          <AlertDescription>
            <pre class="mt-2 text-sm whitespace-pre-wrap">{{ jsonStructureResult }}</pre>
          </AlertDescription>
        </Alert>
      </div>

      <!-- JSON结构查看错误 -->
      <div v-if="jsonStructureError" class="mb-6">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>JSON结构查看失败</AlertTitle>
          <AlertDescription>
            {{ jsonStructureError }}
          </AlertDescription>
        </Alert>
      </div>

      <!-- 阈值分析结果 -->
      <div v-if="thresholdAnalysisResult" class="mb-6">
        <Alert>
          <BarChart3 class="h-4 w-4" />
          <AlertTitle>阈值分析结果</AlertTitle>
          <AlertDescription>
            <pre class="mt-2 text-sm whitespace-pre-wrap">{{ thresholdAnalysisResult }}</pre>
          </AlertDescription>
        </Alert>
      </div>

      <!-- 阈值分析错误 -->
      <div v-if="thresholdAnalysisError" class="mb-6">
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>阈值分析失败</AlertTitle>
          <AlertDescription>
            {{ thresholdAnalysisError }}
          </AlertDescription>
        </Alert>
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
          <Input v-model="dataFilePath" placeholder="analysis_data_20241201_120000.json" />
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
          <h4 class="font-semibold text-primary">1. 收集原始LCU数据</h4>
          <p class="text-muted-foreground">
            从LCU API获取完整的原始对局数据，包含参与者信息、队伍信息、时间线等。这是进行深度分析的基础。
          </p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">2. 分析数据</h4>
          <p class="text-muted-foreground">输入生成的数据文件名，点击"分析数据"按钮查看统计摘要。</p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">3. 分享数据</h4>
          <p class="text-muted-foreground">将生成的数据文件分享给开发者，用于优化算法和调整阈值。</p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">4. 分享数据</h4>
          <p class="text-muted-foreground">将生成的原始数据文件分享给开发者，用于优化算法、调整阈值和改进位置识别。</p>
        </div>
        <div>
          <h4 class="font-semibold text-primary">5. 队列说明</h4>
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

// 原始数据收集相关
const rawGameCount = ref(50)
const rawSelectedQueue = ref('all')
const isCollectingRaw = ref(false)
const isAnalyzingRaw = ref(false)
const rawDataFilePath = ref('')
const rawCollectionResult = ref('')
const rawAnalysisResult = ref('')
const rawCollectionError = ref('')
const rawAnalysisError = ref('')

// JSON结构查看相关
const jsonStructureIndex = ref(0)
const isShowingJson = ref(false)
const jsonStructureResult = ref('')
const jsonStructureError = ref('')

// 阈值分析相关
const isAnalyzingThresholds = ref(false)
const thresholdAnalysisResult = ref('')
const thresholdAnalysisError = ref('')

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

// 收集原始LCU数据
const collectRawData = async () => {
  isCollectingRaw.value = true
  rawCollectionResult.value = ''
  rawCollectionError.value = ''

  try {
    const queueId = rawSelectedQueue.value === 'all' ? null : parseInt(rawSelectedQueue.value)

    const result = await invoke<string>('collect_raw_match_data', {
      count: rawGameCount.value,
      queueId: queueId,
      includeTimeline: true
    })

    rawCollectionResult.value = result
    console.log('原始数据收集成功:', result)
  } catch (error: any) {
    rawCollectionError.value = error.message || '收集原始数据失败'
    console.error('原始数据收集失败:', error)
  } finally {
    isCollectingRaw.value = false
  }
}

// 分析原始数据时间线
const analyzeRawData = async () => {
  if (!rawDataFilePath.value.trim()) {
    rawAnalysisError.value = '请输入原始数据文件路径'
    return
  }

  isAnalyzingRaw.value = true
  rawAnalysisResult.value = ''
  rawAnalysisError.value = ''

  try {
    const result = await invoke<string>('analyze_raw_match_timeline', {
      filePath: rawDataFilePath.value.trim()
    })

    rawAnalysisResult.value = result
    console.log('原始数据分析成功:', result)
  } catch (error: any) {
    rawAnalysisError.value = error.message || '分析原始数据失败'
    console.error('原始数据分析失败:', error)
  } finally {
    isAnalyzingRaw.value = false
  }
}

// 查看JSON结构
const showJsonStructure = async () => {
  if (!rawDataFilePath.value.trim()) {
    jsonStructureError.value = '请输入原始数据文件路径'
    return
  }

  isShowingJson.value = true
  jsonStructureResult.value = ''
  jsonStructureError.value = ''

  try {
    const result = await invoke<string>('show_raw_json_structure', {
      filePath: rawDataFilePath.value.trim(),
      matchIndex: jsonStructureIndex.value
    })

    jsonStructureResult.value = result
    console.log('JSON结构查看成功:', result)
  } catch (error: any) {
    jsonStructureError.value = error.message || '查看JSON结构失败'
    console.error('JSON结构查看失败:', error)
  } finally {
    isShowingJson.value = false
  }
}

// 阈值分析
const analyzeThresholds = async () => {
  if (!rawDataFilePath.value.trim()) {
    thresholdAnalysisError.value = '请输入原始数据文件路径'
    return
  }

  isAnalyzingThresholds.value = true
  thresholdAnalysisResult.value = ''
  thresholdAnalysisError.value = ''

  try {
    const result = await invoke<string>('analyze_thresholds_from_raw_data', {
      filePath: rawDataFilePath.value.trim()
    })

    thresholdAnalysisResult.value = result
    console.log('阈值分析成功:', result)
  } catch (error: any) {
    thresholdAnalysisError.value = error.message || '阈值分析失败'
    console.error('阈值分析失败:', error)
  } finally {
    isAnalyzingThresholds.value = false
  }
}
</script>
