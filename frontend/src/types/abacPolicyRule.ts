// 规则效果枚举
export enum RuleEffect {
    ALLOW = 'ALLOW',
    DENY = 'DENY',
}

// 规则状态
export enum RuleStatus {
    ENABLED = 1,
    DISABLED = 0,
}

// 规则实体（对应后端 Model）
export interface AbacPolicyRule {
    id: number
    rule_code: string
    rule_name: string
    subject_type: string
    subject_id?: number | null
    resource_type: string
    resource_id?: number | null
    action: string
    condition_json: any // JSON 对象
    effect: RuleEffect
    priority: number
    status: RuleStatus
    deleted: number // 0=未删除,1=已删除
    create_time?: string
    update_time?: string
}

// 创建规则请求体
export interface CreateRuleReq {
    rule_code: string
    rule_name: string
    subject_type: string
    subject_id?: number | null
    resource_type: string
    resource_id?: number | null
    action: string
    condition_json: any
    effect: RuleEffect
    priority: number
}

// 更新规则请求体
export interface UpdateRuleReq {
    rule_code?: string
    rule_name?: string
    subject_type?: string
    subject_id?: number | null
    resource_type?: string
    resource_id?: number | null
    action?: string
    condition_json?: any
    effect?: RuleEffect
    priority?: number
    status?: RuleStatus
}

// 查询参数
export interface QueryRulesReq {
    subject_type?: string
    resource_type?: string
    action?: string
    status?: RuleStatus
}