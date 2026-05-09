// 飞行员信息
export interface PilotInfo {
    id: number;
    health_status: string;
    mental_status: string;
    level: string;
    max_daily_flight_hours?: number; // 可选，不传则后端用默认值
    allowed_aircraft_models: string[]; // 可飞机型名称列表
}

// 飞机信息
export interface AircraftInfo {
    id: number;
    model: string;      // 机型名称，如“歼-10”
    plane_no: string;   // 编号，仅用于显示
}

// 任务信息
export interface TaskInfo {
    id: string;                 // 唯一标识，如“7_雷达测试”
    aircraft_id: number;        // 绑定的飞机ID
    duration: number;           // 时长（小时）
    weight: number;             // 优先级权重，默认1
    risk: number;               // 1=高危，0=普通
    required_pilots: number;    // 该任务需要的飞行员数量
}

// 约束条件
export interface MilpConstraint {
    enable_work_time: boolean;
    work_start: number;     // 小时
    work_end: number;       // 小时
    check_health: boolean;
    check_mental: boolean;
    match_aircraft_type: boolean;
    high_risk_need_level1: boolean;
    max_duration: number;           // 单任务最大时长
    max_daily_flight_hours: number; // 飞行员日最大飞行时间
}

// MILP 请求
export interface MilpRequest {
    pilots: PilotInfo[];
    aircrafts: AircraftInfo[];
    tasks: TaskInfo[];
    constraints: MilpConstraint;
}

// MILP 响应保持不变...