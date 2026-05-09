<template>
  <div class="subject-attribute-manage">
    <el-card class="attribute-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Avatar /></el-icon>
            <span class="header-title">主体属性管理</span>
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
              <el-icon><Plus /></el-icon><span>新增主体</span>
            </el-button>
          </div>
        </div>
      </template>

      <!-- 搜索区域 -->
      <div class="search-area">
        <el-form :model="queryParams" inline>
          <el-form-item label="主体类型">
            <div v-if="isSubjectTypeLocked" class="locked-type">
              <el-tag type="info" effect="plain">{{ routeSubjectType }}</el-tag>
              <span class="lock-tip">（由路径锁定）</span>
            </div>
            <el-input
              v-else
              v-model="queryParams.subject_type"
              placeholder="请输入主体类型"
              clearable
            />
          </el-form-item>
          <el-form-item label="主体ID">
            <el-input-number
              v-model="queryParams.subject_id"
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
        <el-table-column prop="subject_type" label="主体类型" width="120" />
        <el-table-column
          prop="subject_id"
          label="主体ID"
          width="100"
          align="center"
        />
        <el-table-column label="用户名" width="150" align="center">
          <template #default="{ row }">
            <span v-if="userNameMap.get(row.subject_id)">{{
              userNameMap.get(row.subject_id)
            }}</span>
            <span v-else-if="loadingUserNameMap.get(row.subject_id)"
              >加载中...</span
            >
            <span v-else>-</span>
          </template>
        </el-table-column>
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
            <el-tooltip content="删除主体" placement="top" v-if="canDelete">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click="handleDeleteSubject(row)"
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

    <!-- 新增主体弹窗 -->
    <el-dialog
      v-model="createDialogVisible"
      title="新增主体"
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
        <el-form-item label="选择用户" prop="user_id">
          <el-select
            v-model="createForm.user_id"
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
        <el-form-item label="主体类型" prop="subject_type">
          <el-input
            v-model="createForm.subject_type"
            placeholder="例如：USER"
            clearable
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
            @click="handleCreateSubject"
            >确定创建</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 属性管理弹窗（可编辑） -->
    <el-dialog
      v-model="attributeDialogVisible"
      :title="`管理属性 - ${currentSubjectType}:${currentSubjectId}`"
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
            <pre class="value-preview">{{ formatValue(row.value) }}</pre>
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
            rows="3"
            placeholder='可以是字符串、数字、JSON对象等，如：{"a":1}'
            :class="{ 'is-error': attrValueError }"
          />
          <div v-if="attrValueError" class="value-error">
            {{ attrValueError }}
          </div>
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
        description="请输入一个 JSON 对象，键值对将会合并到现有属性中（相同键会覆盖）。如果希望完全替换所有属性，请先清空。"
        show-icon
        :closable="false"
        style="margin-bottom: 16px"
      />
      <el-input
        v-model="batchJsonStr"
        type="textarea"
        rows="8"
        placeholder='{"role": "admin", "level": 5, "tags": ["vip"]}'
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
      :title="`完整属性 - ${fullAttrSubjectType}:${fullAttrSubjectId}`"
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
  Avatar,
  View,
} from "@element-plus/icons-vue";
import * as XLSX from "xlsx";
import pLimit from "p-limit";
import { usePagePermission } from "@/directives/usePermission";
import {
  setAttributes,
  getAllRecords,
  getRecordsByType,
  getAttributesBySubject,
  removeAttribute,
  deleteAllAttributes,
  deleteSubject,
} from "@/api/subjectAttribute";
import { listUsers, getUser } from "@/api/user";
import type { SubjectAttributeRecord } from "@/types/subjectAttribute";

const { canCreate, canUpdate, canDelete, canView, canExport, canImport } =
  usePagePermission();

// ========== 路由参数处理 ==========
const route = useRoute();
const routeSubjectType = computed(() => (route.params.type as string) || "");
const isSubjectTypeLocked = computed(() => !!routeSubjectType.value);

// ---------- 列表相关 ----------
const loading = ref(false);
const tableData = ref<SubjectAttributeRecord[]>([]);
const total = ref(0);
const queryParams = reactive({
  page: 1,
  page_size: 10,
  subject_type: "",
  subject_id: undefined as number | undefined,
});

