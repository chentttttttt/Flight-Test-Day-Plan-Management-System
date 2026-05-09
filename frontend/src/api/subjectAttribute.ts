import type { SubjectAttributeRecord } from '@/types/subjectAttribute'
import request from '@/utils/request'



/**
 * 批量设置属性（创建/更新）
 * POST /api/subject-attribute
 */
export function setAttributes(
    subjectType: string,
    subjectId: number,
    attributes: Record<string, any>
): Promise<void> {
    return request({
        url: '/subject-attribute',
        method: 'post',
        data: {
            subject_type: subjectType,
            subject_id: subjectId,
            attributes,
        },
    })
}

/**
 * 获取所有主体属性记录
 * GET /api/subject-attribute
 */
export function getAllRecords(): Promise<SubjectAttributeRecord[]> {
    return request({
        url: '/subject-attribute',
        method: 'get',
    })
}

/**
 * 根据主体类型获取所有记录
 * GET /api/subject-attribute/{subject_type}
 */
export function getRecordsByType(subjectType: string): Promise<SubjectAttributeRecord[]> {
    return request({
        url: `/subject-attribute/${subjectType}`,
        method: 'get',
    })
}

/**
 * 获取指定主体的属性（返回 JSON 对象，不含记录 ID）
 * GET /api/subject-attribute/{subject_type}/{id}
 */
export function getAttributesBySubject(
    subjectType: string,
    subjectId: number
): Promise<Record<string, any>> {
    return request({
        url: `/subject-attribute/${subjectType}/${subjectId}`,
        method: 'get',
    })
}

/**
 * 获取单个属性值
 * GET /api/subject-attribute/{subject_type}/{id}/{key}
 */
export function getAttribute(
    subjectType: string,
    subjectId: number,
    key: string
): Promise<any> {
    return request({
        url: `/subject-attribute/${subjectType}/${subjectId}/${key}`,
        method: 'get',
    })
}

/**
 * 删除单个属性
 * DELETE /api/subject-attribute/{subject_type}/{id}/{key}
 */
export function removeAttribute(
    subjectType: string,
    subjectId: number,
    key: string
): Promise<void> {
    return request({
        url: `/subject-attribute/${subjectType}/${subjectId}/${key}`,
        method: 'delete',
    })
}


/**
 * 删除【主体所有属性】
 * DELETE /api/subject-attribute/{id}
 */
export function deleteAllAttributes(id: number): Promise<void> {
    return request({
        url: `/subject-attribute/${id}`,
        method: 'delete',
    })
}

/**
 * 删除【主体】（根据记录ID软删除）
 * DELETE /api/subject-attribute/attr/{id}
 */
export function deleteSubject(id: number): Promise<void> {
    return request({
        url: `/subject-attribute/attr/${id}`,
        method: 'delete',
    })
}