import request from '@/utils/request'
import type { Notification, NotificationListResult, CreateNotificationParams, QueryNotificationParams } from '@/types/notification'

// 获取当前用户通知列表（POST + JSON 传参）
export function getNotificationList(params: QueryNotificationParams): Promise<NotificationListResult> {
    return request({
        url: '/notification/list',
        method: 'post',
        data: params,           // JSON 请求体
    })
}

// 管理员获取指定用户的通知列表（POST + JSON 传参）
export function getNotificationListByUser(userId: number, params: QueryNotificationParams): Promise<NotificationListResult> {
    return request({
        url: `/notification/list/${userId}`,
        method: 'post',
        data: params,
    })
}

// 发送通知（不变）
export function createNotification(data: CreateNotificationParams): Promise<Notification[]> {
    return request({
        url: '/notification',
        method: 'post',
        data,
    })
}

// 批量标记已读（不变）
export function markAsRead(notification_ids: number[]): Promise<void> {
    return request({
        url: '/notification/mark-read',
        method: 'post',
        data: { notification_ids },
    })
}

// 删除通知（不变）
export function deleteNotification(id: number): Promise<void> {
    return request({
        url: `/notification/${id}`,
        method: 'delete',
    })
}

// 获取未读数量（不变）
export function getUnreadCount(): Promise<number> {
    return request({
        url: '/notification/unread-count',
        method: 'get',
    })
}