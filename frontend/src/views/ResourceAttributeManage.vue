<template>
  <div class="resource-attribute-manage">
    <el-card class="attribute-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Collection /></el-icon>
            <span class="header-title">资源属性管理</span>
          </div>
          <div class="header-right">
            <el-button
              v-if="canExport"
              type="success"
              :icon="Download"
              @click="handleExport"
              >导出</el-button
            >
            <el-button
              v-if="canImport"
              type="primary"
              :icon="Upload"
              @click="handleImport"
              >导入</el-button
            >
            <el-button
              v-if="canCreate"
              type="primary"
              @click="openCreateDialog"
              class="create-btn"
            >
              <el-icon><Plus /></el-icon><span>新增资源</span>
            </el-button>
          </div>
        </div>
      </template>

      <!-- 搜索区域（支持路径锁定） -->
      <div class="search-area">
        <el-form :model="queryParams" inline>
          <el-form-item label="资源类型">
            <div v-if="isResourceTypeLocked" class="locked-type">
              <el-tag type="info" effect="plain">{{
                routeResourceType
              }}</el-tag>
              <span class="lock-tip">（由路径锁定）</span>
            </div>
            <el-input
              v-else
              v-model="queryParams.resource_type"
              placeholder="请输入资源类型"
              clearable
            />
          </el-form-item>
          <el-form-item label="资源ID">
            <el-input-number
              v-model="queryParams.resource_id"
              :min="1"
              controls-position="right"
            />
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
        class="attribute-table"
      >
        <el-table-column prop="id" label="ID" width="80" align="center" />
        <el-table-column prop="resource_type" label="资源类型" width="150" />
        <el-table-column
          prop="resource_id"
          label="资源ID"
          width="100"
          align="center"
        />
        <el-table-column label="属性预览" min-width="250">
          <template #default="{ row }">
            <pre class="value-preview">{{
              formatPreview(row.attr_key_value)
            }}</pre>
          </template>
        </el-table-column>
        <el-table-column prop="create_time" label="创建时间" min-width="160" />
        <el-table-column label="操作" width="280" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="管理属性" placement="top" v-if="canUpdate">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click="openAttributeDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="查看完整属性" placement="top" v-if="canView">
              <el-button
                size="small"
                circle
                :icon="View"
                @click="viewFullAttributes(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除资源" placement="top" v-if="canDelete">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click="handleDeleteResource(row)"
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

    <!-- 新增资源弹窗 -->
    <el-dialog
      v-model="createDialogVisible"
      title="新增资源"
      width="550px"
      class="attribute-dialog"
      :close-on-click-modal="false"
      @close="resetCreateForm"
    >
      <el-form
        ref="createFormRef"
        :model="createForm"
        :rules="createRules"
        label-width="100px"
      >
        <el-form-item label="资源类型" prop="resource_type">
          <el-input
            v-model="createForm.resource_type"
            placeholder="例如：AIRCRAFT"
            clearable
          />
        </el-form-item>
        <el-form-item label="资源ID" prop="resource_id">
          <el-input-number
            v-model="createForm.resource_id"
            :min="1"
            controls-position="right"
          />
        </el-form-item>
        <el-form-item label="初始属性">
          <div
            v-for="(item, index) in initAttrsList"
            :key="index"
            class="attr-row"
          >
            <el-input
              v-model="item.key"
              placeholder="属性键"
              style="width: 40%; margin-right: 10px"
            />
            <el-input
              v-model="item.value"
              placeholder="属性值"
              style="width: 50%"
            />
            <el-button
              type="danger"
              :icon="Delete"
              circle
              size="small"
              @click="removeAttrRow(index)"
              v-if="initAttrsList.length > 1"
            />
          </div>
          <el-button
            type="primary"
            link
            @click="addAttrRow"
            style="margin-top: 8px"
          >
            <el-icon><Plus /></el-icon> 添加属性
          </el-button>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="createDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="createLoading"
            @click="handleCreateResource"
            >确定创建</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 属性管理弹窗（可编辑） -->
    <el-dialog
      v-model="attributeDialogVisible"
      :title="`管理属性 - ${currentResourceType}:${currentResourceId}`"
      width="800px"
      class="attribute-dialog"
      :close-on-click-modal="false"
      @close="closeAttributeDialog"
    >
      <div class="attr-manage-header">
        <div>
          <el-button
            type="primary"
            size="small"
            @click="openAddAttrDialog"
            v-if="canUpdate"
          >
            <el-icon><Plus /></el-icon> 添加属性
          </el-button>
          <el-button
            type="warning"
            size="small"
            @click="openBatchDialog"
            v-if="canUpdate"
          >
            <el-icon><Edit /></el-icon> 批量设置
          </el-button>
          <el-button
            type="danger"
            size="small"
            @click="handleDeleteAllAttrs"
            v-if="canDelete && Object.keys(attributes).length"
          >
            <el-icon><Delete /></el-icon> 清空所有
          </el-button>
        </div>
      </div>
      <el-table
        :data="attributeEntries"
        v-loading="attrLoading"
        border
        stripe
        class="attr-table"
      >
        <el-table-column label="键" prop="key" min-width="200" />
        <el-table-column label="值" prop="value" min-width="300">
          <template #default="{ row }">
            <pre class="value-preview">{{ formatDisplayValue(row) }}</pre>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top" v-if="canUpdate">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click="openEditAttrDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top" v-if="canDelete">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click="handleDeleteAttr(row)"
              />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 添加/编辑属性弹窗 -->
    <el-dialog
      v-model="attrFormDialogVisible"
      :title="attrDialogTitle"
      width="500px"
      class="attribute-dialog"
      :close-on-click-modal="false"
      @close="resetAttrForm"
    >
      <el-form
        ref="attrFormRef"
        :model="attrFormData"
        :rules="attrFormRules"
        label-width="80px"
      >
        <el-form-item label="键" prop="key">
          <el-input
            v-model="attrFormData.key"
            placeholder="属性键"
            :disabled="isEditAttr"
          />
        </el-form-item>
        <el-form-item label="值" prop="value">
          <el-input
            v-model="attrFormData.value"
            type="textarea"
            :rows="3"
            placeholder="可以是字符串、数字、JSON对象"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="attrFormDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="attrSubmitLoading"
            @click="handleSubmitAttr"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 批量设置弹窗 -->
    <el-dialog
      v-model="batchVisible"
      title="批量设置属性"
      width="600px"
      class="attribute-dialog"
      :close-on-click-modal="false"
      @close="resetBatchForm"
    >
      <el-alert
        title="批量设置说明"
        type="info"
        description="请输入一个 JSON 对象，键值对将会合并到现有属性中（相同键会覆盖）。所有值必须为字符串。"
        show-icon
        :closable="false"
        style="margin-bottom: 16px"
      />
      <el-input
        v-model="batchJsonStr"
        type="textarea"
        rows="8"
        placeholder='{"model": "Boeing 737", "range": "4000km"}'
        :class="{ 'is-error': batchJsonError }"
      />
      <div v-if="batchJsonError" class="value-error">{{ batchJsonError }}</div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="batchVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="batchLoading"
            @click="handleBatchSubmit"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 查看完整属性弹窗（只读） -->
    <el-dialog
      v-model="fullAttrDialogVisible"
      :title="`完整属性 - ${fullAttrResourceType}:${fullAttrResourceId}`"
      width="700px"
      class="attribute-dialog"
    >
      <pre class="full-attributes">{{ fullAttributesJson }}</pre>
      <template #footer>
        <el-button @click="fullAttrDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import { useRoute } from "vue-router";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import {
  Plus,
  Edit,
  Delete,
  Download,
  Upload,
  Collection,
  View,
} from "@element-plus/icons-vue";
import * as XLSX from "xlsx";
import { usePagePermission } from "@/directives/usePermission";
import {
  getAllRecords,
  getRecordsByType,
  getAttributes,
  setAttribute,
  removeAttribute,
  setAllAttributes,
  deleteAllAttributes,
  deleteResource,
} from "@/api/resourceAttribute";
import type { ResourceAttributeRecord } from "@/types/resourceAttribute";

