// 通用 API 响应类型
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  message?: string
}

export interface ApiResponseWithPagination<T = any> extends ApiResponse<T[]> {
  pagination?: {
    page: number
    pageSize: number
    total: number
    totalPages: number
  }
}