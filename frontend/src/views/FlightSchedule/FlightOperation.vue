<template>
  <div class="import-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="header-title">试飞计划导入与执行</span>
          <div class="header-actions">
            <el-button type="primary" @click="uploadExcel" :loading="loading">
              <el-icon><Upload /></el-icon> 手动导入 Excel
            </el-button>
            <el-button
              type="success"
              @click="autoImport"
              :loading="autoLoading"
            >
              自动加载当日审批通过计划
            </el-button>
          </div>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <!-- 时间表（甘特图） -->
        <el-tab-pane label="时间表" name="schedule">
          <div v-if="scheduleData.length === 0" class="empty-placeholder">
            暂无数据，请自动加载或手动导入 Excel
          </div>
          <div v-else class="schedule-wrapper">
            <div class="time-header">
              <div class="plane-label"></div>
              <div v-for="h in 24" :key="h" class="hour-cell">
                {{ (h - 1).toString().padStart(2, "0") }}:00
              </div>
            </div>
            <div class="schedule-body">
              <div
                v-for="(row, idx) in scheduleData"
                :key="idx"
                class="plane-row"
              >
                <div class="plane-label">
                  {{ row.plane.model || `飞机${row.plane.id}` }}
                </div>
                <div class="timeline" :style="{ width: 24 * 60 + 'px' }">
                  <div
                    v-for="h in 24"
                    :key="h"
                    class="hour-slot"
                    :style="{ width: '60px' }"
                  ></div>
                  <div
                    v-for="task in row.tasks"
                    :key="task.id"
                    class="task-card readonly"
                    :style="{
                      left: task.startHour * 60 + 'px',
                      width: task.duration * 60 + 'px',
                    }"
                  >
                    <div class="task-title">
                      {{ task.pilotNames.join("、") }}
                    </div>
                    <div class="task-time">
                      {{ task.start }} ~ {{ task.end }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- 执行监控 -->
        <el-tab-pane label="执行监控" name="monitor">
          <div v-if="executionRows.length === 0" class="empty-placeholder">
            暂无任务数据，请先加载计划
          </div>
          <div v-else class="monitor-container">
            <div class="section-card">
              <h3>任务执行状态</h3>
              <el-table :data="executionRows" border stripe>
                <el-table-column prop="aircraftId" label="飞机ID" width="80" />
                <el-table-column prop="planeNo" label="飞机编号" width="120" />
                <el-table-column
                  prop="subjectName"
                  label="科目名称"
                  min-width="140"
                />
                <el-table-column label="飞行员" min-width="150">
                  <template #default="{ row }">
                    {{ row.pilotNames.join("、") || "-" }}
                  </template>
                </el-table-column>
                <el-table-column label="完成状态" width="120">
                  <template #default="{ row }">
                    <el-switch
                      v-model="row.completed"
                      active-text="已完成"
                      inactive-text="未完成"
                      @change="handleCompleteChange(row)"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="操作时长(h)" width="160">
                  <template #default="{ row }">
                    <el-input-number
                      :model-value="row.operationHours"
                      :min="0"
                      :step="0.5"
                      :precision="1"
                      size="small"
                      @change="(val: number | undefined) => handleHoursChange(row, 'operation', val)"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="飞行时长(h)" width="160">
                  <template #default="{ row }">
                    <el-input-number
                      :model-value="row.flightHours"
                      :min="0"
                      :step="0.5"
                      :precision="1"
                      size="small"
                      @change="(val: number | undefined) => handleHoursChange(row, 'flight', val)"
                    />
                  </template>
                </el-table-column>
              </el-table>
            </div>

            <!-- 飞行员信息 -->
            <div class="section-card">
              <h3>飞行员信息</h3>
              <el-table :data="pilotInfoList" border stripe>
                <el-table-column prop="id" label="ID" width="80" />
                <el-table-column label="姓名">
                  <template #default="{ row }">
                    {{ row.name }}
                  </template>
                </el-table-column>
                <el-table-column
                  prop="totalFlightHours"
                  label="总飞行时间(h)"
                />
                <el-table-column label="操作" width="180">
                  <template #default="{ row }">
                    <el-button
                      size="small"
                      @click="openAddFlightHoursDialog(row)"
                    >
                      增加飞行时间
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>

            <!-- 指挥员指挥时间 -->
            <div class="section-card">
              <h3>指挥员指挥时间</h3>
              <el-table :data="commanderList" border stripe>
                <el-table-column prop="name" label="姓名" />
                <el-table-column label="指挥时间(h)" width="200">
                  <template #default="{ row }">
                    <el-input-number
                      v-model="row.totalCommandHours"
                      :min="0"
                      :step="0.5"
                      :precision="1"
                      size="small"
                      @change="saveCommanderHours(row)"
                    />
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 增加飞行员飞行时间弹窗 -->
    <el-dialog
      v-model="addFlightDialogVisible"
      title="增加飞行时间"
      width="400px"
    >
      <el-form label-width="100px">
        <el-form-item label="飞行员">
          <span>{{ currentEditPilot?.name }}</span>
        </el-form-item>
        <el-form-item label="机型">
          <el-select v-model="addFlightModel" placeholder="选择机型">
            <el-option
              v-for="m in availableModels"
              :key="m"
              :label="m"
              :value="m"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="时长(h)">
          <el-input-number
            v-model="addFlightHours"
            :min="0.5"
            :step="0.5"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addFlightDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmAddFlightHours"
          >确定</el-button
        >
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import { Upload } from "@element-plus/icons-vue";
import * as XLSX from "xlsx";
import { setAttributes } from "@/api/subjectAttribute";
import { getAttributes as getResourceAttributes } from "@/api/resourceAttribute";
import { listFiles, getFileUrl } from "@/api/minio";
import { getRecordsByType } from "@/api/subjectAttribute";
import { getUser } from "@/api/user";
import {
  getAircraftDetail,
  updateAircraft,
  finishFlight,
  getPilots,
  updatePilot,
  listModels,
} from "@/api/aircraft";
import { upsertPilotModelFlightTime } from "@/api/pilotModelFlightTime";

// ========== 类型定义 ==========
interface Task {
  id: number;
  pilotIds: number[];
  pilotNames: string[];
  planeId: number;
  startHour: number;
  duration: number;
  start: string;
  end: string;
}

interface PlaneRow {
  plane: { id: number; model: string };
  tasks: Task[];
}

interface PilotInfo {
  id: number;
  name: string;
  totalFlightHours: number;
}

interface MonitorTask {
  taskId: number;
  planeId: number;
  planeModel: string;
  pilotIds: number[];
  pilotNames: string[];
  subjectName: string;
}

interface ExecutionRow {
  aircraftId: number;
  planeNo: string;
  subjectId: number;
  subjectName: string;
  pilotNames: string[];
  pilotIds: number[];
  completed: boolean;
  operationHours: number;
  flightHours: number;
  origOperationHours: number;
  origFlightHours: number;
  origCompleted: boolean;
  modelName: string; // 新增：机型名称，用于更新飞行员机型时间
}

interface CommanderInfo {
  id: number;
  name: string;
  totalCommandHours: number;
}

// ========== 响应式数据 ==========
const activeTab = ref("schedule");
const loading = ref(false);
const autoLoading = ref(false);
const scheduleData = ref<PlaneRow[]>([]);
const pilotInfoList = ref<PilotInfo[]>([]);
const taskList = ref<MonitorTask[]>([]);
const executionRows = ref<ExecutionRow[]>([]);
const pilotInfoMap = ref<Map<number, PilotInfo>>(new Map());
const commanderList = ref<CommanderInfo[]>([]);
const availableModels = ref<string[]>([]);
const modelNameToId = ref<Map<string, number>>(new Map()); // 机型名称 -> 机型ID 映射

const addFlightDialogVisible = ref(false);
const currentEditPilot = ref<PilotInfo | null>(null);
const addFlightModel = ref("");
const addFlightHours = ref(1);

// ========== 解析飞行计划 Sheet（支持多飞行员） ==========
const parseFlightPlanSheet = (
  rows: any[][]
): {
  scheduleData: PlaneRow[];
  monitorTasks: MonitorTask[];
  pilotIds: Set<number>;
  planeIds: Set<number>;
} => {
  if (rows.length < 2) {
    return {
      scheduleData: [],
      monitorTasks: [],
      pilotIds: new Set(),
      planeIds: new Set(),
    };
  }
  const header = rows[0].map((h: any) => String(h).trim());
  const colIdx: Record<string, number> = {};
  for (let i = 0; i < header.length; i++) colIdx[header[i]] = i;

  const requiredCols = [
    "飞机ID",
    "飞机编号",
    "机型",
    "飞行员ID",
    "飞行员",
    "科目",
    "开始时间",
    "结束时间",
    "时长(小时)",
  ];
  for (const col of requiredCols) {
    if (!(col in colIdx)) {
      ElMessage.error(`飞行计划表缺少 ${col} 列`);
      return {
        scheduleData: [],
        monitorTasks: [],
        pilotIds: new Set(),
        planeIds: new Set(),
      };
    }
  }

  const planeTasksMap = new Map<
    number,
    { planeNo: string; model: string; tasks: Task[] }
  >();
  const monitorTasks: MonitorTask[] = [];
  const pilotIds = new Set<number>();
  const planeIds = new Set<number>();

  const splitIds = (str: string): number[] =>
    str
      .split(",")
      .map((s) => parseInt(s.trim(), 10))
      .filter((n) => !isNaN(n));
  const splitNames = (str: string): string[] =>
    str
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);

  for (let i = 1; i < rows.length; i++) {
    const row = rows[i];
    if (!row || row.length === 0) continue;

    const aircraftId = Number(row[colIdx["飞机ID"]]);
    const planeNo = String(row[colIdx["飞机编号"]]).trim();
    const model = String(row[colIdx["机型"]]).trim();
    const pilotIdStr = String(row[colIdx["飞行员ID"]]).trim();
    const pilotNameStr = String(row[colIdx["飞行员"]]).trim();
    const subjectName = String(row[colIdx["科目"]]).trim();
    const startTime = String(row[colIdx["开始时间"]]).trim();
    const endTime = String(row[colIdx["结束时间"]]).trim();
    const durationStr = String(row[colIdx["时长(小时)"]]).trim();

    if (isNaN(aircraftId)) continue;

    const pids = splitIds(pilotIdStr);
    const pnames = splitNames(pilotNameStr);
    if (pids.length === 0) continue;

    pids.forEach((id) => pilotIds.add(id));
    planeIds.add(aircraftId);

    const startParts = startTime.split(":").map(Number);
    const endParts = endTime.split(":").map(Number);
    const startHour = startParts[0] + (startParts[1] || 0) / 60;
    const endHour = endParts[0] + (endParts[1] || 0) / 60;
    const duration =
      parseFloat(durationStr) || Math.max(0, endHour - startHour);

    const task: Task = {
      id: Date.now() + Math.random(),
      pilotIds: pids,
      pilotNames: pnames,
      planeId: aircraftId,
      startHour,
      duration,
      start: startTime,
      end: endTime,
    };

    if (!planeTasksMap.has(aircraftId)) {
      planeTasksMap.set(aircraftId, { planeNo, model, tasks: [] });
    }
    planeTasksMap.get(aircraftId)!.tasks.push(task);

    monitorTasks.push({
      taskId: Date.now() + Math.random() + i,
      planeId: aircraftId,
      planeModel: model,
      pilotIds: pids,
      pilotNames: pnames,
      subjectName,
    });
  }

  const scheduleData: PlaneRow[] = [];
  for (const [planeId, { planeNo, model, tasks }] of planeTasksMap) {
    scheduleData.push({
      plane: { id: planeId, model: `${planeNo} (${model})` },
      tasks,
    });
  }

  return { scheduleData, monitorTasks, pilotIds, planeIds };
};

