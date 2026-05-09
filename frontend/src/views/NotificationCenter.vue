<!-- NotificationCenter.vue -->
<template>
  <div class="notification-center">
    <el-card class="notification-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Bell /></el-icon>
            <span class="header-title">消息中心</span>
            <el-badge
              :value="notificationStore.unreadCount"
              :hidden="notificationStore.unreadCount === 0"
              class="unread-badge"
            >
              <el-tag size="small" type="info">未读消息</el-tag>
            </el-badge>
          </div>
          <div class="header-right">
            <el-button v-if="canCreate" type="primary" @click="openSendDialog">
              <el-icon><Plus /></el-icon> 发送通知
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
          <el-form-item label="类型">
            <el-select
              v-model="queryParams.type"
              placeholder="全部"
              clearable
              style="width: 120px"
            >
              <el-option label="系统消息" value="SYSTEM" />
              <el-option label="审批消息" value="APPROVAL" />
              <el-option label="任务消息" value="TASK" />
            </el-select>
          </el-form-item>
          <el-form-item label="状态">
            <el-select
              v-model="queryParams.is_read"
              placeholder="全部"
              clearable
              style="width: 120px"
            >
              <el-option label="未读" :value="0" />
              <el-option label="已读" :value="1" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="handleSearch">查询</el-button>
            <el-button @click="resetFilter">重置</el-button>
          </el-form-item>
        </el-form>
        <div class="batch-actions" v-if="selectedRows.length > 0">
          <el-button
            v-if="canUpdate"
            type="success"
            size="small"
            @click="batchMarkRead"
            >批量标记已读</el-button
          >
          <el-button
            v-if="canDelete"
            type="danger"
            size="small"
            @click="batchDelete"
            >批量删除</el-button
          >
        </div>
      </div>

      <!-- 通知列表 -->
      <el-table
        :data="tableData"
        v-loading="loading"
        border
        stripe
        @selection-change="handleSelectionChange"
        class="notification-table"
      >
        <el-table-column type="selection" width="55" />
        <el-table-column
          prop="title"
          label="标题"
          min-width="200"
          show-overflow-tooltip
        />
        <el-table-column
          prop="content"
          label="内容"
          min-width="250"
          show-overflow-tooltip
        />
        <el-table-column prop="type" label="类型" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getTypeTag(row.type)" size="small">{{
              getTypeName(row.type)
            }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="is_read" label="状态" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.is_read === 1 ? 'info' : 'warning'" size="small">
              {{ row.is_read === 1 ? "已读" : "未读" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          prop="create_time"
          label="时间"
          width="180"
          align="center"
        />
        <el-table-column label="操作" width="180" fixed="right" align="center">
          <template #default="{ row }">
            <el-tooltip
              content="标记已读"
              placement="top"
              v-if="canUpdate && row.is_read === 0"
            >
              <el-button
                size="small"
                circle
                :icon="Select"
                @click="markSingleRead(row.id)"
              />
            </el-tooltip>
            <el-tooltip content="查看详情" placement="top">
              <el-button
                size="small"
                circle
                :icon="View"
                @click="viewDetail(row)"
              />
            </el-tooltip>
            <el-tooltip content="删除" placement="top" v-if="canDelete">
              <el-button
                size="small"
                circle
                type="danger"
                :icon="Delete"
                @click="handleDelete(row.id)"
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

    <!-- 发送通知弹窗 -->
    <el-dialog
      v-model="sendDialogVisible"
      title="发送通知"
      width="550px"
      class="notification-dialog"
      :close-on-click-modal="false"
      @close="resetSendForm"
    >
      <el-form
        ref="sendFormRef"
        :model="sendForm"
        :rules="sendRules"
        label-width="100px"
      >
        <el-form-item label="发送目标" prop="targetType">
          <el-radio-group v-model="sendForm.targetType">
            <el-radio label="role">按角色广播</el-radio>
            <el-radio label="user">指定用户</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item
          label="角色"
          prop="role"
          v-if="sendForm.targetType === 'role'"
        >
          <el-input v-model="sendForm.role" placeholder="例如：admin" />
        </el-form-item>
        <el-form-item
          label="用户ID"
          prop="user_id"
          v-if="sendForm.targetType === 'user'"
        >
          <el-input-number
            v-model="sendForm.user_id"
            :min="1"
            controls-position="right"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="通知类型" prop="type">
          <el-select v-model="sendForm.type" style="width: 100%">
            <el-option label="系统消息" value="SYSTEM" />
            <el-option label="审批消息" value="APPROVAL" />
            <el-option label="任务消息" value="TASK" />
          </el-select>
        </el-form-item>
        <el-form-item label="标题" prop="title">
          <el-input v-model="sendForm.title" placeholder="通知标题" />
        </el-form-item>
        <el-form-item label="内容" prop="content">
          <el-input
            v-model="sendForm.content"
            type="textarea"
            rows="4"
            placeholder="通知内容"
          />
        </el-form-item>
        <el-form-item label="附件链接" prop="attachment_url">
          <el-input v-model="sendForm.attachment_url" placeholder="可选" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sendDialogVisible = false">取消</el-button>
          <el-button type="primary" :loading="sendLoading" @click="handleSend"
            >发送</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 通知详情弹窗 -->
    <el-dialog
      v-model="detailVisible"
      title="通知详情"
      width="500px"
      class="notification-dialog"
    >
      <el-descriptions :column="1" border>
        <el-descriptions-item label="标题">{{
          currentDetail.title
        }}</el-descriptions-item>
        <el-descriptions-item label="内容">{{
          currentDetail.content
        }}</el-descriptions-item>
        <el-descriptions-item label="类型">{{
          getTypeName(currentDetail.type)
        }}</el-descriptions-item>
        <el-descriptions-item label="时间">{{
          currentDetail.create_time
        }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="currentDetail.is_read === 1 ? 'info' : 'warning'">
            {{ currentDetail.is_read === 1 ? "已读" : "未读" }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="附件" v-if="currentDetail.attachment_url">
          <el-link :href="currentDetail.attachment_url" target="_blank"
            >点击查看</el-link
          >
        </el-descriptions-item>
      </el-descriptions>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import {
  ElMessage,
  ElMessageBox,
  type FormInstance,
  type FormRules,
} from "element-plus";
import {
  Bell,
  Plus,
  Refresh,
  Select,
  View,
  Delete,
} from "@element-plus/icons-vue";
import { usePagePermission } from "@/directives/usePermission";
import { useNotificationStore } from "@/stores/notification";
import {
  getNotificationList,
  createNotification,
  markAsRead,
  deleteNotification,
} from "@/api/notification";
import type {
  Notification,
  CreateNotificationParams,
  QueryNotificationParams,
} from "@/types/notification";

const { canCreate, canUpdate, canDelete } = usePagePermission();
const notificationStore = useNotificationStore();

const loading = ref(false);
const tableData = ref<Notification[]>([]);
const total = ref(0);
const selectedRows = ref<Notification[]>([]);

const queryParams = reactive<QueryNotificationParams>({
  page: 1,
  page_size: 10,
  type: "",
  is_read: undefined,
});

// 发送通知相关
const sendDialogVisible = ref(false);
const sendFormRef = ref<FormInstance>();
const sendLoading = ref(false);
const sendForm = reactive({
  targetType: "role" as "role" | "user",
  role: "",
  user_id: undefined as number | undefined,
  type: "SYSTEM",
  title: "",
  content: "",
  attachment_url: "",
});
const sendRules: FormRules = {
  role: [{ required: true, message: "请输入角色", trigger: "blur" }],
  user_id: [
    {
      required: true,
      message: "请输入用户ID",
      trigger: "blur",
      type: "number",
    },
  ],
  type: [{ required: true, message: "请选择类型", trigger: "change" }],
  title: [{ required: true, message: "请输入标题", trigger: "blur" }],
  content: [{ required: true, message: "请输入内容", trigger: "blur" }],
};

// 详情弹窗
const detailVisible = ref(false);
const currentDetail = ref<Notification>({} as Notification);

// 辅助函数
const getTypeName = (type: string) => {
  const map: Record<string, string> = {
    SYSTEM: "系统消息",
    APPROVAL: "审批消息",
    TASK: "任务消息",
  };
  return map[type] || type;
};
const getTypeTag = (type: string) => {
  const map: Record<string, string> = {
    SYSTEM: "info",
    APPROVAL: "warning",
    TASK: "primary",
  };
  return map[type] || "";
};

// 加载通知列表
const fetchList = async () => {
  loading.value = true;
  try {
    const res = await getNotificationList({
      page: queryParams.page,
      page_size: queryParams.page_size,
      type: queryParams.type || undefined,
      is_read: queryParams.is_read,
    });
    tableData.value = res.list;
    total.value = res.total;
  } catch (error) {
    ElMessage.error("加载通知列表失败");
  } finally {
    loading.value = false;
  }
};

// 搜索重置
const handleSearch = () => {
  queryParams.page = 1;
  fetchList();
};
const resetFilter = () => {
  queryParams.type = "";
  queryParams.is_read = undefined;
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

// 表格多选
const handleSelectionChange = (rows: Notification[]) => {
  selectedRows.value = rows;
};

// 标记单个已读
const markSingleRead = async (id: number) => {
  try {
    await markAsRead([id]);
    ElMessage.success("已标记为已读");
    fetchList();
    notificationStore.fetchUnreadCount(); // 更新全局未读数量
  } catch (error) {
    ElMessage.error("操作失败");
  }
};

// 批量标记已读
const batchMarkRead = async () => {
  const ids = selectedRows.value.map((row) => row.id);
  if (ids.length === 0) return;
  try {
    await markAsRead(ids);
    ElMessage.success("批量标记成功");
    fetchList();
    notificationStore.fetchUnreadCount();
    selectedRows.value = [];
  } catch (error) {
    ElMessage.error("操作失败");
  }
};

// 删除单个
const handleDelete = (id: number) => {
  ElMessageBox.confirm("确认删除该通知吗？", "提示", { type: "warning" })
    .then(async () => {
      await deleteNotification(id);
      ElMessage.success("删除成功");
      fetchList();
      notificationStore.fetchUnreadCount();
    })
    .catch(() => {});
};

// 批量删除
const batchDelete = () => {
  const ids = selectedRows.value.map((row) => row.id);
  if (ids.length === 0) return;
  ElMessageBox.confirm(`确认删除选中的 ${ids.length} 条通知吗？`, "提示", {
    type: "warning",
  })
    .then(async () => {
      for (const id of ids) {
        await deleteNotification(id);
      }
      ElMessage.success("批量删除成功");
      fetchList();
      notificationStore.fetchUnreadCount();
      selectedRows.value = [];
    })
    .catch(() => {});
};

// 查看详情（自动标记已读）
const viewDetail = (row: Notification) => {
  currentDetail.value = row;
  detailVisible.value = true;
  if (row.is_read === 0) {
    markSingleRead(row.id);
  }
};

// 发送通知
const openSendDialog = () => {
  sendForm.targetType = "role";
  sendForm.role = "";
  sendForm.user_id = undefined;
  sendForm.type = "SYSTEM";
  sendForm.title = "";
  sendForm.content = "";
  sendForm.attachment_url = "";
  sendDialogVisible.value = true;
};
const resetSendForm = () => {
  sendFormRef.value?.resetFields();
};
const handleSend = async () => {
  if (!sendFormRef.value) return;
  await sendFormRef.value.validate(async (valid) => {
    if (!valid) return;
    const params: CreateNotificationParams = {
      title: sendForm.title,
      content: sendForm.content,
      type: sendForm.type,
      attachment_url: sendForm.attachment_url || undefined,
    };
    if (sendForm.targetType === "role") {
      params.role = sendForm.role;
    } else {
      params.user_id = sendForm.user_id;
    }
    sendLoading.value = true;
    try {
      await createNotification(params);
      ElMessage.success("发送成功");
      sendDialogVisible.value = false;
      fetchList();
      notificationStore.fetchUnreadCount();
    } catch (error) {
      ElMessage.error("发送失败");
    } finally {
      sendLoading.value = false;
    }
  });
};

onMounted(() => {
  fetchList();
  notificationStore.fetchUnreadCount();
});
</script>

<style scoped>
.notification-center {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.notification-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  border: none;
  overflow: hidden;
}
.notification-card :deep(.el-card__header) {
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
.unread-badge {
  margin-left: 12px;
}
.filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
  padding: 0 8px;
}
.batch-actions {
  display: flex;
  gap: 8px;
}
.notification-table {
  border-radius: 16px;
  overflow: hidden;
}
.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}
.notification-dialog :deep(.el-dialog) {
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