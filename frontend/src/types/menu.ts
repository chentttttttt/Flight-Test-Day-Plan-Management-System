export enum MenuType {
    DIR = 0,      // 目录
    MENU = 1,     // 菜单
    BUTTON = 2,   // 按钮
}
export interface Menu {
    id: number
    name: string
    parent_id?: number
    order_num: number
    path: string
    query_params?: string
    menu_type: MenuType
    permission?: string
    remark?: string
    create_by?: number
    create_time?: string
    update_by?: number
    update_time?: string
    children?: Menu[]
}

export interface CreateMenuParams {
    name: string
    parent_id?: number
    order_num?: number
    path?: string
    query_params?: string
    menu_type: MenuType
    permission?: string
    remark?: string
}

export interface UpdateMenuParams {
    id: number
    name?: string
    parent_id?: number
    order_num?: number
    path?: string
    query_params?: string
    menu_type?: MenuType
    permission?: string
    remark?: string
}

export interface RoleMenu {
    id: number
    role: string
    menu_id: number
    create_time?: string
    update_time?: string
}

export type GrantMenusParams = number[]