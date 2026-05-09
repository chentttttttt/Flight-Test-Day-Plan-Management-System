<!-- src/views/PilotManagement.vue -->
<template>
  <div class="pilot-management">
    <el-card class="pilot-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><User /></el-icon>
            <span class="header-title">飞行员管理</span>
          </div>
          <div class="header-right">
            <el-button type="primary" @click="openCreateDialog">
              <el-icon><Plus /></el-icon> 新增飞行员
            </el-button>
            <el-button @click="fetchList">
              <el-icon><Refresh /></el-icon> 刷新
            </el-button>
          </div>
        </div>
      </template>

      <!-- 筛选栏 -->
      <div class="filter-bar">
        <el-form :model="queryParams" inline>
          <el-form-item label="姓名">
            <el-input
              v-model="queryParams.name"
              placeholder="请输入姓名"
              clearable
            />
          </el-form-item>
          <el-form-item label="等级">
            <el-select v-model="queryParams.level" placeholder="全部" clearable>
              <el-option label="一级飞行员" value="一级飞行员" />
              <el-option label="二级飞行员" value="二级飞行员" />
            </el-select>
          </el-form-item>
          <el-form-item label="状态">
            <el-select
              v-model="queryParams.status"
              placeholder="全部"
              clearable
            >
              <el-option label="启用" :value="1" />
              <el-option label="禁用" :value="0" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="handleSearch">查询</el-button>
            <el-button @click="resetFilter">重置</el-button>
          </el-form-item>
        </el-form>
      </div>

      <!-- 飞行员列表 -->
      <el-table
        :data="tableData"
        v-loading="loading"
        border
        stripe
        class="pilot-table"
      >
        <el-table-column prop="id" label="ID" width="80" align="center" />
        <el-table-column prop="username" label="姓名" min-width="100" />
        <el-table-column prop="code" label="工号" min-width="120" />
        <el-table-column prop="gender" label="性别" width="80" align="center" />
        <el-table-column prop="age" label="年龄" width="80" align="center" />
        <el-table-column prop="level" label="等级" width="120" align="center">
          <template #default="{ row }">
            <el-tag :type="row.level === '一级飞行员' ? 'danger' : 'info'">
              {{ row.level }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          prop="health_status"
          label="身体状态"
          width="100"
          align="center"
        />
        <el-table-column
          prop="mental_status"
          label="心理状态"
          width="100"
          align="center"
        />
        <el-table-column
          prop="total_flight_hours"
          label="总飞行时间(h)"
          width="120"
          align="center"
        />
        <el-table-column prop="status" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.status === 1 ? 'success' : 'danger'">
              {{ row.status === 1 ? "启用" : "禁用" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click="openEditDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="可飞机型" placement="top">
              <el-button
                size="small"
                circle
                :icon="Plus"
                @click="openModelsDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click="handleDelete(row)"
              />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="queryParams.page"
          v-model:page-size="queryParams.page_size"
          :page-sizes="[10, 20, 50, 100]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 新增/编辑飞行员弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="600px"
      class="pilot-dialog"
      :close-on-click-modal="false"
      @close="resetForm"
    >
      <el-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-width="100px"
      >
        <el-form-item label="关联用户" prop="user_id">
          <el-select
            v-model="formData.user_id"
            filterable
            remote
            :remote-method="searchUsers"
            :loading="userLoading"
            placeholder="请输入用户名搜索"
            clearable
            style="width: 100%"
          >
            <el-option
              v-for="user in userOptions"
              :key="user.id"
              :label="user.username"
              :value="user.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="工号" prop="code">
          <el-input v-model="formData.code" placeholder="工号" />
        </el-form-item>
        <el-form-item label="性别" prop="gender">
          <el-radio-group v-model="formData.gender">
            <el-radio label="男">男</el-radio>
            <el-radio label="女">女</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="年龄" prop="age">
          <el-input-number v-model="formData.age" :min="18" :max="60" />
        </el-form-item>
        <el-form-item label="身体状态" prop="health_status">
          <el-select v-model="formData.health_status">
            <el-option label="良好" value="良好" />
            <el-option label="一般" value="一般" />
            <el-option label="不佳" value="不佳" />
          </el-select>
        </el-form-item>
        <el-form-item label="心理状态" prop="mental_status">
          <el-select v-model="formData.mental_status">
            <el-option label="稳定" value="稳定" />
            <el-option label="优秀" value="优秀" />
            <el-option label="需关注" value="需关注" />
          </el-select>
        </el-form-item>
        <el-form-item label="等级" prop="level">
          <el-select v-model="formData.level">
            <el-option label="一级飞行员" value="一级飞行员" />
            <el-option label="二级飞行员" value="二级飞行员" />
          </el-select>
        </el-form-item>
        <el-form-item label="总飞行时间(h)" prop="total_flight_hours">
          <el-input-number
            v-model="formData.total_flight_hours"
            :min="0"
            :step="10"
            :precision="1"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="备注" prop="remark">
          <el-input type="textarea" v-model="formData.remark" rows="3" />
        </el-form-item>
        <el-form-item label="状态" prop="status">
          <el-switch
            v-model="formData.statusSwitch"
            :active-value="1"
            :inactive-value="0"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" :loading="submitLoading" @click="submitForm"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 分配可飞机型弹窗 -->
    <el-dialog
      v-model="modelsDialogVisible"
      :title="`分配可飞机型 - ${currentPilot?.username || ''}`"
      width="500px"
      class="pilot-dialog"
    >
      <el-checkbox-group v-model="selectedModelIds">
        <el-checkbox
          v-for="model in aircraftModels"
          :key="model.id"
          :label="model.id"
        >
          {{ model.name }} ({{ model.code }})
        </el-checkbox>
      </el-checkbox-group>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="modelsDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="modelSubmitLoading"
            @click="submitModels"
            >保存</el-button
          >
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { ElMessage, ElMessageBox, type FormInstance } from "element-plus";
import { Plus, Refresh, Edit, Delete, User } from "@element-plus/icons-vue";
import { listUsers } from "@/api/user";
import { listModels } from "@/api/aircraft";
import type { Pilot } from "@/types/aircraft";

// API 函数（需根据后端实现）
import {
  getPilots,
  createPilot,
  updatePilot,
  deletePilot,
  getPilotModels,
  setPilotModels,
} from "@/api/aircraft";

// ========== 列表相关 ==========
const loading = ref(false);
const tableData = ref<Pilot[]>([]);
const total = ref(0);
const queryParams = reactive({
  page: 1,
  page_size: 10,
  name: "",
  level: "",
  status: undefined as number | undefined,
});

// 机型列表（用于分配）
const aircraftModels = ref<{ id: number; name: string; code: string }[]>([]);
const modelsDialogVisible = ref(false);
const currentPilot = ref<Pilot | null>(null);
const selectedModelIds = ref<number[]>([]);
const modelSubmitLoading = ref(false);

// ========== 表单相关 ==========
const dialogVisible = ref(false);
const dialogTitle = ref("");
const formRef = ref<FormInstance>();
const submitLoading = ref(false);
const formData = reactive({
  id: undefined as number | undefined,
  user_id: undefined as number | undefined,
  code: "",
  gender: "男",
  age: 30,
  health_status: "良好",
  mental_status: "稳定",
  level: "二级飞行员",
  total_flight_hours: 0,
  remark: "",
  statusSwitch: 1, // 1启用 0禁用
});
const formRules = {
  user_id: [{ required: true, message: "请选择关联用户", trigger: "change" }],
  code: [{ required: true, message: "请输入工号", trigger: "blur" }],
};

// 用户搜索相关（用于关联用户选择）
const userOptions = ref<{ id: number; username: string }[]>([]);
const userLoading = ref(false);

// 加载飞行员列表
const fetchList = async () => {
  loading.value = true;
  try {
    const res = await getPilots(queryParams);
    let allData = Array.isArray(res) ? res : res.list || [];
    // 前端筛选（姓名、等级、状态）
    let filtered = allData.filter((item) => {
      if (queryParams.name && !item.username.includes(queryParams.name))
        return false;
      if (queryParams.level && item.level !== queryParams.level) return false;
      if (
        queryParams.status !== undefined &&
        item.status !== queryParams.status
      )
        return false;
      return true;
    });
    total.value = filtered.length;
    const start = (queryParams.page - 1) * queryParams.page_size;
    const end = start + queryParams.page_size;
    tableData.value = filtered.slice(start, end);
  } catch (error) {
    ElMessage.error("加载飞行员列表失败");
  } finally {
    loading.value = false;
  }
};
// 搜索用户
const searchUsers = async (query: string) => {
  if (!query) {
    userOptions.value = [];
    return;
  }
  userLoading.value = true;
  try {
    const res = await listUsers({ page: 1, page_size: 20, username: query });
    userOptions.value = res.list.map((u: any) => ({
      id: u.id,
      username: u.username,
    }));
  } catch (error) {
    console.error("搜索用户失败", error);
  } finally {
    userLoading.value = false;
  }
};

// 查询与重置
const handleSearch = () => {
  queryParams.page = 1;
  fetchList();
};
const resetFilter = () => {
  queryParams.name = "";
  queryParams.level = "";
  queryParams.status = undefined;
  queryParams.page = 1;
  fetchList();
};

// 分页
const handleSizeChange = (size: number) => {
  queryParams.page_size = size;
  queryParams.page = 1;
  fetchList();
};
const handleCurrentChange = (page: number) => {
  queryParams.page = page;
  fetchList();
};

// 打开新增对话框
const openCreateDialog = () => {
  dialogTitle.value = "新增飞行员";
  formData.id = undefined;
  formData.user_id = undefined;
  formData.code = "";
  formData.gender = "男";
  formData.age = 30;
  formData.health_status = "良好";
  formData.mental_status = "稳定";
  formData.level = "二级飞行员";
  formData.total_flight_hours = 0;
  formData.remark = "";
  formData.statusSwitch = 1;
  dialogVisible.value = true;
};

// 打开编辑对话框
const openEditDialog = (row: Pilot) => {
  dialogTitle.value = "编辑飞行员";
  formData.id = row.id;
  formData.user_id = row.user_id;
  formData.code = row.code || "";
  formData.gender = row.gender || "男";
  formData.age = row.age || 30;
  formData.health_status = row.health_status || "良好";
  formData.mental_status = row.mental_status || "稳定";
  formData.level = row.level || "二级飞行员";
  formData.total_flight_hours = row.total_flight_hours || 0;
  formData.remark = row.remark || "";
  formData.statusSwitch = row.status;
  dialogVisible.value = true;
};

// 提交表单
const submitForm = async () => {
  if (!formRef.value) return;
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    submitLoading.value = true;
    try {
      const payload = {
        user_id: formData.user_id,
        code: formData.code,
        gender: formData.gender,
        age: formData.age,
        health_status: formData.health_status,
        mental_status: formData.mental_status,
        level: formData.level,
        total_flight_hours: formData.total_flight_hours,
        remark: formData.remark,
        status: formData.statusSwitch,
      };
      if (formData.id) {
        await updatePilot(formData.id, payload);
        ElMessage.success("更新成功");
      } else {
        await createPilot(payload);
        ElMessage.success("创建成功");
      }
      dialogVisible.value = false;
      fetchList();
    } catch (error) {
      ElMessage.error("操作失败");
    } finally {
      submitLoading.value = false;
    }
  });
};

// 删除飞行员
const handleDelete = (row: Pilot) => {
  ElMessageBox.confirm(`确认删除飞行员“${row.username}”吗？`, "提示", {
    type: "warning",
  })
    .then(async () => {
      await deletePilot(row.id);
      ElMessage.success("删除成功");
      fetchList();
    })
    .catch(() => {});
};

// 重置表单（关闭弹窗时）
const resetForm = () => {
  formRef.value?.resetFields();
};

// ========== 可飞机型分配 ==========
// 加载机型列表
const loadAircraftModels = async () => {
  const models = await listModels();
  aircraftModels.value = models.map((m: any) => ({
    id: m.id,
    name: m.name,
    code: m.code,
  }));
};

// 打开机型分配弹窗
const openModelsDialog = async (pilot: Pilot) => {
  currentPilot.value = pilot;
  selectedModelIds.value = [];
  try {
    const existing = await getPilotModels(pilot.id);
    selectedModelIds.value = existing.map((m: any) => m.id);
  } catch (error) {
    console.error("加载可飞机型失败", error);
  }
  modelsDialogVisible.value = true;
};

// 提交机型分配
const submitModels = async () => {
  if (!currentPilot.value) return;
  modelSubmitLoading.value = true;
  try {
    await setPilotModels(currentPilot.value.id, selectedModelIds.value);
    ElMessage.success("分配成功");
    modelsDialogVisible.value = false;
  } catch (error) {
    ElMessage.error("分配失败");
  } finally {
    modelSubmitLoading.value = false;
  }
};

onMounted(() => {
  fetchList();
  loadAircraftModels();
});
</script>

<style scoped>
.pilot-management {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.pilot-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1);
  border: none;
  overflow: hidden;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-left {
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
.filter-bar {
  margin-bottom: 16px;
  padding: 0 8px;
}
.pilot-table {
  border-radius: 16px;
  overflow: hidden;
}
.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}
.pilot-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
.dialog-footer .el-button {
  border-radius: 40px;
  padding: 10px 20px;
}
.dialog-footer .el-button--primary {
  background: linear-gradient(135deg, #409eff, #36d1dc);
  border: none;
  box-shadow: 0 4px 10px rgba(64, 158, 255, 0.3);
}
</style>