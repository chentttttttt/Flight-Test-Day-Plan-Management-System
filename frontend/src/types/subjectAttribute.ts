// 主体属性记录（后端返回的完整记录）
export interface SubjectAttributeRecord {
    attr_key_value: {}
    id: number
    subject_type: string
    subject_id: number
    //attributes: Record<string, any>
    deleted?: number
    create_time?: string
    update_time?: string
}