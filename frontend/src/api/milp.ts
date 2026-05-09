import request from '@/utils/request';
import type { MilpRequest, MilpResponse } from '@/types/milp';

const BASE_URL = '/milp';

export function runMilpSchedule(params: MilpRequest): Promise<MilpResponse> {
    return request.post(`${BASE_URL}/schedule`, params);
}