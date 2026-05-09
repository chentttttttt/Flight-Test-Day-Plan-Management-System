<template>
  <div class="menu-manage">
    <el-card class="menu-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span class="header-title">菜单管理</span>
          </div>
          <el-button
            v-if="canCreate"
            type="primary"
            @click="handleCreateRoot"
            class="create-btn"
          >
            <el-icon><Plus /></el-icon>
            <span>新建目录</span>
          </el-button>
        </div>
      </template>

      <el-table
        ref="tableRef"
        :data="menuTree"
        row-key="id"
        :tree-props="{ children: 'children', hasChildren: 'hasChildren' }"
        border
        stripe
        :row-class-name="getRowClassName"
        class="menu-table"
        :header-cell-style="headerStyle"
        @row-click="handleRowClick"
      >
        <el-table-column label="序号" width="70" align="center">
          <template #default="{ $index }">
            <span class="node-index">{{ $index + 1 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="菜单名称" min-width="180" show-overflow-tooltip>
          <template #default="{ row }">
            <span
              class="node-name"
              :style="{ paddingLeft: (row.depth || 0) * 24 + 'px' }"
            >
              {{ row.name }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="类型" width="100" align="center">
          <template #default="{ row }">
            <el-tag
              :type="getTypeTagType(row.menu_type)"
              size="small"
              effect="plain"
            >
              <el-icon v-if="row.menu_type === 0"><FolderOpened /></el-icon>
              <el-icon v-else-if="row.menu_type === 1"><Document /></el-icon>
              <el-icon v-else><Cpu /></el-icon>
              {{ getTypeName(row.menu_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="路由路径" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="node-path">{{ row.path || "-" }}</span>
          </template>
        </el-table-column>
        <el-table-column label="权限标识" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="node-query">{{ row.permission || "-" }}</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip content="编辑" placement="top" v-if="canUpdate">
              <el-button
                size="small"
                circle
                :icon="Edit"
                @click.stop="handleEdit(row)"
              />
            </el-tooltip>
            <el-tooltip
              content="添加子菜单"
              placement="top"
              v-if="canCreate && row.menu_type !== 2"
            >
              <el-button
                size="small"
                circle
                :icon="Plus"
                @click.stop="handleAddChild(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top" v-if="canDelete">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click.stop="handleDelete(row)"
              />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 菜单表单弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="550px"
      class="menu-dialog"
      :close-on-click-modal="false"
      @close="resetForm"
    >
      <el-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-width="100px"
        label-position="right"
        class="menu-form"
      >
        <el-form-item label="父级菜单" prop="parent_id">
          <el-select
            v-model="formData.parent_id"
            placeholder="请选择父级菜单"
            clearable
            filterable
            class="full-width"
          >
            <el-option label="无（根菜单）" :value="undefined" />
            <el-option
              v-for="item in parentMenuOptions"
              :key="item.id"
              :label="item.name"
              :value="item.id"
              :disabled="item.disabled"
            />
          </el-select>
        </el-form-item>
        <el-form-item
          :label="formData.menu_type === 2 ? '按钮名称' : '菜单名称'"
          prop="name"
        >
          <el-input
            v-model="formData.name"
            :placeholder="
              formData.menu_type === 2 ? '请输入按钮名称' : '请输入菜单名称'
            "
            clearable
          />
        </el-form-item>
        <el-form-item label="类型" prop="menu_type">
          <el-radio-group v-model="formData.menu_type">
            <el-radio :label="0">目录</el-radio>
            <el-radio :label="1">菜单</el-radio>
            <el-radio :label="2">按钮</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item
          label="路由路径"
          prop="path"
          v-if="formData.menu_type !== 2"
        >
          <el-input
            v-model="formData.path"
            placeholder="例如：/user"
            clearable
          />
        </el-form-item>
        <el-form-item
          label="路由参数"
          prop="query_params"
          v-if="formData.menu_type !== 2"
        >
          <el-input
            v-model="formData.query_params"
            placeholder="例如：id=1&type=2"
            clearable
          />
        </el-form-item>
        <el-form-item
          label="权限标识"
          prop="permission"
          v-if="formData.menu_type === 2"
        >
          <el-input
            v-model="formData.permission"
            placeholder="例如：user:add"
            clearable
          />
        </el-form-item>
        <el-form-item label="排序号" prop="order_num">
          <el-input-number
            v-model="formData.order_num"
            :min="0"
            controls-position="right"
          />
        </el-form-item>
        <el-form-item label="备注" prop="remark">
          <el-input
            v-model="formData.remark"
            type="textarea"
            rows="2"
            placeholder="可选"
            resize="none"
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
            <el-icon><Check /></el-icon>
            <span>确定</span>
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, nextTick } from "vue";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import {
  Menu as MenuIcon,
  Plus,
  Edit,
  Delete,
  Check,
  FolderOpened,
  Document,
  Cpu,
} from "@element-plus/icons-vue";
import { getMenuTree, createMenu, updateMenu, deleteMenu } from "@/api/menu";
import type { Menu, CreateMenuParams, UpdateMenuParams } from "@/types/menu";
import { MenuType } from "@/types/menu";
import { usePagePermission } from "@/directives/usePermission";

// 权限控制
const { canCreate, canUpdate, canDelete } = usePagePermission();

const tableRef = ref();
const menuTree = ref<Menu[]>([]);
const dialogVisible = ref(false);
const dialogTitle = ref("");
const isEdit = ref(false);
const submitLoading = ref(false);
const formRef = ref<FormInstance>();

const formData = reactive({
  id: undefined as number | undefined,
  name: "",
  menu_type: 1,
  parent_id: undefined as number | undefined,
  path: "",
  query_params: "",
  order_num: 0,
  permission: "",
  remark: "",
});

const rules: FormRules = {
  name: [{ required: true, message: "请输入菜单名称", trigger: "blur" }],
  menu_type: [{ required: true, message: "请选择类型", trigger: "change" }],
  path: [{ required: true, message: "请输入路由路径", trigger: "blur" }],
  permission: [{ required: true, message: "请输入权限标识", trigger: "blur" }],
};

// 当前选中行
const currentRowId = ref<number | null>(null);

// 节点映射
const nodeMap = new Map<number, Menu>();
const parentMap = new Map<number, number>();

// 构建映射表
const buildMaps = (nodes: Menu[], parentId: number | null = null) => {
  for (const node of nodes) {
    nodeMap.set(node.id, node);
    if (parentId !== null) parentMap.set(node.id, parentId);
    if (node.children?.length) buildMaps(node.children, node.id);
  }
};

// 判断是否为后代节点
const isDescendant = (nodeId: number, ancestorId: number | null): boolean => {
  if (!ancestorId) return false;
  let current = nodeId;
  while (parentMap.has(current)) {
    const parent = parentMap.get(current)!;
    if (parent === ancestorId) return true;
    current = parent;
  }
  return false;
};

// 表头样式
const headerStyle = {
  background:
    "linear-gradient(90deg, rgba(64, 158, 255, 0.06), rgba(64, 158, 255, 0.03))",
  color: "#1f2937",
  fontWeight: 600,
  fontSize: "13px",
};

// 高亮当前行 + 所有子菜单
const getRowClassName = ({ row }: { row: Menu }) => {
  if (
    currentRowId.value === row.id ||
    isDescendant(row.id, currentRowId.value)
  ) {
    return "menu-row-active";
  }
  return "";
};

// 父菜单选项
const parentMenuOptions = computed(() => {
  if (!isEdit.value || !formData.id) {
    return getAllMenuOptions(menuTree.value, null);
  } else {
    return getAllMenuOptions(menuTree.value, formData.id);
  }
});

function getAllMenuOptions(menus: Menu[], excludeId: number | null): any[] {
  const options: any[] = [];
  const traverse = (node: Menu, level: number = 0) => {
    if (node.menu_type === MenuType.BUTTON) return;
    if (excludeId !== null && node.id === excludeId) return;
    options.push({
      id: node.id,
      name: "　".repeat(level) + node.name,
      disabled: false,
    });
    if (node.children?.length)
      node.children.forEach((child) => traverse(child, level + 1));
  };
  menus.forEach((menu) => traverse(menu));
  return options;
}

const getTypeName = (type: number): string => {
  switch (type) {
    case MenuType.DIR:
      return "目录";
    case MenuType.MENU:
      return "菜单";
    case MenuType.BUTTON:
      return "按钮";
    default:
      return "未知";
  }
};

const getTypeTagType = (type: number): string => {
  switch (type) {
    case MenuType.DIR:
      return "warning";
    case MenuType.MENU:
      return "primary";
    case MenuType.BUTTON:
      return "info";
    default:
      return "";
  }
};

// 递归添加深度
const addDepth = (nodes: Menu[], depth: number = 0): Menu[] => {
  return nodes.map((node) => ({
    ...node,
    depth,
    children: node.children ? addDepth(node.children, depth + 1) : [],
  }));
};

// 加载菜单树
const loadMenuTree = async () => {
  try {
    const rawTree = await getMenuTree();
    menuTree.value = addDepth(rawTree);
    nodeMap.clear();
    parentMap.clear();
    buildMaps(menuTree.value);
    nextTick(() => {
      if (tableRef.value) {
        menuTree.value.forEach((row) => {
          if (row.children?.length)
            tableRef.value.toggleRowExpansion(row, true);
        });
      }
    });
  } catch (error) {
    console.error("加载菜单树失败", error);
    ElMessage.error("加载菜单树失败");
  }
};

// 行点击：切换展开 + 切换高亮
const handleRowClick = (row: Menu) => {
  if (row.children?.length) tableRef.value.toggleRowExpansion(row);
  currentRowId.value = currentRowId.value === row.id ? null : row.id;
};

const handleCreateRoot = () => {
  isEdit.value = false;
  dialogTitle.value = "新建目录";
  resetForm();
  formData.parent_id = undefined;
  dialogVisible.value = true;
};

const handleAddChild = (menu: Menu) => {
  isEdit.value = false;
  dialogTitle.value = `为【${menu.name}】添加子菜单`;
  resetForm();
  formData.parent_id = menu.id;

  if (menu.menu_type === MenuType.DIR) {
    formData.menu_type = MenuType.MENU;
    formData.permission = "";
  } else if (menu.menu_type === MenuType.MENU) {
    formData.menu_type = MenuType.BUTTON;
    formData.path = "";
    formData.query_params = "";
  }
  dialogVisible.value = true;
};

const handleEdit = (menu: Menu) => {
  isEdit.value = true;
  dialogTitle.value = `编辑菜单【${menu.name}】`;
  Object.assign(formData, menu);
  dialogVisible.value = true;
};

const handleDelete = (menu: Menu) => {
  ElMessageBox.confirm(
    `确认删除菜单【${menu.name}】吗？删除后其子菜单也将被删除。`,
    "提示",
    {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      type: "warning",
      customClass: "delete-confirm",
    }
  )
    .then(async () => {
      try {
        await deleteMenu(menu.id);
        ElMessage.success("删除成功");
        loadMenuTree();
      } catch (error: any) {
        const msg =
          error?.response?.data?.message || error?.message || "删除失败";
        ElMessage.error(msg);
      }
    })
    .catch(() => {});
};

const handleSubmit = async () => {
  if (!formData.name) {
    ElMessage.warning("请填写菜单名称");
    return;
  }
  submitLoading.value = true;
  try {
    if (isEdit.value) {
      await updateMenu(formData.id!, formData as UpdateMenuParams);
      ElMessage.success("更新成功");
    } else {
      await createMenu(formData as CreateMenuParams);
      ElMessage.success("创建成功");
    }
    dialogVisible.value = false;
    loadMenuTree();
  } catch (error: any) {
    const msg = error?.response?.data?.message || error?.message || "操作失败";
    ElMessage.error(msg);
  } finally {
    submitLoading.value = false;
  }
};

const resetForm = () => {
  formData.id = undefined;
  formData.name = "";
  formData.menu_type = 0;
  formData.parent_id = undefined;
  formData.path = "";
  formData.query_params = "";
  formData.order_num = 0;
  formData.permission = "";
  formData.remark = "";
  formRef.value?.clearValidate();
};

onMounted(() => {
  loadMenuTree();
});
</script>

<style scoped>
.menu-manage {
  height: 100%;
  background: #f8fafc;
  padding: 24px;
  box-sizing: border-box;
}

.menu-card {
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.06);
  transition: all 0.3s ease;
  border: none;
  overflow: hidden;
}

.menu-card:hover {
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.08);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 24px;
  color: #409eff;
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
  border-radius: 8px;
  padding: 8px 16px;
  transition: all 0.2s ease;
}

.create-btn:hover {
  transform: translateY(-1px);
}

/* 表格核心样式 */
.menu-table {
  border-radius: 12px;
  overflow: hidden;
  margin-top: 8px;
  --el-table-row-hover-bg-color: rgba(64, 158, 255, 0.08);
}

/* 行基础样式 */
:deep(.el-table__row) {
  transition: all 0.2s ease !important;
  cursor: pointer;
}

/* 鼠标悬停高亮 */
:deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.12) !important;
  transform: scale(1.002);
}

/* 点击选中 + 子菜单高亮 */
:deep(.el-table__row.menu-row-active) {
  background-color: rgba(64, 158, 255, 0.18) !important;
  font-weight: 500;
}

/* 高亮行左侧装饰条 */
:deep(.el-table__row.menu-row-active td:first-child) {
  border-left: 3px solid #409eff;
}

/* 表格内容排版 */
.node-name {
  font-weight: 500;
  color: #2d3748;
  display: inline-block;
}

.node-path,
.node-query {
  color: #64748b;
  font-family: monospace;
  font-size: 13px;
}

/* 按钮间距 */
:deep(.el-button[circle]) {
  margin: 0 3px;
}

/* 弹窗优化 */
.menu-dialog :deep(.el-dialog) {
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.12);
}

.menu-dialog :deep(.el-dialog__header) {
  padding: 18px 24px;
  border-bottom: 1px solid #f1f5f9;
}

.menu-dialog :deep(.el-dialog__title) {
  font-size: 17px;
  font-weight: 600;
  color: #1f2937;
}

.menu-form .el-form-item {
  margin-bottom: 18px;
}

.full-width {
  width: 100%;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.dialog-footer .el-button {
  border-radius: 8px;
  padding: 8px 18px;
}

/* 删除确认框圆角 */
:deep(.delete-confirm) {
  border-radius: 16px;
}
</style>