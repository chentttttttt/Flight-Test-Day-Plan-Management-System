import request from '@/utils/request';
import type {
    AircraftModel, Subject, Aircraft, AircraftDetail,
    CreateModelDto, UpdateModelDto,
    CreateSubjectDto, UpdateSubjectDto,
    CreateAircraftDto, UpdateAircraftDto,
    StartFlightDto, EndFlightDto,
    Pilot,
    CreatePilotDto,
    UpdatePilotDto,
    PilotQueryParams,
    PageResponse,
} from '@/types/aircraft';
export function finishFlight(data: {
    aircraft_id: number;
    subject_id: number;
    flight_hours?: number;
    operation_hours?: number;
    completed?: boolean;
}): Promise<void> {
    return request({ url: '/aircraft/finish', method: 'post', data });
}
// ========== 机型 ==========
export function createModel(data: CreateModelDto): Promise<AircraftModel> {
    return request({ url: '/aircraft/model', method: 'post', data });
}
export function updateModel(id: number, data: UpdateModelDto): Promise<AircraftModel> {
    return request({ url: `/aircraft/model/${id}`, method: 'put', data });
}
export function deleteModel(id: number): Promise<void> {
    return request({ url: `/aircraft/model/${id}`, method: 'delete' });
}
export function listModels(): Promise<AircraftModel[]> {
    return request({ url: '/aircraft/models', method: 'get' });
}

// ========== 科目 ==========
export function createSubject(data: CreateSubjectDto): Promise<Subject> {
    return request({ url: '/aircraft/subject', method: 'post', data });
}
export function updateSubject(id: number, data: UpdateSubjectDto): Promise<Subject> {
    return request({ url: `/aircraft/subject/${id}`, method: 'put', data });
}
export function deleteSubject(id: number): Promise<void> {
    return request({ url: `/aircraft/subject/${id}`, method: 'delete' });
}
export function listSubjects(): Promise<Subject[]> {
    return request({ url: '/aircraft/subjects', method: 'get' });
}

// ========== 飞机 ==========
export function createAircraft(data: CreateAircraftDto): Promise<Aircraft> {
    return request({ url: '/aircraft', method: 'post', data });
}
export function updateAircraft(id: number, data: UpdateAircraftDto): Promise<Aircraft> {
    return request({ url: `/aircraft/${id}`, method: 'put', data });
}
export function deleteAircraft(id: number): Promise<void> {
    return request({ url: `/aircraft/${id}`, method: 'delete' });
}
export function listAircraft(): Promise<Aircraft[]> {
    return request({ url: '/aircraft/list', method: 'get' });
}
export function getAircraftDetail(id: number): Promise<AircraftDetail> {
    return request({ url: `/aircraft/detail/${id}`, method: 'get' });
}

// ========== 试飞记录 ==========
export function startFlight(data: StartFlightDto): Promise<void> {
    return request({ url: '/aircraft/start', method: 'post', data });
}
export function endFlight(data: EndFlightDto): Promise<void> {
    return request({ url: '/aircraft/end', method: 'post', data });
}
export function getIncompleteSubjects(aircraftId: number): Promise<Subject[]> {
    return request({ url: `/aircraft/incomplete/${aircraftId}`, method: 'get' });
}

// 飞行员列表（支持分页筛选）
export function getPilots(params: any): Promise<{
    data: Pilot[]; list: Pilot[]; total: number
}> {
    return request({ url: '/aircraft/pilots', method: 'get', params })
}

// 创建飞行员
export function createPilot(data: any): Promise<Pilot> {
    return request({ url: '/aircraft/pilot', method: 'post', data })
}

// 更新飞行员
export function updatePilot(id: number, data: any): Promise<Pilot> {
    return request({ url: `/aircraft/pilot/${id}`, method: 'put', data })
}

// 删除飞行员
export function deletePilot(id: number): Promise<void> {
    return request({ url: `/aircraft/pilot/${id}`, method: 'delete' })
}

// 获取飞行员的已分配机型ID列表
export function getPilotModels(pilotId: number): Promise<{ id: number; name: string }[]> {
    return request({ url: `/aircraft/pilot/${pilotId}/models`, method: 'get' })
}

// 分配可飞机型（传入机型ID数组）
export function setPilotModels(pilotId: number, modelIds: number[]): Promise<void> {
    return request({ url: `/aircraft/pilot/${pilotId}/models`, method: 'post', data: { model_ids: modelIds } })
}