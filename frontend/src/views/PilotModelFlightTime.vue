<template>
  <div class="pilot-flight-time-manage">
    <el-card>
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon"><Timer /></el-icon>
          <span class="header-title">飞行时间管理</span>
        </div>
      </template>

      <!-- 筛选栏 -->
      <div class="toolbar">
        <el-form :inline="true" :model="queryParams" size="default">
          <el-form-item label="飞行员">
            <el-select
              v-model="queryParams.pilot_id"
              placeholder="全部飞行员"
              clearable
              filterable
              style="width: 200px"
            >
              <el-option
                v-for="p in pilots"
                :key="p.id"
                :label="p.username"
                :value="p.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="机型">
            <el-select
              v-model="queryParams.model_id"
              placeholder="全部机型"
              clearable
              filterable
              style="width: 200px"
            >
              <el-option
                v-for="m in models"
                :key="m.id"
                :label="m.name"
                :value="m.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button @click="resetQuery">重置</el-button>
          </el-form-item>
        </el-form>
        <div class="toolbar-actions">
          <el-button type="success" @click="openCreateDialog">
            <el-icon><Plus /></el-icon> 新增
          </el-button>
          <el-button type="warning" @click="openUpsertDialog">
            <el-icon><Upload /></el-icon> 累加飞行时间
          </el-button>
          <el-button type="danger" @click="handleBatchDeleteByPilot">
            <el-icon><Delete /></el-icon> 按飞行员批量删除
          </el-button>
        </div>
      </div>

      <!-- 表格 -->
      <el-table
        :data="filteredData"
        v-loading="loading"
        border
        stripe
        class="flight-time-table"
      >
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column label="飞行员" min-width="120">
          <template #default="{ row }">
            {{ getPilotName(row.pilot_id) }}
          </template>
        </el-table-column>
        <el-table-column label="机型" min-width="120">
          <template #default="{ row }">
            {{ getModelName(row.model_id) }}
          </template>
        </el-table-column>
        <el-table-column label="累计飞行时间(h)" width="150">
          <template #default="{ row }">
            {{ row.total_flight_hours.toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column label="间断飞行时间(h)" width="160">
          <template #default="{ row }">
            {{ calcBreakHours(row.update_time) }}
          </template>
        </el-table-column>
        <el-table-column prop="create_time" label="创建时间" width="180" />
        <el-table-column prop="update_time" label="最后飞行时间" width="180" />
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEditDialog(row)">
              <el-icon><Edit /></el-icon> 编辑
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(row.id)">
              <el-icon><Delete /></el-icon> 删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 新增 / 编辑 弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑飞行时间' : '新增飞行时间'"
      width="500px"
      @close="resetForm"
    >
      <el-form
        :model="formData"
        :rules="formRules"
        ref="formRef"
        label-width="120px"
      >
        <el-form-item label="飞行员" prop="pilot_id">
          <el-select
            v-model="formData.pilot_id"
            placeholder="选择飞行员"
            filterable
            :disabled="isEdit"
          >
            <el-option
              v-for="p in pilots"
              :key="p.id"
              :label="p.username"
              :value="p.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="机型" prop="model_id">
          <el-select
            v-model="formData.model_id"
            placeholder="选择机型"
            filterable
            :disabled="isEdit"
          >
            <el-option
              v-for="m in models"
              :key="m.id"
              :label="m.name"
              :value="m.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="飞行时间(h)" prop="total_flight_hours">
          <el-input-number
            v-model="formData.total_flight_hours"
            :min="0"
            :step="0.5"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm" :loading="submitLoading">
          确定
        </el-button>
      </template>
    </el-dialog>

    <!-- Upsert（累加）弹窗 -->
    <el-dialog
      v-model="upsertDialogVisible"
      title="累加飞行时间"
      width="500px"
      @close="resetUpsertForm"
    >
      <el-form
        :model="upsertForm"
        :rules="formRules"
        ref="upsertFormRef"
        label-width="120px"
      >
        <el-form-item label="飞行员" prop="pilot_id">
          <el-select
            v-model="upsertForm.pilot_id"
            placeholder="选择飞行员"
            filterable
          >
            <el-option
              v-for="p in pilots"
              :key="p.id"
              :label="p.username"
              :value="p.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="机型" prop="model_id">
          <el-select
            v-model="upsertForm.model_id"
            placeholder="选择机型"
            filterable
          >
            <el-option
              v-for="m in models"
              :key="m.id"
              :label="m.name"
              :value="m.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="本次飞行时间(h)" prop="total_flight_hours">
          <el-input-number
            v-model="upsertForm.total_flight_hours"
            :min="0"
            :step="0.5"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="upsertDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          @click="submitUpsert"
          :loading="upsertLoading"
        >
          确定
        </el-button>
      </template>
    </el-dialog>

    <!-- 按飞行员批量删除弹窗 -->
    <el-dialog
      v-model="batchDeleteDialogVisible"
      title="按飞行员批量删除"
      width="400px"
    >
      <el-form>
        <el-form-item label="飞行员">
          <el-select
            v-model="batchDeletePilotId"
            placeholder="选择飞行员"
            filterable
          >
            <el-option
              v-for="p in pilots"
              :key="p.id"
              :label="p.username"
              :value="p.id"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="batchDeleteDialogVisible = false">取消</el-button>
        <el-button type="danger" @click="confirmBatchDelete"
          >确认删除</el-button
        >
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import { Timer, Plus, Edit, Delete, Upload } from "@element-plus/icons-vue";
import {
  createPilotModelFlightTime,
  listPilotModelFlightTimes,
  updatePilotModelFlightTime,
  upsertPilotModelFlightTime,
  deletePilotModelFlightTime,
  deleteByPilot,
} from "@/api/pilotModelFlightTime";
import { listModels, getPilots } from "@/api/aircraft";
import type {
  PilotModelFlightTime,
  CreatePilotModelFlightTimeDto,
} from "@/types/pilotModelFlightTime";
import type { Pilot, AircraftModel } from "@/types/aircraft";

// ========== 预加载数据 ==========
const pilots = ref<Pilot[]>([]);
const models = ref<AircraftModel[]>([]);

// ========== 查询参数（仅用于前端筛选） ==========
const queryParams = reactive({
  pilot_id: undefined as number | undefined,
  model_id: undefined as number | undefined,
});

const tableData = ref<PilotModelFlightTime[]>([]);
const loading = ref(false);

// ========== 前端筛选 ==========
const filteredData = computed(() => {
  return tableData.value.filter((item) => {
    const matchPilot =
      !queryParams.pilot_id || item.pilot_id === queryParams.pilot_id;
    const matchModel =
      !queryParams.model_id || item.model_id === queryParams.model_id;
    return matchPilot && matchModel;
  });
});

// ========== 辅助映射 ==========
const getPilotName = (pilotId: number) => {
  const pilot = pilots.value.find((p) => p.id === pilotId);
  return pilot ? pilot.username : `ID:${pilotId}`;
};

const getModelName = (modelId: number) => {
  const model = models.value.find((m) => m.id === modelId);
  return model ? model.name : `机型${modelId}`;
};

// ========== 弹窗表单 ==========
const dialogVisible = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<FormInstance>();
const submitLoading = ref(false);

const formData = reactive<CreatePilotModelFlightTimeDto>({
  pilot_id: 0,
  model_id: 0,
  total_flight_hours: 0,
});

const formRules: FormRules = {
  pilot_id: [{ required: true, message: "请选择飞行员", trigger: "change" }],
  model_id: [{ required: true, message: "请选择机型", trigger: "change" }],
  total_flight_hours: [
    { required: true, message: "请输入飞行时间", trigger: "blur" },
  ],
};

// ========== Upsert 弹窗 ==========
const upsertDialogVisible = ref(false);
const upsertFormRef = ref<FormInstance>();
const upsertLoading = ref(false);
const upsertForm = reactive<CreatePilotModelFlightTimeDto>({
  pilot_id: 0,
  model_id: 0,
  total_flight_hours: 0,
});

// ========== 批量删除弹窗 ==========
const batchDeleteDialogVisible = ref(false);
const batchDeletePilotId = ref<number | undefined>(undefined);

// ========== 工具函数 ==========
const calcBreakHours = (updateTime: string) => {
  if (!updateTime) return "0.00";
  const last = new Date(updateTime).getTime();
  const now = Date.now();
  const diffHours = (now - last) / (1000 * 60 * 60);
  return diffHours.toFixed(2);
};

// ========== 加载所有飞行记录 ==========
const loadData = async () => {
  loading.value = true;
  try {
    const { list } = await listPilotModelFlightTimes({});
    tableData.value = list;
  } catch {
    ElMessage.error("加载飞行时间数据失败");
  } finally {
    loading.value = false;
  }
};

// 重置筛选条件
const resetQuery = () => {
  queryParams.pilot_id = undefined;
  queryParams.model_id = undefined;
};

// ========== 新增 ==========
const openCreateDialog = () => {
  isEdit.value = false;
  editingId.value = null;
  formData.pilot_id = 0;
  formData.model_id = 0;
  formData.total_flight_hours = 0;
  dialogVisible.value = true;
};

// ========== 编辑 ==========
const openEditDialog = (row: PilotModelFlightTime) => {
  isEdit.value = true;
  editingId.value = row.id;
  formData.pilot_id = row.pilot_id;
  formData.model_id = row.model_id;
  formData.total_flight_hours = row.total_flight_hours;
  dialogVisible.value = true;
};

const resetForm = () => {
  formRef.value?.resetFields();
};

const submitForm = async () => {
  if (!formRef.value) return;
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    submitLoading.value = true;
    try {
      if (isEdit.value && editingId.value !== null) {
        await updatePilotModelFlightTime(editingId.value, {
          total_flight_hours: formData.total_flight_hours,
        });
        ElMessage.success("更新成功");
      } else {
        await createPilotModelFlightTime(formData);
        ElMessage.success("创建成功");
      }
      dialogVisible.value = false;
      await loadData();
    } catch {
      ElMessage.error("操作失败");
    } finally {
      submitLoading.value = false;
    }
  });
};

