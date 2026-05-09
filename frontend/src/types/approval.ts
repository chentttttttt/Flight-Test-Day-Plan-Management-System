// 流程定义
export interface ApprovalFlow {
    id: number
    code: string
    name: string
    description?: string
    status: number // 1启用 0禁用
    create_time: string
    update_time: string
}

// 节点定义
export interface ApprovalNode {
    id: number
    flow_id: number
    node_order: number
    node_name: string
    approver_type: string // ROLE, USER, DEPARTMENT_HEAD
    approver_value?: string
    approve_strategy?: string // ALL, ANY
    status: number
    create_time: string
    update_time: string
}

// 树节点（前端展示用，与后端返回结构一致）
export interface TreeNode {
    flow_id: number
    id: number
    name: string
    type?: 'flow' | 'node' // 可选，根据后端返回判断
    status?: number
    code?: string
    description?: string
    children?: TreeNode[]
    // 节点特有字段
    node_order?: number
    node_name?: string
    approver_type?: string
    approver_value?: string
    approve_strategy?: string
    // 可能还包含原始数据，根据需要
}


// 审批单状态
export type ApprovalStatus = 'PENDING' | 'APPROVED' | 'REJECTED' | 'CANCELLED'

// 审批单
export interface ApprovalOrder {
    id: number
    flow_code: string
    biz_id: number
    biz_type: string
    applicant_id: number
    applicant_name: string
    status: ApprovalStatus
    current_node_order?: number
    current_approver_ids?: string
    attachment_url?: string
    remark?: string
    submitted_time?: string
    finished_time?: string
    create_time: string
    update_time: string
}

// 审批记录
export interface ApprovalRecord {
    id: number
    order_id: number
    node_order: number
    approver_id: number
    approver_name: string
    action: 'APPROVE' | 'REJECT'
    opinion?: string
    attachment_url?: string
    create_time: string
}

// 提交审批申请 DTO
export interface SubmitApprovalDto {
    flow_code: string
    biz_id: number
    biz_type: string
    attachment_url?: string
    remark?: string
}

// 审批操作 DTO
export interface ApproveDto {
    order_id: number
    opinion?: string
    attachment_url?: string
}

export interface RejectDto {
    order_id: number
    opinion?: string
    attachment_url?: string
}