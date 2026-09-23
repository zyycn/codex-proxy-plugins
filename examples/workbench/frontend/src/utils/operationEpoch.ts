import type { OperationEpoch } from '../types'
/** 让迟到的异步结果失效，不依赖请求本身是否支持取消。 */
export function createOperationEpoch(): OperationEpoch {
  let epoch = 0
  return {
    current: () => epoch,
    advance: () => ++epoch,
    isCurrent: token => token === epoch,
  }
}
