import type { WorkbenchSnapshot } from '../types/workbench'
import request from '../request'
import { parseSnapshot } from '../schemas/workbench'

export function getSnapshot(): Promise<WorkbenchSnapshot> {
  return request({
    url: 'api/snapshot',
    method: 'GET',
  }).then(parseSnapshot)
}
