<template>
  <div class="flow-manage">
    <el-card class="flow-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Stamp /></el-icon>
            <span class="header-title">审批流程配置</span>
          </div>
          <el-button type="primary" @click="openCreateFlowDialog">
            <el-icon><Plus /></el-icon> 新建流程
          </el-button>
        </div>
      </template>

      <!-- 树组件：点击行展开/收起 -->
      <el-tree
        ref="treeRef"
        :data="treeData"
        node-key="id"
        :props="{ label: 'name', children: 'children' }"
        :expand-on-click-node="true"
        highlight-current
        class="flow-tree"
      >
        <template #default="{ data }">
          <div
            class="tree-node"
            :class="{
              'is-flow': data.type === 'flow',
              'is-node': data.type === 'node',
            }"
          >
            <span class="node-name">
              <el-icon v-if="data.type === 'flow'"><Folder /></el-icon>
              <el-icon v-else><Document /></el-icon>
              {{ data.name }}
            </span>
            <span class="node-info" v-if="data.type === 'node'">
              <el-tag size="small" type="info">{{
                getApproverTypeText(data.approver_type)
              }}</el-tag>
              <el-tag
                size="small"
                type="warning"
                v-if="data.approve_strategy"
                >{{ getStrategyText(data.approve_strategy) }}</el-tag
              >
              <el-tag
                size="small"
                :type="data.status === 1 ? 'success' : 'danger'"
              >
                {{ data.status === 1 ? "启用" : "禁用" }}
              </el-tag>
            </span>
            <span class="node-info" v-else-if="data.type === 'flow'">
              <el-tag
                size="small"
                :type="data.status === 1 ? 'success' : 'danger'"
              >
                {{ data.status === 1 ? "启用" : "禁用" }}
              </el-tag>
            </span>
            <span class="node-actions">
              <el-tooltip
                :content="data.type === 'flow' ? '编辑流程' : '编辑节点'"
                placement="top"
              >
                <el-button
                  size="small"
                  circle
                  :icon="Edit"
                  @click.stop="handleEdit(data)"
                />
              </el-tooltip>
              <el-tooltip
                v-if="data.type === 'flow'"
                content="添加节点"
                placement="top"
              >
                <el-button
                  size="small"
                  circle
                  :icon="Plus"
                  @click.stop="openCreateNodeDialog(data.id)"
                />
              </el-tooltip>
              <el-tooltip content="删除" placement="top">
                <el-button
                  size="small"
                  circle
                  type="danger"
                  :icon="Delete"
                  @click.stop="handleDelete(data)"
                />
              </el-tooltip>
            </span>
          </div>
        </template>
      </el-tree>
    </el-card>

    <!-- 流程表单弹窗 -->
    <el-dialog
      v-model="flowDialogVisible"
      :title="flowDialogTitle"
      width="500px"
      class="flow-dialog"
      :close-on-click-modal="false"
      @close="resetFlowForm"
    >
      <el-form
        ref="flowFormRef"
        :model="flowForm"
        :rules="flowRules"
        label-width="100px"
      >
        <el-form-item label="流程编码" prop="code" v-if="!isFlowEdit">
          <el-input v-model="flowForm.code" placeholder="唯一编码" />
        </el-form-item>
        <el-form-item label="流程名称" prop="name">
          <el-input v-model="flowForm.name" placeholder="流程名称" />
        </el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input
            v-model="flowForm.description"
            type="textarea"
            rows="3"
            placeholder="可选"
            resize="none"
          />
        </el-form-item>
        <el-form-item label="状态" prop="status">
          <el-radio-group v-model="flowForm.status">
            <el-radio :label="1">启用</el-radio>
            <el-radio :label="0">禁用</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="flowDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="flowSubmitLoading"
            @click="handleFlowSubmit"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 节点表单弹窗（支持多选用户） -->
    <el-dialog
      v-model="nodeDialogVisible"
      :title="nodeDialogTitle"
      width="500px"
      class="node-dialog"
      :close-on-click-modal="false"
      @close="resetNodeForm"
    >
      <el-form
        ref="nodeFormRef"
        :model="nodeForm"
        :rules="nodeRules"
        label-width="120px"
      >
        <el-form-item label="节点顺序" prop="node_order">
          <el-input-number
            v-model="nodeForm.node_order"
            :min="1"
            controls-position="right"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="节点名称" prop="node_name">
          <el-input v-model="nodeForm.node_name" placeholder="节点名称" />
        </el-form-item>
        <el-form-item label="审批人类型" prop="approver_type">
          <el-select
            v-model="nodeForm.approver_type"
            placeholder="请选择"
            style="width: 100%"
          >
            <el-option label="角色" value="ROLE" />
            <el-option label="用户" value="USER" />
            <el-option label="部门负责人" value="DEPARTMENT_HEAD" />
          </el-select>
        </el-form-item>
        <el-form-item
          label="审批人值"
          prop="approver_value"
          v-if="nodeForm.approver_type"
        >
          <!-- 用户类型：多选下拉框 -->
          <el-select
            v-if="nodeForm.approver_type === 'USER'"
            v-model="approverUserList"
            multiple
            filterable
            remote
            :remote-method="searchUsers"
            :loading="userLoading"
            placeholder="请输入用户名搜索，可多选"
            clearable
            style="width: 100%"
          >
            <el-option
              v-for="user in userOptions"
              :key="user.username"
              :label="user.username"
              :value="user.username"
            />
          </el-select>
          <!-- 其他类型：普通输入框 -->
          <el-input
            v-else
            v-model="nodeForm.approver_value"
            placeholder="例如：admin 或 角色编码"
          />
        </el-form-item>
        <el-form-item label="审批策略" prop="approve_strategy">
          <el-select
            v-model="nodeForm.approve_strategy"
            placeholder="请选择"
            clearable
            style="width: 100%"
          >
            <el-option label="会签（所有审批人）" value="ALL" />
            <el-option label="或签（任一审批人）" value="ANY" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态" prop="status">
          <el-radio-group v-model="nodeForm.status">
            <el-radio :label="1">启用</el-radio>
            <el-radio :label="0">禁用</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="nodeDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="nodeSubmitLoading"
            @click="handleNodeSubmit"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from "vue";
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
  Folder,
  Document,
  Stamp,
} from "@element-plus/icons-vue";
import {
  getTree,
  createFlow,
  updateFlow,
  deleteFlow,
  createNode,
  updateNode,
  deleteNode,
} from "@/api/approvalManage";
import { listUsers } from "@/api/user";
import type { TreeNode } from "@/types/approval";

