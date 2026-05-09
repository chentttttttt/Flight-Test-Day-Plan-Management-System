// src/api/user.ts
import request from '@/utils/request'
import type {
    LoginParams,
    LoginResult,
    UserInfo,
    CreateUserParams,
    UpdateUserParams,
    QueryUserParams,
    UserListResult
} from '@/types/user'

// 登录
export function login(data: LoginParams): Promise<LoginResult> {
    return request({
        url: '/user/login',
        method: 'post',
        data
    })
}

// 创建用户
export function createUser(data: CreateUserParams): Promise<UserInfo> {
    return request({
        url: '/user/create',
        method: 'post',
        data
    })
}

// 更新用户
export function updateUser(data: UpdateUserParams): Promise<UserInfo> {
    return request({
        url: '/user/update',
        method: 'put',
        data
    })
}

export function updateUserSelf(data: UpdateUserParams): Promise<UserInfo> {
    return request({
        url: '/user/update-self',
        method: 'put',
        data
    })
}

// 获取当前登录用户信息（用于个人中心刷新）
export function getCurrentUserInfo(): Promise<UserInfo> {
    return request({
        url: '/user/info',
        method: 'get'
    })
}


// 分页查询用户列表
export function listUsers(data: QueryUserParams): Promise<UserListResult> {
    return request({
        url: '/user/list',
        method: 'post',
        data
    })
}

// 删除用户
export function deleteUser(id: number, updateBy: number): Promise<boolean> {
    return request({
        url: `/user/delete/${id}`,
        method: 'delete',
        params: { update_by: updateBy }
    })
}

// 获取用户详情
export function getUser(id: number): Promise<UserInfo> {
    return request({
        url: `/user/${id}`,
        method: 'get'
    })
}
