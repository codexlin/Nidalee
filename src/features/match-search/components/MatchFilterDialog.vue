<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>战绩过滤</DialogTitle>
        <DialogDescription> 选择要显示的比赛类型 </DialogDescription>
      </DialogHeader>
      <div class="space-y-4">
        <div class="space-y-2">
          <Label>比赛类型</Label>
          <div class="grid grid-cols-2 gap-2">
            <div v-for="queueType in availableQueueTypes" :key="queueType.id" class="flex items-center space-x-2">
              <Checkbox
                :id="String(queueType.id)"
                :checked="selectedQueueTypes.includes(queueType.id)"
                @update:checked="toggleQueueType(queueType.id)"
              />
              <Label :for="String(queueType.id)" class="text-sm">{{ queueType.name }}</Label>
            </div>
          </div>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" @click="reset">重置</Button>
        <Button @click="apply">应用</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { getQueueDisplayName } from '@/common/queueCatalog'

interface Props {
  open: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { selectedQueueTypes, setFilterTypes } = useSearchMatches()

const availableQueueTypes = computed(() =>
  [420, 440, 450, 2400, 490, 400, 700, 1700, 1400, 900, 1020].map((id) => ({
    id,
    name: getQueueDisplayName(id)
  }))
)

const open = computed({
  get: () => props.open,
  set: (value) => emit('update:open', value)
})

const toggleQueueType = (queueTypeId: number) => {
  const index = selectedQueueTypes.value.indexOf(queueTypeId)
  if (index > -1) {
    selectedQueueTypes.value.splice(index, 1)
  } else {
    selectedQueueTypes.value.push(queueTypeId)
  }
}

const reset = () => {
  selectedQueueTypes.value = []
}

const apply = () => {
  setFilterTypes(selectedQueueTypes.value)
  open.value = false
}
</script>
