<template>
  <div class="role-list">
    <el-card class="role-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Avatar /></el-icon>
            <span class="header-title">角色管理</span>
          </div>
          <el-button
            type="primary"
            @click="openCreateDialog"
            class="create-btn"
          >
            <el-icon><Plus /></el-icon><span>新增角色</span>
          </el-button>
        </div>
      </template>

      <el-table
        :data="tableData"
        v-loading="loading"
        border
        stripe
        class="role-table"
        :header-cell-style="{
          background: 'rgba(64, 158, 255, 0.05)',
          color: '#2c3e50',
        }"
      >
        <el-table-column prop="id" label="ID" width="80" align="center" />
        <el-table-column
          prop="role"
          label="角色名称"
          min-width="150"
          show-overflow-tooltip
        />
        <el-table-column label="已授权菜单" min-width="200">
          <template #default="{ row }">
            <el-tag
              v-for="menuId in row.menu_ids"
              :key="menuId"
              size="small"
              effect="plain"
              class="menu-tag"
            >
              {{ getMenuNameById(menuId) }}
            </el-tag>
            <span v-if="!row.menu_ids?.length" class="empty-text">无</span>
          </template>
        </el-table-column>
        <el-table-column prop="create_time" label="创建时间" width="160" />
        <el-table-column prop="update_time" label="更新时间" width="160" />
        <el-table-column label="操作" width="200" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top">
              <el-button
                type="primary"
                size="small"
                circle
                :icon="Edit"
                @click="openEditDialog(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top">
              <el-button
                type="danger"
                size="small"
                circle
                :icon="Delete"
                @click="handleDelete(row)"
              />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 新增/编辑角色对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="600px"
      class="role-dialog"
      :close-on-click-modal="false"
      @close="resetForm"
    >
      <el-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-width="100px"
        label-position="right"
        class="role-form"
      >
        <el-form-item label="角色名称" prop="role">
          <el-input
            v-model="formData.role"
            placeholder="请输入角色名称（唯一）"
            clearable
          />
        </el-form-item>
        <el-form-item label="菜单权限" prop="menu_ids">
          <el-tree
            ref="treeRef"
            :data="menuTree"
            show-checkbox
            node-key="id"
            :props="{ label: 'name', children: 'children' }"
            :check-strictly="false"
            @check="handleTreeCheck"
            class="menu-tree"
          />
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
            <el-icon><Check /></el-icon><span>确定</span>
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, nextTick } from "vue";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import { Plus, Edit, Delete, Check, Avatar } from "@element-plus/icons-vue";
import { getRoleList, createRole, updateRole, deleteRole } from "@/api/role";
import { getMenuTree } from "@/api/menu";
import type { Menu } from "@/types/menu";

// ---------- 数据 ----------
const tableData = ref<any[]>([]);
const loading = ref(false);
const menuTree = ref<Menu[]>([]);
const menuNameMap = ref<Map<number, string>>(new Map());
const treeRef = ref();
const dialogVisible = ref(false);
const dialogTitle = ref("");
const isEdit = ref(false);
const currentId = ref<number | undefined>();
const formRef = ref<FormInstance>();
const submitLoading = ref(false);

const formData = reactive({
  role: "",
  menu_ids: [] as number[],
});

const rules: FormRules = {
  role: [{ required: true, message: "请输入角色名称", trigger: "blur" }],
};

// ---------- 加载角色列表 ----------
const loadRoleList = async () => {
  loading.value = true;
  try {
    const list = await getRoleList();
    tableData.value = list || [];
  } catch (error) {
    ElMessage.error("加载角色列表失败");
  } finally {
    loading.value = false;
  }
};

// ---------- 加载菜单树 ----------
const loadMenuTree = async () => {
  try {
    const tree = await getMenuTree();
    menuTree.value = tree;
    // 构建 ID -> 名称映射
    const map = new Map<number, string>();
    const walk = (nodes: Menu[]) => {
      nodes.forEach((node) => {
        map.set(node.id, node.name);
        if (node.children) walk(node.children as Menu[]);
      });
    };
    walk(tree);
    menuNameMap.value = map;
  } catch (error) {
    ElMessage.error("加载菜单树失败");
  }
};

