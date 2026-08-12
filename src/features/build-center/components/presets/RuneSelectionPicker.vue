<template>
  <div>
    <div v-if="isLoading" class="flex min-h-72 items-center justify-center gap-2 text-sm text-muted-foreground">
      <Spinner />
      加载符文数据
    </div>

    <Alert v-else-if="error" variant="destructive">
      <AlertCircle />
      <AlertTitle>符文数据加载失败</AlertTitle>
      <AlertDescription>{{ error }}</AlertDescription>
    </Alert>

    <div v-else class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
      <section class="surface-inset flex min-w-0 flex-col gap-4 rounded-xl border p-4">
        <header class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-sm font-semibold">主系 · {{ primaryStyle?.name || '未选择' }}</span>
              <Badge variant="secondary">4 枚</Badge>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">选择符文系，再为每一层选择一枚。</p>
          </div>

          <div class="grid grid-cols-5 gap-1.5" role="radiogroup" aria-label="主系符文">
            <button
              v-for="style in perkStyles"
              :key="style.id"
              type="button"
              role="radio"
              :aria-checked="primaryStyleId === style.id"
              :title="style.name"
              :class="styleButtonClass(primaryStyleId === style.id)"
              @click="selectPrimaryStyle(style.id)"
            >
              <img
                :src="runeData.getStyleIconUrl(style.iconPath)"
                :alt="style.name"
                class="size-7 object-contain"
                @error="handleImageError"
              />
              <span class="truncate text-xs">{{ style.name }}</span>
            </button>
          </div>
        </header>

        <Separator />

        <div v-if="primaryStyle" class="flex flex-col gap-2.5">
          <div
            v-for="(slot, slotIndex) in primaryStyleSlots"
            :key="`${primaryStyle.id}-${slotIndex}`"
            class="grid grid-cols-[64px_minmax(0,1fr)] items-stretch gap-2"
          >
            <div class="flex items-center text-xs text-muted-foreground">
              {{ slot.slotLabel || getSlotLabel(slot.type, slotIndex) }}
            </div>
            <div
              class="grid min-w-0 gap-2"
              :style="{ gridTemplateColumns: `repeat(${slot.perks.length}, minmax(0, 1fr))` }"
            >
              <button
                v-for="perkId in slot.perks"
                :key="perkId"
                type="button"
                :aria-pressed="selectedPrimaryPerks[slotIndex] === perkId"
                :title="getPerkName(perkId)"
                :class="perkButtonClass(selectedPrimaryPerks[slotIndex] === perkId)"
                @click="selectPerk(slotIndex, perkId)"
              >
                <span class="relative">
                  <img
                    :src="getPerkIconUrl(perkId)"
                    :alt="getPerkName(perkId)"
                    :class="slot.type === 'kKeyStone' ? 'size-11' : 'size-9'"
                    class="object-contain"
                    @error="handleImageError"
                  />
                  <Check
                    v-if="selectedPrimaryPerks[slotIndex] === perkId"
                    class="absolute -right-2 -bottom-1 size-3.5 text-primary"
                  />
                </span>
                <span class="w-full truncate text-center text-xs">{{ getPerkName(perkId) }}</span>
              </button>
            </div>
          </div>
        </div>
      </section>

      <div class="grid min-w-0 gap-4 sm:grid-cols-2 xl:grid-cols-1">
        <section class="surface-inset flex min-w-0 flex-col gap-3 rounded-xl border p-4">
          <header class="flex items-center justify-between gap-3">
            <div>
              <div class="flex items-center gap-2">
                <span class="text-sm font-semibold">副系 · {{ subStyle?.name || '未选择' }}</span>
                <Badge variant="secondary">{{ selectedSubPerks.length }} / 2</Badge>
              </div>
              <p class="mt-1 text-xs text-muted-foreground">两枚符文必须来自不同层。</p>
            </div>
          </header>

          <div class="grid grid-cols-4 gap-1.5" role="radiogroup" aria-label="副系符文">
            <button
              v-for="style in availableSubStyles"
              :key="style.id"
              type="button"
              role="radio"
              :aria-checked="subStyleId === style.id"
              :title="style.name"
              :class="styleButtonClass(subStyleId === style.id)"
              @click="selectSubStyle(style.id)"
            >
              <img
                :src="runeData.getStyleIconUrl(style.iconPath)"
                :alt="style.name"
                class="size-7 object-contain"
                @error="handleImageError"
              />
              <span class="truncate text-xs">{{ style.name }}</span>
            </button>
          </div>

          <div v-if="subStyle" class="flex flex-col gap-2">
            <div
              v-for="(slot, slotIndex) in subStyleSlots"
              :key="`${subStyle.id}-${slotIndex}`"
              class="grid grid-cols-[42px_repeat(3,minmax(0,1fr))] items-stretch gap-1.5"
            >
              <span class="text-xs text-muted-foreground">第 {{ slotIndex + 2 }} 层</span>
              <button
                v-for="perkId in slot.perks"
                :key="perkId"
                type="button"
                :disabled="isSubPerkDisabled(slotIndex, perkId)"
                :aria-pressed="selectedSubPerks.includes(perkId)"
                :title="getPerkName(perkId)"
                :class="iconButtonClass(selectedSubPerks.includes(perkId))"
                @click="toggleSubPerk(slotIndex, perkId)"
              >
                <img
                  :src="getPerkIconUrl(perkId)"
                  :alt="getPerkName(perkId)"
                  class="size-9 object-contain"
                  @error="handleImageError"
                />
                <span class="w-full truncate text-center text-xs">{{ getPerkName(perkId) }}</span>
                <Check v-if="selectedSubPerks.includes(perkId)" class="absolute right-1 bottom-1 size-3 text-primary" />
              </button>
            </div>
          </div>
        </section>

        <section class="surface-inset flex min-w-0 flex-col gap-3 rounded-xl border p-4">
          <header>
            <div class="flex items-center gap-2">
              <span class="text-sm font-semibold">属性碎片</span>
              <Badge variant="secondary">{{ selectedStatCount }} / 3</Badge>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">每一行选择一枚属性。</p>
          </header>

          <div class="flex flex-col gap-2">
            <div
              v-for="(slot, slotIndex) in statModSlots"
              :key="`${primaryStyleId}-stat-${slotIndex}`"
              class="grid grid-cols-[42px_repeat(3,minmax(0,1fr))] items-stretch gap-1.5"
            >
              <span class="text-xs text-muted-foreground">{{ getStatModLabel(slotIndex) }}</span>
              <button
                v-for="perkId in slot.perks"
                :key="perkId"
                type="button"
                :aria-pressed="selectedStatMods[slotIndex] === perkId"
                :title="getPerkName(perkId)"
                :class="iconButtonClass(selectedStatMods[slotIndex] === perkId)"
                @click="selectStatMod(slotIndex, perkId)"
              >
                <img
                  :src="getPerkIconUrl(perkId)"
                  :alt="getPerkName(perkId)"
                  class="size-8 object-contain"
                  @error="handleImageError"
                />
                <span class="w-full truncate text-center text-xs">{{ getPerkName(perkId) }}</span>
                <Check
                  v-if="selectedStatMods[slotIndex] === perkId"
                  class="absolute right-1 bottom-1 size-3 text-primary"
                />
              </button>
            </div>
          </div>
        </section>
      </div>

      <div class="flex items-center justify-between gap-3 xl:col-span-2">
        <p class="text-xs text-muted-foreground">主系 4 枚 · 副系 2 枚 · 属性 3 枚</p>
        <Badge :variant="totalSelectedPerks === 9 ? 'default' : 'outline'">
          {{ totalSelectedPerks === 9 ? '配置完整' : `已选 ${totalSelectedPerks} / 9` }}
        </Badge>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, Check } from 'lucide-vue-next'
