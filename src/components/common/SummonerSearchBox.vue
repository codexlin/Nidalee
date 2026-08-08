<template>
  <Card class="w-full max-w-lg mx-auto p-6">
    <form @submit.prevent="onSearch" class="flex items-center gap-2">
      <NeonBorder :animation-type="'none'">
        <Input
          type="text"
          v-model="summonerName"
          placeholder="输入召唤师名称,支持批量查询,使用','分割"
          class="flex-1 rounded-lg px-4 text-sm"
          @keyup.enter="onSearch"
        />
      </NeonBorder>
      <Button type="submit" :disabled="loading || !summonerName.trim()" class="shrink-0">
        <Search class="w-4 h-4 mr-1" />
        查询
      </Button>
    </form>
  </Card>
</template>

<script setup lang="ts">
import { Search } from 'lucide-vue-next'

withDefaults(defineProps<{ loading?: boolean }>(), {
  loading: false
})
const summonerName = defineModel<string>('summonerName', { default: '' })
const emits = defineEmits(['onSearch'])
const onSearch = () => emits('onSearch')
</script>