// ========== 加载飞行员信息（使用飞行员管理接口） ==========
const loadPilotInfo = async (pilotIds: Set<number>) => {
  const map = new Map<number, PilotInfo>();
  try {
    const res = await getPilots({ page: 1, page_size: 9999 });
    const allPilots = Array.isArray(res) ? res : res.list || [];
    for (const id of pilotIds) {
      const pilot = allPilots.find((p: any) => p.id === id);
      if (pilot) {
        map.set(id, {
          id: pilot.id,
          name: pilot.username || `飞行员${id}`,
          totalFlightHours: pilot.total_flight_hours
            ? parseFloat(pilot.total_flight_hours)
            : 0,
        });
      } else {
        map.set(id, { id, name: `飞行员${id}`, totalFlightHours: 0 });
      }
    }
  } catch {
    for (const id of pilotIds) {
      map.set(id, { id, name: `飞行员${id}`, totalFlightHours: 0 });
    }
  }
  return map;
};

const loadPlaneInfo = async (planeIds: Set<number>) => {
  const map = new Map<number, string>();
  for (const id of planeIds) {
    try {
      const attrs = await getResourceAttributes("AIRCRAFT", id);
      map.set(id, attrs.model || `飞机${id}`);
    } catch {
      /* */
    }
  }
  return map;
};

