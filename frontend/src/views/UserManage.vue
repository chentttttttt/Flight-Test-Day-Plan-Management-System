<template>
  <div class="user-manage">
    <el-card class="user-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><User /></el-icon>
            <span class="header-title">用户管理</span>
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
              @click="handleCreate"
              class="create-btn"
            >
              <el-icon><Plus /></el-icon><span>新增用户</span>
            </el-button>
          </div>
        </div>
      </template>

      <!-- 搜索区域 -->
      <div class="search-area">
        <el-form :model="queryParams" inline>
          <el-form-item label="用户名">
            <el-input
              v-model="queryParams.username"
              placeholder="请输入用户名"
              clearable
            />
          </el-form-item>
          <el-form-item label="真实姓名">
            <el-input
              v-model="queryParams.real_name"
              placeholder="请输入真实姓名"
              clearable
            />
          </el-form-item>
          <el-form-item label="角色">
            <el-input
              v-model="queryParams.role"
              placeholder="请输入角色名称"
              clearable
            />
          </el-form-item>
          <el-form-item label="状态">
            <el-select
              v-model="queryParams.status"
              placeholder="请选择状态"
              clearable
              style="width: 120px"
            >
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
        class="user-table"
      >
        <el-table-column prop="id" label="ID" width="80" align="center" />
        <el-table-column
          prop="username"
          label="用户名"
          min-width="120"
          show-overflow-tooltip
        />
        <el-table-column
          prop="real_name"
          label="真实姓名"
          min-width="120"
          show-overflow-tooltip
        />
        <el-table-column prop="role" label="角色" width="120" align="center" />
        <el-table-column
          prop="phone"
          label="电话"
          min-width="120"
          show-overflow-tooltip
        />
        <el-table-column
          prop="email"
          label="邮箱"
          min-width="150"
          show-overflow-tooltip
        />
        <el-table-column prop="status" label="状态" width="100" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.status === 1 ? 'success' : 'danger'"
              size="small"
              effect="plain"
            >
              {{ row.status === 1 ? "启用" : "禁用" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          prop="create_time"
          label="创建时间"
          min-width="160"
          align="center"
        />
        <el-table-column label="操作" width="240" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top" v-if="canUpdate">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click="handleEdit(row.id)"
              />
            </el-tooltip>
            <el-tooltip content="详情" placement="top" v-if="canView">
              <el-button
                size="small"
                circle
                :icon="View"
                @click="handleDetail(row.id)"
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

    <!-- 新增/编辑弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="550px"
      class="user-dialog"
      :close-on-click-modal="false"
      @close="handleDialogClose"
    >
      <el-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-width="100px"
        label-position="right"
        class="user-form"
      >
        <el-form-item label="用户名" prop="username" v-if="isCreate">
          <el-input
            v-model="formData.username"
            placeholder="请输入用户名"
            clearable
          />
        </el-form-item>
        <el-form-item label="密码" prop="password" v-if="isCreate">
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="请输入密码"
            show-password
            clearable
          />
        </el-form-item>
        <el-form-item label="真实姓名" prop="real_name">
          <el-input
            v-model="formData.real_name"
            placeholder="请输入真实姓名"
            clearable
          />
        </el-form-item>
        <el-form-item label="角色" prop="role">
          <el-input
            v-model="formData.role"
            placeholder="请输入角色名称"
            clearable
          />
        </el-form-item>
        <el-form-item label="电话" prop="phone">
          <el-input
            v-model="formData.phone"
            placeholder="请输入电话号码"
            clearable
          />
        </el-form-item>
        <el-form-item label="邮箱" prop="email">
          <el-input
            v-model="formData.email"
            placeholder="请输入邮箱地址"
            clearable
          />
        </el-form-item>
        <el-form-item label="状态" prop="status">
          <el-radio-group v-model="formData.status">
            <el-radio :label="1">启用</el-radio>
            <el-radio :label="0">禁用</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="新密码" prop="newPassword" v-if="!isCreate">
          <el-input
            v-model="formData.newPassword"
            type="password"
            placeholder="留空则不修改密码"
            show-password
            clearable
          />
        </el-form-item>
        <div v-if="!isCreate" class="password-tip">
          <el-alert
            title="提示：管理员可直接设置新密码，无需填写旧密码"
            type="info"
            :closable="false"
            show-icon
          />
        </div>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="submitLoading"
            @click="handleSubmit"
          >
            <el-icon><Check /></el-icon><span>确定</span>
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 详情弹窗 -->
    <el-dialog
      v-model="detailVisible"
      title="用户详情"
      width="500px"
      class="user-dialog"
    >
      <el-descriptions :column="1" border>
        <el-descriptions-item label="ID">{{
          detailData.id
        }}</el-descriptions-item>
        <el-descriptions-item label="用户名">{{
          detailData.username
        }}</el-descriptions-item>
        <el-descriptions-item label="真实姓名">{{
          detailData.real_name || "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="角色">{{
          detailData.role || "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="电话">{{
          detailData.phone || "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="邮箱">{{
          detailData.email || "-"
        }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="detailData.status === 1 ? 'success' : 'danger'">
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
  View,
  Delete,
  Check,
  User,
  Download,
  Upload,
} from "@element-plus/icons-vue";
import * as XLSX from "xlsx";
import { useUserStore } from "@/stores/user";
import { usePagePermission } from "@/directives/usePermission";
import {
  listUsers,
  createUser,
  updateUser,
  deleteUser,
  getUser,
} from "@/api/user";
import type {
  UserInfo,
  QueryUserParams,
  CreateUserParams,
  UpdateUserParams,
} from "@/types/user";

const route = useRoute();
const routeRole = (route.params.role as string) || "";
const isRoleLocked = computed(() => !!routeRole);
const effectiveRole = computed(() =>
  isRoleLocked.value ? routeRole : queryParams.role
);
const userStore = useUserStore();
const { canCreate, canUpdate, canDelete, canView, canExport, canImport } =
  usePagePermission();

const loading = ref(false);
const submitLoading = ref(false);
const tableData = ref<UserInfo[]>([]);
const total = ref(0);

// 查询参数（从路由参数预填角色）
const queryParams = reactive<QueryUserParams>({
  page: 1,
  page_size: 10,
  username: "",
  real_name: "",
  role: (route.params.role as string) || "",
  status: undefined,
});

// 表单数据
const dialogVisible = ref(false);
const dialogTitle = ref("");
const isCreate = ref(true);
const formRef = ref<FormInstance>();
const formData = reactive({
  id: undefined as number | undefined,
  username: "",
  password: "",
  real_name: "",
  role: "",
  phone: "",
  email: "",
  status: 1,
  newPassword: "",
});

const formRules: FormRules = {
  username: [
    { required: true, message: "请输入用户名", trigger: "blur" },
    { min: 2, max: 20, message: "长度在 2 到 20 个字符", trigger: "blur" },
  ],
  password: [
    { required: true, message: "请输入密码", trigger: "blur" },
    { min: 3, max: 20, message: "长度在 3 到 20 个字符", trigger: "blur" },
  ],
  real_name: [
    { max: 50, message: "真实姓名不能超过50个字符", trigger: "blur" },
  ],
  phone: [
    {
      pattern: /^1[3-9]\d{9}$/,
      message: "请输入正确的手机号",
      trigger: "blur",
    },
  ],
  email: [{ type: "email", message: "请输入正确的邮箱地址", trigger: "blur" }],
};

const detailVisible = ref(false);
const detailData = ref<UserInfo>({} as UserInfo);

// 获取用户列表
const fetchList = async () => {
  loading.value = true;
  try {
    const params = { ...queryParams, role: effectiveRole.value };
    const res = await listUsers(params);
    tableData.value = res.list || [];
    total.value = res.total || 0;
  } catch (error) {
    console.error("获取列表失败", error);
  } finally {
    loading.value = false;
  }
};

// 搜索与重置
const handleSearch = () => {
  queryParams.page = 1;
  fetchList();
};
const resetSearch = () => {
  queryParams.username = "";
  queryParams.real_name = "";
  queryParams.role = "";
  queryParams.status = undefined;
  queryParams.page = 1;
  fetchList();
};

// 分页处理
const handleSizeChange = (size: number) => {
  queryParams.page_size = size;
  queryParams.page = 1;
  fetchList();
};
const handleCurrentChange = (page: number) => {
  queryParams.page = page;
  fetchList();
};

// 新增用户
const handleCreate = () => {
  if (!canCreate.value) return;
  isCreate.value = true;
  dialogTitle.value = "新增用户";
  formData.id = undefined;
  formData.username = "";
  formData.password = "";
  formData.real_name = "";
  formData.role = "";
  formData.phone = "";
  formData.email = "";
  formData.status = 1;
  formData.newPassword = "";
  dialogVisible.value = true;
  setTimeout(() => formRef.value?.clearValidate(), 0);
};

// 编辑用户
const handleEdit = async (id: number) => {
  if (!canUpdate.value) return;
  try {
    const user = await getUser(id);
    isCreate.value = false;
    dialogTitle.value = "编辑用户";
    formData.id = user.id;
    formData.username = user.username;
    formData.real_name = user.real_name || "";
    formData.role = user.role || "";
    formData.phone = user.phone || "";
    formData.email = user.email || "";
    formData.status = user.status ?? 1;
    formData.password = "";
    formData.newPassword = "";
    dialogVisible.value = true;
    setTimeout(() => formRef.value?.clearValidate(), 0);
  } catch (error) {
    ElMessage.error("获取用户信息失败");
  }
};

// 详情
const handleDetail = async (id: number) => {
  if (!canView.value) return;
  try {
    const user = await getUser(id);
    detailData.value = user;
    detailVisible.value = true;
  } catch (error) {
    ElMessage.error("获取详情失败");
  }
};

// 删除用户
const handleDelete = (row: UserInfo) => {
  if (!canDelete.value) return;
  ElMessageBox.confirm(`确认删除用户“${row.username}”吗？`, "提示", {
    confirmButtonText: "确定",
    cancelButtonText: "取消",
    type: "warning",
    customClass: "delete-confirm",
  })
    .then(async () => {
      try {
        const updateBy = userStore.currentUserId;
        if (!updateBy) {
          ElMessage.error("无法获取当前操作人信息");
          return;
        }
        await deleteUser(row.id, updateBy);
        ElMessage.success("删除成功");
        fetchList();
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

// 提交表单（新增/编辑）
const handleSubmit = async () => {
  if (!formRef.value) return;
  await formRef.value.validate(async (valid) => {
    if (!valid) return;

    submitLoading.value = true;
    try {
      if (isCreate.value) {
        const params: CreateUserParams = {
          username: formData.username,
          password: formData.password,
          real_name: formData.real_name,
          phone: formData.phone,
          email: formData.email,
          status: formData.status,
          role: formData.role,
        };
        await createUser(params);
        ElMessage.success("新增成功");
      } else {
        const params: UpdateUserParams = {
          id: formData.id!,
          real_name: formData.real_name,
          phone: formData.phone,
          email: formData.email,
          status: formData.status,
          role: formData.role,
        };
        if (formData.newPassword && formData.newPassword.trim()) {
          params.password = formData.newPassword;
        }
        await updateUser(params);
        ElMessage.success("更新成功");
      }
      dialogVisible.value = false;
      fetchList();
    } catch (error) {
      // 错误已全局处理
    } finally {
      submitLoading.value = false;
    }
  });
};

// 弹窗关闭时重置表单
const handleDialogClose = () => {
  formRef.value?.resetFields();
  formData.password = "";
  formData.newPassword = "";
};

// ========== 导入导出功能 ==========
// 导出当前表格数据为 Excel（纯前端）
const handleExport = () => {
  if (!canExport.value) return;
  // 准备导出数据（过滤敏感字段，仅保留展示字段）
  const exportData = tableData.value.map((user) => ({
    ID: user.id,
    用户名: user.username,
    真实姓名: user.real_name || "",
    角色: user.role || "",
    电话: user.phone || "",
    邮箱: user.email || "",
    状态: user.status === 1 ? "启用" : "禁用",
    创建时间: user.create_time || "",
  }));
  const ws = XLSX.utils.json_to_sheet(exportData);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, "用户列表");
  XLSX.writeFile(
    wb,
    `用户列表_${new Date().toISOString().slice(0, 19).replace(/:/g, "-")}.xlsx`
  );
  ElMessage.success("导出成功");
};

// 导入 Excel：解析文件，逐条调用 createUser
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
      const rows = XLSX.utils.sheet_to_json(worksheet);
      if (!rows.length) {
        ElMessage.warning("文件无数据");
        return;
      }

      // 将解析结果转换为 CreateUserParams 格式
      const users: CreateUserParams[] = rows.map((row: any) => ({
        username: row["用户名"] || row["username"],
        password: row["密码"] || row["password"] || "123456", // 默认密码
        real_name: row["真实姓名"] || row["real_name"] || "",
        phone: row["电话"] || row["phone"] || "",
        email: row["邮箱"] || row["email"] || "",
        role: row["角色"] || row["role"] || "",
        status: row["状态"] === "启用" ? 1 : 0,
      }));

      // 逐条调用 createUser 接口
      let successCount = 0;
      let failCount = 0;
      for (let i = 0; i < users.length; i++) {
        try {
          await createUser(users[i]);
          successCount++;
        } catch (error) {
          failCount++;
          console.error(`导入第 ${i + 1} 条失败:`, error);
        }
      }
      ElMessage.success(
        `导入完成：成功 ${successCount} 条，失败 ${failCount} 条`
      );
      if (successCount > 0) {
        fetchList(); // 刷新列表
      }
    };
    reader.readAsBinaryString(file);
  };
  input.click();
};

onMounted(() => {
  fetchList();
});
</script>

<style scoped>
/* 与之前样式相同，此处略去 */
.user-manage {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.user-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  transition: all 0.3s ease;
  border: none;
  overflow: hidden;
}
.user-card :deep(.el-card__header) {
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
.create-btn span {
  margin-left: 5px;
}
.search-area {
  margin-bottom: 20px;
  padding: 0 8px;
}
.user-table {
  border-radius: 16px;
  overflow: hidden;
}
.user-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.user-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.user-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.user-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
.user-table :deep(.el-table__row .el-button) {
  margin: 0 4px;
}
.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  padding: 0 8px;
}
.user-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.user-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  margin: 0;
  padding: 20px 24px;
}
.user-dialog :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #409eff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.user-dialog :deep(.el-dialog__body) {
  padding: 24px;
}
.user-dialog :deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}
.user-form .el-form-item {
  margin-bottom: 20px;
}
.password-tip {
  margin-top: -10px;
  margin-bottom: 20px;
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