const { canCreate, canUpdate, canDelete, canView, canExport, canImport } =
  usePagePermission();

// ========== 路由参数处理 ==========
const route = useRoute();
const routeResourceType = computed(() => (route.params.type as string) || "");
const isResourceTypeLocked = computed(() => !!routeResourceType.value);

// ---------- 列表相关 ----------
const loading = ref(false);
const tableData = ref<ResourceAttributeRecord[]>([]);
const total = ref(0);
const queryParams = reactive({
  page: 1,
  page_size: 10,
  resource_type: "",
  resource_id: undefined as number | undefined,
});

// 加载列表（优先使用路由参数）
const loadList = async () => {
  loading.value = true;
  try {
    let res: ResourceAttributeRecord[];
    if (routeResourceType.value) {
      res = await getRecordsByType(routeResourceType.value);
    } else if (queryParams.resource_type) {
      res = await getRecordsByType(queryParams.resource_type);
    } else {
      res = await getAllRecords();
    }
    let filtered = res;
    if (queryParams.resource_id) {
      filtered = res.filter(
        (item) => item.resource_id === queryParams.resource_id
      );
    }
    total.value = filtered.length;
    const start = (queryParams.page - 1) * queryParams.page_size;
    const end = start + queryParams.page_size;
    tableData.value = filtered.slice(start, end);
  } catch (error) {
    ElMessage.error("加载列表失败");
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  queryParams.page = 1;
  loadList();
};
const resetSearch = () => {
  if (!isResourceTypeLocked.value) {
    queryParams.resource_type = "";
  }
  queryParams.resource_id = undefined;
  queryParams.page = 1;
  loadList();
};
const handleSizeChange = (size: number) => {
  queryParams.page_size = size;
  queryParams.page = 1;
  loadList();
};
const handleCurrentChange = (page: number) => {
  queryParams.page = page;
  loadList();
};

// ---------- 查看完整属性 ----------
const fullAttrDialogVisible = ref(false);
const fullAttrResourceType = ref("");
const fullAttrResourceId = ref(0);
const fullAttributesJson = ref("");

const viewFullAttributes = async (row: ResourceAttributeRecord) => {
  fullAttrResourceType.value = row.resource_type;
  fullAttrResourceId.value = row.resource_id;
  try {
    const attrs = await getAttributes(row.resource_type, row.resource_id);

    // ======================
    // ✅ 深度解析所有嵌套 JSON（去掉转义符）
    // ======================
    const parsedAttrs = deepParseJson(attrs);

    fullAttributesJson.value = JSON.stringify(parsedAttrs, null, 2);
    fullAttrDialogVisible.value = true;
  } catch (error) {
    ElMessage.error("获取属性失败");
  }
};

// ======================
// ✅ 新增：深度递归解析 JSON 字符串
// ======================
function deepParseJson(obj: Record<string, any>): Record<string, any> {
  const result: Record<string, any> = {};
  for (const key in obj) {
    let value = obj[key];
    try {
      // 尝试解析
      const parsed = JSON.parse(value);
      // 如果是对象，递归解析
      if (typeof parsed === "object" && parsed !== null) {
        result[key] = deepParseJson(parsed);
      } else {
        result[key] = parsed;
      }
    } catch {
      // 解析失败就保留原值
      result[key] = value;
    }
  }
  return result;
}

// ---------- 属性管理（编辑） ----------
const attributeDialogVisible = ref(false);
const currentResourceType = ref("");
const currentResourceId = ref(0);
const attributes = ref<Record<string, string>>({});
const attrLoading = ref(false);

// 将属性对象转为表格数据，并处理值的显示格式
const attributeEntries = computed(() => {
  return Object.entries(attributes.value).map(([key, value]) => ({
    key,
    rawValue: value,
    displayValue: formatDisplayValue(value),
  }));
});

const formatDisplayValue = (row: any): string => {
  // 如果 row 本身是 null/undefined
  if (row == null) return "";

  // 如果 row 是普通值（字符串、数字等），直接返回
  if (typeof row !== "object") return String(row);

  // 如果 row 是数组（一般不会），返回 JSON
  if (Array.isArray(row)) return JSON.stringify(row, null, 2);

  // 如果 row 包含 value 字段（我们的预期结构 { key, value }）
  if ("value" in row) {
    const val = row.value;
    if (val === null || val === undefined) return "";
    if (typeof val === "object") return JSON.stringify(val, null, 2);
    return String(val);
  }

  // 如果 row 包含 rawValue 字段（旧结构兼容）
  if ("rawValue" in row) {
    const val = row.rawValue;
    if (val === null || val === undefined) return "";
    if (typeof val === "object") return JSON.stringify(val, null, 2);
    return String(val);
  }

  // 否则将整个对象转为 JSON 字符串（备选）
  return JSON.stringify(row, null, 2);
};

// 编辑时使用：彻底保证传入文本框的是字符串，不是对象
const parseValueForEdit = (val: string | Record<string, any>): string => {
  if (!val) return "";

  // 如果是对象 → 转成 JSON 字符串
  if (typeof val === "object" && !Array.isArray(val)) {
    return JSON.stringify(val, null, 2);
  }

  // 如果是字符串 → 尝试格式化
  try {
    const parsed = JSON.parse(val as string);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return val as string;
  }
};

const openAttributeDialog = async (row: ResourceAttributeRecord) => {
  currentResourceType.value = row.resource_type;
  currentResourceId.value = row.resource_id;
  attrLoading.value = true;
  attributeDialogVisible.value = true;
  try {
    const attrs = await getAttributes(row.resource_type, row.resource_id);
    attributes.value = attrs;
  } catch (error) {
    ElMessage.error("加载属性失败");
  } finally {
    attrLoading.value = false;
  }
};
const closeAttributeDialog = () => {
  attributeDialogVisible.value = false;
  attributes.value = {};
  currentResourceType.value = "";
  currentResourceId.value = 0;
};

// 添加/编辑属性
const attrFormDialogVisible = ref(false);
const attrDialogTitle = ref("");
const isEditAttr = ref(false);
const attrFormRef = ref<FormInstance>();
const attrSubmitLoading = ref(false);
const attrFormData = reactive({ key: "", value: "" });
const attrFormRules: FormRules = {
  key: [{ required: true, message: "请输入键", trigger: "blur" }],
};

const openAddAttrDialog = () => {
  isEditAttr.value = false;
  attrDialogTitle.value = "添加属性";
  attrFormData.key = "";
  attrFormData.value = "";
  attrFormDialogVisible.value = true;
};
const openEditAttrDialog = (row: { key: string; rawValue: string }) => {
  isEditAttr.value = true;
  attrDialogTitle.value = "编辑属性";
  attrFormData.key = row.key;
  attrFormData.value = parseValueForEdit(row.rawValue);
  attrFormDialogVisible.value = true;
};
const resetAttrForm = () => {
  attrFormRef.value?.resetFields();
};
const handleSubmitAttr = async () => {
  if (!attrFormRef.value) return;
  await attrFormRef.value.validate(async (valid) => {
    if (!valid) return;
    attrSubmitLoading.value = true;
    const finalValue = attrFormData.value.trim();
    try {
      // 提交时，值保持字符串形式（后端存储字符串，前端无需额外处理）
      await setAttribute(
        currentResourceType.value,
        currentResourceId.value,
        attrFormData.key,
        finalValue
      );
      ElMessage.success(isEditAttr.value ? "更新成功" : "添加成功");
      const attrs = await getAttributes(
        currentResourceType.value,
        currentResourceId.value
      );
      attributes.value = attrs;
      attrFormDialogVisible.value = false;
      await loadList();
    } catch (error) {
      ElMessage.error(isEditAttr.value ? "更新失败" : "添加失败");
    } finally {
      attrSubmitLoading.value = false;
    }
  });
};

const handleDeleteAttr = (row: { key: string }) => {
  ElMessageBox.confirm(`确认删除属性“${row.key}”吗？`, "提示", {
    type: "warning",
  })
    .then(async () => {
      try {
        await removeAttribute(
          currentResourceType.value,
          currentResourceId.value,
          row.key
        );
        ElMessage.success("删除成功");
        const attrs = await getAttributes(
          currentResourceType.value,
          currentResourceId.value
        );
        attributes.value = attrs;
        await loadList();
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

const handleDeleteAllAttrs = () => {
  ElMessageBox.confirm(
    `确认清空资源“${currentResourceType.value}:${currentResourceId.value}”的所有属性吗？`,
    "提示",
    { type: "warning" }
  )
    .then(async () => {
      try {
        await deleteAllAttributes(
          currentResourceType.value,
          currentResourceId.value
        );
        ElMessage.success("清空成功");
        attributes.value = {};
        await loadList();
      } catch (error) {
        ElMessage.error("清空失败");
      }
    })
    .catch(() => {});
};

// 批量设置
const batchVisible = ref(false);
const batchJsonStr = ref("");
const batchJsonError = ref("");
const batchLoading = ref(false);
const openBatchDialog = () => {
  batchJsonStr.value = JSON.stringify(attributes.value, null, 2);
  batchJsonError.value = "";
  batchVisible.value = true;
};
const resetBatchForm = () => {
  batchJsonStr.value = "";
  batchJsonError.value = "";
};
const handleBatchSubmit = async () => {
  if (!batchJsonStr.value.trim()) {
    batchJsonError.value = "请输入 JSON 对象";
    return;
  }
  let attrs: Record<string, string>;
  try {
    const parsed = JSON.parse(batchJsonStr.value);
    if (typeof parsed !== "object" || Array.isArray(parsed)) throw new Error();
    // 将所有值转为字符串（后端要求字符串）
    attrs = Object.fromEntries(
      Object.entries(parsed).map(([k, v]) => [k, String(v)])
    );
  } catch {
    batchJsonError.value = "JSON 格式错误，请输入有效的对象";
    return;
  }
  batchJsonError.value = "";
  batchLoading.value = true;
  try {
    await setAllAttributes(
      currentResourceType.value,
      currentResourceId.value,
      attrs
    );
    ElMessage.success("批量设置成功");
    batchVisible.value = false;
    const newAttrs = await getAttributes(
      currentResourceType.value,
      currentResourceId.value
    );
    attributes.value = newAttrs;
    await loadList();
  } catch (error) {
    ElMessage.error("批量设置失败");
  } finally {
    batchLoading.value = false;
  }
};

// 删除资源
const handleDeleteResource = (row: ResourceAttributeRecord) => {
  ElMessageBox.confirm(
    `确认删除资源“${row.resource_type}:${row.resource_id}”吗？所有属性将被清除。`,
    "提示",
    { type: "warning" }
  )
    .then(async () => {
      try {
        await deleteResource(row.resource_type, row.resource_id);
        ElMessage.success("删除成功");
        await loadList();
        if (
          currentResourceId.value === row.resource_id &&
          currentResourceType.value === row.resource_type
        ) {
          closeAttributeDialog();
        }
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

// ---------- 新增资源 ----------
const createDialogVisible = ref(false);
const createFormRef = ref<FormInstance>();
const createLoading = ref(false);
const initAttrsList = ref([{ key: "", value: "" }]);
const createForm = reactive({
  resource_type: "",
  resource_id: undefined as number | undefined,
});
const createRules: FormRules = {
  resource_type: [
    { required: true, message: "请输入资源类型", trigger: "blur" },
  ],
  resource_id: [{ required: true, message: "请输入资源ID", trigger: "blur" }],
};

const openCreateDialog = () => {
  createForm.resource_type = routeResourceType.value || "";
  createForm.resource_id = undefined;
  initAttrsList.value = [{ key: "", value: "" }];
  createDialogVisible.value = true;
};
const resetCreateForm = () => {
  createFormRef.value?.resetFields();
  initAttrsList.value = [{ key: "", value: "" }];
};
const addAttrRow = () => initAttrsList.value.push({ key: "", value: "" });
const removeAttrRow = (index: number) => initAttrsList.value.splice(index, 1);

const handleCreateResource = async () => {
  if (!createFormRef.value) return;
  await createFormRef.value.validate(async (valid) => {
    if (!valid) return;
    createLoading.value = true;
    try {
      const initAttrs: Record<string, string> = {};
      for (const item of initAttrsList.value) {
        if (item.key.trim()) {
          initAttrs[item.key] = item.value.trim() || "";
        }
      }
      await setAllAttributes(
        createForm.resource_type,
        createForm.resource_id!,
        initAttrs
      );
      ElMessage.success("资源创建成功");
      createDialogVisible.value = false;
      await loadList();
    } catch (error) {
      ElMessage.error("创建失败");
    } finally {
      createLoading.value = false;
    }
  });
};

// ---------- 导入导出 ----------
const handleExport = () => {
  if (!canExport.value) return;
  const exportData = tableData.value.map((record) => ({
    ID: record.id,
    资源类型: record.resource_type,
    资源ID: record.resource_id,
    // ✅ 这里导出完整 JSON，不截断、不省略
    属性预览: JSON.stringify(record.attr_key_value, null, 2),
    创建时间: record.create_time,
  }));
  const ws = XLSX.utils.json_to_sheet(exportData);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, "资源属性列表");
  XLSX.writeFile(
    wb,
    `资源属性列表_${new Date()
      .toISOString()
      .slice(0, 19)
      .replace(/:/g, "-")}.xlsx`
  );
  ElMessage.success("导出成功");
};

const handleImport = () => {
  if (!canImport.value) return;
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".xlsx, .xls, .csv";
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (evt) => {
      const data = evt.target?.result;
      const workbook = XLSX.read(data, { type: "binary" });
      const sheetName = workbook.SheetNames[0];
      const worksheet = workbook.Sheets[sheetName];
      const rows = XLSX.utils.sheet_to_json(worksheet) as any[];
      if (!rows.length) {
        ElMessage.warning("文件无数据");
        return;
      }
      let successCount = 0;
      let failCount = 0;
      for (const row of rows) {
        try {
          const resourceType = row["资源类型"] || row["resource_type"];
          const resourceId = row["资源ID"] || row["resource_id"];
          if (!resourceType || !resourceId) {
            failCount++;
            continue;
          }

          // ======================
          // ✅ 【修复核心】正确解析 JSON，支持嵌套对象
          // ======================
          let attrs: Record<string, string> = {};
          const preview = row["属性预览"] || row["attr_key_value"];

          if (preview && typeof preview === "string") {
            let parsedAttrs: Record<string, any> = {};

            try {
              // 1. 优先尝试直接解析完整 JSON（导出的格式）
              parsedAttrs = JSON.parse(preview);
            } catch {
              // 2. 如果解析失败，尝试按 key:value 格式兼容解析
              const pairs = preview
                .split(/[,;，；\n]/)
                .map((p) => p.trim())
                .filter(Boolean);
              for (const pair of pairs) {
                let colonIndex = pair.indexOf(":");
                if (colonIndex === -1) colonIndex = pair.indexOf("：");
                if (colonIndex === -1) continue;
                const k = pair.substring(0, colonIndex).trim();
                const v = pair.substring(colonIndex + 1).trim();
                if (k) parsedAttrs[k] = v;
              }
            }

            // ✅ 把所有值 安全转字符串（嵌套对象也能正常存储）
            attrs = Object.fromEntries(
              Object.entries(parsedAttrs).map(([k, v]) => {
                if (typeof v === "object" && v !== null) {
                  return [k, JSON.stringify(v)]; // 对象 → JSON字符串
                } else {
                  return [k, String(v)]; // 其他 → 字符串
                }
              })
            );
          }

          await setAllAttributes(resourceType, Number(resourceId), attrs);
          successCount++;
        } catch (error) {
          console.error("导入失败", error);
          failCount++;
        }
      }
      ElMessage.success(
        `导入完成：成功 ${successCount} 条，失败 ${failCount} 条`
      );
      if (successCount > 0) await loadList();
    };
    reader.readAsBinaryString(file);
  };
  input.click();
};

// 辅助函数：导出 + 表格预览
const formatPreview = (
  attrs: Record<string, string | Record<string, any>>
): string => {
  if (!attrs) return "";

  const entries = Object.entries(attrs).slice(0, 3);

  // 统一处理：如果值是对象，转成字符串
  const preview = entries
    .map(([k, v]) => {
      if (typeof v === "object" && v !== null) {
        return `${k}:${JSON.stringify(v)}`;
      }
      return `${k}:${v}`;
    })
    .join(", ");

  return Object.keys(attrs).length > 3 ? `${preview}...` : preview;
};

onMounted(() => {
  loadList();
});
</script>

<style scoped>
/* 新增锁定样式 */
.locked-type {
  display: flex;
  align-items: center;
  gap: 8px;
}
.lock-tip {
  font-size: 12px;
  color: #909399;
}
/* 原有样式保持不变，略... */
.resource-attribute-manage {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.attribute-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  border: none;
  overflow: hidden;
}
.attribute-card :deep(.el-card__header) {
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
.header-right {
  display: flex;
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
.search-area {
  margin-bottom: 20px;
  padding: 0 8px;
}
.attribute-table {
  border-radius: 16px;
  overflow: hidden;
}
.attribute-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.attribute-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.attribute-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.attribute-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
.value-preview {
  margin: 0;
  font-family: monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 80px;
  overflow: auto;
}
.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  padding: 0 8px;
}
.attribute-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.attribute-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  margin: 0;
  padding: 20px 24px;
}
.attribute-dialog :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #409eff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.attribute-dialog :deep(.el-dialog__body) {
  padding: 24px;
}
.attribute-dialog :deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}
.attr-row {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
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
.attr-manage-header {
  margin-bottom: 16px;
  text-align: right;
}
.attr-table {
  margin-top: 8px;
}
.value-error {
  color: #f56c6c;
  font-size: 12px;
  margin-top: 4px;
}
.is-error :deep(.el-textarea__inner) {
  border-color: #f56c6c;
}
</style>