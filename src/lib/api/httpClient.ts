import { createFetch } from '@vueuse/core'

/** 公共 HTTP 客户端（DDragon / CDragon 等） */
export const useApiFetch = createFetch({
  options: {
    timeout: 10000,
    beforeFetch({ url, options }) {
      console.log(`🚀 正在请求: ${url}`)

      options.headers = {
        ...options.headers,
        'User-Agent': 'Nidalee-LoL-Assistant/1.0',
        Accept: 'application/json'
      }

      return { options }
    },
    afterFetch({ data, response }) {
      console.log(`✅ 请求成功: ${response.url} (${response.status})`)
      return { data }
    },
    onFetchError({ error, response }) {
      console.error(`❌ 请求失败: ${response?.url} (${response?.status})`, error)
      return { error }
    }
  },
  fetchOptions: {
    mode: 'cors'
  }
})

export interface ApiResponse<T> {
  success: boolean
  data: T | null
  error?: string
  version?: string
}