// ========== 加载所有机型（用于名称->ID映射） ==========
const loadAllModels = async () => {
  try {
    const models = await listModels();
    modelNameToId.value = new Map(models.map((m: any) => [m.name, m.id]));
  } catch {
    ElMessage.error("加载机型列表失败");
  }
};

// ========== 构建执行监控行（基于 monitorTasks + 飞机详情） ==========
const buildExecutionRows = async (
  monitorTasks: MonitorTask[],
  planeIds: Set<number>
) => {
  const planeDetailsMap = new Map<number, any>();
  for (const id of planeIds) {
    try {
      const detail = await getAircraftDetail(id);
      planeDetailsMap.set(id, detail);
    } catch {
      /* */
    }
  }

  const rows: ExecutionRow[] = [];
  for (const mt of monitorTasks) {
    const detail = planeDetailsMap.get(mt.planeId);
    if (!detail) continue;
    const planeNo = detail.aircraft.plane_no;

    let subjectId: number | undefined;
    let completed = false;
    let operationHours = 0;
    let flightHours = 0;

    const subRecord = detail.subjects.find(
      (s: any) => s.subject.name === mt.subjectName
    );
    if (subRecord) {
      subjectId = subRecord.subject.id;
      completed = subRecord.completed;
      operationHours = subRecord.operation_hours || 0;
      flightHours = subRecord.flight_hours || 0;
    }

    rows.push({
      aircraftId: mt.planeId,
      planeNo,
      subjectId: subjectId || 0,
      subjectName: mt.subjectName,
      pilotNames: mt.pilotNames,
      pilotIds: mt.pilotIds,
      completed,
      operationHours,
      flightHours,
      origOperationHours: operationHours,
      origFlightHours: flightHours,
      origCompleted: completed,
      modelName: mt.planeModel, // 保存机型名称
    });
  }
  executionRows.value = rows;
};

