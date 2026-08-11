/**
 * 构建数据源抽象：UI 只依赖 provider 能力，不绑死 OP.GG。
 * 后续 101 等源实现同一套 view 契约即可接入。
 */
export type BuildProviderId = 'opgg' | 'hextech' | 'lol101'

export type BuildWorkspaceView = 'tier' | 'build'

export type BuildProviderCapability =
  | 'tierList'
  | 'championBuild'
  | 'applyRunes'

export interface BuildProviderMeta {
  id: BuildProviderId
  label: string
  /** false 时 UI 可选中展示但禁用加载 */
  available: boolean
  hint?: string
  capabilities: BuildProviderCapability[]
}

export const BUILD_PROVIDERS: BuildProviderMeta[] = [
  {
    id: 'opgg',
    label: 'OP.GG',
    available: true,
    capabilities: ['tierList', 'championBuild', 'applyRunes']
  },
  {
    id: 'hextech',
    label: '海克斯',
    available: true,
    hint: '大乱斗增强 · 国服统计',
    capabilities: ['tierList', 'championBuild']
  },
  {
    id: 'lol101',
    label: '101',
    available: false,
    hint: '即将接入',
    capabilities: ['tierList', 'championBuild']
  }
]

export function getBuildProvider(id: BuildProviderId): BuildProviderMeta {
  return BUILD_PROVIDERS.find((p) => p.id === id) ?? BUILD_PROVIDERS[0]!
}

export function providerSupports(id: BuildProviderId, cap: BuildProviderCapability): boolean {
  const p = getBuildProvider(id)
  return p.available && p.capabilities.includes(cap)
}
