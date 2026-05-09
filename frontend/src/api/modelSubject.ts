import request from '@/utils/request';

const BASE_URL = '/model-subject';

/** 创建机型-科目关联 */
export function createModelSubject(modelId: number, subjectId: number) {
    return request.post(`${BASE_URL}/${modelId}/${subjectId}`);
}

/** 删除单个关联 */
export function deleteModelSubject(modelId: number, subjectId: number) {
    return request.delete(`${BASE_URL}/${modelId}/${subjectId}`);
}

/** 查询关联列表（可按 model_id 或 subject_id 过滤） */
export function listModelSubjects(params?: { model_id?: number; subject_id?: number }) {
    return request.get(BASE_URL, { params });
}

/** 删除某机型下的所有科目关联 */
export function deleteByModel(modelId: number) {
    return request.delete(`${BASE_URL}/model/${modelId}`);
}

/** 删除某科目关联的所有机型 */
export function deleteBySubject(subjectId: number) {
    return request.delete(`${BASE_URL}/subject/${subjectId}`);
}