// 根据菜单ID获取名称
const getMenuNameById = (id: number): string => {
  return menuNameMap.value.get(id) || `菜单${id}`;
};

// 树节点勾选事件（只同步数据，父子联动由 el-tree 自动处理）
const handleTreeCheck = (checkedNodes: any, { checkedKeys }: any) => {
  formData.menu_ids = checkedKeys;
};

// 打开新增对话框
const openCreateDialog = () => {
  isEdit.value = false;
  dialogTitle.value = "新增角色";
  formData.role = "";
  formData.menu_ids = [];
  dialogVisible.value = true;
  nextTick(() => {
    treeRef.value?.setCheckedKeys([]);
  });
};

// 打开编辑对话框
const openEditDialog = (row: any) => {
  isEdit.value = true;
  dialogTitle.value = "编辑角色";
  currentId.value = row.id;
  formData.role = row.role;
  formData.menu_ids = [...(row.menu_ids || [])];
  dialogVisible.value = true;
  nextTick(() => {
    treeRef.value?.setCheckedKeys(formData.menu_ids);
  });
};

// 提交表单
const handleSubmit = async () => {
  if (!formRef.value) return;
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    submitLoading.value = true;
    try {
      if (isEdit.value) {
        await updateRole(currentId.value!, {
          role: formData.role,
          menu_ids: formData.menu_ids,
        });
      } else {
        await createRole({
          role: formData.role,
          menu_ids: formData.menu_ids,
        });
      }
      ElMessage.success("操作成功");
      dialogVisible.value = false;
      loadRoleList();
    } catch (error) {
      ElMessage.error("操作失败");
    } finally {
      submitLoading.value = false;
    }
  });
};

// 删除角色
const handleDelete = (row: any) => {
  ElMessageBox.confirm("确认删除该角色吗？", "提示", {
    confirmButtonText: "确定",
    cancelButtonText: "取消",
    type: "warning",
    customClass: "delete-confirm",
  })
    .then(async () => {
      try {
        await deleteRole(row.id);
        ElMessage.success("删除成功");
        loadRoleList();
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

// 重置表单
const resetForm = () => {
  formRef.value?.resetFields();
  formData.menu_ids = [];
};

onMounted(() => {
  loadRoleList();
  loadMenuTree();
});
</script>

<style scoped>
.role-list {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.role-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  transition: all 0.3s ease;
  border: none;
  overflow: hidden;
}
.role-card :deep(.el-card__header) {
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
  color: #409eff;
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
.role-table {
  border-radius: 16px;
  overflow: hidden;
  margin-top: 8px;
}
.role-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.role-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.role-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.role-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
.menu-tag {
  margin-right: 5px;
  margin-bottom: 5px;
  border-radius: 20px;
  background: rgba(64, 158, 255, 0.1);
  border: none;
  color: #409eff;
}
.empty-text {
  color: #909399;
  font-size: 12px;
}
.role-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.role-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  margin: 0;
  padding: 20px 24px;
}
.role-dialog :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #409eff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.role-dialog :deep(.el-dialog__body) {
  padding: 24px;
}
.role-dialog :deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}
.role-form .el-form-item {
  margin-bottom: 20px;
}
.menu-tree {
  border: 1px solid #e4e7ed;
  border-radius: 12px;
  padding: 12px;
  background: #fafbfc;
  max-height: 400px;
  overflow: auto;
}
.menu-tree :deep(.el-tree-node__content) {
  height: 34px;
  transition: background 0.2s;
}
.menu-tree :deep(.el-tree-node__content:hover) {
  background-color: rgba(64, 158, 255, 0.08);
}
.menu-tree :deep(.el-checkbox) {
  margin-right: 8px;
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