// 树数据
const treeData = ref<TreeNode[]>([]);
const treeRef = ref();

// 流程表单
const flowDialogVisible = ref(false);
const flowDialogTitle = ref("");
const isFlowEdit = ref(false);
const currentFlowId = ref<number>();
const flowFormRef = ref<FormInstance>();
const flowSubmitLoading = ref(false);
const flowForm = reactive({
  code: "",
  name: "",
  description: "",
  status: 1,
});
const flowRules: FormRules = {
  code: [{ required: true, message: "请输入流程编码", trigger: "blur" }],
  name: [{ required: true, message: "请输入流程名称", trigger: "blur" }],
};

// 节点表单
const nodeDialogVisible = ref(false);
const nodeDialogTitle = ref("");
const isNodeEdit = ref(false);
const currentNodeId = ref<number>();
const nodeFormRef = ref<FormInstance>();
const nodeSubmitLoading = ref(false);
const nodeForm = reactive({
  flow_id: 0,
  node_order: 1,
  node_name: "",
  approver_type: "",
  approver_value: "", // 字符串形式，用于后端存储（多选时用逗号分隔）
  approve_strategy: "",
  status: 1,
});
// 多选用途的数组
const approverUserList = ref<string[]>([]);
const nodeRules: FormRules = {
  node_order: [{ required: true, message: "请输入节点顺序", trigger: "blur" }],
  node_name: [{ required: true, message: "请输入节点名称", trigger: "blur" }],
  approver_type: [
    { required: true, message: "请选择审批人类型", trigger: "change" },
  ],
};

// 用户远程搜索
const userOptions = ref<Array<{ username: string }>>([]);
const userLoading = ref(false);

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
    userOptions.value = (res.list || []).map((u) => ({ username: u.username }));
  } catch (error) {
    console.error("搜索用户失败", error);
  } finally {
    userLoading.value = false;
  }
};

