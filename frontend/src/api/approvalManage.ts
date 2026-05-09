import request from '@/utils/request'
import type { ApprovalFlow, ApprovalNode, TreeNode } from '@/types/approval'

// 获取树形数据
export function getTree(): Promise<TreeNode[]> {
    return request({
        url: '/approval-manage/tree',
        method: 'get',
    })
}

// 流程管理
export function createFlow(data: { code: string; name: string; description?: string; status?: number }) {
    return request({
        url: '/approval-manage/flow',
        method: 'post',
        data,
    })
}

export function updateFlow(id: number, data: { name?: string; description?: string; status?: number }) {
    return request({
        url: `/approval-manage/flow/${id}`,
        method: 'put',
        data,
    })
}

export function deleteFlow(id: number) {
    return request({
        url: `/approval-manage/flow/${id}`,
        method: 'delete',
    })
}

export function getFlow(id: number): Promise<ApprovalFlow> {
    return request({
        url: `/approval-manage/flow/${id}`,
        method: 'get',
    })
}


export function getActiveFlows(): Promise<ApprovalFlow[]> {
    return request({
        url: '/approval-manage/active',
        method: 'get',
    })
}

// 节点管理
export function createNode(data: {
    flow_id: number
    node_order: number
    node_name: string
    approver_type: string
    approver_value?: string
    approve_strategy?: string
    status?: number
}) {
    return request({
        url: '/approval-manage/node',
        method: 'post',
        data,
    })
}

export function updateNode(id: number, data: {
    node_name?: string
    approver_type?: string
    approver_value?: string
    approve_strategy?: string
    status?: number
}) {
    return request({
        url: `/approval-manage/node/${id}`,
        method: 'put',
        data,
    })
}

export function deleteNode(id: number) {
    return request({
        url: `/approval-manage/node/${id}`,
        method: 'delete',
    })
}

export function getNode(id: number): Promise<ApprovalNode> {
    return request({
        url: `/approval-manage/node/${id}`,
        method: 'get',
    })
}