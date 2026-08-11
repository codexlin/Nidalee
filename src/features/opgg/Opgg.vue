<template>
  <!--
    浏览态：整页锁高 = 视口 − TitleBar(2.5) − TopNav(3) − 页 padding(3) = 8.5rem
    配置卡/强度榜说明行都在此盒内用 flex 分配，避免再估算它们的高度导致「还差一点点」外滚。
  -->
  <div
    class="flex w-full flex-col gap-4"
    :class="showingDetail ? '' : 'h-[calc(100dvh-8.5rem)] overflow-hidden'"
  >
    <Card class="shrink-0 gap-0 py-0">
      <CardContent class="space-y-3 p-4">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <h1 class="text-xl font-medium leading-tight">构建推荐</h1>
            <p class="mt-1 text-sm text-muted-foreground">
              {{ showingDetail ? '方案详情' : '按分路看强度，点英雄看指南' }}
            </p>
          </div>
          <div class="flex gap-0.5 rounded-full surface-inset p-0.5">
            <button
              v-for="p in BUILD_PROVIDERS"
              :key="p.id"
              type="button"
              class="rounded-full px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-40"
              :class="
                providerId === p.id ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground'
              "
              :disabled="!p.available"
              :title="p.hint"
              @click="switchProvider(p.id)"
            >
              {{ p.label }}
            </button>
          </div>
        </div>

        <div class="flex flex-wrap items-center justify-between gap-2 border-t border-border/50 pt-3">
          <OpggConfigPanel
            :config="activeConfig"
            :regions="opggData.regions"
            :modes="opggData.modes"
            :tiers="opggData.tiers"
            :positions="opggData.positions"
            :compact="isHextech"
            @update:config="handleConfigUpdate"
          />
          <div class="flex flex-wrap items-center gap-2">
            <Button size="sm" variant="outline" class="h-9" :disabled="loading" @click="handleRefresh">
              <RefreshCw :class="['mr-1.5 size-3.5', loading && 'animate-spin']" />
              刷新
            </Button>
            <Button
              v-if="showingDetail && !isHextech"
              size="sm"
              class="h-9"
              :disabled="!canApplyRunes"
              @click="handleApplyBestRunes"
            >
              <Wand2 class="mr-1.5 size-3.5" />
              套最佳符文
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>

    <div
      v-if="activeError"
      class="flex shrink-0 items-start gap-2 rounded-xl border border-destructive/40 bg-destructive/5 px-3 py-2 text-sm"
    >
      <AlertCircle class="mt-0.5 size-4 shrink-0 text-destructive" />
      <p class="text-destructive">{{ activeError }}</p>
    </div>

    <Card
      v-if="!showingDetail"
      class="flex min-h-0 flex-1 flex-col gap-0 overflow-hidden py-0"
    >
      <div class="flex min-h-0 flex-1 overflow-hidden">
        <ChampionGrid :selected-id="activeChampionId" @select="onSelectChampion" />
        <TierListPanel
          ref="tierPanelRef"
          embedded
          :tier-list="activeTierList"
          :loading="loading"
          :default-position="opggData.config.value.position"
          :mode="tierMode"
          @select-champion="onSelectFromTier"
          @update:position="onBrowsePosition"
        />
      </div>
    </Card>

    <!-- 详情 -->
    <template v-else>
      <HextechDetailPanel
        v-if="isHextech && hextechData.detail.value"
        :detail="hextechData.detail.value"
        @back="goBack"
      />
      <BuildDetailPanel
        v-else-if="!isHextech && opggData.championBuild.value"
        :build="opggData.championBuild.value"
        :mode="opggData.config.value.mode"
        @back="goBack"
        @apply-runes="handleApplySpecificRunes"
      />
      <Card v-else class="gap-0 py-0">
        <CardContent class="flex h-40 flex-col items-center justify-center gap-1 text-muted-foreground">
          <p class="text-sm">{{ loading ? '正在加载方案…' : '暂无方案数据' }}</p>
          <button type="button" class="text-xs text-primary hover:underline" @click="goBack">返回强度榜</button>
        </CardContent>
      </Card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, RefreshCw, Wand2 } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useOpggData, type OpggConfig } from './composables/useOpggData'
import { useHextechData } from './composables/useHextechData'
import { useOpggRunes } from './composables/useOpggRunes'
import { BUILD_PROVIDERS, providerSupports, type BuildProviderId } from './types/buildProvider'
import { buildRequestPosition, usesLanePosition } from './types/modes'
import TierListPanel from './components/TierListPanel.vue'
import OpggConfigPanel from './components/OpggConfigPanel.vue'
import BuildDetailPanel from './components/BuildDetailPanel.vue'
import HextechDetailPanel from './components/HextechDetailPanel.vue'
import ChampionGrid from './components/ChampionGrid.vue'

const providerId = ref<BuildProviderId>('opgg')
const showingDetail = ref(false)
const tierPanelRef = ref<{ activePosition?: { value: string } } | null>(null)

const isHextech = computed(() => providerId.value === 'hextech')
const browseMode = computed(() => !showingDetail.value)

