import request from '@/utils/request'
import type {
    Menu,
    CreateMenuParams,
    UpdateMenuParams,
    GrantMenusParams
} from '@/types/menu'

// ---------- 菜单管理 ----------
export function createMenu(data: CreateMenuParams): Promise<Menu> {
    return request({
        url: '/menu',
        method: 'post',
        data
    })
}

export function updateMenu(id: number, data: UpdateMenuParams): Promise<Menu> {
    return request({
        url: `/menu/${id}`,
        method: 'put',
        data
    })
}

export function deleteMenu(id: number): Promise<void> {
    return request({
        url: `/menu/${id}`,
        method: 'delete'
    })
}

export function getMenuTree(): Promise<Menu[]> {
    return request({
        url: '/menu/tree',
        method: 'get'
    })
}

export function getMenuList(): Promise<Menu[]> {
    return request({
        url: '/menu/list',
        method: 'get'
    })
}

// ---------- 角色菜单 ----------
export function getMenuTreeByRole(roleCode: string): Promise<Menu[]> {
    return request({
        url: `/menu/role/tree/${roleCode}`,
        method: 'get'
    })
}

export function getMenuListByRole(roleCode: string): Promise<Menu[]> {
    return request({
        url: `/menu/role/list/${roleCode}`,
        method: 'get'
    })
}

export function getMenuIdsByRole(role: string): Promise<number[]> {
    return request({
        url: `/role-menu/role/menus/${role}`,
        method: 'get'
    })
}

export function grantMenus(role: string, menuIds: GrantMenusParams): Promise<void> {
    return request({
        url: `/role-menu/role/grant/${role}`,
        method: 'post',
        data: menuIds
    })
}

export function replaceMenus(role: string, menuIds: GrantMenusParams): Promise<void> {
    return request({
        url: `/role-menu/role/replace/${role}`,
        method: 'post',
        data: menuIds
    })
}