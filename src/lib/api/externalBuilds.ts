import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from './httpClient'

/**
 * OP.GG / Hextech：走 Tauri IPC，不与 DDragon/CDragon HTTP 混放。
 */

export async function fetchOpggChampionBuildRaw(params: {
  region: string
  mode: string
  champion_id: number
  position?: string
  tier: string
}): Promise<ApiResponse<unknown>> {
  try {
    const data = await invoke<unknown>('get_opgg_champion_build_raw', {
      region: params.region,
      mode: params.mode,
      championId: params.champion_id,
      position: params.position,
      tier: params.tier
    })
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function fetchOpggChampionBuild(params: {
  region: string
  mode: string
  champion_id: number
  position?: string
  tier: string
}): Promise<ApiResponse<OpggChampionBuild>> {
  try {
    const data = await invoke<OpggChampionBuild>('get_opgg_champion_build', {
      region: params.region,
      mode: params.mode,
      championId: params.champion_id,
      position: params.position,
      tier: params.tier
    })
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function fetchOpggTierList(params: {
  region: string
  mode: string
  tier: string
}): Promise<ApiResponse<OpggTierList>> {
  try {
    const data = await invoke<OpggTierList>('get_opgg_tier_list', params)
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function fetchOpggChampionPositions(params: {
  region: string
  champion_id: number
  tier: string
}): Promise<ApiResponse<string[]>> {
  try {
    const data = await invoke<string[]>('get_opgg_champion_positions', {
      region: params.region,
      championId: params.champion_id,
      tier: params.tier
    })
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function fetchHextechTierList(): Promise<ApiResponse<HextechTierList>> {
  try {
    const data = await invoke<HextechTierList>('get_hextech_tier_list')
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function fetchHextechChampionDetail(championId: number): Promise<ApiResponse<HextechChampionDetail>> {
  try {
    const data = await invoke<HextechChampionDetail>('get_hextech_champion_detail', {
      championId
    })
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function applyOpggRunes(params: {
  region: string
  mode: string
  champion_id: number
  position?: string
  tier: string
  build_index?: number
}): Promise<ApiResponse<string>> {
  try {
    const data = await invoke<string>('apply_opgg_runes', {
      region: params.region,
      mode: params.mode,
      championId: params.champion_id,
      champion_name: `Champion_${params.champion_id}`,
      position: params.position,
      tier: params.tier,
      build_index: params.build_index
    })
    return { success: true, data }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}
