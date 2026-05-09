export interface PilotModelFlightTime {
    id: number;
    pilot_id: number;
    model_id: number;
    total_flight_hours: number;
    create_time: string;   // ISO 格式时间字符串
    update_time: string;   // ISO 格式时间字符串
}

export interface CreatePilotModelFlightTimeDto {
    pilot_id: number;
    model_id: number;
    total_flight_hours: number;
}

export interface UpdatePilotModelFlightTimeDto {
    total_flight_hours: number;
}

export interface ListQuery {
    pilot_id?: number;
    model_id?: number;
    page?: number;
    page_size?: number;
}

export interface PaginatedResponse<T> {
    list: T[];
    total: number;
}