// ========== 自动加载 / 手动导入 ==========
const autoImport = async () => {
  autoLoading.value = true;
  try {
    const today = new Date().toISOString().slice(0, 10);
    const { files } = await listFiles("approval");
    const target = files.find((f: any) =>
      f.key.endsWith(`${today}-审批通过.xlsx`)
    );
    if (!target) {
      ElMessage.warning(`未找到 ${today}-审批通过.xlsx`);
      return;
    }
    const { url } = await getFileUrl(target.key);
    const response = await fetch(url);
    const blob = await response.blob();
    await parseExcelBlob(blob);
    ElMessage.success("自动加载成功");
  } catch (error) {
    ElMessage.error("自动加载失败");
    console.error(error);
  } finally {
    autoLoading.value = false;
  }
};

const uploadExcel = () => {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".xlsx, .xls";
  input.onchange = async (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    await parseExcelBlob(file);
  };
  input.click();
};

const parseExcelBlob = async (blob: Blob) => {
  loading.value = true;
  try {
    const buffer = await blob.arrayBuffer();
    const workbook = XLSX.read(buffer);
    const sheetNames = workbook.SheetNames;

    const sheet1 = workbook.Sheets[sheetNames[0]];
    const rows = XLSX.utils.sheet_to_json(sheet1, {
      header: 1,
      defval: "",
    }) as any[][];
    const {
      scheduleData: sd,
      monitorTasks,
      pilotIds,
      planeIds,
    } = parseFlightPlanSheet(rows);
    scheduleData.value = sd;
    taskList.value = monitorTasks;

    const [pilotMap, planeModelMap] = await Promise.all([
      loadPilotInfo(pilotIds),
      loadPlaneInfo(planeIds),
    ]);
    pilotInfoMap.value = pilotMap;
    pilotInfoList.value = Array.from(pilotMap.values());

    const modelSet = new Set<string>();
    for (const model of planeModelMap.values()) {
      if (model && !model.startsWith("飞机")) modelSet.add(model);
    }
    availableModels.value = Array.from(modelSet);

    // 确保机型映射已加载
    if (modelNameToId.value.size === 0) {
      await loadAllModels();
    }

    await buildExecutionRows(monitorTasks, planeIds);
    await loadCommanders();
  } catch (error) {
    console.error(error);
    ElMessage.error("解析失败");
  } finally {
    loading.value = false;
  }
};