// 用户名映射（缓存 + 加载状态）
const userNameMap = ref<Map<number, string>>(new Map());
const loadingUserNameMap = ref<Map<number, boolean>>(new Map());

// 并发控制（限制同时最多 10 个请求）
const limit = pLimit(10);

// 获取单个用户名
const fetchUserName = async (subjectId: number): Promise<string> => {
  if (!subjectId || subjectId <= 0) return "-";
  if (userNameMap.value.has(subjectId)) {
    return userNameMap.value.get(subjectId)!;
  }
  loadingUserNameMap.value.set(subjectId, true);
  try {
    const user = await getUser(subjectId);
    const name = user.real_name || user.username;
    userNameMap.value.set(subjectId, name);
    return name;
  } catch {
    userNameMap.value.set(subjectId, "-");
    return "-";
  } finally {
    loadingUserNameMap.value.set(subjectId, false);
  }
};

// 批量加载用户名（并行，限制并发）
const batchLoadUserNames = async (records: SubjectAttributeRecord[]) => {
  const ids = records
    .map((r) => r.subject_id)
    .filter((id) => id && id > 0 && !userNameMap.value.has(id));
  if (ids.length === 0) return;
  const promises = ids.map((id) => limit(() => fetchUserName(id)));
  await Promise.all(promises);
};