import { cn } from '@/lib/utils'

interface Props {
  primaryStyleId: number
  subStyleId: number
  selectedPerkIds: number[]
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:primaryStyleId': [value: number]
  'update:subStyleId': [value: number]
  'update:selectedPerkIds': [value: number[]]
}>()

const runeData = useRuneData()
const { perkStyles, perks, isLoading, error } = runeData
const selectedPrimaryPerks = ref<number[]>([])
const selectedSubPerks = ref<number[]>([])
const selectedStatMods = ref<number[]>([])

const primaryStyle = computed(() => perkStyles.value.find((style) => style.id === props.primaryStyleId))
const subStyle = computed(() => perkStyles.value.find((style) => style.id === props.subStyleId))
const availableSubStyles = computed(() => {
  if (!primaryStyle.value) return []
  return perkStyles.value.filter((style) => primaryStyle.value?.allowedSubStyles.includes(style.id))
})
const primaryStyleSlots = computed(() =>
  (primaryStyle.value?.slots ?? []).filter(
    (slot) => slot.type === 'kKeyStone' || slot.type === 'kMixedRegularSplashable'
  )
)
const subStyleSlots = computed(() =>
  (subStyle.value?.slots ?? []).filter((slot) => slot.type === 'kMixedRegularSplashable')
)
const statModSlots = computed(() => (primaryStyle.value?.slots ?? []).filter((slot) => slot.type === 'kStatMod'))
const selectedPrimaryCount = computed(() => selectedPrimaryPerks.value.filter((id) => id > 0).length)
const selectedStatCount = computed(() => selectedStatMods.value.filter((id) => id > 0).length)
const totalSelectedPerks = computed(
  () => selectedPrimaryCount.value + selectedSubPerks.value.length + selectedStatCount.value
)

const styleButtonClass = (selected: boolean) =>
  cn(
    'flex min-w-0 flex-col items-center justify-center gap-1 rounded-lg border px-1.5 py-2 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    selected ? 'border-primary bg-primary/10' : 'border-border bg-background hover:bg-accent'
  )