// ========== 执行监控交互 ==========
const handleCompleteChange = async (row: ExecutionRow) => {
  try {
    await finishFlight({
      aircraft_id: row.aircraftId,
      subject_id: row.subjectId,
      completed: row.completed,
    });
    row.origCompleted = row.completed;
    ElMessage.success("状态已更新");
  } catch {
    row.completed = !row.completed;
    ElMessage.error("更新失败");
  }
};

const handleHoursChange = async (
  row: ExecutionRow,
  type: "operation" | "flight",
  newVal: number | undefined
) => {
  const val = newVal || 0;
  const original =
    type === "operation" ? row.origOperationHours : row.origFlightHours;
  const delta = val - original;
  if (delta <= 0) {
    ElMessage.warning("必须输入大于当前值的正增量");
    return;
  }

  try {
    // 1. 更新试飞记录
    await finishFlight({
      aircraft_id: row.aircraftId,
      subject_id: row.subjectId,
      flight_hours: type === "flight" ? delta : undefined,
      operation_hours: type === "operation" ? delta : undefined,
      completed: row.completed,
    });

    // 2. 如果是飞行时长，更新飞机总飞行时间
    if (type === "flight") {
      const detail = await getAircraftDetail(row.aircraftId);
      const currentTotal = Number(detail.aircraft.total_running_hours) || 0;
      await updateAircraft(row.aircraftId, {
        total_running_hours: currentTotal + delta,
      } as any);
    }

    // 3. 如果是飞行时长，同步更新飞行员机型飞行时间表（pilot_model_flight_time）
    if (type === "flight") {
      const modelId = modelNameToId.value.get(row.modelName);
      if (modelId) {
        const promises = row.pilotIds.map((pilotId) =>
          upsertPilotModelFlightTime({
            pilot_id: pilotId,
            model_id: modelId,
            total_flight_hours: delta, // 累加
          })
        );
        await Promise.all(promises);
      } else {
        console.warn("未找到机型ID，无法更新飞行员机型飞行时间");
      }
    }

    // 更新本地值
    if (type === "operation") {
      row.operationHours = val;
      row.origOperationHours = val;
    } else {
      row.flightHours = val;
      row.origFlightHours = val;
    }
    ElMessage.success("时长已更新");
  } catch (error) {
    ElMessage.error("更新失败");
    // 回滚本地值
    if (type === "operation") row.operationHours = row.origOperationHours;
    else row.flightHours = row.origFlightHours;
  }
};

