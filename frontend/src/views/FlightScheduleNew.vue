<template>
  <div class="flight-schedule-new">
    <div class="toolbar">
      <el-button type="primary" @click="milpSchedule">MILP智能编排</el-button>
      <el-button @click="autoSchedule">贪心编排</el-button>
      <el-button type="success" @click="exportSchedule">导出计划</el-button>
      <el-button @click="clearAll">清空时间表</el-button>
      <el-button @click="openConstraintDialog">约束条件</el-button>
      <el-switch
        v-model="use10minSlot"
        active-text="10分钟刻度"
        inactive-text="小时刻度"
        @change="rebuildTimeSlots"
      />
    </div>

    <div class="main-layout">
      <!-- 左侧面板：飞行员、飞机、指挥员（默认收纳） -->
      <div class="left-panel">
        <!-- 飞行员卡片 -->
        <el-card class="pilot-card" shadow="hover">
          <template #header>
            <div class="card-header" @click="togglePilotSection">
              <span>飞行员 (勾选可执行科目)</span>
              <div class="header-actions">
                <el-button
                  link
                  type="primary"
                  size="small"
                  @click.stop="toggleSelectAllPilots"
                >
                  {{ allPilotsSelected ? "取消全选" : "全部全选" }}
                </el-button>
                <span class="toggle-icon">{{
                  pilotSectionExpanded ? "▼" : "▶"
                }}</span>
              </div>
            </div>
          </template>
          <div v-show="pilotSectionExpanded">
            <div v-for="pilot in pilotList" :key="pilot.id" class="pilot-item">
              <div class="pilot-header" @click="togglePilotCollapse(pilot.id)">
                <div>
                  <span class="pilot-name">{{ pilot.name }}</span>
                  <span class="pilot-info"
                    >{{ pilot.level }} | 可飞:{{
                      pilot.allowedModels.join(",")
                    }}</span
                  >
                </div>
                <el-button
                  link
                  type="primary"
                  size="small"
                  @click.stop="toggleSelectAllPilot(pilot)"
                >
                  {{
                    pilot.selectedSubjects.length ===
                    pilot.availableSubjects.length
                      ? "取消全选"
                      : "全选"
                  }}
                </el-button>
                <span class="toggle-icon">{{
                  pilotCollapsed[pilot.id] ? "▶" : "▼"
                }}</span>
              </div>
              <div v-show="!pilotCollapsed[pilot.id]">
                <div
                  v-if="pilot.availableSubjects.length === 0"
                  class="empty-subject"
                >
                  无可用科目
                </div>
                <el-checkbox-group
                  v-model="pilot.selectedSubjects"
                  @change="updatePilotSubjects(pilot)"
                >
                  <el-checkbox
                    v-for="subj in pilot.availableSubjects"
                    :key="subj.id"
                    :label="subj.id"
                  >
                    {{ subj.name }} ({{ subj.default_duration }}h, 需{{
                      subj.required_pilots
                    }}人)
                  </el-checkbox>
                </el-checkbox-group>
                <div class="draggable-tasks">
                  <div
                    v-for="subjId in pilot.selectedSubjects"
                    :key="subjId"
                    class="draggable-task"
                    draggable="true"
                    @dragstart="
                      (e) =>
                        onDragStartPilotTask(e, {
                          pilotId: pilot.id,
                          subjectId: subjId,
                        })
                    "
                    @dragend="onDragEnd"
                  >
                    {{ getSubjectName(pilot.availableSubjects, subjId) }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 飞机卡片 -->
        <el-card class="aircraft-card" shadow="hover">
          <template #header>
            <div class="card-header" @click="toggleAircraftSection">
              <span>飞机 (勾选未完成科目)</span>
              <div class="header-actions">
                <el-button
                  link
                  type="primary"
                  size="small"
                  @click.stop="toggleSelectAllAircrafts"
                >
                  {{ allAircraftsSelected ? "取消全选" : "全部全选" }}
                </el-button>
                <span class="toggle-icon">{{
                  aircraftSectionExpanded ? "▼" : "▶"
                }}</span>
              </div>
            </div>
          </template>
          <div v-show="aircraftSectionExpanded">
            <div v-for="ac in aircraftList" :key="ac.id" class="aircraft-item">
              <div
                class="aircraft-header"
                @click="toggleAircraftCollapse(ac.id)"
              >
                <div>
                  <span class="aircraft-name"
                    >{{ ac.plane_no }} ({{ ac.model_name }})</span
                  >
                  <span class="aircraft-status">{{ ac.status }}</span>
                </div>
                <el-button
                  link
                  type="primary"
                  size="small"
                  @click.stop="toggleSelectAllAircraft(ac)"
                >
                  {{
                    ac.selectedSubjects.length === ac.availableSubjects.length
                      ? "取消全选"
                      : "全选"
                  }}
                </el-button>
                <span class="toggle-icon">{{
                  aircraftCollapsed[ac.id] ? "▶" : "▼"
                }}</span>
              </div>
              <div v-show="!aircraftCollapsed[ac.id]">
                <div
                  v-if="ac.availableSubjects.length === 0"
                  class="empty-subject"
                >
                  无未完成科目
                </div>
                <el-checkbox-group
                  v-model="ac.selectedSubjects"
                  @change="updateAircraftSubjects(ac)"
                >
                  <el-checkbox
                    v-for="subj in ac.availableSubjects"
                    :key="subj.id"
                    :label="subj.id"
                  >
                    {{ subj.name }} ({{ subj.default_duration }}h, 需{{
                      subj.required_pilots
                    }}人)
                  </el-checkbox>
                </el-checkbox-group>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 指挥员卡片 -->
        <el-card class="commander-card" shadow="hover">
          <template #header>
            <div class="card-header" @click="toggleCommanderSection">
              <span>指挥员 (可勾选，不影响时间表)</span>
              <div class="header-actions">
                <el-button
                  link
                  type="primary"
                  size="small"
                  @click.stop="toggleSelectAllCommanders"
                >
                  {{ allCommandersSelected ? "取消全选" : "全部全选" }}
                </el-button>
                <span class="toggle-icon">{{
                  commanderSectionExpanded ? "▼" : "▶"
                }}</span>
              </div>
            </div>
          </template>
          <div v-show="commanderSectionExpanded">
            <div
              v-for="cmd in commanderList"
              :key="cmd.id"
              class="commander-item"
            >
              <el-checkbox v-model="cmd.selected">
                {{ cmd.name }} {{ cmd.level ? "(" + cmd.level + ")" : "" }}
              </el-checkbox>
            </div>
            <div v-if="commanderList.length === 0" class="empty-subject">
              暂无指挥员数据
            </div>
          </div>
        </el-card>
      </div>

      <!-- 右侧时间表 -->
      <div class="right-panel" v-if="showSchedule">
        <div class="schedule-wrapper">
          <div class="time-header">
            <div class="plane-label-col">飞机/时间</div>
            <div class="time-cells">
              <div
                v-for="slot in timeSlots"
                :key="slot.index"
                class="time-cell"
                :style="{ width: cellWidth + 'px' }"
              >
                {{ formatTime(slot.startMin) }}
              </div>
            </div>
          </div>
          <div class="schedule-body">
            <div v-for="ac in filteredAircrafts" :key="ac.id" class="plane-row">
              <div class="plane-label">{{ ac.plane_no }}</div>
              <div class="timeline" :style="{ width: totalWidth + 'px' }">
                <div
                  v-for="slot in timeSlots"
                  :key="slot.index"
                  class="time-slot"
                  :style="{ width: cellWidth + 'px' }"
                  @dragover.prevent
                  @drop="(e) => onDrop(e, ac.id, slot.startMin)"
                ></div>
                <div
                  v-for="task in ac.tasks"
                  :key="task.id"
                  class="task-card"
                  draggable="true"
                  @dragstart="(e) => onDragStartTask(e, task)"
                  @dragend="onDragEnd"
                  :style="getTaskStyle(task)"
                  @click="editTask(task)"
                >
                  <div class="task-title">
                    {{ task.subjectName }}<br />{{ task.pilotName }}
                  </div>
                  <div class="task-time">
                    {{ formatTimeRange(task.startMin, task.endMin) }}
                  </div>
                  <el-icon class="del" @click.stop="deleteTask(task)"
                    ><Close
                  /></el-icon>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="zoom-control">
          <el-slider
            v-model="cellWidth"
            :min="30"
            :max="120"
            :step="5"
            show-input
            size="small"
          />
          <span class="zoom-label">时间列宽度(px)</span>
        </div>
      </div>
      <div v-else class="empty-schedule">
        请至少为一架飞机勾选科目，时间表将自动显示
      </div>
    </div>

    <!-- 编辑任务弹窗 -->
    <el-dialog v-model="editDialogVisible" title="编辑任务" width="400px">
      <el-form :model="editForm" label-width="100px">
        <el-form-item label="开始时间">
          <el-time-select
            v-model="editForm.startTime"
            :start="'00:00'"
            :step="use10minSlot ? '00:10' : '01:00'"
            :end="'23:50'"
            placeholder="选择开始时间"
          />
        </el-form-item>
        <el-form-item label="飞行时长(小时)">
          <el-input-number
            v-model="editForm.duration"
            :min="0.5"
            :step="0.5"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveEditTask">保存</el-button>
      </template>
    </el-dialog>

    <!-- 约束条件对话框 -->
    <el-dialog
      v-model="constraintDialogVisible"
      title="约束条件配置"
      width="600px"
    >
      <el-form :model="constraintConfig" label-width="180px">
        <el-form-item label="启用工作时间限制">
          <el-switch v-model="constraintConfig.enableWorkTime" />
        </el-form-item>
        <el-form-item
          v-if="constraintConfig.enableWorkTime"
          label="工作时间范围"
        >
          <el-time-picker
            v-model="workTimeRange"
            is-range
            format="HH:mm"
            value-format="HH:mm"
            range-separator=" - "
            start-placeholder="开始"
            end-placeholder="结束"
          />
        </el-form-item>

        <el-form-item label="启用心理健康检查">
          <el-switch v-model="constraintConfig.enableMentalCheck" />
        </el-form-item>
        <el-form-item v-if="constraintConfig.enableMentalCheck">
          <span class="constraint-tip"
            >仅允许“优秀”或“良好”心理状态的飞行员执行任务</span
          >
        </el-form-item>

        <el-form-item label="启用健康检查">
          <el-switch v-model="constraintConfig.enableHealthCheck" />
        </el-form-item>
        <el-form-item v-if="constraintConfig.enableHealthCheck">
          <span class="constraint-tip">仅允许“良好”健康状态的飞行员</span>
        </el-form-item>

        <el-form-item label="启用机型匹配">
          <el-switch v-model="constraintConfig.matchAircraftType" />
        </el-form-item>

        <el-form-item label="高危任务需一级飞行员">
          <el-switch v-model="constraintConfig.enableHighRiskLevel1" />
        </el-form-item>

        <el-form-item label="单任务最大时长(h)">
          <el-input-number
            v-model="constraintConfig.maxDuration"
            :min="1"
            :step="1"
          />
        </el-form-item>

        <el-form-item label="飞行员每日最大飞行时长(h)">
          <el-input-number
            v-model="constraintConfig.maxDailyFlightHours"
            :min="1"
            :max="24"
            :step="1"
          />
        </el-form-item>

        <el-divider />

        <el-form-item label="飞机飞行间隔限制">
          <el-switch v-model="constraintConfig.enableAircraftInterval" />
        </el-form-item>
        <el-form-item
          v-if="constraintConfig.enableAircraftInterval"
          label="最小间隔(分钟)"
        >
          <el-input-number
            v-model="constraintConfig.minAircraftInterval"
            :min="0"
            :step="10"
          />
        </el-form-item>

        <el-form-item label="飞行员飞行间隔限制">
          <el-switch v-model="constraintConfig.enablePilotInterval" />
        </el-form-item>
        <el-form-item
          v-if="constraintConfig.enablePilotInterval"
          label="最小间隔(分钟)"
        >
          <el-input-number
            v-model="constraintConfig.minPilotInterval"
            :min="0"
            :step="10"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="constraintDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from "vue";
import { ElMessage } from "element-plus";
import { Close } from "@element-plus/icons-vue";
import {
  listAircraft,
  getAircraftDetail,
  getPilotModels,
  getPilots,
} from "@/api/aircraft";
import { getRecordsByType } from "@/api/subjectAttribute";
import { getUser } from "@/api/user";
import * as XLSX from "xlsx";
import { uploadFile } from "@/api/minio";
import { runMilpSchedule } from "@/api/milp";
import type {
  AircraftInfo,
  MilpRequest,
  PilotInfo,
  TaskInfo,
} from "@/types/milp";

// ---------- 类型定义 ----------
interface Subject {
  id: number;
  name: string;
  default_duration: number;
  danger_level: string;
  required_pilots: number; // 新增
}

interface Pilot {
  id: number;
  name: string;
  level: string;
  allowedModels: string[];
  availableSubjects: Subject[];
  selectedSubjects: number[];
  mentalStatus: string;
  healthStatus: string;
}

interface Aircraft {
  id: number;
  plane_no: string;
  model_name: string;
  status: string;
  availableSubjects: Subject[];
  selectedSubjects: number[];
  tasks: any[];
}

interface CommanderAttr {
  指挥时间?: number;
  [key: string]: any;
}

interface Commander {
  id: number;
  attr: CommanderAttr;
  name: string;
  level: string;
  selected: boolean;
}

interface ConstraintConfig {
  enableWorkTime: boolean;
  workStart: number;
  workEnd: number;
  enableMentalCheck: boolean;
  enableHealthCheck: boolean;
  matchAircraftType: boolean;
  enableHighRiskLevel1: boolean;
  maxDuration: number;
  maxDailyFlightHours: number;
  enableAircraftInterval: boolean;
  minAircraftInterval: number;
  enablePilotInterval: boolean;
  minPilotInterval: number;
}

function parseJsonAttr<T>(data: any): T {
  if (!data) return {} as T;
  if (typeof data === "object") return data;
  try {
    return JSON.parse(data);
  } catch {
    return {} as T;
  }
}

// ---------- 响应式数据 ----------
const pilotList = ref<Pilot[]>([]);
const aircraftList = ref<Aircraft[]>([]);
const commanderList = ref<Commander[]>([]);

const use10minSlot = ref(true);
const cellWidth = ref(60);
const timeSlots = ref<{ index: number; startMin: number }[]>([]);
const editDialogVisible = ref(false);
const editForm = ref({ id: 0, startTime: "", duration: 0 });
let editingTask: any = null;

let draggingPayload: any = null;
let draggingTask: any = null;

const pilotSectionExpanded = ref(false);
const aircraftSectionExpanded = ref(false);
const commanderSectionExpanded = ref(false);
const pilotCollapsed = ref<Record<number, boolean>>({});
const aircraftCollapsed = ref<Record<number, boolean>>({});

const constraintDialogVisible = ref(false);
const constraintConfig = reactive<ConstraintConfig>({
  enableWorkTime: false,
  workStart: 8 * 60,
  workEnd: 18 * 60,
  enableMentalCheck: false,
  enableHealthCheck: false,
  matchAircraftType: true,
  enableHighRiskLevel1: false,
  maxDuration: 8,
  maxDailyFlightHours: 8,
  enableAircraftInterval: false,
  minAircraftInterval: 30,
  enablePilotInterval: false,
  minPilotInterval: 30,
});
const workTimeRange = ref<[string, string]>(["08:00", "18:00"]);

watch(workTimeRange, (val) => {
  if (val && val.length === 2) {
    const [h1, m1] = val[0].split(":").map(Number);
    const [h2, m2] = val[1].split(":").map(Number);
    constraintConfig.workStart = h1 * 60 + m1;
    constraintConfig.workEnd = h2 * 60 + m2;
  }
});

const showSchedule = computed(() =>
  aircraftList.value.some((ac) => ac.selectedSubjects.length > 0)
);
const filteredAircrafts = computed(() =>
  aircraftList.value.filter((ac) => ac.selectedSubjects.length > 0)
);
const totalWidth = computed(() => cellWidth.value * timeSlots.value.length);

const allPilotsSelected = computed(
  () =>
    pilotList.value.length > 0 &&
    pilotList.value.every(
      (p) =>
        p.selectedSubjects.length === p.availableSubjects.length &&
        p.availableSubjects.length > 0
    )
);
const allAircraftsSelected = computed(
  () =>
    aircraftList.value.length > 0 &&
    aircraftList.value.every(
      (ac) =>
        ac.selectedSubjects.length === ac.availableSubjects.length &&
        ac.availableSubjects.length > 0
    )
);
const allCommandersSelected = computed(
  () =>
    commanderList.value.length > 0 &&
    commanderList.value.every((c) => c.selected)
);

const formatTime = (minutes: number) => {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`;
};
const formatTimeRange = (startMin: number, endMin: number) =>
  `${formatTime(startMin)}~${formatTime(endMin)}`;
const timeToMinutes = (timeStr: string) => {
  const [h, m] = timeStr.split(":").map(Number);
  return h * 60 + m;
};

const rebuildTimeSlots = () => {
  const slots = [];
  const step = use10minSlot.value ? 10 : 60;
  for (let m = 0; m < 24 * 60; m += step) {
    slots.push({ index: slots.length, startMin: m });
  }
  timeSlots.value = slots;
};
rebuildTimeSlots();

const getTaskStyle = (task: any) => {
  const step = use10minSlot.value ? 10 : 60;
  const startIdx = task.startMin / step;
  const durationSlots = (task.duration * 60) / step;
  return {
    left: `${startIdx * cellWidth.value}px`,
    width: `${durationSlots * cellWidth.value}px`,
  };
};

const togglePilotSection = () => {
  pilotSectionExpanded.value = !pilotSectionExpanded.value;
};
const toggleAircraftSection = () => {
  aircraftSectionExpanded.value = !aircraftSectionExpanded.value;
};
const toggleCommanderSection = () => {
  commanderSectionExpanded.value = !commanderSectionExpanded.value;
};
const togglePilotCollapse = (id: number) => {
  pilotCollapsed.value[id] = !pilotCollapsed.value[id];
};
const toggleAircraftCollapse = (id: number) => {
  aircraftCollapsed.value[id] = !aircraftCollapsed.value[id];
};

const openConstraintDialog = () => {
  workTimeRange.value = [
    `${Math.floor(constraintConfig.workStart / 60)
      .toString()
      .padStart(2, "0")}:${(constraintConfig.workStart % 60)
      .toString()
      .padStart(2, "0")}`,
    `${Math.floor(constraintConfig.workEnd / 60)
      .toString()
      .padStart(2, "0")}:${(constraintConfig.workEnd % 60)
      .toString()
      .padStart(2, "0")}`,
  ];
  constraintDialogVisible.value = true;
};

// ---------- 后端数据加载 ----------
const loadData = async () => {
  try {
    const planes = await listAircraft();
    const aircraftDetails = await Promise.all(
      planes.map(async (plane: any) => {
        const detail = await getAircraftDetail(plane.id);
        const subjects = detail.subjects
          .filter((s: any) => !s.completed)
          .map((s: any) => ({
            id: s.subject.id,
            name: s.subject.name,
            default_duration: parseFloat(s.subject.default_duration) || 0,
            danger_level: s.subject.danger_level,
            required_pilots: s.subject.required_pilots || 1, // 新增字段
          }));
        return {
          id: plane.id,
          plane_no: plane.plane_no,
          model_name: detail.model?.name || "未知机型",
          status: plane.status,
          availableSubjects: subjects,
          selectedSubjects: [] as number[],
          tasks: [],
        };
      })
    );
    aircraftList.value = aircraftDetails;
    aircraftDetails.forEach((ac) => {
      aircraftCollapsed.value[ac.id] = false;
    });

    const pilotsRes = await getPilots({ page: 1, page_size: 1000 });
    const pilotsData = Array.isArray(pilotsRes)
      ? pilotsRes
      : pilotsRes.list || [];
    const pilotsWithModels = await Promise.all(
      pilotsData.map(async (pilot: any) => {
        try {
          const models = await getPilotModels(pilot.id);
          const allowedModels = models.map((m: any) => m.name);
          return { ...pilot, allowedModels };
        } catch (error) {
          console.error(`获取飞行员 ${pilot.id} 可飞机型失败`, error);
          return { ...pilot, allowedModels: [] };
        }
      })
    );
    const pilots: Pilot[] = pilotsWithModels.map((pilot) => {
      const allowedModels = pilot.allowedModels;
      const subjectMap = new Map<number, Subject>();
      for (const ac of aircraftDetails) {
        if (allowedModels.includes(ac.model_name)) {
          ac.availableSubjects.forEach((subj) => {
            if (!subjectMap.has(subj.id)) subjectMap.set(subj.id, subj);
          });
        }
      }
      return {
        id: pilot.id,
        name: pilot.username,
        level: pilot.level || "二级飞行员",
        allowedModels,
        availableSubjects: Array.from(subjectMap.values()),
        selectedSubjects: [],
        mentalStatus: pilot.mental_status || "稳定",
        healthStatus: pilot.health_status || "良好",
      };
    });
    pilotList.value = pilots;
    pilots.forEach((pilot) => {
      pilotCollapsed.value[pilot.id] = false;
    });

    const commanders = await getRecordsByType("COMMANDER");
    const commanderPromises = commanders.map(async (p: any) => {
      const name = (await getUser(p.subject_id)).real_name || "未知";
      const attr = parseJsonAttr<CommanderAttr>(p.attr_key_value);
      return {
        id: p.id,
        attr,
        name,
        level: attr.level || "",
        selected: false,
      };
    });
    commanderList.value = await Promise.all(commanderPromises);
  } catch (error) {
    console.error(error);
    ElMessage.error("加载数据失败");
  }
};

const getSubjectName = (subjects: Subject[], id: number) =>
  subjects.find((s) => s.id === id)?.name || "";

const toggleSelectAllPilot = (pilot: Pilot) => {
  if (pilot.selectedSubjects.length === pilot.availableSubjects.length) {
    pilot.selectedSubjects = [];
  } else {
    pilot.selectedSubjects = pilot.availableSubjects.map((s) => s.id);
  }
  pilot.selectedSubjects = [...pilot.selectedSubjects];
};
const toggleSelectAllAircraft = (ac: Aircraft) => {
  if (ac.selectedSubjects.length === ac.availableSubjects.length) {
    ac.selectedSubjects = [];
  } else {
    ac.selectedSubjects = ac.availableSubjects.map((s) => s.id);
  }
  ac.selectedSubjects = [...ac.selectedSubjects];
};

const updatePilotSubjects = (pilot: Pilot) => {};
const updateAircraftSubjects = (ac: Aircraft) => {};

const toggleSelectAllPilots = () => {
  const targetState = !allPilotsSelected.value;
  pilotList.value.forEach((pilot) => {
    if (targetState) {
      pilot.selectedSubjects = pilot.availableSubjects.map((s) => s.id);
    } else {
      pilot.selectedSubjects = [];
    }
    pilot.selectedSubjects = [...pilot.selectedSubjects];
  });
};
const toggleSelectAllAircrafts = () => {
  const targetState = !allAircraftsSelected.value;
  aircraftList.value.forEach((ac) => {
    if (targetState) {
      ac.selectedSubjects = ac.availableSubjects.map((s) => s.id);
    } else {
      ac.selectedSubjects = [];
    }
    ac.selectedSubjects = [...ac.selectedSubjects];
  });
};
const toggleSelectAllCommanders = () => {
  const targetState = !allCommandersSelected.value;
  commanderList.value.forEach((c) => (c.selected = targetState));
};

// ---------- 冲突检测（单飞行员拖拽用） ----------
const checkConflict = (
  aircraftId: number,
  startMin: number,
  endMin: number,
  excludeTaskId?: number
): boolean => {
  const aircraft = aircraftList.value.find((a) => a.id === aircraftId);
  if (!aircraft) return false;

  if (constraintConfig.enableWorkTime) {
    if (
      startMin < constraintConfig.workStart ||
      endMin > constraintConfig.workEnd
    )
      return true;
  }

  for (const t of aircraft.tasks) {
    if (excludeTaskId && t.id === excludeTaskId) continue;
    if (!(endMin <= t.startMin || startMin >= t.endMin)) return true;
    if (constraintConfig.enableAircraftInterval) {
      const gapBefore = t.endMin + constraintConfig.minAircraftInterval;
      const gapAfter = t.startMin - constraintConfig.minAircraftInterval;
      if (!(endMin <= gapAfter || startMin >= gapBefore)) return true;
    }
  }

  let pilotId: number | null = null;
  if (draggingPayload?.pilotId) pilotId = draggingPayload.pilotId;
  else if (draggingTask?.pilotId) pilotId = draggingTask.pilotId;
  if (pilotId) {
    for (const ac of aircraftList.value) {
      for (const t of ac.tasks) {
        if (excludeTaskId && t.id === excludeTaskId) continue;
        if (
          t.pilotId === pilotId ||
          (Array.isArray(t.pilotId) && t.pilotId.includes(pilotId))
        ) {
          if (!(endMin <= t.startMin || startMin >= t.endMin)) return true;
          if (constraintConfig.enablePilotInterval) {
            const gapBefore = t.endMin + constraintConfig.minPilotInterval;
            const gapAfter = t.startMin - constraintConfig.minPilotInterval;
            if (!(endMin <= gapAfter || startMin >= gapBefore)) return true;
          }
        }
      }
    }
  }
  return false;
};

// ---------- 多飞行员冲突检测（用于编排） ----------
const checkConflictMulti = (
  aircraftId: number,
  startMin: number,
  endMin: number,
  pilotIds: number[],
  excludeTaskId?: number
): boolean => {
  const aircraft = aircraftList.value.find((a) => a.id === aircraftId);
  if (!aircraft) return false;

  if (constraintConfig.enableWorkTime) {
    if (
      startMin < constraintConfig.workStart ||
      endMin > constraintConfig.workEnd
    )
      return true;
  }

  // 飞机冲突
  for (const t of aircraft.tasks) {
    if (excludeTaskId && t.id === excludeTaskId) continue;
    if (!(endMin <= t.startMin || startMin >= t.endMin)) return true;
    if (constraintConfig.enableAircraftInterval) {
      const gapBefore = t.endMin + constraintConfig.minAircraftInterval;
      const gapAfter = t.startMin - constraintConfig.minAircraftInterval;
      if (!(endMin <= gapAfter || startMin >= gapBefore)) return true;
    }
  }

  // 飞行员冲突
  for (const pid of pilotIds) {
    for (const ac of aircraftList.value) {
      for (const t of ac.tasks) {
        if (excludeTaskId && t.id === excludeTaskId) continue;
        const taskPilots = Array.isArray(t.pilotId) ? t.pilotId : [t.pilotId];
        if (taskPilots.includes(pid)) {
          if (!(endMin <= t.startMin || startMin >= t.endMin)) return true;
          if (constraintConfig.enablePilotInterval) {
            const gapBefore = t.endMin + constraintConfig.minPilotInterval;
            const gapAfter = t.startMin - constraintConfig.minPilotInterval;
            if (!(endMin <= gapAfter || startMin >= gapBefore)) return true;
          }
        }
      }
    }
  }
  return false;
};

// ---------- 拖拽事件 ----------
const onDragStartPilotTask = (e: DragEvent, payload: any) => {
  e.dataTransfer?.setData(
    "text/plain",
    JSON.stringify({ type: "pilotTask", ...payload })
  );
  e.dataTransfer!.effectAllowed = "move";
  const dragImage = new Image();
  dragImage.src =
    "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";
  e.dataTransfer?.setDragImage(dragImage, 0, 0);
  draggingPayload = payload;
};

const onDragStartTask = (e: DragEvent, task: any) => {
  e.dataTransfer?.setData(
    "text/plain",
    JSON.stringify({ type: "task", taskId: task.id })
  );
  e.dataTransfer!.effectAllowed = "move";
  const dragImage = new Image();
  dragImage.src =
    "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";
  e.dataTransfer?.setDragImage(dragImage, 0, 0);
  draggingTask = task;
};

const onDragEnd = () => {
  draggingPayload = null;
  draggingTask = null;
};

const onDrop = async (
  e: DragEvent,
  aircraftId: number,
  slotStartMin: number
) => {
  e.preventDefault();
  const raw = e.dataTransfer?.getData("text/plain");
  if (!raw) return;
  const data = JSON.parse(raw);

  if (data.type === "pilotTask") {
    const pilot = pilotList.value.find((p) => p.id === data.pilotId);
    if (!pilot) return;
    const subject = pilot.availableSubjects.find(
      (s) => s.id === data.subjectId
    );
    if (!subject) return;
    const aircraft = aircraftList.value.find((a) => a.id === aircraftId);
    if (!aircraft) return;
    if (!aircraft.availableSubjects.some((s) => s.id === subject.id)) {
      ElMessage.warning("该飞机不能执行此科目");
      return;
    }
    if (
      constraintConfig.matchAircraftType &&
      !pilot.allowedModels.includes(aircraft.model_name)
    ) {
      ElMessage.warning("飞行员不可飞该机型");
      return;
    }
    if (
      constraintConfig.enableMentalCheck &&
      pilot.mentalStatus !== "优秀" &&
      pilot.mentalStatus !== "良好"
    ) {
      ElMessage.warning("该飞行员心理状态不符合要求");
      return;
    }
    if (constraintConfig.enableHealthCheck && pilot.healthStatus !== "良好") {
      ElMessage.warning("该飞行员健康状态不符合要求");
      return;
    }
    if (
      constraintConfig.enableHighRiskLevel1 &&
      subject.danger_level === "高危" &&
      pilot.level !== "一级飞行员"
    ) {
      ElMessage.warning("高危任务需一级飞行员");
      return;
    }
    const duration = subject.default_duration;
    if (
      constraintConfig.maxDuration > 0 &&
      duration > constraintConfig.maxDuration
    ) {
      ElMessage.warning(
        `任务时长 ${duration}h 超过最大限制 ${constraintConfig.maxDuration}h`
      );
      return;
    }
    const endMin = slotStartMin + duration * 60;
    if (endMin > 24 * 60) {
      ElMessage.warning("超出时间范围");
      return;
    }
    if (checkConflict(aircraftId, slotStartMin, endMin)) {
      ElMessage.warning("时间冲突或违反间隔约束");
      return;
    }
    const newTask = {
      id: Date.now() + Math.random(),
      aircraftId,
      pilotId: pilot.id,
      pilotName: pilot.name,
      subjectId: subject.id,
      subjectName: subject.name,
      startMin: slotStartMin,
      endMin,
      duration,
    };
    aircraft.tasks.push(newTask);
    sortTasks(aircraft);
    ElMessage.success("任务已添加");
  } else if (data.type === "task") {
    if (!draggingTask) return;
    const task = draggingTask;
    const oldAircraft = aircraftList.value.find(
      (a) => a.id === task.aircraftId
    );
    if (!oldAircraft) return;
    const newStart = slotStartMin;
    const newEnd = newStart + task.duration * 60;
    if (newEnd > 24 * 60) {
      ElMessage.warning("超出时间范围");
      return;
    }
    oldAircraft.tasks = oldAircraft.tasks.filter((t: any) => t.id !== task.id);
    const conflict = checkConflict(aircraftId, newStart, newEnd, task.id);
    if (conflict) {
      oldAircraft.tasks.push(task);
      sortTasks(oldAircraft);
      ElMessage.warning("目标位置冲突或违反间隔约束，移动失败");
      return;
    }
    task.aircraftId = aircraftId;
    task.startMin = newStart;
    task.endMin = newEnd;
    const newAircraft = aircraftList.value.find((a) => a.id === aircraftId);
    if (newAircraft) {
      newAircraft.tasks.push(task);
      sortTasks(newAircraft);
    } else {
      oldAircraft.tasks.push(task);
      sortTasks(oldAircraft);
      ElMessage.warning("目标飞机不存在");
      return;
    }
    sortTasks(oldAircraft);
    ElMessage.success("任务已移动");
    draggingTask = null;
  }
};

const sortTasks = (aircraft: Aircraft) => {
  aircraft.tasks.sort((a, b) => a.startMin - b.startMin);
};

const deleteTask = (task: any) => {
  const aircraft = aircraftList.value.find((a) => a.id === task.aircraftId);
  if (aircraft) {
    aircraft.tasks = aircraft.tasks.filter((t: any) => t.id !== task.id);
    ElMessage.success("删除成功");
  }
};

const editTask = (task: any) => {
  editingTask = task;
  editForm.value = {
    id: task.id,
    startTime: formatTime(task.startMin),
    duration: task.duration,
  };
  editDialogVisible.value = true;
};

const saveEditTask = () => {
  if (!editingTask) return;
  const newStart = timeToMinutes(editForm.value.startTime);
  const newEnd = newStart + editForm.value.duration * 60;
  if (newEnd > 24 * 60) {
    ElMessage.warning("超出24点");
    return;
  }
  const aircraft = aircraftList.value.find(
    (a) => a.id === editingTask.aircraftId
  );
  if (!aircraft) return;
  const oldTasks = [...aircraft.tasks];
  aircraft.tasks = aircraft.tasks.filter((t: any) => t.id !== editingTask.id);
  const pilots = Array.isArray(editingTask.pilotId)
    ? editingTask.pilotId
    : [editingTask.pilotId];
  const newTask = {
    ...editingTask,
    startMin: newStart,
    endMin: newEnd,
    duration: editForm.value.duration,
  };
  if (checkConflictMulti(aircraft.id, newStart, newEnd, pilots, newTask.id)) {
    ElMessage.warning("冲突");
    aircraft.tasks = oldTasks;
    return;
  }
  aircraft.tasks.push(newTask);
  sortTasks(aircraft);
  editDialogVisible.value = false;
  ElMessage.success("修改成功");
};

// ---------- 贪心自动编排（支持多飞行员） ----------
const autoSchedule = () => {
  aircraftList.value.forEach((ac) => (ac.tasks = []));

  // 收集所有任务需求：每个飞机科目为一个任务，需要 required_pilots 个飞行员
  const taskRequirements: {
    aircraftId: number;
    subjectId: number;
    subjectName: string;
    duration: number;
    danger_level: string;
    required_pilots: number;
  }[] = [];

  // 只考虑已勾选的飞机和科目
  const selectedAircrafts = aircraftList.value.filter(
    (ac) => ac.selectedSubjects.length > 0
  );
  for (const ac of selectedAircrafts) {
    for (const subjId of ac.selectedSubjects) {
      const subject = ac.availableSubjects.find((s) => s.id === subjId);
      if (!subject) continue;
      // 必须有至少一个飞行员选择了该科目才创建需求
      const hasPilot = pilotList.value.some((p) =>
        p.selectedSubjects.includes(subjId)
      );
      if (!hasPilot) continue;
      taskRequirements.push({
        aircraftId: ac.id,
        subjectId: subject.id,
        subjectName: subject.name,
        duration: subject.default_duration,
        danger_level: subject.danger_level,
        required_pilots: subject.required_pilots || 1,
      });
    }
  }

  // 按高危优先、所需飞行员数量降序排序
  taskRequirements.sort((a, b) => {
    if (a.danger_level === "高危" && b.danger_level !== "高危") return -1;
    if (a.danger_level !== "高危" && b.danger_level === "高危") return 1;
    return b.required_pilots - a.required_pilots;
  });

  const step = use10minSlot.value ? 10 : 60;
  const maxSlots = timeSlots.value.length;

  // 为每个需求寻找时间槽和飞行员
  for (const req of taskRequirements) {
    const aircraft = aircraftList.value.find((a) => a.id === req.aircraftId);
    if (!aircraft) continue;

    // 候选飞行员：选择了该科目，且满足机型、健康、心理、等级等约束
    const candidates = pilotList.value.filter((p) => {
      if (!p.selectedSubjects.includes(req.subjectId)) return false;
      if (
        constraintConfig.matchAircraftType &&
        !p.allowedModels.includes(aircraft.model_name)
      )
        return false;
      if (
        constraintConfig.enableMentalCheck &&
        p.mentalStatus !== "优秀" &&
        p.mentalStatus !== "良好"
      )
        return false;
      if (constraintConfig.enableHealthCheck && p.healthStatus !== "良好")
        return false;
      if (
        constraintConfig.enableHighRiskLevel1 &&
        req.danger_level === "高危" &&
        p.level !== "一级飞行员"
      )
        return false;
      return true;
    });

    if (candidates.length < req.required_pilots) {
      console.warn(
        `科目 ${req.subjectName} 所需飞行员不足 (需要 ${req.required_pilots}, 可用 ${candidates.length})`
      );
      continue;
    }

    // 按某种优先级排序（例如等级、飞行时间少优先），这里简单按 ID 排序
    candidates.sort((a, b) => a.id - b.id);

    // 尝试在每个时间槽分配
    let assigned = false;
    for (let slotIdx = 0; slotIdx <= maxSlots; slotIdx++) {
      const startMin = slotIdx * step;
      const endMin = startMin + req.duration * 60;
      if (endMin > 24 * 60) break;

      // 工作时间检查
      if (constraintConfig.enableWorkTime) {
        if (
          startMin < constraintConfig.workStart ||
          endMin > constraintConfig.workEnd
        )
          continue;
      }

      // 检查飞机是否可用
      let aircraftConflict = false;
      for (const t of aircraft.tasks) {
        if (!(endMin <= t.startMin || startMin >= t.endMin)) {
          aircraftConflict = true;
          break;
        }
        if (constraintConfig.enableAircraftInterval) {
          const gapBefore = t.endMin + constraintConfig.minAircraftInterval;
          const gapAfter = t.startMin - constraintConfig.minAircraftInterval;
          if (!(endMin <= gapAfter || startMin >= gapBefore)) {
            aircraftConflict = true;
            break;
          }
        }
      }
      if (aircraftConflict) continue;

      // 尝试选择 req.required_pilots 个飞行员，要求他们在此时间段空闲且满足间隔
      const selectedPilots: Pilot[] = [];
      const usedPilotIds = new Set<number>();
      for (const pilot of candidates) {
        if (usedPilotIds.size >= req.required_pilots) break;
        // 检查此飞行员是否与已有任务冲突
        let pilotConflict = false;
        for (const ac of aircraftList.value) {
          for (const t of ac.tasks) {
            const taskPilots = Array.isArray(t.pilotId)
              ? t.pilotId
              : [t.pilotId];
            if (taskPilots.includes(pilot.id)) {
              if (!(endMin <= t.startMin || startMin >= t.endMin)) {
                pilotConflict = true;
                break;
              }
              if (constraintConfig.enablePilotInterval) {
                const gapBefore = t.endMin + constraintConfig.minPilotInterval;
                const gapAfter = t.startMin - constraintConfig.minPilotInterval;
                if (!(endMin <= gapAfter || startMin >= gapBefore)) {
                  pilotConflict = true;
                  break;
                }
              }
            }
          }
          if (pilotConflict) break;
        }
        // 还要检查是否与已选中的飞行员冲突（他们将在同一时间执行任务，所以相互之间也需要检查？实际上同一个任务内部飞行员是协同的，不需要间隔，但间隔约束应该仅针对不同任务。这里不需要检查彼此。）
        if (!pilotConflict) {
          selectedPilots.push(pilot);
          usedPilotIds.add(pilot.id);
        }
      }

      if (selectedPilots.length >= req.required_pilots) {
        // 分配任务
        const pilotIds = selectedPilots.map((p) => p.id);
        const pilotNames = selectedPilots.map((p) => p.name);
        aircraft.tasks.push({
          id: Date.now() + Math.random(),
          aircraftId: req.aircraftId,
          pilotId: pilotIds,
          pilotName: pilotNames.join("、"),
          subjectId: req.subjectId,
          subjectName: req.subjectName,
          startMin,
          endMin,
          duration: req.duration,
        });
        sortTasks(aircraft);
        assigned = true;
        break;
      }
    }
    if (!assigned) {
      console.warn(
        `科目 ${req.subjectName} 在飞机 ${aircraft.plane_no} 上无法调度`
      );
    }
  }
  ElMessage.success("贪心编排完成");
};

// ---------- MILP智能编排 ----------
const milpSchedule = async () => {
  if (!showSchedule.value) {
    ElMessage.warning("请至少为一架飞机勾选科目");
    return;
  }

  const selectedPilots = pilotList.value.filter(
    (p) => p.selectedSubjects.length > 0
  );
  const pilotInfos: PilotInfo[] = selectedPilots.map((p) => ({
    id: p.id,
    health_status: p.healthStatus,
    mental_status: p.mentalStatus,
    level: p.level,
    allowed_aircraft_models: p.allowedModels,
    max_daily_flight_hours: constraintConfig.maxDailyFlightHours,
  }));

  const selectedAircrafts = aircraftList.value.filter(
    (ac) => ac.selectedSubjects.length > 0
  );
  const aircraftInfos: AircraftInfo[] = selectedAircrafts.map((ac) => ({
    id: ac.id,
    model: ac.model_name,
    plane_no: ac.plane_no,
  }));

  const tasks: TaskInfo[] = [];
  selectedAircrafts.forEach((ac) => {
    ac.selectedSubjects.forEach((subjId) => {
      const subject = ac.availableSubjects.find((s) => s.id === subjId);
      if (!subject) return;
      tasks.push({
        id: `${ac.id}_${subject.name}`,
        aircraft_id: ac.id,
        duration: subject.default_duration,
        weight: 1,
        risk: subject.danger_level === "高危" ? 1 : 0,
        required_pilots: subject.required_pilots || 1, // 使用实际所需人数
      });
    });
  });

  if (tasks.length === 0) {
    ElMessage.warning("没有可执行的任务");
    return;
  }

  const requestData: MilpRequest = {
    pilots: pilotInfos,
    aircrafts: aircraftInfos,
    tasks: tasks,
    constraints: {
      enable_work_time: constraintConfig.enableWorkTime,
      work_start: Math.floor(constraintConfig.workStart / 60),
      work_end: Math.floor(constraintConfig.workEnd / 60),
      check_health: constraintConfig.enableHealthCheck,
      check_mental: constraintConfig.enableMentalCheck,
      match_aircraft_type: constraintConfig.matchAircraftType,
      high_risk_need_level1: constraintConfig.enableHighRiskLevel1,
      max_duration: constraintConfig.maxDuration,
      max_daily_flight_hours: constraintConfig.maxDailyFlightHours,
    },
  };

  console.log("MILP请求数据:", JSON.stringify(requestData, null, 2));
  try {
    const res = await runMilpSchedule(requestData);
    if (res.status !== "Optimal") {
      ElMessage.warning(`未找到最优解，状态：${res.status}，请调整约束`);
      return;
    }

    aircraftList.value.forEach((ac) => (ac.tasks = []));
    const newTasksMap = new Map<number, any[]>();
    for (const assign of res.assignments) {
      const aircraftId = assign.aircraft_id;
      if (!newTasksMap.has(aircraftId)) newTasksMap.set(aircraftId, []);
      const subjectName = assign.task_id.substring(
        String(aircraftId).length + 1
      );
      const pilotNames = assign.pilot_id.map((pid: number) => {
        const pilot = pilotList.value.find((p) => p.id === pid);
        return pilot ? pilot.name : `飞行员${pid}`;
      });
      const startMin = assign.start_hour * 60;
      const endMin = startMin + assign.duration * 60;
      newTasksMap.get(aircraftId)!.push({
        id: Date.now() + Math.random(),
        aircraftId: aircraftId,
        pilotId: assign.pilot_id,
        pilotName: pilotNames.join("、"),
        subjectId: 0,
        subjectName: subjectName,
        startMin,
        endMin,
        duration: assign.duration,
      });
    }

    for (const [acId, tasks] of newTasksMap) {
      const aircraft = aircraftList.value.find((a) => a.id === acId);
      if (aircraft) {
        aircraft.tasks = tasks;
        sortTasks(aircraft);
      }
    }

    ElMessage.success(`MILP 编排完成，目标值：${res.objective.toFixed(2)}`);
  } catch (error) {
    ElMessage.error("MILP 调度请求失败");
    console.error(error);
  }
};

const clearAll = () => {
  aircraftList.value.forEach((ac) => (ac.tasks = []));
  ElMessage.success("已清空");
};

// ========== 导出计划 ==========
const exportSchedule = async () => {
  const activePlanes = filteredAircrafts.value;
  const activePilotIds = new Set<number>();
  activePlanes.forEach((ac) => {
    ac.tasks.forEach((t: any) => {
      const pilots = Array.isArray(t.pilotId) ? t.pilotId : [t.pilotId];
      pilots.forEach((pid: number) => activePilotIds.add(pid));
    });
  });
  const activePilots = pilotList.value.filter((p) => activePilotIds.has(p.id));
  const activeCommanders = commanderList.value;

  const taskRows: any[] = [];
  activePlanes.forEach((ac) => {
    ac.tasks.forEach((t: any) => {
      taskRows.push({
        飞机ID: ac.id,
        飞机编号: ac.plane_no,
        机型: ac.model_name,
        飞行员ID: Array.isArray(t.pilotId) ? t.pilotId.join(",") : t.pilotId,
        飞行员: t.pilotName,
        科目: t.subjectName,
        开始时间: formatTime(t.startMin),
        结束时间: formatTime(t.endMin),
        "时长(小时)": t.duration,
      });
    });
  });
  const ws1 = XLSX.utils.json_to_sheet(taskRows);
  const ws2 = XLSX.utils.json_to_sheet([
    ...activePlanes.map((ac) => ({
      类型: "飞机",
      ID: ac.id,
      "编号/姓名": ac.plane_no,
      "机型/备注": ac.model_name,
    })),
    ...activePilots.map((p) => ({
      类型: "飞行员",
      ID: p.id,
      "编号/姓名": p.name,
      "机型/备注": p.level || "",
    })),
    ...activeCommanders.map((c) => ({
      类型: "指挥员",
      ID: c.id,
      "编号/姓名": c.name,
      "机型/备注": c.level || "",
    })),
  ]);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws1, "飞行计划");
  XLSX.utils.book_append_sheet(wb, ws2, "ID映射");

  const excelBuffer = XLSX.write(wb, { bookType: "xlsx", type: "array" });
  const blob = new Blob([excelBuffer], {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
  const today = new Date().toISOString().slice(0, 10);
  const fileName = `${today}-未审批.xlsx`;

  try {
    const formData = new FormData();
    formData.append("file", blob, fileName);
    const res = await uploadFile(formData, fileName);
    ElMessage.success(`计划已导出至 MinIO，文件名：${fileName}`);
    console.log("MinIO key:", res.key);
  } catch (error) {
    ElMessage.error("导出至 MinIO 失败");
  }
};

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.flight-schedule-new {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
  padding: 16px;
}
.toolbar {
  margin-bottom: 16px;
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.main-layout {
  display: flex;
  flex: 1;
  gap: 16px;
  overflow: hidden;
}
.left-panel {
  width: 340px;
  flex-shrink: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.pilot-card,
.aircraft-card,
.commander-card {
  border-radius: 12px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  user-select: none;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.pilot-item,
.aircraft-item,
.commander-item {
  margin-bottom: 8px;
}
.commander-item {
  padding: 4px 0;
}
.pilot-header,
.aircraft-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  cursor: pointer;
  user-select: none;
}
.pilot-name,
.aircraft-name {
  font-weight: bold;
}
.pilot-info,
.aircraft-status {
  font-size: 12px;
  color: #666;
  margin-left: 8px;
}
.toggle-icon {
  font-size: 14px;
  color: #909399;
  margin-left: 8px;
}
.empty-subject {
  font-size: 12px;
  color: #aaa;
  padding: 4px 0;
}
.draggable-tasks {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.draggable-task {
  background: #ecf5ff;
  border-radius: 16px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: grab;
  display: inline-block;
}
.right-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
  padding: 8px;
}
.empty-schedule {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #909399;
  background: white;
  border-radius: 12px;
}
.schedule-wrapper {
  flex: 1;
  overflow-x: auto;
  overflow-y: auto;
}
.time-header {
  display: flex;
  background: #fafbfc;
  border-bottom: 1px solid #eee;
  position: sticky;
  top: 0;
  z-index: 10;
}
.plane-label-col {
  width: 120px;
  flex-shrink: 0;
  text-align: center;
  line-height: 40px;
  border-right: 1px solid #eee;
  font-weight: bold;
}
.time-cells {
  display: flex;
}
.time-cell {
  flex-shrink: 0;
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
  height: 70px;
}
.plane-label {
  width: 120px;
  flex-shrink: 0;
  text-align: center;
  line-height: 70px;
  border-right: 1px solid #eee;
  font-weight: 500;
}
.timeline {
  position: relative;
  height: 70px;
}
.time-slot {
  display: inline-block;
  height: 70px;
  border-right: 1px dashed #eee;
}
.task-card {
  position: absolute;
  top: 5px;
  height: 60px;
  background: #ecf5ff;
  border-left: 4px solid #409eff;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: grab;
  user-select: none;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.task-title {
  font-weight: bold;
  font-size: 11px;
}
.task-time {
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
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}
.constraint-tip {
  color: #666;
  font-size: 12px;
}
</style>