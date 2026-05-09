import sys
import json
from pulp import *

def solve_from_json(input_json: str) -> str:
    """
    输入：JSON 字符串，包含问题数据
    输出：JSON 字符串，包含求解结果
    """
    try:
        data = json.loads(input_json)
        result = build_model(data)
        return json.dumps(result, ensure_ascii=False)
    except Exception as e:
        return json.dumps({"status": "ERROR", "message": str(e)})

def build_model(data):
    """构建并求解 MILP 模型，支持动态约束配置"""
    # ---------- 读取基础数据 ----------
    pilots = data['pilots']
    aircrafts = data['aircrafts']
    tasks = data['tasks']
    time_slots = data['time_slots']
    work_start = data['work_start']
    work_end = data['work_end']
    allow = data.get('allow', [])
    health = data.get('health', {})
    mental = data.get('mental', {})
    level = data.get('level', {})
    max_daily = data.get('max_daily', {p: 8 for p in pilots})
    max_duration_per_task = data.get('max_duration_per_task', 4)

    # ---------- 约束配置 ----------
    default_constraints = {
        "pilot_health": False,
        "pilot_mental": False,
        "pilot_level": False,
        "pilot_max_daily": True,
        "task_max_duration": True,
        "cooling_hours": 1,        # 冷却小时数，设为0则禁用冷却
        "required_pilots": True,   # 是否使用多飞行员约束
        "aircraft_binding": True   # 是否强制任务绑定到指定飞机
    }
    constraints = data.get('constraints', {})
    # 合并配置
    cfg = default_constraints.copy()
    cfg.update(constraints)
    cooling = cfg.get("cooling_hours", 1)

    # ---------- 构建 allow 映射 ----------
    allow_map = {(p, a): v for p, a, v in allow}

    # ---------- 任务属性预处理 ----------
    task_info = {}
    for t in tasks:
        tid = t['id']
        # 如果禁用 aircraft_binding，则忽略任务中的 aircraft 字段，所有飞机均可选
        if cfg["aircraft_binding"]:
            a = t.get('aircraft')
            if a is None:
                raise ValueError(f"任务 {tid} 缺少 aircraft 字段")
            if a not in aircrafts:
                raise ValueError(f"任务 {tid} 的飞机 {a} 不在 aircrafts 列表中")
        else:
            a = None   # 表示不绑定，后面变量生成时遍历所有飞机
        task_info[tid] = {
            'aircraft': a,
            'duration': t['duration'],
            'weight': t['weight'],
            'required_pilots': t.get('required_pilots', 1) if cfg["required_pilots"] else 1,
            'risk': t.get('risk', 0)
        }
        if cfg["task_max_duration"] and t['duration'] > max_duration_per_task:
            raise ValueError(f"任务 {tid} 持续时间 {t['duration']} 超过最大允许 {max_duration_per_task}")

    # 按飞机分组任务（仅在 aircraft_binding 启用时使用）
    aircraft_tasks = {a: [] for a in aircrafts} if cfg["aircraft_binding"] else None
    if cfg["aircraft_binding"]:
        for tid, info in task_info.items():
            a = info['aircraft']
            aircraft_tasks[a].append(tid)

    # 有效时间槽
    valid_slots = [h for h in time_slots if work_start <= h <= work_end]

    # ---------- 创建问题 ----------
    prob = LpProblem("DailyFlightSchedule", LpMaximize)

    # ---------- 变量 ----------
    # w[tid, a, h] 或 w[tid, h] 取决于是否绑定飞机
    w = {}
    # x[tid, p, a, h] 或 x[tid, p, h] 取决于是否绑定飞机
    x = {}

    for tid, info in task_info.items():
        dur = info['duration']
        req_pilots = info['required_pilots']
        risk = info['risk']
        # 确定可用的飞机列表
        if cfg["aircraft_binding"]:
            aircraft_list = [info['aircraft']]
        else:
            aircraft_list = aircrafts
        for a in aircraft_list:
            # 飞行员资质检查（可能需要在生成x时检查，但w不需要检查资质）
            for h in valid_slots:
                if h + dur  > work_end:
                    continue
                # w 变量：任务-飞机-时间
                w_key = (tid, a, h)
                w[w_key] = LpVariable(f"w_{tid}_{a}_{h}", 0, 1, LpBinary)
                # x 变量：需要飞行员资质
                for p in pilots:
                    if not allow_map.get((p, a), 0):
                        continue
                    if cfg["pilot_health"] and health.get(p, 0) == 0:
                        continue
                    if cfg["pilot_mental"] and mental.get(p, 0) == 0:
                        continue
                    if cfg["pilot_level"] and risk == 1 and level.get(p, 0) == 0:
                        continue
                    x_key = (tid, p, a, h)
                    x[x_key] = LpVariable(f"x_{tid}_{p}_{a}_{h}", 0, 1, LpBinary)

    # ---------- 目标函数 ----------
    obj_terms = []
    for (tid, a, h), var in w.items():
        weight = task_info[tid]['weight']
        dur = task_info[tid]['duration']
        obj_terms.append(weight * var - 0.01 * dur * var)
    prob += lpSum(obj_terms)

    # ---------- 约束 ----------
    # 1. 每个任务最多执行一次（同一任务不同飞机、不同时间视为互斥）
    for tid in task_info:
        if cfg["aircraft_binding"]:
            prob += lpSum(w.get((tid, task_info[tid]['aircraft'], h), 0) for h in valid_slots) <= 1
        else:
            prob += lpSum(w.get((tid, a, h), 0) for a in aircrafts for h in valid_slots) <= 1

    # 2. x <= w（飞行员指派不能超出任务执行）
    for (tid, p, a, h), var in x.items():
        prob += var <= w[(tid, a, h)]

    # 3. 每个任务指派的飞行员数量等于所需数量（如果任务执行）
    if cfg["required_pilots"]:
        for (tid, a, h), wvar in w.items():
            req = task_info[tid]['required_pilots']
            prob += lpSum(x.get((tid, p, a, h), 0) for p in pilots) == req * wvar

    # 4. 飞机占用 + 冷却
    if cooling > 0:
        for a in aircrafts:
            for s in valid_slots:
                # 找出所有可能占用时刻 s 的任务
                prob += lpSum(
                    w.get((tid, a, h), 0)
                    for tid in (aircraft_tasks[a] if cfg["aircraft_binding"] else task_info.keys())
                    for h in valid_slots
                    if (tid, a, h) in w and h <= s <= h + task_info[tid]['duration'] + cooling - 1
                ) <= 1
    else:
        # 无冷却，只保证同一时刻不重叠
        for a in aircrafts:
            for s in valid_slots:
                prob += lpSum(
                    w.get((tid, a, h), 0)
                    for tid in (aircraft_tasks[a] if cfg["aircraft_binding"] else task_info.keys())
                    for h in valid_slots
                    if (tid, a, h) in w and h <= s < h + task_info[tid]['duration']
                ) <= 1

    # 5. 飞行员占用 + 冷却
    if cooling > 0:
        for p in pilots:
            for s in valid_slots:
                prob += lpSum(
                    x.get((tid, p, a, h), 0)
                    for tid in task_info
                    for a in aircrafts
                    for h in valid_slots
                    if (tid, p, a, h) in x and h <= s <= h + task_info[tid]['duration'] + cooling - 1
                ) <= 1
    else:
        for p in pilots:
            for s in valid_slots:
                prob += lpSum(
                    x.get((tid, p, a, h), 0)
                    for tid in task_info
                    for a in aircrafts
                    for h in valid_slots
                    if (tid, p, a, h) in x and h <= s < h + task_info[tid]['duration']
                ) <= 1

    # 6. 飞行员每日总飞行时长
    if cfg["pilot_max_daily"]:
        for p in pilots:
            prob += lpSum(
                task_info[tid]['duration'] * x.get((tid, p, a, h), 0)
                for tid in task_info
                for a in aircrafts
                for h in valid_slots
            ) <= max_daily.get(p, 8)

    # ---------- 求解 ----------
    prob.solve(PULP_CBC_CMD(msg=False))

    # ---------- 结果处理 ----------
    if prob.status != LpStatusOptimal:
        return {
            "status": LpStatus[prob.status],
            "objective": 0,
            "assignments": [],
            "message": "未找到最优解，可能资源不足。请检查约束或增加飞行员/飞机/时间窗口。"
        }

    assignments = []
    for (tid, a, h), wvar in w.items():
        if wvar.varValue > 0.5:
            # 找到指派给该任务的飞行员
            assigned_pilots = [
                p for p in pilots
                if (tid, p, a, h) in x and x[(tid, p, a, h)].varValue > 0.5
            ]
            info = task_info[tid]
            assignments.append({
                "task_id": tid,
                "pilots": assigned_pilots,
                "aircraft": a,
                "start_hour": h,
                "duration": info['duration'],
                "weight": info['weight'],
            })

    return {
        "status": LpStatus[prob.status],
        "objective": value(prob.objective),
        "assignments": assignments
    }

if __name__ == "__main__":
    input_str = sys.stdin.read()
    output_str = solve_from_json(input_str)
    print(output_str)