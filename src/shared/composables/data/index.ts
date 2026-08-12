/**
 * 数据查询层
 *
 * 提供统一的数据查询接口，使用 TanStack Query 进行缓存管理
 * 静态数据使用版本号作为缓存 key，避免不必要的请求
 */

export * from './useVersionedData'