const opggData = useOpggData({
  tierListEnabled: computed(() => !isHextech.value && browseMode.value),
  buildEnabled: computed(() => !isHextech.value && showingDetail.value)
})
const hextechData = useHextechData({
  tierListEnabled: computed(() => isHextech.value && browseMode.value),
  detailEnabled: computed(() => isHextech.value && showingDetail.value)
})
const opggRunes = useOpggRunes()
const route = useRoute()
const router = useRouter()

const loading = computed(() =>
  isHextech.value ? hextechData.loading.value : opggData.loading.value
)
const activeError = computed(() =>
  isHextech.value ? hextechData.error.value : opggData.error.value
)
const activeTierList = computed(() =>
  isHextech.value ? hextechData.tierListAsOpgg.value : opggData.tierList.value
)
const tierMode = computed(() => (isHextech.value ? 'hextech' : opggData.config.value.mode))
const activeChampionId = computed(() =>
  isHextech.value ? hextechData.championId.value : opggData.config.value.championId
)
const activeConfig = computed(() => {
  if (!isHextech.value) return opggData.config.value
  return {
    ...opggData.config.value,
    championId: hextechData.championId.value,
    mode: 'hextech',
    region: 'cn'
  }
})

const canApplyRunes = computed(() => {
  if (isHextech.value) return false
  return !!opggData.championBuild.value?.perks?.length && !opggData.loading.value
})

const resolveBrowsePosition = () => {
  const fromPanel = tierPanelRef.value?.activePosition?.value
  if (fromPanel && fromPanel !== 'all') return fromPanel
  return opggData.config.value.position || 'MID'
}

const goBack = () => {
  showingDetail.value = false
  if (route.query.championId) {
    const next = { ...route.query }
    delete next.championId
    void router.replace({ query: next })
  }
}

const openDetail = (championId: number, position?: string) => {
  if (isHextech.value) {
    hextechData.selectChampion(championId)
  } else {
    opggData.config.value.championId = championId
    if (position) opggData.config.value.position = position
  }
  showingDetail.value = true
}

const onSelectChampion = (championId: number) => {
  const pos = usesLanePosition(opggData.config.value.mode) ? resolveBrowsePosition() : 'none'
  openDetail(championId, pos)
}

const onSelectFromTier = (championId: number, position: string) => {
  openDetail(championId, position)
}

const onBrowsePosition = (position: string) => {
  if (!isHextech.value) opggData.config.value.position = position
}

const switchProvider = (id: BuildProviderId) => {
  if (providerId.value === id) return
  if (!providerSupports(id, 'tierList') && !showingDetail.value) {
    toast.message('当前数据源暂不支持强度榜')
  }
  providerId.value = id
  showingDetail.value = false
}

const handleConfigUpdate = (newConfig: OpggConfig) => {
  if (isHextech.value) return

  const modeChanged = newConfig.mode !== opggData.config.value.mode
  Object.assign(opggData.config.value, newConfig)

  if (modeChanged) {
    if (!usesLanePosition(newConfig.mode)) {
      opggData.config.value.position = 'MID'
    }
    // 换模式后回到浏览；Query key 变化会自动拉新榜
    if (!showingDetail.value || !newConfig.championId) {
      showingDetail.value = false
    }
  }
}

const handleRefresh = () => {
  if (!providerSupports(providerId.value, 'tierList') && !showingDetail.value) {
    toast.message('当前数据源暂不支持强度榜')
    return
  }
  void (isHextech.value ? hextechData.refreshCurrent() : opggData.refreshCurrent())
}

const handleApplyBestRunes = async () => {
  try {
    await opggRunes.applyBestRunes(opggData.config.value.championId, opggData.config.value)
    if (opggRunes.applySuccess.value) toast.success('已套用最佳符文')
    if (opggRunes.applyError.value) toast.error(opggRunes.applyError.value)
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '应用符文失败')
  }
}

const handleApplySpecificRunes = async (runeIndex: number) => {
  try {
    await opggRunes.applySpecificRunes(runeIndex, opggData.config.value.championId, opggData.config.value)
    if (opggRunes.applySuccess.value) toast.success(`已套用方案 ${runeIndex + 1}`)
    if (opggRunes.applyError.value) toast.error(opggRunes.applyError.value)
  } catch (error) {
    toast.error(error instanceof Error ? error.message : '应用符文失败')
  }
}

const loadChampionFromRoute = (championIdRaw: string | number | (string | null)[] | undefined) => {
  let championId: number | undefined
  if (Array.isArray(championIdRaw)) championId = Number(championIdRaw[0])
  else championId = Number(championIdRaw)
  if (!championId || isNaN(championId)) return
  const pos = buildRequestPosition(opggData.config.value.mode, opggData.config.value.position) || undefined
  openDetail(championId, pos === 'none' ? undefined : pos)
}

watch(
  () => route.query.championId,
  (newId) => {
    if (newId) loadChampionFromRoute(newId)
  },
  { immediate: true }
)

watch(activeError, (msg, prev) => {
  if (msg && msg !== prev) toast.error(msg)
})
</script>
