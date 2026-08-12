import { nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { BuildPreset, BuildTarget } from '@/shared/models/buildPreset'

const harness = vi.hoisted(() => ({
  game: null as unknown as {
    currentPhase: string
    champSelectSession: ChampSelectSession | null
  },
  presetStore: null as unknown as {
    autoBuild: { enabled: boolean; opggTier: 'diamond_plus'; showToast: boolean }
    findAutoPreset: ReturnType<typeof vi.fn>
  },
  applyPreset: vi.fn(),
  applyRecommendation: vi.fn(),
  fetchTierList: vi.fn(),
  fetchBuild: vi.fn()
}))

vi.mock('@/shared/stores/features/gameStore', () => ({ useGameStore: () => harness.game }))
vi.mock('@/shared/stores/features/buildPresetStore', () => ({ useBuildPresetStore: () => harness.presetStore }))
vi.mock('@/lib', () => ({ getChampionName: (id: number) => `champion-${id}` }))
vi.mock('@/lib/dataApi', () => ({
  fetchOpggTierList: harness.fetchTierList,
  fetchOpggChampionBuild: harness.fetchBuild
}))
vi.mock('./useBuildApplication', () => ({
  runeSnapshotFromOpgg: (_perk: OpggPerk, context: { target: BuildTarget }) => ({ target: context.target }),
  useBuildApplication: () => ({
    applyPreset: harness.applyPreset,
    applyRecommendation: harness.applyRecommendation
  })
}))

import { useAutoBuild } from './useAutoBuild'

const perk: OpggPerk = {
  primaryId: 8000,
  secondaryId: 8200,
  perks: [8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001],
  win: 10,
  play: 20,
  pickRate: 0.5
}

function session(
  championId = 59,
  queueId = 420,
  assignedPosition: string | null = 'JUNGLE',
  isCustomGame = false
): ChampSelectSession {
  return {
    localPlayerCellId: 0,
    queueId,
    isCustomGame,
    myTeam: [
      {
        cellId: 0,
        puuid: 'local',
        summonerId: '1',
        championId,
        championPickIntent: championId,
        selectedSkinId: 0,
        spell1Id: 4,
        spell2Id: 11,
        assignedPosition
      }
    ],
    theirTeam: [],
    bans: { myTeamBans: [], theirTeamBans: [] },
    timer: { phase: 'FINALIZATION' },
    actions: [
      [
        {
          actorCellId: 0,
          championId,
          completed: true,
          id: 1,
          isAllyAction: true,
          isInProgress: false,
          pickTurn: 1,
          type: 'pick',
          isCurrentUser: true
        }
      ]
    ]
  }
}

function preset(): BuildPreset {
  return {
    id: 'saved',
    name: '皇子打野',
    target: { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' },
    components: {
      runes: {
        primaryStyleId: 8000,
        subStyleId: 8200,
        selectedPerkIds: [...perk.perks]
      }
    },
    source: { kind: 'custom' },
    autoUse: true,
    createdAt: 1,
    updatedAt: 1,
    usageCount: 0
  }
}

function tierList(championId = 59): OpggTierList {
  return {
    meta: { version: '1', region: 'kr', mode: 'ranked', tier: 'diamond_plus' },
    data: [
      {
        championId,
        averageStats: {
          play: 0,
          winRate: 0,
          pickRate: 0,
          banRate: 0,
          kda: 0,
          tier: 0,
          rank: 0,
          firstPlace: null,
          totalPlace: null
        },
        positions: [
          {
            name: 'TOP',
            stats: {
              play: 10,
              winRate: 0,
              pickRate: 0,
              banRate: 0,
              kda: 0,
              tier: 0,
              rank: 0,
              firstPlace: null,
              totalPlace: null
            },
            counters: []
          },
          {
            name: 'JUNGLE',
            stats: {
              play: 90,
              winRate: 0,
              pickRate: 0,
              banRate: 0,
              kda: 0,
              tier: 0,
              rank: 0,
              firstPlace: null,
              totalPlace: null
            },
            counters: []
          }
        ],
        roles: []
      }
    ]
  }
}

async function runDelay(): Promise<void> {
  await nextTick()
  await vi.advanceTimersByTimeAsync(1500)
  await nextTick()
}

describe('useAutoBuild', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    harness.game = reactive({ currentPhase: 'ChampSelect', champSelectSession: session() })
    harness.presetStore = reactive({
      autoBuild: { enabled: true, opggTier: 'diamond_plus' as const, showToast: false },
      findAutoPreset: vi.fn(() => null)
    })
    harness.applyPreset.mockResolvedValue('saved')
    harness.applyRecommendation.mockResolvedValue('recommended')
    harness.fetchTierList.mockResolvedValue({ success: true, data: tierList() })
    harness.fetchBuild.mockResolvedValue({ success: true, data: { perks: [perk] } })
  })

  afterEach(() => vi.useRealTimers())

  it('uses the exact automatic preset before requesting a provider', async () => {
    harness.presetStore.findAutoPreset.mockReturnValue(preset())
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()

    await runDelay()

    expect(harness.presetStore.findAutoPreset).toHaveBeenCalledWith(59, 'ranked-jungle')
    expect(harness.applyPreset).toHaveBeenCalledOnce()
    expect(harness.fetchBuild).not.toHaveBeenCalled()
    autoBuild.stopAutoBuildWatch()
  })

  it('resolves a normal-game recommendation through the champion main position', async () => {
    harness.game.champSelectSession = session(59, 490, null)
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()

    await runDelay()

    expect(harness.fetchTierList).toHaveBeenCalledOnce()
    expect(harness.fetchBuild).toHaveBeenCalledWith(
      expect.objectContaining({ mode: 'ranked', champion_id: 59, position: 'JUNGLE' })
    )
    expect(harness.applyRecommendation).toHaveBeenCalledWith(
      expect.objectContaining({ target: expect.objectContaining({ scenario: 'normal-sr' }) })
    )
    autoBuild.stopAutoBuildWatch()
  })

  it('does not write runes for a custom or unsupported queue', async () => {
    harness.game.champSelectSession = session(59, 420, 'JUNGLE', true)
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()

    await runDelay()

    expect(harness.applyPreset).not.toHaveBeenCalled()
    expect(harness.applyRecommendation).not.toHaveBeenCalled()
    expect(harness.fetchTierList).not.toHaveBeenCalled()
    autoBuild.stopAutoBuildWatch()
  })

  it('drops an old provider response after the locked champion changes', async () => {
    let resolveTierList: ((value: { success: true; data: OpggTierList }) => void) | undefined
    harness.game.champSelectSession = session(59, 490, null)
    harness.fetchTierList.mockReturnValue(
      new Promise((resolve) => {
        resolveTierList = resolve
      })
    )
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()
    await runDelay()
    expect(harness.fetchTierList).toHaveBeenCalledOnce()

    harness.game.champSelectSession = session(60, 490, null)
    await nextTick()
    resolveTierList?.({ success: true, data: tierList(59) })
    await Promise.resolve()
    await nextTick()

    expect(harness.fetchBuild).not.toHaveBeenCalled()
    expect(harness.applyRecommendation).not.toHaveBeenCalled()
    autoBuild.stopAutoBuildWatch()
  })

  it('claims one effective selection while the LCU write is in flight', async () => {
    let finishApply: (() => void) | undefined
    harness.presetStore.findAutoPreset.mockReturnValue(preset())
    harness.applyPreset.mockReturnValue(
      new Promise((resolve) => {
        finishApply = () => resolve('saved')
      })
    )
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()

    await runDelay()
    expect(harness.applyPreset).toHaveBeenCalledOnce()

    harness.presetStore.autoBuild.enabled = false
    await nextTick()
    harness.presetStore.autoBuild.enabled = true
    await runDelay()
    expect(harness.applyPreset).toHaveBeenCalledOnce()

    finishApply?.()
    await Promise.resolve()
    await nextTick()
    expect(harness.applyPreset).toHaveBeenCalledOnce()
    autoBuild.stopAutoBuildWatch()
  })

  it('reapplies after the effective automatic preset changes', async () => {
    const selected = reactive(preset())
    harness.presetStore.findAutoPreset.mockImplementation(() => selected)
    const autoBuild = useAutoBuild()
    autoBuild.startAutoBuildWatch()

    await runDelay()
    expect(harness.applyPreset).toHaveBeenCalledOnce()

    selected.updatedAt += 1
    selected.components.runes.selectedPerkIds = [8008, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001]
    await runDelay()

    expect(harness.applyPreset).toHaveBeenCalledTimes(2)
    autoBuild.stopAutoBuildWatch()
  })
})
