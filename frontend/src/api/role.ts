import request from '@/utils/request'

// 角色关联记录（一个角色对应一条记录）
export interface RoleMenu {
    id: number
    role: string
    menu_ids: number[]
    create_time?: string
    update_time?: string
}

// 创建/更新角色时使用的参数（与后端 DTO 一致）
export interface RoleMenuCreateDto {
    role: string
    menu_ids: number[]
}

export interface RoleMenuUpdateDto {
    role?: string
    menu_ids?: number[]
}

// 获取所有角色记录
export function getRoleList(): Promise<RoleMenu[]> {
    return request({
        url: '/role/list',
        method: 'get'
    })
}

// 创建角色（同时指定菜单权限）
export function createRole(data: RoleMenuCreateDto): Promise<RoleMenu> {
    return request({
        url: '/role',
        method: 'post',
        data
    })
}

// 更新角色（可修改角色名称和菜单权限）
export function updateRole(id: number, data: RoleMenuUpdateDto): Promise<RoleMenu> {
    return request({
        url: `/role/${id}`,
        method: 'put',
        data
    })
}

// 删除角色
export function deleteRole(id: number): Promise<void> {
    return request({
        url: `/role/${id}`,
        method: 'delete'
    })
}

// 根据角色名获取菜单ID列表（用于回显）
export function getMenuIdsByRole(role: string): Promise<number[]> {
    return request({
        url: `/role/role/menus/${role}`,
        method: 'get'
    })
}

// 批量授权（添加菜单）—— 注意后端 grant 会创建新记录，如果角色已存在可能会失败，建议使用 replace
export function grantMenus(role: string, menuIds: number[]): Promise<RoleMenu> {
    return request({
        url: `/role/role/grant/${role}`,
        method: 'post',
        data: menuIds
    })
}

// 替换角色的菜单权限（先删除后添加）—— 推荐使用
export function replaceMenus(role: string, menuIds: number[]): Promise<RoleMenu> {
    return request({
        url: `/role/role/replace/${role}`,
        method: 'post',
        data: menuIds
    })
}