// 监听审批人类型变化，清空相关值
watch(
  () => nodeForm.approver_type,
  (newType) => {
    if (newType === "USER") {
      // 清空原有字符串值，从 approverUserList 同步
      nodeForm.approver_value = approverUserList.value.join(",");
    } else {
      approverUserList.value = [];
      // 其他类型保留输入框的值，不清空
    }
  }
);

// 监听多选数组变化，同步到 nodeForm.approver_value
watch(approverUserList, (newList) => {
  if (nodeForm.approver_type === "USER") {
    nodeForm.approver_value = newList.join(",");
  }
});

// 辅助函数
const getApproverTypeText = (type: string) => {
  const map: Record<string, string> = {
    ROLE: "角色",
    USER: "用户",
    DEPARTMENT_HEAD: "部门负责人",
  };
  return map[type] || type;
};

const getStrategyText = (strategy: string) => {
  const map: Record<string, string> = {
    ALL: "会签",
    ANY: "或签",
  };
  return map[strategy] || strategy;
};

// 加载数据
const loadData = async () => {
  try {
    treeData.value = await getTree();
  } catch (error) {
    ElMessage.error("加载数据失败");
  }
};

// 流程相关
const openCreateFlowDialog = () => {
  isFlowEdit.value = false;
  flowDialogTitle.value = "新建流程";
  flowForm.code = "";
  flowForm.name = "";
  flowForm.description = "";
  flowForm.status = 1;
  flowDialogVisible.value = true;
  setTimeout(() => flowFormRef.value?.clearValidate(), 0);
};

const openEditFlowDialog = (flow: TreeNode) => {
  isFlowEdit.value = true;
  flowDialogTitle.value = "编辑流程";
  currentFlowId.value = flow.id;
  flowForm.code = flow.code || "";
  flowForm.name = flow.name;
  flowForm.description = flow.description || "";
  flowForm.status = flow.status ?? 1;
  flowDialogVisible.value = true;
  setTimeout(() => flowFormRef.value?.clearValidate(), 0);
};

const handleFlowSubmit = async () => {
  if (!flowFormRef.value) return;
  await flowFormRef.value.validate(async (valid) => {
    if (!valid) return;
    flowSubmitLoading.value = true;
    try {
      if (isFlowEdit.value) {
        await updateFlow(currentFlowId.value!, {
          name: flowForm.name,
          description: flowForm.description,
          status: flowForm.status,
        });
        ElMessage.success("更新成功");
      } else {
        await createFlow({
          code: flowForm.code,
          name: flowForm.name,
          description: flowForm.description,
          status: flowForm.status,
        });
        ElMessage.success("创建成功");
      }
      flowDialogVisible.value = false;
      await loadData();
    } catch (error) {
      ElMessage.error(isFlowEdit.value ? "更新失败" : "创建失败");
    } finally {
      flowSubmitLoading.value = false;
    }
  });
};

// 节点相关
const openCreateNodeDialog = (flowId: number) => {
  isNodeEdit.value = false;
  nodeDialogTitle.value = "添加节点";
  nodeForm.flow_id = flowId;
  nodeForm.node_order = 1;
  nodeForm.node_name = "";
  nodeForm.approver_type = "";
  nodeForm.approver_value = "";
  nodeForm.approve_strategy = "";
  nodeForm.status = 1;
  approverUserList.value = [];
  nodeDialogVisible.value = true;
  setTimeout(() => nodeFormRef.value?.clearValidate(), 0);
};

const openEditNodeDialog = (node: TreeNode) => {
  isNodeEdit.value = true;
  nodeDialogTitle.value = "编辑节点";
  currentNodeId.value = node.id;
  nodeForm.flow_id = node.flow_id || 0;
  nodeForm.node_order = node.node_order || 1;
  nodeForm.node_name = node.node_name || "";
  nodeForm.approver_type = node.approver_type || "";
  nodeForm.approver_value = node.approver_value || "";
  nodeForm.approve_strategy = node.approve_strategy || "";
  nodeForm.status = node.status ?? 1;
  // 如果审批人类型是用户，将逗号分隔的字符串转为数组
  if (nodeForm.approver_type === "USER" && nodeForm.approver_value) {
    approverUserList.value = nodeForm.approver_value
      .split(",")
      .filter((v) => v.trim());
  } else {
    approverUserList.value = [];
  }
  nodeDialogVisible.value = true;
  setTimeout(() => nodeFormRef.value?.clearValidate(), 0);
};