// ========== 指挥员 ==========
const loadCommanders = async () => {
  try {
    const records = await getRecordsByType("COMMANDER");
    const list: CommanderInfo[] = [];
    for (const p of records) {
      const name = (await getUser(p.subject_id)).real_name || "未知";
      const attr = typeof p.attr_key_value === "object" ? p.attr_key_value : {};
      const hours = Number((attr as any).指挥时间) || 0;
      list.push({ id: p.subject_id, name, totalCommandHours: hours });
    }
    commanderList.value = list;
  } catch {
    /* */
  }
};

const saveCommanderHours = async (row: CommanderInfo) => {
  try {
    await setAttributes("COMMANDER", row.id, {
      指挥时间: row.totalCommandHours.toString(),
    });
    ElMessage.success("指挥时间已更新");
  } catch {
    ElMessage.error("保存失败");
  }
};

// 飞行员增加飞行时间（使用飞行员管理接口更新总飞行时间）
const openAddFlightHoursDialog = (pilot: PilotInfo) => {
  currentEditPilot.value = pilot;
  addFlightModel.value = availableModels.value[0] || "";
  addFlightHours.value = 1;
  addFlightDialogVisible.value = true;
};

const confirmAddFlightHours = async () => {
  if (!currentEditPilot.value || !addFlightModel.value) return;
  const pilot = currentEditPilot.value;
  const hours = addFlightHours.value;
  try {
    // 直接更新总飞行时间
    const newTotal = pilot.totalFlightHours + hours;
    await updatePilot(pilot.id, { total_flight_hours: newTotal.toString() });
    pilot.totalFlightHours = newTotal;

    // 同时更新飞行员机型飞行时间表（此处机型名称由用户选择）
    const modelId = modelNameToId.value.get(addFlightModel.value);
    if (modelId) {
      await upsertPilotModelFlightTime({
        pilot_id: pilot.id,
        model_id: modelId,
        total_flight_hours: hours,
      });
    }

    ElMessage.success("飞行时间已增加");
    addFlightDialogVisible.value = false;
  } catch {
    ElMessage.error("操作失败");
  }
};

onMounted(() => {
  // 预先加载机型映射
  loadAllModels();
  autoImport();
});
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
.import-page {
  height: 100%;
  background: #f5f7fa;
  padding: 20px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-actions {
  display: flex;
  gap: 8px;
}
.empty-placeholder {
  text-align: center;
  padding: 80px 0;
  color: #909399;
}
.schedule-wrapper {
  overflow-x: auto;
}
.time-header {
  display: flex;
  background: #fafbfc;
  border-bottom: 1px solid #eee;
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
  width: 60px;
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
  height: 60px;
}
.timeline {
  position: relative;
  height: 60px;
}
.hour-slot {
  display: inline-block;
  width: 60px;
  height: 60px;
  border-right: 1px dashed #eee;
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
  pointer-events: none;
}
.task-title {
  font-weight: 600;
  font-size: 11px;
}
.task-time {
  font-size: 9px;
  color: #666;
}
.monitor-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
}
.section-card {
  background: #fff;
  padding: 16px;
  border-radius: 8px;
}
</style>