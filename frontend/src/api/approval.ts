import request from '@/utils/request'
import type {
    ApprovalOrder,
    ApprovalRecord,
    SubmitApprovalDto,
    ApproveDto,
    RejectDto,
} from '@/types/approval'

// 提交审批申请
export function submitApproval(data: SubmitApprovalDto): Promise<ApprovalOrder> {
    return request({
        url: '/approval/submit',
        method: 'post',
        data,
    })
}

// 审批通过
export function approve(data: ApproveDto): Promise<ApprovalOrder> {
    return request({
        url: '/approval/approve',
        method: 'post',
        data,
    })
}

// 审批驳回
export function reject(data: RejectDto): Promise<ApprovalOrder> {
    return request({
        url: '/approval/reject',
        method: 'post',
        data,
    })
}

// 获取待办列表
export function getTodoList(): Promise<ApprovalOrder[]> {
    return request({
        url: '/approval/todo',
        method: 'get',
    })
}

// 获取我的申请列表（可选状态筛选）
// 获取我的申请列表
export function getMyOrders(params?: { status?: string }): Promise<ApprovalOrder[]> {
    return request({
        url: '/approval/my-orders',
        method: 'get',
        params,
    });
}

// 获取审批记录
export function getApprovalRecords(orderId: number): Promise<ApprovalRecord[]> {
    return request({
        url: `/approval/records/${orderId}`,
        method: 'get',
    })
}