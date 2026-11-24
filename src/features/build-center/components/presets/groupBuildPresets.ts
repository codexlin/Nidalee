import type { BuildPreset } from '@/shared/models/buildPreset'

interface PresetGroup {
  championId: number
  championName: string
  presets: BuildPreset[]
  autoCount: number
}

export function groupBuildPresets(presets: readonly BuildPreset[]): PresetGroup[] {
  const groups = new Map<number, PresetGroup>()
  // Creation order stays stable when a switch updates autoUse and updatedAt.
  const newestFirst = [...presets].sort((a, b) => b.createdAt - a.createdAt || a.id.localeCompare(b.id))
  for (const preset of newestFirst) {
    const { championId, championName } = preset.target
    const group = groups.get(championId) ?? { championId, championName, presets: [], autoCount: 0 }
    group.presets.push(preset)
    if (preset.autoUse) group.autoCount++
    groups.set(championId, group)
  }
  return [...groups.values()]
}
