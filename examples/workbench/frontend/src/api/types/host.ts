export interface ManagementReply {
  status: number
  contentType: string
  body: ArrayBuffer
}

export interface PluginHost {
  readonly version: 2
  readonly theme: 'light' | 'dark'
  request: (input: {
    method: 'GET' | 'POST'
    path: string
    query?: string
    contentType?: string
    body?: string
  }) => Promise<ManagementReply>
  callbackTicket: (input: { path: string, ttlSeconds: number }) => Promise<string>
  resourceUrl: (path: string) => string
  models: {
    responses: (input: ModelRequestInput) => Promise<Response>
  }
}

export interface ModelRequestInput {
  clientKeyId: string
  body: Record<string, unknown>
  signal?: AbortSignal
}
