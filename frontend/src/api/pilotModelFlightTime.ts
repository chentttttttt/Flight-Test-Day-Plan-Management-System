import request from '@/utils/request';
import type {
    PilotModelFlightTime,
    CreatePilotModelFlightTimeDto,
    UpdatePilotModelFlightTimeDto,
    ListQuery,
    PaginatedResponse,
} from '@/types/pilotModelFlightTime';

const BASE_URL = '/pilot-model-flight-time';

/** 新增 */
export function createPilotModelFlightTime(
    data: CreatePilotModelFlightTimeDto
): Promise<PilotModelFlightTime> {
    return request.post(BASE_URL, data);
}

/** 根据 ID 查询 */
export function getPilotModelFlightTimeById(
    id: number
): Promise<PilotModelFlightTime> {
    return request.get(`${BASE_URL}/${id}`);
}

/** 根据 pilot_id + model_id 查询 */
export function getByPilotAndModel(
    pilot_id: number,
    model_id: number
): Promise<PilotModelFlightTime> {
    return request.get(`${BASE_URL}/pilot/${pilot_id}/model/${model_id}`);
}

/** 列表查询（支持分页和过滤） */
export function listPilotModelFlightTimes(
    params: { pilot_id?: number; model_id?: number }
): Promise<{ list: PilotModelFlightTime[] }> {
    return request.get(`${BASE_URL}/list`, { params });
}

/** 更新（根据 ID） */
export function updatePilotModelFlightTime(
    id: number,
    data: UpdatePilotModelFlightTimeDto
): Promise<PilotModelFlightTime> {
    return request.put(`${BASE_URL}/${id}`, data);
}

/** Upsert（存在则累加并更新时间，否则新增） */
export function upsertPilotModelFlightTime(
    data: CreatePilotModelFlightTimeDto
): Promise<PilotModelFlightTime> {
    return request.post(`${BASE_URL}/upsert`, data);
}

/** 根据 ID 删除 */
export function deletePilotModelFlightTime(id: number): Promise<void> {
    return request.delete(`${BASE_URL}/${id}`);
}

/** 根据 pilot_id 批量删除 */
export function deleteByPilot(pilot_id: number): Promise<{ deleted: number }> {
    return request.delete(`${BASE_URL}/pilot/${pilot_id}`);
}