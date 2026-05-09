// src/types/user.ts
// 用户信息实体（后端返回）
export interface UserInfo {
    id: number
    username: string
    real_name?: string
    phone?: string
    email?: string
    role?: string
    status?: number // 0:禁用 1:启用
    create_time?: string
    created_at?: string
    update_time?: string
    updated_at?: string
    deleted?: number
}

// 登录请求参数
export interface LoginParams {
    username: string
    password: string
}

// 登录响应结果（包含user和token）
export interface LoginResult {
    user: UserInfo
    token: string
}

// 创建用户参数
export interface CreateUserParams {
    username: string
    password: string
    real_name?: string
    phone?: string
    email?: string
    status?: number
    role?: string
    // 其他可能的字段
}
// 更新用户参数（部分更新，必须包含id）
export interface UpdateUserParams {
    id: number
    old_password?: string
    username?: string
    password?: string
    real_name?: string
    phone?: string
    email?: string
    status?: number
    role?: string
    update_by?: number
}

// 查询参数（分页+筛选）
export interface QueryUserParams {
    page: number
    page_size: number
    username?: string
    real_name?: string
    role?: string
    status?: number
}

// 列表响应结构
export interface UserListResult {
    list: UserInfo[]
    total: number
}
