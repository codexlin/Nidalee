export interface LatestRequestTicket {
  isCurrent: () => boolean
  invalidate: () => void
}

/**
 * 为同一状态所有者签发递增请求票据，只有最后一张票据可以提交结果。
 */
export function createLatestRequestGuard() {
  let revision = 0

  return {
    begin(): LatestRequestTicket {
      const ticketRevision = ++revision

      return {
        isCurrent: () => ticketRevision === revision,
        invalidate: () => {
          if (ticketRevision === revision) revision += 1
        }
      }
    },
    invalidate() {
      revision += 1
    }
  }
}