// ========== Upsert ==========
const openUpsertDialog = () => {
  upsertForm.pilot_id = 0;
  upsertForm.model_id = 0;
  upsertForm.total_flight_hours = 0;
  upsertDialogVisible.value = true;
};

const resetUpsertForm = () => {
  upsertFormRef.value?.resetFields();
};

const submitUpsert = async () => {
  if (!upsertFormRef.value) return;
  await upsertFormRef.value.validate(async (valid) => {
    if (!valid) return;
    upsertLoading.value = true;
    try {
      await upsertPilotModelFlightTime(upsertForm);
      ElMessage.success("累加成功");
      upsertDialogVisible.value = false;
      await loadData();
    } catch {
      ElMessage.error("操作失败");
    } finally {
      upsertLoading.value = false;
    }
  });
};

// ========== 单条删除 ==========
const handleDelete = async (id: number) => {
  try {
    await ElMessageBox.confirm("确认删除此记录吗？", "提示", {
      type: "warning",
    });
    await deletePilotModelFlightTime(id);
    ElMessage.success("删除成功");
    await loadData();
  } catch {
    // 用户取消
  }
};

// ========== 批量删除 ==========
const handleBatchDeleteByPilot = () => {
  batchDeletePilotId.value = undefined;
  batchDeleteDialogVisible.value = true;
};

