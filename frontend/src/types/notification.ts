export interface Notification {
    id: number
    user_id: number
    title: string
    content: string
    type: string        // SYSTEM, APPROVAL, TASK...
    attachment_url?: string
    is_read: number     // 0:未读 1:已读
    create_time: string
}
export interface QueryNotificationParams {
    page: number
    page_size: number
    type?: string
    is_read?: number   // 0:未读 1:已读
}

export interface NotificationListResult {
    list: Notification[]
    total: number
}

export interface CreateNotificationParams {
    role?: string       // 广播角色，与 user_id 二选一
    user_id?: number
    title: string
    content: string
    type: string
    attachment_url?: string
}

export interface MarkReadParams {
    notification_ids: number[]
}