import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'

export function useGameHelper() {
  const updatingNote = ref(false)
  const updatingRank = ref(false)

  /**
   * 设置召唤师生涯背景皮肤
   * @param skinId 皮肤ID
   * @returns Promise<void>
   */
  async function setSummonerBackgroundSkin(skinId: number, skinName?: string): Promise<void> {
    try {
      await invoke('set_summoner_background_skin', { skinId })
      toast.success(`皮肤${skinName ? `"${skinName}"` : ''}已设置为生涯背景`, {
        description: '生涯背景设置成功完成',
        duration: 3000
      })
    } catch (error) {
      toast.error('设置生涯背景失败', {
        description: String(error),
        duration: 5000
      })
    }
  }

  /**
   * 通用设置召唤师聊天资料（签名/段位信息）
   */
  async function setSummonerChatProfile({
    statusMessage,
    queue,
    tier,
    division
  }: {
    statusMessage?: string
    queue?: string
    tier?: string
    division?: string
  }): Promise<void> {
    const providedAny = !!statusMessage || !!queue || !!tier || !!division
    if (!providedAny) {
      toast.error('无有效内容可修改', { description: '请提供签名或完整段位信息', duration: 4000 })
      return
    }

    const isNote = !!statusMessage && !queue && !tier && !division
    const isRank = !!queue && !!tier && !!division
    const hasPartialRank = (queue || tier || division) && !isRank

    if (hasPartialRank) {
      toast.error('段位信息不完整', { description: '需同时提供 队列/段位/分段', duration: 5000 })
      return
    }

    if (isNote) {
      await setSummonerNote(statusMessage!)
      return
    }
    if (isRank) {
      await setSummonerRank({ queue: queue!, tier: tier!, division: division! })
      return
    }

    // 兜底：既非纯签名也非完整段位（如全部为空已提前 return），直接透传
    try {
      await invoke('set_summoner_chat_profile', { statusMessage, queue, tier, division })
      toast.success('资料修改成功', { duration: 3000 })
    } catch (error) {
      toast.error('资料修改失败', { description: String(error), duration: 5000 })
    }
  }

  /**
   * 仅修改签名
   */
  async function setSummonerNote(statusMessage: string): Promise<void> {
    if (!statusMessage?.trim()) {
      toast.error('签名内容为空', { description: '请输入有效的签名内容', duration: 4000 })
      return
    }
    updatingNote.value = true
    try {
      await invoke('set_summoner_chat_profile', { statusMessage })
      toast.success('签名修改成功', { description: '召唤师签名已更新', duration: 3000 })
    } catch (error) {
      toast.error('签名修改失败', { description: String(error), duration: 5000 })
    } finally {
      updatingNote.value = false
    }
  }

  /**
   * 仅修改段位信息（需全部提供）
   */
  async function setSummonerRank({
    queue,
    tier,
    division
  }: {
    queue: string
    tier: string
    division: string
  }): Promise<void> {
    if (!queue || !tier || !division) {
      toast.error('段位信息不完整', { description: '需同时提供 队列/段位/分段', duration: 5000 })
      return
    }
    updatingRank.value = true
    try {
      await invoke('set_summoner_chat_profile', { queue, tier, division })
      toast.success('段位信息修改成功', { description: '聊天段位已更新', duration: 3000 })
    } catch (error) {
      toast.error('段位信息修改失败', { description: String(error), duration: 5000 })
    } finally {
      updatingRank.value = false
    }
  }

  return {
    setSummonerBackgroundSkin,
    setSummonerChatProfile,
    setSummonerNote,
    setSummonerRank,
    updatingNote,
    updatingRank
  }
}
