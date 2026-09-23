export interface RequestConfig {
  url: string
  method: 'GET' | 'POST'
  data?: unknown
}
