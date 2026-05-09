import request from '@/utils/request'
import type {
    AbacPolicyRule,
    CreateRuleReq,
    UpdateRuleReq,
    QueryRulesReq,
} from '@/types/abacPolicyRule'

// 创建规则
export function createRule(data: CreateRuleReq): Promise<AbacPolicyRule> {
    return request({
        url: '/rule',
        method: 'post',
        data,
    })
}

// 查询所有规则
export function getAllRules(): Promise<AbacPolicyRule[]> {
    return request({
        url: '/rule/list',
        method: 'get',
    })
}

// 按条件查询规则
export function queryRules(params: QueryRulesReq): Promise<AbacPolicyRule[]> {
    return request({
        url: '/rule/query',
        method: 'get',
        params,
    })
}

// 更新规则
export function updateRule(id: number, data: UpdateRuleReq): Promise<AbacPolicyRule> {
    return request({
        url: `/rule/${id}`,
        method: 'put',
        data,
    })
}

// 删除规则
export function deleteRule(id: number): Promise<void> {
    return request({
        url: `/rule/${id}`,
        method: 'delete',
    })
}