const confirmBatchDelete = async () => {
  if (!batchDeletePilotId.value) {
    ElMessage.warning("请选择飞行员");
    return;
  }
  try {
    await ElMessageBox.confirm(
      `确认删除该飞行员的所有飞行时间记录吗？`,
      "提示",
      { type: "warning" }
    );
    const { deleted } = await deleteByPilot(batchDeletePilotId.value);
    ElMessage.success(`已删除 ${deleted} 条记录`);
    batchDeleteDialogVisible.value = false;
    await loadData();
  } catch {
    // 用户取消
  }
};

// ========== 初始化 ==========
onMounted(async () => {
  try {
    const [pilotsRes, modelsList] = await Promise.all([
      getPilots({ page: 1, page_size: 999 }),
      listModels(),
    ]);
    const pilotsData = pilotsRes?.data || pilotsRes?.list || pilotsRes;
    pilots.value = Array.isArray(pilotsData) ? pilotsData : [];
    models.value = modelsList || [];
  } catch {
    ElMessage.error("加载基础数据失败");
  }
  loadData();
});
</script>

<style scoped>
.pilot-flight-time-manage {
  height: 100%;
  background: #f5f7fa;
  padding: 20px;
  box-sizing: border-box;
}
.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.header-icon {
  font-size: 28px;
  background: linear-gradient(135deg, #409eff, #36d1dc);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.header-title {
  font-size: 20px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #3498db);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 20px;
}
.toolbar-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.flight-time-table {
  border-radius: 16px;
  overflow: hidden;
}
.flight-time-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.flight-time-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.flight-time-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.flight-time-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
</style>