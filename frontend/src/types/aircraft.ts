// 飞行员
export interface Pilot {
    id: number;
    user_id: number;
    username: string;          // 来自关联用户
    code?: string;
    gender?: string;
    age?: number;
    health_status?: string;
    mental_status?: string;
    level?: string;
    total_flight_hours?: number;
    flight_hours_by_type?: string;
    break_hours_by_type?: string;
    remark?: string;
    status: number;
    create_time?: string;
    update_time?: string;
}


// 飞行员创建/更新 DTO
export interface CreatePilotDto {
    user_id: number;
    code?: string;
    gender?: string;
    age?: number;
    health_status?: string;
    mental_status?: string;
    level?: string;
    model_ids?: number[];
    remark?: string;
    status?: number;
}

export type UpdatePilotDto = Partial<CreatePilotDto>;

// 分页查询参数
export interface PilotQueryParams {
    page?: number;
    page_size?: number;
    keyword?: string;
    level?: string;
    status?: number;
}

// 分页响应
export interface PageResponse<T> {
    list: T[];
    total: number;
}

// 机型
export interface AircraftModel {
    id: number;
    name: string;
    code?: string;
    description?: string;
    status: number; // 1启用 0禁用
    create_time: string;
    update_time: string;
}

// 科目
export interface Subject {
    id: number;
    name: string;
    code?: string;
    danger_level: string; // 普通 / 高危
    default_duration: number; // 小数
    subject_type?: string;
    description?: string;
    status: number;
    create_time: string;
    update_time: string;
}

// 飞机
export interface Aircraft {
    id: number;
    model_id: number;
    plane_no: string;
    status: string; // '测试中' | '完成'

    total_running_hours?: number;

    remark?: string;
    extra_attrs?: Record<string, any>;
    create_time: string;
    update_time: string;
}

// 飞机详细信息（含机型名称、科目及完成状态）
export interface AircraftDetail {
    aircraft: Aircraft;
    model?: AircraftModel;
    subjects: Array<{
        subject: Subject;
        completed: boolean;
        start_time?: string;
        end_time?: string;
        operation_hours?: number;
    }>;
}

// 创建/更新机型 DTO
export interface CreateModelDto {
    name: string;
    code?: string;
    description?: string;
    status?: number;
}
export type UpdateModelDto = Partial<CreateModelDto>;

// 创建/更新科目 DTO
export interface CreateSubjectDto {
    name: string;
    code?: string;
    danger_level: string;
    default_duration: number;
    subject_type?: string;
    description?: string;
    status?: number;
    required_pilots?: number;
}
export type UpdateSubjectDto = Partial<CreateSubjectDto>;

// 创建/更新飞机 DTO
export interface CreateAircraftDto {
    model_id: number;
    plane_no: string;
    status?: string;
    remark?: string;
    extra_attrs?: Record<string, any>;
}
export type UpdateAircraftDto = Partial<CreateAircraftDto>;

// 试飞记录 DTO
export interface StartFlightDto {
    aircraft_id: number;
    subject_id: number;
}
export interface EndFlightDto {
    aircraft_id: number;
    subject_id: number;
    flight_hours: number; // 空中飞行小时
}

export interface ModelSubject {
    model_id: number;
    subject_id: number;
}