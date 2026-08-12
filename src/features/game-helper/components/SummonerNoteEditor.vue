<template>
  <div v-if="embedded" class="flex flex-col gap-3 sm:flex-row sm:items-center">
    <Input
      v-model="note"
      placeholder="输入新的召唤师签名…"
      maxlength="60"
      class="h-9 flex-1 text-sm"
      aria-label="召唤师签名"
    />
    <Button class="h-9 shrink-0" :disabled="!note.trim() || updatingNote" @click="handleSave">
      {{ updatingNote ? '保存中…' : '保存签名' }}
    </Button>
  </div>
  <Card v-else class="gap-0 py-0">
    <CardHeader class="gap-1 px-4 py-3 sm:px-5">
      <CardTitle class="flex items-center gap-2 text-base font-medium">
        <MessageSquareText class="size-4 shrink-0 text-muted-foreground" />
        个人签名
      </CardTitle>
      <p class="text-xs text-muted-foreground">自定义你的个性签名</p>
    </CardHeader>
    <CardContent class="px-4 pb-4 sm:px-5">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
        <Input
          v-model="note"
          placeholder="输入新的召唤师签名…"
          maxlength="60"
          class="h-9 flex-1 text-sm"
          aria-label="召唤师签名"
        />
        <Button class="h-9 shrink-0" :disabled="!note.trim() || updatingNote" @click="handleSave">
          {{ updatingNote ? '保存中…' : '保存签名' }}
        </Button>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { MessageSquareText } from 'lucide-vue-next'
import { useGameHelper } from '../composables/useGameHelper'

withDefaults(
  defineProps<{
    embedded?: boolean
  }>(),
  { embedded: false }
)

const note = ref('')
const { setSummonerChatProfile, updatingNote } = useGameHelper()

const handleSave = async () => {
  if (!note.value.trim()) return
  await setSummonerChatProfile({ statusMessage: note.value })
}
</script>