const perkButtonClass = (selected: boolean) =>
  cn(
    'flex min-h-20 min-w-0 flex-col items-center justify-center gap-1 rounded-lg border px-2 py-1.5 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    selected ? 'border-primary bg-primary/10' : 'border-border bg-background hover:bg-accent'
  )

const iconButtonClass = (selected: boolean) =>
  cn(
    'relative flex min-h-16 min-w-0 flex-col items-center justify-center gap-0.5 rounded-lg border bg-background px-1 py-1.5 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-35',
    selected ? 'border-primary bg-primary/10' : 'border-border hover:bg-accent'
  )

const selectPrimaryStyle = (styleId: number) => {
  if (styleId === props.primaryStyleId) return
  emit('update:primaryStyleId', styleId)
  selectedPrimaryPerks.value = []

  const nextStyle = perkStyles.value.find((style) => style.id === styleId)
  if (nextStyle && !nextStyle.allowedSubStyles.includes(props.subStyleId)) {
    emit('update:subStyleId', nextStyle.allowedSubStyles[0] ?? 8000)
    selectedSubPerks.value = []
  }
  updateSelectedPerkIds()
}

const selectSubStyle = (styleId: number) => {
  if (styleId === props.subStyleId) return
  emit('update:subStyleId', styleId)
  selectedSubPerks.value = []
  updateSelectedPerkIds()
}

const selectPerk = (slotIndex: number, perkId: number) => {
  selectedPrimaryPerks.value[slotIndex] = perkId
  updateSelectedPerkIds()
}

const toggleSubPerk = (slotIndex: number, perkId: number) => {
  const selectedIndex = selectedSubPerks.value.indexOf(perkId)
  if (selectedIndex >= 0) {
    selectedSubPerks.value.splice(selectedIndex, 1)
    updateSelectedPerkIds()
    return
  }

  const sameSlotPerk = selectedSubPerks.value.find((selectedId) =>
    subStyleSlots.value[slotIndex]?.perks.includes(selectedId)
  )
  if (sameSlotPerk !== undefined) {
    selectedSubPerks.value.splice(selectedSubPerks.value.indexOf(sameSlotPerk), 1, perkId)
  } else if (selectedSubPerks.value.length < 2) {
    selectedSubPerks.value.push(perkId)
  }
  updateSelectedPerkIds()
}

const isSubPerkDisabled = (slotIndex: number, perkId: number) => {
  if (selectedSubPerks.value.includes(perkId) || selectedSubPerks.value.length < 2) return false
  return !selectedSubPerks.value.some((selectedId) => subStyleSlots.value[slotIndex]?.perks.includes(selectedId))
}

const selectStatMod = (slotIndex: number, perkId: number) => {
  selectedStatMods.value = selectedStatMods.value.map((selectedId, index) =>
    index === slotIndex ? perkId : selectedId
  )
  updateSelectedPerkIds()
}

const updateSelectedPerkIds = () => {
  emit('update:selectedPerkIds', [
    ...selectedPrimaryPerks.value.filter((id) => id > 0),
    ...selectedSubPerks.value,
    ...selectedStatMods.value.filter((id) => id > 0)
  ])
}

const initializeFromProps = () => {
  const remaining = [...props.selectedPerkIds]
  const consumeSlotPerk = (perkIds: readonly number[]) => {
    const index = remaining.findIndex((perkId) => perkIds.includes(perkId))
    if (index < 0) return 0
    const [perkId] = remaining.splice(index, 1)
    return perkId ?? 0
  }

  selectedPrimaryPerks.value = primaryStyleSlots.value.map((slot) => consumeSlotPerk(slot.perks))
  selectedSubPerks.value = subStyleSlots.value.map((slot) => consumeSlotPerk(slot.perks)).filter((perkId) => perkId > 0)
  selectedStatMods.value = statModSlots.value.map((slot) => consumeSlotPerk(slot.perks))
}

const getPerkIconUrl = (perkId: number) => {
  const perk = perks.value.find((item) => item.id === perkId)
  return perk ? runeData.getPerkIconUrl(perk.iconPath) : ''
}

const getPerkName = (perkId: number) => perks.value.find((item) => item.id === perkId)?.name ?? `符文 ${perkId}`
const getSlotLabel = (type: string, index: number) => (type === 'kKeyStone' ? '基石' : `第 ${index + 1} 层`)
const getStatModLabel = (index: number) => ['进攻', '灵活', '防御'][index] ?? '属性'

const handleImageError = (event: Event) => {
  ;(event.target as HTMLImageElement).style.visibility = 'hidden'
}

onMounted(async () => {
  if (perkStyles.value.length === 0) await runeData.loadRuneData()
  initializeFromProps()
})

watch(
  [() => props.selectedPerkIds, () => props.primaryStyleId, () => props.subStyleId, () => perkStyles.value.length],
  initializeFromProps,
  { deep: true }
)
</script>
