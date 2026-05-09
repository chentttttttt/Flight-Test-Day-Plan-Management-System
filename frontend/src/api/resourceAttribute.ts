import request from '@/utils/request'
import type { ResourceAttributeRecord } from '@/types/resourceAttribute'

// 获取所有资源属性记录
export function getAllRecords(): Promise<ResourceAttributeRecord[]> {
    return request({
        url: '/resource-attribute',
        method: 'get',
    })
}

// 按资源类型获取记录
export function getRecordsByType(resourceType: string): Promise<ResourceAttributeRecord[]> {
    return request({
        url: `/resource-attribute/type/${resourceType}`,
        method: 'get',
    })
}

// 获取指定资源的所有属性（JSON 对象）
export function getAttributes(resourceType: string, resourceId: number): Promise<Record<string, string>> {
    return request({
        url: `/resource-attribute/${resourceType}/${resourceId}`,
        method: 'get',
    })
}

// 获取单个属性值
export function getAttribute(resourceType: string, resourceId: number, key: string): Promise<string> {
    return request({
        url: `/resource-attribute/${resourceType}/${resourceId}/${key}`,
        method: 'get',
    })
}

// 设置单个属性（合并）
export function setAttribute(resourceType: string, resourceId: number, key: string, value: string): Promise<void> {
    return request({
        url: `/resource-attribute/${resourceType}/${resourceId}/${key}`,
        method: 'post',
        data: { key, value },
    })
}

// 删除单个属性
export function removeAttribute(resourceType: string, resourceId: number, key: string): Promise<void> {
    return request({
        url: `/resource-attribute/${resourceType}/${resourceId}/${key}`,
        method: 'delete',
    })
}

// 批量设置所有属性（替换整个 JSON）
export function setAllAttributes(resourceType: string, resourceId: number, attributes: Record<string, string>): Promise<void> {
    return request({
        url: '/resource-attribute',
        method: 'post',
        data: {
            resource_type: resourceType,
            resource_id: resourceId,
            attributes,
        },
    })
}

// 删除资源的所有属性（清空 JSON 对象）
export function deleteAllAttributes(resourceType: string, resourceId: number): Promise<void> {
    return request({
        url: `/resource-attribute/${resourceType}/${resourceId}`,
        method: 'delete',
    })
}

// 删除资源（软删除整个记录）
export function deleteResource(resourceType: string, resourceId: number): Promise<void> {
    return request({
        url: `/resource-attribute/res/${resourceType}/${resourceId}`,
        method: 'delete',
    })
}

// 根据资源类型和ID获取完整记录（含数据库ID）
export function findRecord(resourceType: string, resourceId: number): Promise<ResourceAttributeRecord | null> {
    return request({
        url: `/resource-attribute/record/${resourceType}/${resourceId}`,
        method: 'get',
    })
}