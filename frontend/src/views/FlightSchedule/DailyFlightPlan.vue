<template>
  <div class="flight-plan-page">
    <el-card class="main-card">
      <template #header>
        <div class="card-header">
          <span class="header-title">试飞日计划编排</span>
          <div class="tool-bar">
            <el-button type="warning" @click="openConstraintPanel">
              <el-icon><Setting /></el-icon> 约束配置
            </el-button>
            <el-button type="primary" @click="autoSchedule">
              <el-icon><Magic /></el-icon> 自动编排
            </el-button>
            <el-button type="success" @click="savePlan">
              <el-icon><Download /></el-icon> 保存计划
            </el-button>
            <el-button @click="clearAll">清空所有</el-button>
          </div>
        </div>
      </template>

      <div class="main-layout">
        <!-- 左侧：飞行员 + 飞机 -->
        <div class="left-fixed-box">
          <div class="left-panel">
            <!-- 飞行员 -->
            <div class="panel-section">
              <div class="panel-header" @click="togglePilotPanel">
                <span>可调配飞行员</span>
                <span class="toggle-icon">{{
                  pilotCollapsed ? "+" : "-"
                }}</span>
              </div>
              <div v-show="!pilotCollapsed" class="panel-body scroll-body">
                <div
                  v-for="pilot in pilotList"
                  :key="pilot.subject_id"
                  class="resource-item pilot"
                  draggable="true"
                  @dragstart="onDragPilot($event, pilot)"
                  @dragend="onDragEnd"
                >
                  <div class="resource-main">
                    <div>
                      {{ pilot.attr.name || `飞行员${pilot.subject_id}` }}
                    </div>
                    <el-button
                      size="mini"
                      type="primary"
                      text
                      @click.stop="showPilotDetail(pilot)"
                      >详情</el-button
                    >
                  </div>
                  <div class="small-info">
                    可飞：{{ pilot.attr.allowed_aircraft || "不限" }}
                  </div>
                  <div class="small-info">
                    身体：{{ pilot.attr.health_status || "正常" }} | 心理：{{
                      pilot.attr.mental_status || "正常"
                    }}
                  </div>
                  <div class="small-info">
                    总飞行：{{ pilot.attr.total_flight_hours || 0 }}h
                  </div>
                </div>
              </div>
            </div>

            <!-- 飞机 -->
            <div class="panel-section">
              <div class="panel-header" @click="togglePlanePanel">
                <span>可调配飞机</span>
                <span class="toggle-icon">{{
                  planeCollapsed ? "+" : "-"
                }}</span>
              </div>
              <div v-show="!planeCollapsed" class="panel-body scroll-body">
                <div
                  v-for="plane in planeList"
                  :key="plane.resource_id"
                  class="resource-item plane"
                  draggable="true"
                  @dragstart="onDragPlane($event, plane)"
                  @dragend="onDragEnd"
                >
                  <div class="resource-main">
                    <div>
                      {{ plane.attr.model || `飞机${plane.resource_id}` }}
                    </div>
                    <el-button
                      size="mini"
                      type="primary"
                      text
                      @click.stop="showPlaneDetail(plane)"
                      >详情</el-button
                    >
                  </div>
                  <div class="small-info">
                    编号：{{ plane.attr.plane_no || "N/A" }}
                  </div>
                  <div class="small-info">
                    科目：{{ plane.attr.subject || "未设置" }}
                  </div>
                  <div class="small-info">
                    危险等级：{{ plane.attr.danger_level || "普通" }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：时间表 -->
        <div class="right-scroll-box">
          <div class="right-panel">
            <div class="schedule-container">
              <div class="time-header">
                <div class="plane-label"></div>
                <div
                  v-for="h in 24"
                  :key="h"
                  class="hour-cell"
                  :style="{ width: columnWidth + 'px' }"
                >
                  {{ (h - 1).toString().padStart(2, "0") }}:00
                </div>
              </div>
              <div class="schedule-body">
                <div
                  v-for="(planeRow, idx) in planeRows"
                  :key="idx"
                  class="plane-row"
                >
                  <div class="plane-label">
                    {{
                      planeRow.plane.attr.model ||
                      `飞机${planeRow.plane.resource_id}`
                    }}
                  </div>
                  <div
                    class="timeline"
                    :style="{ width: 24 * columnWidth + 'px' }"
                  >
                    <div
                      v-for="h in 24"
                      :key="h"
                      class="hour-slot"
                      :style="{ width: columnWidth + 'px' }"
                      @dragover.prevent="onDragOver"
                      @dragleave="onDragLeave"
                      @drop="(e) => onDrop(e, planeRow, h - 1)"
                    ></div>
                    <div
                      v-for="task in planeRow.tasks"
                      :key="task.id"
                      class="task-card"
                      :class="{ conflict: task.conflict }"
                      :style="getTaskStyle(task)"
                      @click="editTask(task)"
                      @dragover.prevent
                      @drop="(e) => onDropOnTask(e, task, planeRow)"
                    >
                      <div class="task-title">
                        {{ task.pilotNames.join(", ") }}
                      </div>
                      <div class="task-time">
                        {{ task.start }} ~ {{ task.end }}
                      </div>
                      <div class="task-duration">{{ task.duration }}h</div>
                      <el-icon
                        class="del"
                        size="12"
                        @click.stop="deleteTask(task)"
                      >
                        <Close />
                      </el-icon>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div class="zoom-control">
              <el-slider
                v-model="columnWidth"
                :min="35"
                :max="200"
                :step="5"
                show-input
                size="small"
              />
              <span class="zoom-label">时间列宽度</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 约束配置面板 -->
    <el-dialog
      v-model="constraintVisible"
      title="⚙️ 自动编排约束配置"
      width="700px"
    >
      <!-- 内容不变 -->
      <el-form :model="constraintConfig" label-width="180px" size="small">
        <el-divider content-position="left">⏱️ 时间规则</el-divider>
        <el-form-item label="启用工作时段限制">
          <el-switch v-model="constraintConfig.enableWorkTime" />
          仅在 {{ constraintConfig.workStart }}:00 ~
          {{ constraintConfig.workEnd }}:00 编排
        </el-form-item>
        <el-form-item
          label="工作开始时间"
          :disabled="!constraintConfig.enableWorkTime"
        >
          <el-input-number
            v-model="constraintConfig.workStart"
            :min="0"
            :max="23"
          />
        </el-form-item>
        <el-form-item
          label="工作结束时间"
          :disabled="!constraintConfig.enableWorkTime"
        >
          <el-input-number
            v-model="constraintConfig.workEnd"
            :min="1"
            :max="24"
          />
        </el-form-item>

        <el-divider content-position="left">👨‍✈️ 飞行员资质约束</el-divider>
        <el-form-item label="仅身体状态良好">
          <el-switch v-model="constraintConfig.checkHealth" />
        </el-form-item>
        <el-form-item label="仅心理状态合格">
          <el-switch v-model="constraintConfig.checkMental" />
        </el-form-item>
        <el-form-item label="必须匹配可飞机型">
          <el-switch v-model="constraintConfig.matchAircraftType" />
        </el-form-item>
        <el-form-item label="单人单日最多任务数">
          <el-input-number
            v-model="constraintConfig.maxTasksPerPilot"
            :min="1"
            :max="5"
          />
        </el-form-item>

        <el-divider content-position="left">✈️ 科目与风险约束</el-divider>
        <el-form-item label="高危科目必须一级飞行员">
          <el-switch v-model="constraintConfig.highRiskNeedLevel1" />
        </el-form-item>
        <el-form-item label="单任务最大时长（小时）">
          <el-input-number
            v-model="constraintConfig.maxDuration"
            :min="1"
            :max="8"
            step="0.5"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="constraintVisible = false">取消</el-button>
        <el-button type="primary" @click="saveConstraintConfig"
          >保存配置</el-button
        >
      </template>
    </el-dialog>

    <!-- 飞行员详情弹窗 -->
    <el-dialog
      v-model="pilotDetailVisible"
      title="飞行员详细信息"
      width="580px"
    >
      <!-- 内容不变 -->
      <el-descriptions :column="2" border size="small">
        <el-descriptions-item label="飞行员ID">{{
          currentPilot?.subject_id
        }}</el-descriptions-item>
        <el-descriptions-item label="姓名">{{
          currentPilot?.attr.name
        }}</el-descriptions-item>
        <el-descriptions-item label="性别">{{
          currentPilot?.attr.gender
        }}</el-descriptions-item>
        <el-descriptions-item label="年龄">{{
          currentPilot?.attr.age
        }}</el-descriptions-item>
        <el-descriptions-item label="身体状态">{{
          currentPilot?.attr.health_status
        }}</el-descriptions-item>
        <el-descriptions-item label="心理状态">{{
          currentPilot?.attr.mental_status
        }}</el-descriptions-item>
        <el-descriptions-item label="可飞机型">{{
          currentPilot?.attr.allowed_aircraft
        }}</el-descriptions-item>
        <el-descriptions-item label="总飞行时间"
          >{{ currentPilot?.attr.total_flight_hours }}h</el-descriptions-item
        >
        <el-descriptions-item label="各机型累计时间" :span="2">{{
          currentPilot?.attr.flight_hours_by_type || "无数据"
        }}</el-descriptions-item>
        <el-descriptions-item label="各机型间断时间" :span="2">{{
          currentPilot?.attr.break_hours_by_type || "无数据"
        }}</el-descriptions-item>
        <el-descriptions-item label="等级" :span="2">{{
          currentPilot?.attr.level
        }}</el-descriptions-item>
        <el-descriptions-item label="备注" :span="2">{{
          currentPilot?.attr.remark
        }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>

    <!-- 飞机详情弹窗 -->
    <el-dialog v-model="planeDetailVisible" title="飞机详细信息" width="580px">
      <!-- 内容不变 -->
      <el-descriptions :column="2" border size="small">
        <el-descriptions-item label="飞机ID">{{
          currentPlane?.resource_id
        }}</el-descriptions-item>
        <el-descriptions-item label="机型">{{
          currentPlane?.attr.model
        }}</el-descriptions-item>
        <el-descriptions-item label="飞机编号">{{
          currentPlane?.attr.plane_no
        }}</el-descriptions-item>
        <el-descriptions-item label="状态">{{
          currentPlane?.attr.status
        }}</el-descriptions-item>
        <el-descriptions-item label="科目">{{
          currentPlane?.attr.subject
        }}</el-descriptions-item>
        <el-descriptions-item label="所需人数">{{
          currentPlane?.attr.required_pilots
        }}</el-descriptions-item>
        <el-descriptions-item label="任务类型">{{
          currentPlane?.attr.task_type
        }}</el-descriptions-item>
        <el-descriptions-item label="科目类型">{{
          currentPlane?.attr.subject_type
        }}</el-descriptions-item>
        <el-descriptions-item label="危险等级">{{
          currentPlane?.attr.danger_level
        }}</el-descriptions-item>
        <el-descriptions-item label="默认时长"
          >{{ currentPlane?.attr.duration }}h</el-descriptions-item
        >
        <el-descriptions-item label="所属单位">{{
          currentPlane?.attr.unit
        }}</el-descriptions-item>
        <el-descriptions-item label="维护状态">{{
          currentPlane?.attr.maintain_status
        }}</el-descriptions-item>
        <el-descriptions-item label="备注" :span="2">{{
          currentPlane?.attr.remark
        }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>

    <!-- 编辑任务弹窗 -->
    <el-dialog v-model="editDialogVisible" title="编辑任务" width="400px">
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="开始小时">
          <el-input-number v-model="editForm.startHour" :min="0" :max="23" />
        </el-form-item>
        <el-form-item label="飞行时长">
          <el-input-number
            v-model="editForm.duration"
            :min="1"
            :max="12"
            :step="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveEditTask">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { Download, Close, Setting } from "@element-plus/icons-vue";
import { getRecordsByType as getSubjectRecords } from "@/api/subjectAttribute";
import { getRecordsByType as getResourceRecords } from "@/api/resourceAttribute";
import type { SubjectAttributeRecord } from "@/types/subjectAttribute";
import type { ResourceAttributeRecord } from "@/types/resourceAttribute";
import { milpSchedule } from "@/api/milp";

// 类型定义
interface PilotAttr {
  name?: string;
  gender?: string;
  age?: number;
  health_status?: string;
  mental_status?: string;
  allowed_aircraft?: string;
  total_flight_hours?: number;
  flight_hours_by_type?: string;
  break_hours_by_type?: string;
  level?: string;
  remark?: string;
  [key: string]: any;
}
interface Pilot extends SubjectAttributeRecord {
  attr: PilotAttr;
}

interface PlaneAttr {
  model?: string;
  plane_no?: string;
  status?: string;
  subject?: string;
  task_type?: string;
  subject_type?: string;
  danger_level?: string;
  duration?: number;
  unit?: string;
  maintain_status?: string;
  remark?: string;
  [key: string]: any;
}
interface Plane extends ResourceAttributeRecord {
  attr: PlaneAttr;
}

interface FlightTask {
  id: number;
  pilotIds: number[];
  pilotNames: string[];
  planeId: number;
  startHour: number;
  duration: number;
  start: string;
  end: string;
  conflict: boolean;
}

interface PlaneRow {
  plane: Plane;
  tasks: FlightTask[];
}

interface ConstraintConfig {
  enableWorkTime: boolean;
  workStart: number;
  workEnd: number;
  checkHealth: boolean;
  checkMental: boolean;
  matchAircraftType: boolean;
  maxTasksPerPilot: number;
  highRiskNeedLevel1: boolean;
  maxDuration: number;
}

// 工具函数
function parseJsonAttr<T>(data: any): T {
  if (!data) return {} as T;
  if (typeof data === "object") return data;
  try {
    return JSON.parse(data);
  } catch {
    return {} as T;
  }
}

function hhmm(h: number) {
  const hour = Math.floor(h);
  const min = Math.round((h % 1) * 60);
  return `${hour.toString().padStart(2, "0")}:${min
    .toString()
    .padStart(2, "0")}`;
}

// 数据
const pilotList = ref<Pilot[]>([]);
const planeList = ref<Plane[]>([]);
const columnWidth = ref(120);
const planeRows = ref<PlaneRow[]>([]);

const draggingType = ref<"pilot" | "plane" | null>(null);
const draggingPilot = ref<Pilot | null>(null);
const draggingPlane = ref<Plane | null>(null);

const pilotCollapsed = ref(false);
const planeCollapsed = ref(false);
const togglePilotPanel = () => (pilotCollapsed.value = !pilotCollapsed.value);
const togglePlanePanel = () => (planeCollapsed.value = !planeCollapsed.value);

const pilotDetailVisible = ref(false);
const currentPilot = ref<Pilot | null>(null);
const planeDetailVisible = ref(false);
const currentPlane = ref<Plane | null>(null);

const editDialogVisible = ref(false);
const editForm = ref({ id: 0, startHour: 0, duration: 0 });
let editingTask: FlightTask | null = null;

// 约束配置
const constraintVisible = ref(false);
const constraintConfig = ref<ConstraintConfig>({
  enableWorkTime: true,
  workStart: 8,
  workEnd: 18,
  checkHealth: true,
  checkMental: true,
  matchAircraftType: true,
  maxTasksPerPilot: 1,
  highRiskNeedLevel1: true,
  maxDuration: 4,
});

const openConstraintPanel = () => {
  constraintVisible.value = true;
};

const saveConstraintConfig = () => {
  constraintVisible.value = false;
  ElMessage.success("约束配置已保存");
};

// 加载数据
const loadData = async () => {
  try {
    const pilots = await getSubjectRecords("PILOT");
    pilotList.value = pilots.map((p) => ({
      ...p,
      attr: parseJsonAttr<PilotAttr>(p.attr_key_value),
    }));
    const planes = await getResourceRecords("AIRCRAFT");
    planeList.value = planes.map((p) => ({
      ...p,
      attr: parseJsonAttr<PlaneAttr>(p.attr_key_value),
    }));
    planeRows.value = planeList.value.map((p) => ({ plane: p, tasks: [] }));
  } catch {
    ElMessage.error("加载失败");
  }
};

// 冲突检测（支持多飞行员）
const checkConflict = (task: FlightTask, row: PlaneRow): boolean => {
  const s1 = task.startHour;
  const e1 = s1 + task.duration;
  // 1. 同飞机时间重叠
  for (const t of row.tasks) {
    if (t.id === task.id) continue;
    const s2 = t.startHour;
    const e2 = s2 + t.duration;
    if (s1 < e2 && e1 > s2) return true;
  }
  // 2. 不同飞机，但飞行员时间重叠
  for (const r of planeRows.value) {
    for (const t of r.tasks) {
      if (t.id === task.id) continue;
      const s2 = t.startHour;
      const e2 = s2 + t.duration;
      if (s1 < e2 && e1 > s2) {
        if (task.pilotIds.some((pid) => t.pilotIds.includes(pid))) {
          return true;
        }
      }
    }
  }
  return false;
};

const refreshAllConflicts = () => {
  planeRows.value.forEach((r) =>
    r.tasks.forEach((t) => (t.conflict = checkConflict(t, r)))
  );
};

// 拖拽相关
const onDragPilot = (e: DragEvent, p: Pilot) => {
  e.dataTransfer!.setData("text", "pilot");
  draggingType.value = "pilot";
  draggingPilot.value = p;
};

const onDragPlane = (e: DragEvent, p: Plane) => {
  e.dataTransfer!.setData("text", "plane");
  draggingType.value = "plane";
  draggingPlane.value = p;
};

const onDragEnd = () => {
  draggingType.value = null;
  draggingPilot.value = null;
  draggingPlane.value = null;
};

const onDragOver = (e: DragEvent) => {
  (e.currentTarget as HTMLElement).classList.add("drag-over");
};

const onDragLeave = (e: DragEvent) => {
  (e.currentTarget as HTMLElement).classList.remove("drag-over");
};

// 拖拽到空白槽（创建新任务）
const onDrop = async (e: DragEvent, row: PlaneRow, startHour: number) => {
  e.preventDefault();
  const el = e.currentTarget as HTMLElement;
  el.classList.remove("drag-over");
  if (draggingType.value !== "pilot" || !draggingPilot.value) return;

  const pilot = draggingPilot.value;
  const plane = row.plane;
  const defaultDur = plane.attr.duration || 2;

  try {
    const { value: durInput } = await ElMessageBox.prompt(
      "输入时长",
      "创建任务",
      {
        inputValue: String(defaultDur),
      }
    );
    if (!durInput) return;
    const duration = parseFloat(durInput);
    if (isNaN(duration) || duration <= 0 || startHour + duration > 24) {
      ElMessage.warning("输入无效或超出24点");
      return;
    }

    const newTask: FlightTask = {
      id: Date.now(),
      pilotIds: [pilot.subject_id],
      pilotNames: [pilot.attr.name || `飞行员${pilot.subject_id}`],
      planeId: plane.resource_id,
      startHour,
      duration,
      start: hhmm(startHour),
      end: hhmm(startHour + duration),
      conflict: false,
    };

    if (checkConflict(newTask, row)) {
      ElMessage.warning("冲突");
      return;
    }
    row.tasks.push(newTask);
    refreshAllConflicts();
    ElMessage.success("创建成功");
  } catch {
    // 取消
  }
};

// 拖拽到已有任务（追加飞行员）
const onDropOnTask = async (
  e: DragEvent,
  existingTask: FlightTask,
  row: PlaneRow
) => {
  e.preventDefault();
  e.stopPropagation();
  if (draggingType.value !== "pilot" || !draggingPilot.value) return;
  const pilot = draggingPilot.value;
  // 去重
  if (existingTask.pilotIds.includes(pilot.subject_id)) {
    ElMessage.info("该飞行员已在任务中");
    return;
  }
  // 保存原任务状态以便回滚
  const originalTask = { ...existingTask };
  const originalPilotIds = [...existingTask.pilotIds];
  const originalPilotNames = [...existingTask.pilotNames];

  // 临时添加飞行员
  existingTask.pilotIds.push(pilot.subject_id);
  existingTask.pilotNames.push(pilot.attr.name || `飞行员${pilot.subject_id}`);
  // 重新检测冲突
  const hasConflict = checkConflict(existingTask, row);
  if (hasConflict) {
    // 回滚
    existingTask.pilotIds = originalPilotIds;
    existingTask.pilotNames = originalPilotNames;
    ElMessage.warning("添加后会导致飞行员时间冲突，操作取消");
    return;
  }
  // 无冲突则刷新显示
  refreshAllConflicts();
  ElMessage.success(`已将 ${pilot.attr.name || pilot.subject_id} 添加到任务`);
};

// 任务样式
const getTaskStyle = (t: FlightTask) => ({
  left: `${t.startHour * columnWidth.value}px`,
  width: `${t.duration * columnWidth.value}px`,
});

// 删除任务
const deleteTask = (task: FlightTask) => {
  for (const r of planeRows.value) {
    const i = r.tasks.findIndex((x) => x.id === task.id);
    if (i >= 0) {
      r.tasks.splice(i, 1);
      refreshAllConflicts();
      ElMessage.success("删除成功");
      return;
    }
  }
};

// 清空所有任务
const clearAll = () => {
  planeRows.value.forEach((r) => (r.tasks = []));
  ElMessage.success("已清空");
};

// 自动编排
const autoSchedule = async () => {
  const tasks = planeRows.value.map((row, idx) => ({
    id: `task_${idx}`,
    duration: Math.min(
      row.plane.attr.duration || 2,
      constraintConfig.value.maxDuration
    ),
    weight: 10,
    risk: row.plane.attr.danger_level === "高危" ? 1 : 0,
  }));

  const request = {
    tasks,
    constraints: {
      enable_work_time: constraintConfig.value.enableWorkTime,
      work_start: constraintConfig.value.workStart,
      work_end: constraintConfig.value.workEnd,
      check_health: constraintConfig.value.checkHealth,
      check_mental: constraintConfig.value.checkMental,
      match_aircraft_type: constraintConfig.value.matchAircraftType,
      max_tasks_per_pilot: constraintConfig.value.maxTasksPerPilot,
      high_risk_need_level1: constraintConfig.value.highRiskNeedLevel1,
      max_duration: constraintConfig.value.maxDuration,
    },
  };

  try {
    const res = await milpSchedule(request);
    const data = res && res.data ? res.data : res;
    const assignments = data?.assignments || [];
    if (assignments.length === 0) {
      ElMessage.warning("编排结果为空，请检查任务或约束条件");
      return;
    }
    // 清空现有任务
    planeRows.value.forEach((row) => (row.tasks = []));
    for (const assign of assignments) {
      const planeRow = planeRows.value.find(
        (r) => r.plane.resource_id === assign.aircraft_id
      );
      if (!planeRow) continue;
      // const pilot = pilotList.value.find(
      //   (p) => p.subject_id === assign.pilot_id
      // );
      // 取出所有匹配当前指派的飞行员数组
      const matchedPilots = pilotList.value.filter((p) =>
        assign.pilot_id.includes(p.subject_id)
      );
      // 取出名字
      const pilotNames = matchedPilots.map((p) => p.attr.name).join("、");

      const task: FlightTask = {
        id: Date.now() + Math.random(),
        pilotIds: [assign.pilot_id],
        pilotNames: [pilotNames],
        planeId: assign.aircraft_id,
        startHour: assign.start_hour,
        duration: assign.duration,
        start: hhmm(assign.start_hour),
        end: hhmm(assign.start_hour + assign.duration),
        conflict: false,
      };
      planeRow.tasks.push(task);
    }
    refreshAllConflicts();
    ElMessage.success(`自动编排完成，共 ${assignments.length} 个任务`);
  } catch (error) {
    console.error(error);
    ElMessage.error("编排失败，请检查后端服务");
  }
};

// 导出 Excel（时间表 + ID映射表 + 任务ID）
const savePlan = async () => {
  if (planeRows.value.length === 0) {
    ElMessage.warning("暂无任务可导出");
    return;
  }

  // 1) 生成时间表
  type ExcelRow = (string | number)[];
  const scheduleRows: ExcelRow[] = [];
  const header: ExcelRow = [
    "飞机编号",
    "型号",
    "科目",
    "危险等级",
    "00:00",
    "01:00",
    "02:00",
    "03:00",
    "04:00",
    "05:00",
    "06:00",
    "07:00",
    "08:00",
    "09:00",
    "10:00",
    "11:00",
    "12:00",
    "13:00",
    "14:00",
    "15:00",
    "16:00",
    "17:00",
    "18:00",
    "19:00",
    "20:00",
    "21:00",
    "22:00",
    "23:00",
  ];
  scheduleRows.push(header);

  // 全局任务编号（不同时间段自动编号 1,2,3...）
  let taskGlobalId = 1;
  // 保存 任务 → 统一编号
  const taskIdMap = new Map<any, number>();

  for (const row of planeRows.value) {
    const p = row.plane;
    const line: ExcelRow = [
      p.attr.plane_no ?? "",
      p.attr.model ?? "",
      p.attr.subject ?? "",
      p.attr.danger_level ?? "",
    ];
    for (let i = 0; i < 24; i++) line.push("");

    // 给每个任务分配唯一任务ID
    row.tasks.forEach((task) => {
      if (!taskIdMap.has(task.id)) {
        taskIdMap.set(task.id, taskGlobalId++);
      }
      const start = task.startHour;
      const hours = task.duration;
      const pilotNamesStr = task.pilotNames.join(",");
      for (let i = 0; i < hours; i++) {
        const h = start + i;
        if (h >= 24) break;
        line[4 + h] = pilotNamesStr;
      }
    });
    scheduleRows.push(line);
  }

  // 2) 生成 ID映射表 + 任务ID
  const idRows: (string | number)[][] = [["任务ID", "飞机ID", "飞行员ID"]];
  for (const row of planeRows.value) {
    for (const task of row.tasks) {
      const tid = taskIdMap.get(task.id) || "";
      for (const pid of task.pilotIds) {
        idRows.push([tid, task.planeId, pid]);
      }
    }
  }

  // 3) 写入Excel
  const XLSX = await import("xlsx");
  const wb = XLSX.utils.book_new();

  // Sheet1 时间表
  const wsSchedule = XLSX.utils.aoa_to_sheet(scheduleRows);
  wsSchedule["!cols"] = [
    { wch: 14 },
    { wch: 10 },
    { wch: 18 },
    { wch: 10 },
    ...Array(24).fill({ wch: 16 }),
  ];
  const border = {
    top: { style: "thin" },
    bottom: { style: "thin" },
    left: { style: "thin" },
    right: { style: "thin" },
  };
  const range = XLSX.utils.decode_range(wsSchedule["!ref"] || "A1:AB1");
  for (let r = range.s.r; r <= range.e.r; r++) {
    for (let c = range.s.c; c <= range.e.c; c++) {
      const cell = wsSchedule[XLSX.utils.encode_cell({ r, c })];
      if (cell) {
        cell.s = {
          alignment: { horizontal: "center", vertical: "center" },
          border,
        };
        if (r === 0) cell.s.fill = { fgColor: { rgb: "E0E0E0" } };
      }
    }
  }
  XLSX.utils.book_append_sheet(wb, wsSchedule, "试飞时间表");

  // Sheet2 ID映射表（带任务ID）
  if (idRows.length > 1) {
    const wsId = XLSX.utils.aoa_to_sheet(idRows);
    XLSX.utils.book_append_sheet(wb, wsId, "ID映射表");
  }

  XLSX.writeFile(wb, `试飞计划_${new Date().toISOString().slice(0, 10)}.xlsx`);
  ElMessage.success("导出成功：时间表 + 带任务ID的映射表");
};

// 编辑任务（仅修改时间）
const editTask = (task: FlightTask) => {
  editingTask = task;
  editForm.value = {
    id: task.id,
    startHour: task.startHour,
    duration: task.duration,
  };
  editDialogVisible.value = true;
};

const saveEditTask = () => {
  if (!editingTask) return;
  const f = editForm.value;
  const end = f.startHour + f.duration;
  if (end > 24) {
    ElMessage.warning("超出24点");
    return;
  }

  let row: PlaneRow | null = null;
  let t: FlightTask | null = null;
  for (const r of planeRows.value) {
    const found = r.tasks.find((x) => x.id === f.id);
    if (found) {
      row = r;
      t = found;
      break;
    }
  }
  if (!row || !t) return;

  // 移除旧任务
  row.tasks = row.tasks.filter((x) => x.id !== f.id);
  // 更新属性
  t.startHour = f.startHour;
  t.duration = f.duration;
  t.start = hhmm(f.startHour);
  t.end = hhmm(end);
  // 检查冲突
  if (checkConflict(t, row)) {
    ElMessage.warning("冲突");
    row.tasks.push(editingTask);
    return;
  }
  row.tasks.push(t);
  refreshAllConflicts();
  editDialogVisible.value = false;
  ElMessage.success("保存成功");
};

// 详情弹窗
const showPilotDetail = (p: Pilot) => {
  currentPilot.value = p;
  pilotDetailVisible.value = true;
};

const showPlaneDetail = (p: Plane) => {
  currentPlane.value = p;
  planeDetailVisible.value = true;
};

onMounted(() => loadData());
</script>

<style scoped>
.header-title {
  font-size: 20px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #3498db);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.flight-plan-page {
  height: 100%;
  background: #f5f7fa;
  display: flex;
  flex-direction: column;
}
.main-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden !important;
}
:deep(.el-card__body) {
  height: calc(100vh - 110px) !important;
  overflow: hidden !important;
  padding: 12px !important;
  display: flex;
  flex-direction: column;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.tool-bar {
  display: flex;
  gap: 10px;
}
.main-layout {
  display: flex;
  gap: 14px;
  flex: 1;
  overflow: hidden !important;
}
.left-fixed-box {
  width: 280px;
  flex-shrink: 0;
  height: 100%;
  overflow: hidden !important;
}
.left-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}
.panel-section {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #eee;
  overflow: hidden;
}
.panel-header {
  padding: 10px 12px;
  background: #fafbfc;
  font-weight: 600;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
}
.toggle-icon {
  font-size: 16px;
  font-weight: bold;
  color: #666;
}
.panel-body {
  padding: 8px;
}
.scroll-body {
  max-height: 260px;
  overflow-y: auto;
}
.resource-item {
  padding: 10px;
  margin-bottom: 8px;
  background: #f9f9f9;
  border-radius: 6px;
  border-left: 4px solid #409eff;
  cursor: grab;
}
.resource-item.plane {
  border-left-color: #67c23a;
}
.resource-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 600;
}
.small-info {
  font-size: 12px;
  color: #666;
  margin-top: 2px;
}
.right-scroll-box {
  flex: 1;
  height: 100%;
  overflow: hidden !important;
}
.right-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.schedule-container {
  flex: 1;
  background: #fff;
  border: 1px solid #eee;
  border-radius: 8px;
  overflow: auto;
}
.time-header {
  display: flex;
  background: #fafbfc;
  border-bottom: 1px solid #eee;
  position: sticky;
  top: 0;
  z-index: 10;
  width: fit-content;
}
.plane-label {
  width: 120px;
  flex-shrink: 0;
  text-align: center;
  line-height: 40px;
  border-right: 1px solid #eee;
}
.hour-cell {
  flex-shrink: 0;
  width: 120px;
  text-align: center;
  line-height: 40px;
  border-right: 1px solid #eee;
  font-size: 12px;
}
.schedule-body {
  width: fit-content;
}
.plane-row {
  display: flex;
  border-bottom: 1px dashed #eee;
  height: 60px;
}
.timeline {
  position: relative;
  height: 100%;
}
.hour-slot {
  display: inline-block;
  width: 120px;
  height: 60px;
  border-right: 1px dashed #eee;
}
.hour-slot.drag-over {
  background: #e1f3ff;
}
.task-card {
  position: absolute;
  top: 5px;
  height: 50px;
  background: #ecf5ff;
  border-left: 4px solid #409eff;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
}
.task-card.conflict {
  background: #fef0f0;
  border-color: #f56c6c;
}
.task-title {
  font-weight: 600;
  font-size: 11px;
}
.task-time,
.task-duration {
  font-size: 9px;
  color: #666;
}
.del {
  position: absolute;
  top: 2px;
  right: 4px;
  color: #999;
  cursor: pointer;
}
.zoom-control {
  display: flex;
  align-items: center;
  gap: 12px;
}
.zoom-label {
  font-size: 12px;
  color: #666;
}
</style>