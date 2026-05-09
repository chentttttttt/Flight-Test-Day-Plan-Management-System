
export interface ResourceAttributeRecord {
    id: number
    resource_id: number
    resource_type: string
    attr_key_value: Record<string, any>
    deleted: number
    create_time?: string
}