const handleNodeSubmit = async () => {
  if (!nodeFormRef.value) return;
  await nodeFormRef.value.validate(async (valid) => {
    if (!valid) return;
    nodeSubmitLoading.value = true;
    try {
      const data = {
        flow_id: nodeForm.flow_id,
        node_order: nodeForm.node_order,
        node_name: nodeForm.node_name,
        approver_type: nodeForm.approver_type,
        approver_value: nodeForm.approver_value || undefined,
        approve_strategy: nodeForm.approve_strategy || undefined,
        status: nodeForm.status,
      };
      if (isNodeEdit.value) {
        await updateNode(currentNodeId.value!, {
          node_name: data.node_name,
          approver_type: data.approver_type,
          approver_value: data.approver_value,
          approve_strategy: data.approve_strategy,
          status: data.status,
        });
        ElMessage.success("更新成功");
      } else {
        await createNode(data);
        ElMessage.success("创建成功");
      }
      nodeDialogVisible.value = false;
      await loadData();
    } catch (error) {
      ElMessage.error(isNodeEdit.value ? "更新失败" : "创建失败");
    } finally {
      nodeSubmitLoading.value = false;
    }
  });
};

// 统一删除处理
const handleDelete = (data: TreeNode) => {
  const isFlow = !data.node_order;
  const name = data.name;
  ElMessageBox.confirm(
    `确认删除${isFlow ? "流程" : "节点"}“${name}”吗？${
      isFlow ? "删除流程将同时删除其下所有节点。" : ""
    }`,
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
        if (isFlow) {
          await deleteFlow(data.id);
        } else {
          await deleteNode(data.id);
        }
        ElMessage.success("删除成功");
        await loadData();
      } catch (error) {
        ElMessage.error("删除失败");
      }
    })
    .catch(() => {});
};

const handleEdit = (data: TreeNode) => {
  if (!data.node_order) {
    openEditFlowDialog(data);
  } else {
    openEditNodeDialog(data);
  }
};

// 重置表单
const resetFlowForm = () => {
  flowFormRef.value?.resetFields();
};

const resetNodeForm = () => {
  nodeFormRef.value?.resetFields();
  userOptions.value = [];
  approverUserList.value = [];
};

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.flow-manage {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.flow-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  border: none;
  overflow: hidden;
}
.flow-card :deep(.el-card__header) {
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
.tree-title {
  margin-bottom: 12px;
  padding: 0 8px;
}
.tree-title-text {
  font-size: 14px;
  font-weight: 500;
  color: #2c3e50;
}
.tree-title .el-divider {
  margin: 8px 0 0 0;
}
.flow-tree {
  margin-top: 8px;
  background: transparent;
}
.flow-tree :deep(.el-tree-node__content) {
  padding-left: 0 !important;
  height: auto;
}
.tree-node {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 100%;
  padding: 8px 12px;
  transition: all 0.2s;
  border-radius: 12px;
}
.tree-node:hover {
  background-color: rgba(64, 158, 255, 0.08);
}
.node-name {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 200px;
  font-weight: 500;
}
.node-info {
  display: flex;
  gap: 8px;
  flex: 1;
}
.node-actions {
  display: flex;
  gap: 8px;
  visibility: hidden;
}
.tree-node:hover .node-actions {
  visibility: visible;
}
.flow-dialog :deep(.el-dialog),
.node-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.flow-dialog :deep(.el-dialog__header),
.node-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  margin: 0;
  padding: 20px 24px;
}
.flow-dialog :deep(.el-dialog__title),
.node-dialog :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  background: linear-gradient(135deg, #2c3e50, #409eff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.flow-dialog :deep(.el-dialog__body),
.node-dialog :deep(.el-dialog__body) {
  padding: 24px;
}
.flow-dialog :deep(.el-dialog__footer),
.node-dialog :deep(.el-dialog__footer) {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
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