// 加载列表（优先使用路由参数）
const loadList = async () => {
  loading.value = true;
  try {
    let res: SubjectAttributeRecord[];
    if (routeSubjectType.value) {
      res = await getRecordsByType(routeSubjectType.value);
    } else if (queryParams.subject_type) {
      res = await getRecordsByType(queryParams.subject_type);
    } else {
      res = await getAllRecords();
    }

    let filtered = res;
    if (queryParams.subject_id) {
      filtered = res.filter(
        (item) => item.subject_id === queryParams.subject_id
      );
    }

    total.value = filtered.length;
    const start = (queryParams.page - 1) * queryParams.page_size;
    const end = start + queryParams.page_size;
    tableData.value = filtered.slice(start, end);
    await batchLoadUserNames(tableData.value);
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
  if (!isSubjectTypeLocked.value) {
    queryParams.subject_type = "";
  }
  queryParams.subject_id = undefined;
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
const fullAttrSubjectType = ref("");
const fullAttrSubjectId = ref(0);
const fullAttributesJson = ref("");

const viewFullAttributes = async (row: SubjectAttributeRecord) => {
  fullAttrSubjectType.value = row.subject_type;
  fullAttrSubjectId.value = row.subject_id;
  try {
    const attrs = await getAttributesBySubject(
      row.subject_type,
      row.subject_id
    );
    fullAttributesJson.value = JSON.stringify(attrs, null, 2);
    fullAttrDialogVisible.value = true;
  } catch (error) {
    ElMessage.error("获取属性失败");
  }
};

// ---------- 属性管理 ----------
const attributeDialogVisible = ref(false);
const currentSubjectType = ref("");
const currentSubjectId = ref(0);
const currentRecordId = ref(0);
const attributes = ref<Record<string, any>>({});
const attrLoading = ref(false);
const attributeEntries = computed(() =>
  Object.entries(attributes.value).map(([key, value]) => ({ key, value }))
);

const openAttributeDialog = async (row: SubjectAttributeRecord) => {
  currentSubjectType.value = row.subject_type;
  currentSubjectId.value = row.subject_id;
  currentRecordId.value = row.id;
  attrLoading.value = true;
  attributeDialogVisible.value = true;
  try {
    const attrs = await getAttributesBySubject(
      row.subject_type,
      row.subject_id
    );
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
  currentSubjectType.value = "";
  currentSubjectId.value = 0;
  currentRecordId.value = 0;
};

// 添加/编辑属性
const attrFormDialogVisible = ref(false);
const attrDialogTitle = ref("");
const isEditAttr = ref(false);
const attrFormRef = ref<FormInstance>();
const attrSubmitLoading = ref(false);
const attrValueError = ref("");
const attrFormData = reactive({ key: "", value: "" });
const attrFormRules: FormRules = {
  key: [{ required: true, message: "请输入键", trigger: "blur" }],
};

const openAddAttrDialog = () => {
  isEditAttr.value = false;
  attrDialogTitle.value = "添加属性";
  attrFormData.key = "";
  attrFormData.value = "";
  attrValueError.value = "";
  attrFormDialogVisible.value = true;
};

const openEditAttrDialog = (row: { key: string; value: any }) => {
  isEditAttr.value = true;
  attrDialogTitle.value = "编辑属性";
  attrFormData.key = row.key;
  attrFormData.value = formatValue(row.value);
  attrValueError.value = "";
  attrFormDialogVisible.value = true;
};

const resetAttrForm = () => {
  attrFormRef.value?.resetFields();
  attrValueError.value = "";
};

const handleSubmitAttr = async () => {
  if (!attrFormRef.value) return;
  await attrFormRef.value.validate(async (valid) => {
    if (!valid) return;
    let parsedValue: any;
    if (
      typeof attrFormData.value === "string" &&
      attrFormData.value.trim() === ""
    ) {
      parsedValue = "";
    } else {
      try {
        parsedValue = JSON.parse(attrFormData.value);
      } catch {
        parsedValue = attrFormData.value;
      }
    }
    attrSubmitLoading.value = true;
    try {
      const newAttrs = { ...attributes.value };
      newAttrs[attrFormData.key] = parsedValue;
      await setAttributes(
        currentSubjectType.value,
        currentSubjectId.value,
        newAttrs
      );
      ElMessage.success(isEditAttr.value ? "更新成功" : "添加成功");
      const attrs = await getAttributesBySubject(
        currentSubjectType.value,
        currentSubjectId.value
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
          currentSubjectType.value,
          currentSubjectId.value,
          row.key
        );
        ElMessage.success("删除成功");
        const attrs = await getAttributesBySubject(
          currentSubjectType.value,
          currentSubjectId.value
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
    `确认清空主体“${currentSubjectType.value}:${currentSubjectId.value}”的所有属性吗？`,
    "提示",
    { type: "warning" }
  )
    .then(async () => {
      try {
        await deleteAllAttributes(currentRecordId.value);
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
  let attrs: Record<string, any>;
  try {
    attrs = JSON.parse(batchJsonStr.value);
    if (typeof attrs !== "object" || Array.isArray(attrs)) throw new Error();
  } catch {
    batchJsonError.value = "JSON 格式错误，请输入有效的对象";
    return;
  }
  batchJsonError.value = "";
  batchLoading.value = true;
  try {
    await setAttributes(
      currentSubjectType.value,
      currentSubjectId.value,
      attrs
    );
    ElMessage.success("批量设置成功");
    batchVisible.value = false;
    const newAttrs = await getAttributesBySubject(
      currentSubjectType.value,
      currentSubjectId.value
    );
    attributes.value = newAttrs;
    await loadList();
  } catch (error) {
    ElMessage.error("批量设置失败");
  } finally {
    batchLoading.value = false;
  }
};

// 删除主体
const handleDeleteSubject = (row: SubjectAttributeRecord) => {
  ElMessageBox.confirm(
    `确认删除主体“${row.subject_type}:${row.subject_id}”吗？所有属性将被清除。`,
    "提示",
    { type: "warning" }
  )
    .then(async () => {
      try {
        await deleteSubject(row.id);
        ElMessage.success("删除成功");
        await loadList();
        if (
          currentSubjectId.value === row.subject_id &&
          currentSubjectType.value === row.subject_type
        ) {
          closeAttributeDialog();
        }
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

// ---------- 新增主体 ----------
const createDialogVisible = ref(false);
const createFormRef = ref<FormInstance>();
const createLoading = ref(false);
const userOptions = ref<Array<{ id: number; username: string }>>([]);
const userLoading = ref(false);
const initAttrsList = ref([{ key: "", value: "" }]);
const createForm = reactive({
  user_id: undefined as number | undefined,
  subject_type: "USER",
});
const createRules: FormRules = {
  user_id: [{ required: true, message: "请选择用户", trigger: "change" }],
  subject_type: [
    { required: true, message: "请输入主体类型", trigger: "blur" },
  ],
};

const searchUsers = async (query: string) => {
  if (!query) {
    userOptions.value = [];
    return;
  }
  userLoading.value = true;
  try {
    const res = await listUsers({
      page: 1,
      page_size: 20,
      username: query,
      real_name: "",
      status: undefined,
    });
    userOptions.value = (res.list || []).map((u) => ({
      id: u.id,
      username: u.username,
    }));
  } catch (error) {
    console.error("搜索用户失败", error);
  } finally {
    userLoading.value = false;
  }
};

const openCreateDialog = () => {
  createForm.user_id = undefined;
  createForm.subject_type = routeSubjectType.value || "USER";
  initAttrsList.value = [{ key: "", value: "" }];
  userOptions.value = [];
  createDialogVisible.value = true;
};

const resetCreateForm = () => {
  createFormRef.value?.resetFields();
  initAttrsList.value = [{ key: "", value: "" }];
  userOptions.value = [];
};

const addAttrRow = () => initAttrsList.value.push({ key: "", value: "" });
const removeAttrRow = (index: number) => initAttrsList.value.splice(index, 1);

const handleCreateSubject = async () => {
  if (!createFormRef.value) return;
  await createFormRef.value.validate(async (valid) => {
    if (!valid) return;
    createLoading.value = true;
    try {
      const initAttrs: Record<string, any> = {};
      for (const item of initAttrsList.value) {
        if (item.key.trim()) {
          let val: any = item.value.trim();
          if (val === "") continue;
          try {
            val = JSON.parse(val);
          } catch {}
          initAttrs[item.key] = val;
        }
      }
      await setAttributes(
        createForm.subject_type,
        createForm.user_id!,
        initAttrs
      );
      ElMessage.success("主体创建成功");
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
    主体类型: record.subject_type,
    主体ID: record.subject_id,
    用户名: userNameMap.value.get(record.subject_id) || "",
    属性预览: formatPreview(record.attr_key_value),
    创建时间: record.create_time,
  }));
  const ws = XLSX.utils.json_to_sheet(exportData);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, "主体属性列表");
  XLSX.writeFile(
    wb,
    `主体属性列表_${new Date()
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
          const subjectType = row["主体类型"] || row["subject_type"];
          const subjectId = row["主体ID"] || row["subject_id"];
          if (!subjectType || !subjectId) {
            failCount++;
            continue;
          }
          let attrs: Record<string, any> = {};
          const preview = row["属性预览"] || row["attr_key_value"];
          if (preview) {
            const val = String(preview).trim();
            // 1. 尝试 JSON 解析
            try {
              attrs = JSON.parse(val);
            } catch {
              // 2. 解析 key:value 格式（支持中英文逗号、分号、换行分隔）
              attrs = {};
              // 先按常见分隔符拆分（英文逗号、中文逗号、分号、换行）
              const pairs = val
                .split(/[,;，；\n]/)
                .map((p) => p.trim())
                .filter(Boolean);
              for (const pair of pairs) {
                // 找到第一个冒号（支持中文冒号？）
                let colonIndex = pair.indexOf(":");
                if (colonIndex === -1) colonIndex = pair.indexOf("：");
                if (colonIndex === -1) continue;
                const k = pair.substring(0, colonIndex).trim();
                let v = pair.substring(colonIndex + 1).trim();
                if (k === "") continue;
                // 如果值是纯数字，转为数字类型
                if (/^-?\d+(\.\d+)?$/.test(v)) {
                  v = String(v);
                }
                attrs[k] = v;
              }
            }
          }
          await setAttributes(subjectType, Number(subjectId), attrs);
          successCount++;
        } catch (error) {
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

// 辅助函数
const formatValue = (val: any): string => {
  if (typeof val === "object") return JSON.stringify(val, null, 2);
  return String(val);
};

const formatPreview = (attrs: Record<string, any>): string => {
  const entries = Object.entries(attrs).slice(0, 3);
  const preview = entries
    .map(([k, v]) => `${k}:${typeof v === "object" ? "{}" : v}`)
    .join(", ");
  return Object.keys(attrs).length > 3 ? `${preview}...` : preview;
};

onMounted(() => {
  loadList();
});
</script>

<style scoped>
.locked-type {
  display: flex;
  align-items: center;
  gap: 8px;
}
.lock-tip {
  font-size: 12px;
  color: #909399;
}
.subject-attribute-manage {
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