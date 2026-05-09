<template>
  <div class="abac-rule-manage">
    <el-card class="rule-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Lock /></el-icon>
            <span class="header-title">策略规则管理</span>
          </div>
          <el-button
            v-if="canCreate"
            type="primary"
            @click="openCreateDialog"
            class="create-btn"
          >
            <el-icon><Plus /></el-icon>
            <span>新增规则</span>
          </el-button>
        </div>
      </template>

      <!-- 搜索区域 -->
      <div class="search-area">
        <el-form :model="queryParams" inline>
          <el-form-item label="主体类型">
            <el-input
              v-model="queryParams.subject_type"
              placeholder="例如：PILOT"
              clearable
            />
          </el-form-item>
          <el-form-item label="资源类型">
            <el-input
              v-model="queryParams.resource_type"
              placeholder="例如：PLAN"
              clearable
            />
          </el-form-item>
          <el-form-item label="操作">
            <el-input
              v-model="queryParams.action"
              placeholder="例如：FLY"
              clearable
            />
          </el-form-item>
          <el-form-item label="状态">
            <el-select
              v-model="queryParams.status"
              placeholder="全部"
              clearable
              style="width: 120px"
              popper-class="select-popper"
            >
              <el-option label="全部" :value="null" />
              <el-option label="启用" :value="1" />
              <el-option label="禁用" :value="0" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="handleSearch">查询</el-button>
            <el-button @click="resetSearch">重置</el-button>
          </el-form-item>
        </el-form>
      </div>

      <!-- 表格区域 -->
      <el-table
        :data="tableData"
        v-loading="loading"
        border
        stripe
        class="rule-table"
        :header-cell-style="{
          background: 'rgba(64, 158, 255, 0.05)',
          color: '#2c3e50',
        }"
      >
        <el-table-column prop="id" label="ID" width="60" align="center" />
        <el-table-column
          prop="rule_code"
          label="规则编码"
          min-width="120"
          show-overflow-tooltip
        />
        <el-table-column
          prop="rule_name"
          label="规则名称"
          min-width="140"
          show-overflow-tooltip
        />
        <el-table-column prop="subject_type" label="主体类型" width="100" />
        <el-table-column
          prop="subject_id"
          label="主体ID"
          width="80"
          align="center"
        >
          <template #default="{ row }">
            {{ row.subject_id ?? "-" }}
          </template>
        </el-table-column>
        <el-table-column prop="resource_type" label="资源类型" width="100" />
        <el-table-column
          prop="resource_id"
          label="资源ID"
          width="80"
          align="center"
        >
          <template #default="{ row }">
            {{ row.resource_id ?? "-" }}
          </template>
        </el-table-column>
        <el-table-column prop="action" label="操作" width="100" />
        <el-table-column prop="effect" label="效果" width="80" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.effect === 'ALLOW' ? 'success' : 'danger'"
              size="small"
              effect="plain"
            >
              {{ row.effect }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          prop="priority"
          label="优先级"
          width="80"
          align="center"
        />
        <el-table-column prop="status" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.status === 1 ? 'success' : 'info'"
              size="small"
              effect="plain"
            >
              {{ row.status === 1 ? "启用" : "禁用" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="180" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top" v-if="canUpdate">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click="openEditDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="详情" placement="top" v-if="canView">
              <el-button
                size="small"
                circle
                :icon="View"
                @click="openDetailDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top" v-if="canDelete">
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
    </el-card>

    <!-- 新增/编辑弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="700px"
      class="rule-dialog"
      :close-on-click-modal="false"
      @close="resetForm"
    >
      <el-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-width="100px"
        label-position="right"
        class="rule-form"
      >
        <el-form-item label="规则编码" prop="rule_code">
          <el-input
            v-model="formData.rule_code"
            placeholder="唯一编码"
            clearable
          />
        </el-form-item>
        <el-form-item label="规则名称" prop="rule_name">
          <el-input
            v-model="formData.rule_name"
            placeholder="规则名称"
            clearable
          />
        </el-form-item>
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="主体类型" prop="subject_type">
              <el-input
                v-model="formData.subject_type"
                placeholder="如：PILOT"
                clearable
              />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="主体ID" prop="subject_id">
              <el-input-number
                v-model="formData.subject_id"
                :min="1"
                controls-position="right"
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
        </el-row>
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="资源类型" prop="resource_type">
              <el-input
                v-model="formData.resource_type"
                placeholder="如：PLAN"
                clearable
              />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="资源ID" prop="resource_id">
              <el-input-number
                v-model="formData.resource_id"
                :min="1"
                controls-position="right"
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item label="操作" prop="action">
          <el-input v-model="formData.action" placeholder="如：FLY" clearable />
        </el-form-item>
        <el-form-item label="条件(JSON)" prop="condition_json">
          <el-input
            v-model="conditionJsonStr"
            type="textarea"
            rows="3"
            placeholder='{"requiredLicense": "A", "minFlightHours": 500}'
            :class="{ 'is-error': jsonError }"
          />
          <div v-if="jsonError" class="json-error">{{ jsonError }}</div>
        </el-form-item>
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="效果" prop="effect">
              <el-radio-group v-model="formData.effect">
                <el-radio label="ALLOW">允许</el-radio>
                <el-radio label="DENY">拒绝</el-radio>
              </el-radio-group>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="优先级" prop="priority">
              <el-input-number
                v-model="formData.priority"
                :min="0"
                controls-position="right"
                style="width: 100%"
              />
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item label="状态" prop="status" v-if="isEdit">
          <el-radio-group v-model="formData.status">
            <el-radio :label="1">启用</el-radio>
            <el-radio :label="0">禁用</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="submitLoading"
            @click="handleSubmit"
          >
            <el-icon><Check /></el-icon>
            <span>确定</span>
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 详情弹窗 -->
    <el-dialog
      v-model="detailVisible"
      title="规则详情"
      width="600px"
      class="rule-dialog"
    >
      <el-descriptions :column="1" border>
        <el-descriptions-item label="ID">{{
          detailData.id
        }}</el-descriptions-item>
        <el-descriptions-item label="规则编码">{{
          detailData.rule_code
        }}</el-descriptions-item>
        <el-descriptions-item label="规则名称">{{
          detailData.rule_name
        }}</el-descriptions-item>
        <el-descriptions-item label="主体类型">{{
          detailData.subject_type
        }}</el-descriptions-item>
        <el-descriptions-item label="主体ID">{{
          detailData.subject_id ?? "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="资源类型">{{
          detailData.resource_type
        }}</el-descriptions-item>
        <el-descriptions-item label="资源ID">{{
          detailData.resource_id ?? "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="操作">{{
          detailData.action
        }}</el-descriptions-item>
        <el-descriptions-item label="条件">
          <pre>{{ JSON.stringify(detailData.condition_json, null, 2) }}</pre>
        </el-descriptions-item>
        <el-descriptions-item label="效果">
          <el-tag
            :type="detailData.effect === 'ALLOW' ? 'success' : 'danger'"
            size="small"
          >
            {{ detailData.effect }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="优先级">{{
          detailData.priority
        }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag
            :type="detailData.status === 1 ? 'success' : 'info'"
            size="small"
          >
            {{ detailData.status === 1 ? "启用" : "禁用" }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="创建时间">{{
          detailData.create_time || "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="更新时间">{{
          detailData.update_time || "-"
        }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import { Plus, Edit, View, Delete, Check, Lock } from "@element-plus/icons-vue";
import { usePagePermission } from "@/directives/usePermission";
import {
  getAllRules,
  queryRules,
  createRule,
  updateRule,
  deleteRule,
} from "@/api/abacPolicyRule";
import {
  type AbacPolicyRule,
  type CreateRuleReq,
  RuleStatus,
  type UpdateRuleReq,
  type QueryRulesReq,
  RuleEffect,
} from "@/types/abacPolicyRule";

// 权限控制
const { canCreate, canUpdate, canDelete, canView } = usePagePermission();

// 数据
const loading = ref(false);
const tableData = ref<AbacPolicyRule[]>([]);

// 查询参数
const queryParams = reactive<QueryRulesReq>({
  subject_type: "",
  resource_type: "",
  action: "",
  status: undefined,
});

// 弹窗状态
const dialogVisible = ref(false);
const dialogTitle = ref("");
const isEdit = ref(false);
const currentId = ref<number>();
const formRef = ref<FormInstance>();
const submitLoading = ref(false);
const conditionJsonStr = ref("");
const jsonError = ref("");

// 表单数据
const formData = reactive<CreateRuleReq & { status?: RuleStatus }>({
  rule_code: "",
  rule_name: "",
  subject_type: "",
  subject_id: undefined,
  resource_type: "",
  resource_id: undefined,
  action: "",
  condition_json: {},
  effect: "ALLOW" as RuleEffect,
  priority: 0,
  status: 1,
});

// 校验规则
const formRules: FormRules = {
  rule_code: [{ required: true, message: "请输入规则编码", trigger: "blur" }],
  rule_name: [{ required: true, message: "请输入规则名称", trigger: "blur" }],
  subject_type: [
    { required: true, message: "请输入主体类型", trigger: "blur" },
  ],
  resource_type: [
    { required: true, message: "请输入资源类型", trigger: "blur" },
  ],
  action: [{ required: true, message: "请输入操作", trigger: "blur" }],
  effect: [{ required: true, message: "请选择效果", trigger: "change" }],
  priority: [{ required: true, message: "请输入优先级", trigger: "blur" }],
};

// 详情
const detailVisible = ref(false);
const detailData = ref<AbacPolicyRule>({} as AbacPolicyRule);

// 加载规则列表
const loadRules = async () => {
  loading.value = true;
  try {
    const hasSearch =
      queryParams.subject_type ||
      queryParams.resource_type ||
      queryParams.action ||
      queryParams.status !== undefined;
    if (hasSearch) {
      tableData.value = await queryRules(queryParams);
    } else {
      tableData.value = await getAllRules();
    }
  } catch (error) {
    ElMessage.error("加载规则列表失败");
  } finally {
    loading.value = false;
  }
};

// 搜索
const handleSearch = () => {
  loadRules();
};

// 重置搜索
const resetSearch = () => {
  queryParams.subject_type = "";
  queryParams.resource_type = "";
  queryParams.action = "";
  queryParams.status = undefined;
  loadRules();
};

// 打开新增对话框
const openCreateDialog = () => {
  isEdit.value = false;
  dialogTitle.value = "新增规则";
  resetFormData();
  dialogVisible.value = true;
};

// 打开编辑对话框
const openEditDialog = (row: AbacPolicyRule) => {
  isEdit.value = true;
  dialogTitle.value = "编辑规则";
  currentId.value = row.id;
  Object.assign(formData, {
    rule_code: row.rule_code,
    rule_name: row.rule_name,
    subject_type: row.subject_type,
    subject_id: row.subject_id ?? undefined,
    resource_type: row.resource_type,
    resource_id: row.resource_id ?? undefined,
    action: row.action,
    condition_json: row.condition_json,
    effect: row.effect,
    priority: row.priority,
    status: row.status,
  });
  conditionJsonStr.value = JSON.stringify(row.condition_json, null, 2);
  jsonError.value = "";
  dialogVisible.value = true;
};

// 打开详情对话框
const openDetailDialog = (row: AbacPolicyRule) => {
  detailData.value = row;
  detailVisible.value = true;
};

// 删除规则
const handleDelete = (row: AbacPolicyRule) => {
  ElMessageBox.confirm(`确认删除规则“${row.rule_name}”吗？`, "提示", {
    confirmButtonText: "确定",
    cancelButtonText: "取消",
    type: "warning",
    customClass: "delete-confirm",
  })
    .then(async () => {
      try {
        await deleteRule(row.id);
        ElMessage.success("删除成功");
        loadRules();
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

// 提交表单
const handleSubmit = async () => {
  if (!formRef.value) return;

  // 验证 JSON 格式
  if (conditionJsonStr.value.trim()) {
    try {
      formData.condition_json = JSON.parse(conditionJsonStr.value);
      jsonError.value = "";
    } catch (e) {
      jsonError.value = "JSON 格式错误";
      return;
    }
  } else {
    formData.condition_json = {};
  }

  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    submitLoading.value = true;
    try {
      if (isEdit.value) {
        const updateData: UpdateRuleReq = {
          rule_code: formData.rule_code,
          rule_name: formData.rule_name,
          subject_type: formData.subject_type,
          subject_id: formData.subject_id,
          resource_type: formData.resource_type,
          resource_id: formData.resource_id,
          action: formData.action,
          condition_json: formData.condition_json,
          effect: formData.effect,
          priority: formData.priority,
          status: formData.status,
        };
        await updateRule(currentId.value!, updateData);
        ElMessage.success("更新成功");
      } else {
        const createData: CreateRuleReq = {
          rule_code: formData.rule_code,
          rule_name: formData.rule_name,
          subject_type: formData.subject_type,
          subject_id: formData.subject_id,
          resource_type: formData.resource_type,
          resource_id: formData.resource_id,
          action: formData.action,
          condition_json: formData.condition_json,
          effect: formData.effect,
          priority: formData.priority,
        };
        await createRule(createData);
        ElMessage.success("创建成功");
      }
      dialogVisible.value = false;
      loadRules();
    } catch (error) {
      ElMessage.error(isEdit.value ? "更新失败" : "创建失败");
    } finally {
      submitLoading.value = false;
    }
  });
};

// 重置表单
const resetFormData = () => {
  formData.rule_code = "";
  formData.rule_name = "";
  formData.subject_type = "";
  formData.subject_id = undefined;
  formData.resource_type = "";
  formData.resource_id = undefined;
  formData.action = "";
  formData.condition_json = {};
  formData.effect = RuleEffect.ALLOW;
  formData.priority = 0;
  formData.status = RuleStatus.ENABLED;
  conditionJsonStr.value = "";
  jsonError.value = "";
};

const resetForm = () => {
  formRef.value?.resetFields();
  resetFormData();
};

onMounted(() => {
  loadRules();
});
</script>

<style scoped>
.abac-rule-manage {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}

.rule-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  transition: all 0.3s ease;
  border: none;
  overflow: hidden;
}

.rule-card :deep(.el-card__header) {
  background: rgba(255, 255, 255, 0.5);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  padding: 18px 24px;
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

.create-btn {
  border-radius: 40px;
  padding: 9px 20px;
  background: linear-gradient(135deg, #409eff, #36d1dc);
  border: none;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
  transition: all 0.2s;
}
.create-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 18px rgba(64, 158, 255, 0.4);
}
.create-btn span {
  margin-left: 5px;
}

.search-area {
  margin-bottom: 20px;
  padding: 0 8px;
}

.rule-table {
  border-radius: 16px;
  overflow: hidden;
}
.rule-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.rule-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.rule-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.rule-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
.rule-table :deep(.el-table__row .el-button) {
  margin: 0 4px;
}

.rule-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.rule-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  margin: 0;
  padding: 20px 24px;
}
.rule-dialog :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #409eff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.rule-dialog :deep(.el-dialog__body) {
  padding: 24px;
}
.rule-dialog :deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}

.rule-form .el-form-item {
  margin-bottom: 20px;
}
.json-error {
  color: #f56c6c;
  font-size: 12px;
  margin-top: 4px;
}
.is-error :deep(.el-textarea__inner) {
  border-color: #f56c6c;
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
.dialog-footer .el-button--primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 14px rgba(64, 158, 255, 0.4);
}

:deep(.delete-confirm) {
  border-radius: 24px;
  backdrop-filter: blur(4px);
}
</style>