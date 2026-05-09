<template>
  <div class="approval-manage">
    <el-card class="approval-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><Stamp /></el-icon>
            <span class="header-title">审批管理</span>
          </div>
        </div>
      </template>

      <el-tabs
        v-model="activeTab"
        class="approval-tabs"
        @tab-click="handleTabClick"
      >
        <!-- 提交申请 -->
        <el-tab-pane label="提交申请" name="submit">
          <div class="submit-form">
            <el-form
              ref="submitFormRef"
              :model="submitForm"
              :rules="submitRules"
              label-width="120px"
              class="approval-form"
            >
              <el-form-item label="审批流程" prop="flow_code">
                <el-select
                  v-model="submitForm.flow_code"
                  placeholder="请选择审批流程"
                  clearable
                  filterable
                  :loading="flowLoading"
                  @change="onFlowChange"
                >
                  <el-option
                    v-for="flow in flowList"
                    :key="flow.code"
                    :label="flow.name"
                    :value="flow.code"
                  />
                </el-select>
                <el-button
                  v-if="flowList.length === 0 && !flowLoading"
                  type="primary"
                  link
                  @click="loadFlows"
                  style="margin-left: 10px"
                  >重试</el-button
                >
              </el-form-item>

              <!-- <el-form-item label="业务ID" prop="biz_id">
                <el-input-number
                  v-model="submitForm.biz_id"
                  :min="1"
                  placeholder="例如：请假单ID"
                  style="width: 100%"
                />
              </el-form-item> -->
              <input type="hidden" v-model="submitForm.biz_id" />

              <el-form-item label="业务类型" prop="biz_type">
                <el-input
                  v-model="submitForm.biz_type"
                  placeholder="例如：请假申请"
                />
              </el-form-item>

              <!-- 日计划审批：从 MinIO 选择附件 -->
              <el-form-item
                v-if="isDailyPlan"
                label="选择计划文件"
                prop="attachment_url"
              >
                <el-select
                  v-model="submitForm.attachment_url"
                  placeholder="请选择已导出的计划文件"
                  clearable
                  filterable
                  :loading="planFileLoading"
                >
                  <el-option
                    v-for="file in planFileList"
                    :key="file.key"
                    :label="file.label"
                    :value="file.key"
                  />
                </el-select>
              </el-form-item>

              <!-- 其他流程：本地上传 + 链接 -->
              <template v-else>
                <el-form-item label="上传附件">
                  <el-upload
                    class="attachment-upload"
                    drag
                    :action="uploadUrl"
                    :headers="uploadHeaders"
                    :on-success="handleUploadSuccess"
                    :on-error="handleUploadError"
                    :before-upload="beforeUpload"
                    :file-list="fileList"
                    :limit="1"
                    :on-remove="handleFileRemove"
                  >
                    <el-icon class="el-icon--upload"><UploadFilled /></el-icon>
                    <div class="el-upload__text">
                      将文件拖到此处，或<em>点击上传</em>
                    </div>
                    <template #tip>
                      <div class="el-upload__tip">
                        支持 jpg/png/pdf/word 等格式，单个文件不超过 10MB
                      </div>
                    </template>
                  </el-upload>
                </el-form-item>

                <el-form-item label="附件链接" prop="attachment_url">
                  <el-input
                    v-model="submitForm.attachment_url"
                    placeholder="也可手动输入文件URL"
                    clearable
                  />
                </el-form-item>
              </template>

              <el-form-item label="备注" prop="remark">
                <el-input
                  v-model="submitForm.remark"
                  type="textarea"
                  rows="3"
                  placeholder="申请说明（可选）"
                  resize="none"
                />
              </el-form-item>

              <el-form-item>
                <el-button
                  type="primary"
                  :loading="submitLoading"
                  @click="handleSubmit"
                  >提交申请</el-button
                >
                <el-button @click="resetSubmitForm">重置</el-button>
              </el-form-item>
            </el-form>
          </div>
        </el-tab-pane>

        <!-- 待办审批 -->
        <el-tab-pane label="待办审批" name="todo">
          <el-table
            :data="todoList"
            v-loading="todoLoading"
            border
            stripe
            class="approval-table"
          >
            <el-table-column prop="id" label="ID" width="80" />
            <el-table-column prop="biz_type" label="业务类型" min-width="120" />
            <!-- <el-table-column prop="biz_id" label="业务ID" width="100" /> -->
            <el-table-column prop="applicant_name" label="申请人" width="120" />
            <el-table-column label="当前审批人" width="150">
              <template #default="{ row }">
                <el-tag size="small" type="info">{{
                  getApproverNames(row.current_approver_ids)
                }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="submitted_time"
              label="申请时间"
              width="160"
            />
            <el-table-column
              prop="remark"
              label="备注"
              min-width="150"
              show-overflow-tooltip
            />

            <el-table-column label="附件" width="100" align="center">
              <template #default="{ row }">
                <el-button
                  v-if="row.attachment_url"
                  type="primary"
                  link
                  @click="viewAttachment(row.attachment_url)"
                >
                  查看
                </el-button>
                <span v-else class="no-attachment">无</span>
              </template>
            </el-table-column>

            <el-table-column
              label="操作"
              width="200"
              fixed="right"
              align="center"
            >
              <template #default="{ row }">
                <el-button
                  type="primary"
                  size="small"
                  @click="openApproveDialog(row)"
                  >通过</el-button
                >
                <el-button
                  type="danger"
                  size="small"
                  @click="openRejectDialog(row)"
                  >驳回</el-button
                >
              </template>
            </el-table-column>
            <template #empty>
              <el-empty description="暂无待办审批" :image-size="80" />
            </template>
          </el-table>
        </el-tab-pane>

        <!-- 我的申请 -->
        <el-tab-pane label="我的申请" name="myOrders">
          <div class="my-orders-header">
            <el-radio-group
              v-model="myOrdersStatus"
              size="small"
              @change="loadMyOrders"
            >
              <el-radio-button label="">全部</el-radio-button>
              <el-radio-button label="PENDING">进行中</el-radio-button>
              <el-radio-button label="APPROVED">已通过</el-radio-button>
              <el-radio-button label="REJECTED">已驳回</el-radio-button>
            </el-radio-group>
          </div>
          <el-table
            :data="myOrdersList"
            v-loading="myOrdersLoading"
            border
            stripe
            class="approval-table"
          >
            <el-table-column prop="id" label="ID" width="80" />
            <el-table-column prop="biz_type" label="业务类型" min-width="120" />
            <!-- <el-table-column prop="biz_id" label="业务ID" width="100" /> -->
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag
                  :type="getStatusType(row.status)"
                  size="small"
                  effect="plain"
                  >{{ getStatusText(row.status) }}</el-tag
                >
              </template>
            </el-table-column>
            <el-table-column
              prop="submitted_time"
              label="申请时间"
              width="160"
            />
            <el-table-column
              prop="finished_time"
              label="完成时间"
              width="160"
            />
            <el-table-column
              prop="remark"
              label="备注"
              min-width="150"
              show-overflow-tooltip
            />
            <el-table-column
              label="操作"
              width="120"
              fixed="right"
              align="center"
            >
              <template #default="{ row }">
                <el-button size="small" link @click="showRecords(row.id)"
                  >查看记录</el-button
                >
              </template>
            </el-table-column>
            <template #empty>
              <el-empty description="暂无申请记录" :image-size="80" />
            </template>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 审批操作弹窗 -->
    <el-dialog
      v-model="approveDialogVisible"
      :title="approveDialogTitle"
      width="500px"
      class="approval-dialog"
      :close-on-click-modal="false"
    >
      <el-form ref="approveFormRef" :model="approveForm" label-width="100px">
        <el-form-item label="审批意见">
          <el-input
            v-model="approveForm.opinion"
            type="textarea"
            rows="3"
            placeholder="请输入审批意见（可选）"
          />
        </el-form-item>

        <!-- 非日计划流程支持上传附件 -->
        <template v-if="currentOrder && currentOrder.flow_code !== '1'">
          <el-form-item label="上传附件">
            <el-upload
              class="attachment-upload"
              drag
              :action="uploadUrl"
              :headers="uploadHeaders"
              :on-success="handleApproveUploadSuccess"
              :on-error="handleUploadError"
              :before-upload="beforeUpload"
              :file-list="approveFileList"
              :limit="1"
              :on-remove="handleApproveFileRemove"
            >
              <el-icon class="el-icon--upload"><UploadFilled /></el-icon>
              <div class="el-upload__text">
                将文件拖到此处，或<em>点击上传</em>
              </div>
            </el-upload>
          </el-form-item>

          <el-form-item label="附件链接">
            <el-input
              v-model="approveForm.attachment_url"
              placeholder="可选，文件URL"
              clearable
            />
          </el-form-item>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="approveDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="approveLoading"
            @click="confirmApprove"
            >确定</el-button
          >
        </span>
      </template>
    </el-dialog>

    <!-- 审批记录弹窗 -->
    <el-dialog
      v-model="recordsDialogVisible"
      title="审批记录"
      width="700px"
      class="approval-dialog"
      :close-on-click-modal="false"
    >
      <el-timeline>
        <el-timeline-item
          v-for="record in recordsList"
          :key="record.id"
          :timestamp="record.create_time"
          placement="top"
          :type="record.action === 'APPROVE' ? 'primary' : 'danger'"
          :icon="
            record.action === 'APPROVE' ? SuccessFilled : CircleCloseFilled
          "
        >
          <el-card class="record-card">
            <div><strong>审批人：</strong>{{ record.approver_name }}</div>
            <div>
              <strong>操作：</strong
              >{{ record.action === "APPROVE" ? "通过" : "驳回" }}
            </div>
            <div v-if="record.opinion">
              <strong>意见：</strong>{{ record.opinion }}
            </div>
            <div v-if="record.attachment_url">
              <strong>附件：</strong>
              <el-button
                type="primary"
                link
                @click="viewAttachment(record.attachment_url)"
                >查看</el-button
              >
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
      <div v-if="recordsList.length === 0" class="empty-text">暂无审批记录</div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed, watch } from "vue";
import { useRoute } from "vue-router";
import {
  ElMessage,
  type FormInstance,
  type FormRules,
  type UploadProps,
  type UploadUserFile,
} from "element-plus";
import {
  Stamp,
  SuccessFilled,
  CircleCloseFilled,
  UploadFilled,
} from "@element-plus/icons-vue";
import {
  submitApproval,
  getTodoList,
  getMyOrders,
  getApprovalRecords,
  approve,
  reject,
} from "@/api/approval";
import { getActiveFlows } from "@/api/approvalManage";
import type {
  ApprovalFlow,
  ApprovalOrder,
  ApprovalRecord,
  SubmitApprovalDto,
  ApproveDto,
  RejectDto,
} from "@/types/approval";
import { getFileUrl, listFiles } from "@/api/minio";
import { useTodoStore } from "@/stores/todo";

// ========== 常量 ==========
const DAILY_PLAN_CODE = "1"; // 日计划审批流程编码

// ========== 基础数据 ==========
const route = useRoute();
const todoStore = useTodoStore();
const activeTab = ref(
  (route.query.tab === "todo" ? "todo" : "submit") as string
);

const flowList = ref<ApprovalFlow[]>([]);
const flowLoading = ref(false);

// ========== 日计划文件列表 ==========
const planFileList = ref<{ key: string; label: string }[]>([]);
const planFileLoading = ref(false);
const isDailyPlan = computed(() => submitForm.flow_code === DAILY_PLAN_CODE);

// ========== 提交申请 ==========
const submitFormRef = ref<FormInstance>();
const submitLoading = ref(false);
const submitForm = reactive<SubmitApprovalDto>({
  flow_code: "",
  biz_id: 0,
  biz_type: "",
  remark: "",
  attachment_url: "",
});
const submitRules: FormRules = {
  flow_code: [{ required: true, message: "请选择审批流程", trigger: "change" }],
  biz_id: [{ required: true, message: "请输入业务ID", trigger: "blur" }],
  biz_type: [{ required: true, message: "请输入业务类型", trigger: "blur" }],
  attachment_url: [
    {
      validator: (_rule, value, callback) => {
        if (isDailyPlan.value && !value) {
          callback(new Error("请选择计划文件"));
        } else {
          callback();
        }
      },
      trigger: "change",
    },
  ],
};

// ========== 待办审批 ==========
const todoList = ref<ApprovalOrder[]>([]);
const todoLoading = ref(false);
const currentOrder = ref<ApprovalOrder | null>(null); // 记录当前操作的订单

// ========== 我的申请 ==========
const myOrdersList = ref<ApprovalOrder[]>([]);
const myOrdersLoading = ref(false);
const myOrdersStatus = ref("");

// ========== 审批弹窗 ==========
const approveDialogVisible = ref(false);
const approveDialogTitle = ref("");
const currentAction = ref<"approve" | "reject">("approve");
const currentOrderId = ref(0);
const approveFormRef = ref<FormInstance>();
const approveLoading = ref(false);
const approveForm = reactive({
  opinion: "",
  attachment_url: "",
});

// ========== 审批记录弹窗 ==========
const recordsDialogVisible = ref(false);
const recordsList = ref<ApprovalRecord[]>([]);

// ========== 文件上传相关 ==========
const uploadUrl = "/api/minio/upload";
const uploadHeaders = computed(() => ({
  Authorization: `Bearer ${localStorage.getItem("token")}`,
}));
const fileList = ref<UploadUserFile[]>([]);
const approveFileList = ref<UploadUserFile[]>([]);

const beforeUpload: UploadProps["beforeUpload"] = (file) => {
  const isLt10M = file.size / 1024 / 1024 < 10;
  if (!isLt10M) {
    ElMessage.error("文件大小不能超过 10MB");
    return false;
  }
  return true;
};

const handleUploadSuccess = (response: any) => {
  if (response.code === 200 && response.data?.key) {
    submitForm.attachment_url = response.data.key;
    ElMessage.success("文件上传成功");
  } else {
    ElMessage.error("上传失败：" + (response.message || "未知错误"));
  }
};

const handleApproveUploadSuccess = (response: any) => {
  if (response.code === 200 && response.data?.key) {
    approveForm.attachment_url = response.data.key;
    ElMessage.success("文件上传成功");
  } else {
    ElMessage.error("上传失败：" + (response.message || "未知错误"));
  }
};

const handleUploadError = (error: any) => {
  console.error(error);
  ElMessage.error("文件上传失败，请重试");
};

const handleFileRemove = () => {
  submitForm.attachment_url = "";
  fileList.value = [];
};

const handleApproveFileRemove = () => {
  approveForm.attachment_url = "";
  approveFileList.value = [];
};

const viewAttachment = async (key: string) => {
  if (!key) return;
  try {
    let realKey = key;
    if (key.startsWith("/api/minio/url/")) {
      realKey = key.replace("/api/minio/url/", "");
    }
    const { url } = await getFileUrl(realKey);
    if (url) window.open(url, "_blank");
  } catch (error) {
    console.error(error);
    ElMessage.error("获取附件链接失败");
  }
};

// ========== 辅助函数 ==========
const getApproverNames = (idsStr?: string): string => {
  if (!idsStr) return "无";
  const ids = idsStr.split(",").filter(Boolean);
  return ids.length ? ids.join(", ") : "无";
};

const getStatusType = (status: string) => {
  switch (status) {
    case "PENDING":
      return "warning";
    case "APPROVED":
      return "success";
    case "REJECTED":
      return "danger";
    default:
      return "info";
  }
};

const getStatusText = (status: string) => {
  switch (status) {
    case "PENDING":
      return "进行中";
    case "APPROVED":
      return "已通过";
    case "REJECTED":
      return "已驳回";
    default:
      return status;
  }
};

// ========== 加载流程 ==========
const loadFlows = async () => {
  flowLoading.value = true;
  try {
    flowList.value = await getActiveFlows();
  } catch {
    ElMessage.error("加载流程列表失败");
  } finally {
    flowLoading.value = false;
  }
};

// ========== 日计划文件列表加载 ==========
const loadPlanFiles = async () => {
  planFileLoading.value = true;
  try {
    const { files } = await listFiles("approval");
    // 只显示 *未审批.xlsx 文件
    planFileList.value = files
      .filter((f: any) => f.key.endsWith("未审批.xlsx"))
      .map((f: any) => {
        const parts = f.key.split("/");
        const fileName = parts[parts.length - 1] || f.key;
        return { key: f.key, label: fileName };
      });
  } catch (error) {
    ElMessage.error("加载计划文件列表失败");
    planFileList.value = [];
  } finally {
    planFileLoading.value = false;
  }
};

// 流程变更时处理
const onFlowChange = () => {
  if (isDailyPlan.value) {
    submitForm.biz_type = "日计划审批";
    submitForm.attachment_url = "";
    loadPlanFiles();
  } else {
    submitForm.biz_type = "其他审批";
    submitForm.attachment_url = "";
    fileList.value = [];
  }
};

// ========== 提交申请 ==========
const handleSubmit = async () => {
  if (!submitFormRef.value) return;
  await submitFormRef.value.validate(async (valid) => {
    if (!valid) return;
    submitLoading.value = true;
    try {
      await submitApproval(submitForm);
      ElMessage.success("申请已提交");
      resetSubmitForm();
      if (activeTab.value === "myOrders") await loadMyOrders();
    } catch {
      ElMessage.error("提交失败");
    } finally {
      submitLoading.value = false;
    }
  });
};

const resetSubmitForm = () => {
  submitFormRef.value?.resetFields();
  submitForm.flow_code = "";
  submitForm.biz_id = 1;
  submitForm.biz_type = "其他审批";
  submitForm.remark = "";
  submitForm.attachment_url = "";
  fileList.value = [];
};

// ========== 待办审批 ==========
const loadTodoList = async () => {
  todoLoading.value = true;
  try {
    const list = await getTodoList();
    todoList.value = list;
    todoStore.todoCount = list.length;
  } catch {
    ElMessage.error("加载待办列表失败");
  } finally {
    todoLoading.value = false;
  }
};

const openApproveDialog = (row: ApprovalOrder) => {
  currentAction.value = "approve";
  currentOrderId.value = row.id;
  currentOrder.value = row;
  approveDialogTitle.value = `审批通过 - 单号 ${row.id}`;
  approveForm.opinion = "";
  approveForm.attachment_url = "";
  approveFileList.value = [];
  approveDialogVisible.value = true;
};

const openRejectDialog = (row: ApprovalOrder) => {
  currentAction.value = "reject";
  currentOrderId.value = row.id;
  currentOrder.value = row;
  approveDialogTitle.value = `审批驳回 - 单号 ${row.id}`;
  approveForm.opinion = "";
  approveForm.attachment_url = "";
  approveFileList.value = [];
  approveDialogVisible.value = true;
};

const confirmApprove = async () => {
  approveLoading.value = true;
  try {
    if (currentAction.value === "approve") {
      const dto: ApproveDto = {
        order_id: currentOrderId.value,
        opinion: approveForm.opinion,
        attachment_url: approveForm.attachment_url,
      };
      await approve(dto);
      ElMessage.success("审批通过");
    } else {
      const dto: RejectDto = {
        order_id: currentOrderId.value,
        opinion: approveForm.opinion,
        attachment_url: approveForm.attachment_url,
      };
      await reject(dto);
      ElMessage.success("审批驳回");
    }
    approveDialogVisible.value = false;
    await loadTodoList();
    if (activeTab.value === "myOrders") await loadMyOrders();
  } catch {
    ElMessage.error("操作失败");
  } finally {
    approveLoading.value = false;
  }
};

// ========== 我的申请 ==========
const loadMyOrders = async () => {
  myOrdersLoading.value = true;
  try {
    const params = myOrdersStatus.value ? { status: myOrdersStatus.value } : {};
    myOrdersList.value = await getMyOrders(params);
  } catch {
    ElMessage.error("加载申请列表失败");
  } finally {
    myOrdersLoading.value = false;
  }
};

const showRecords = async (orderId: number) => {
  try {
    recordsList.value = await getApprovalRecords(orderId);
    recordsDialogVisible.value = true;
  } catch {
    ElMessage.error("加载审批记录失败");
  }
};

// ========== 选项卡切换 ==========
const handleTabClick = (tab: any) => {
  if (tab.paneName === "todo") loadTodoList();
  else if (tab.paneName === "myOrders") loadMyOrders();
};

watch(
  () => route.query.tab,
  (tab) => {
    if (tab === "todo") activeTab.value = "todo";
  }
);

// ========== 初始化 ==========
onMounted(() => {
  loadFlows();
  loadTodoList(); // 更新全局待办计数
  loadMyOrders();
});
</script>

<style scoped>
.approval-manage {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}
.approval-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  border: none;
  overflow: hidden;
}
.approval-card :deep(.el-card__header) {
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
.approval-tabs {
  margin-top: 8px;
}
.submit-form {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}
.approval-table {
  border-radius: 16px;
  overflow: hidden;
}
.approval-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}
.approval-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}
.approval-table :deep(.el-table__row) {
  transition: background 0.2s;
}
.approval-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}
.approval-dialog :deep(.el-dialog) {
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(8px);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}
.my-orders-header {
  margin-bottom: 20px;
  display: flex;
  justify-content: flex-end;
}
.record-card {
  background: rgba(255, 255, 255, 0.5);
  border: none;
  box-shadow: none;
  padding: 8px;
}
.empty-text {
  text-align: center;
  color: #909399;
  padding: 40px;
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
.attachment-upload {
  width: 100%;
}
.attachment-upload :deep(.el-upload-dragger) {
  width: 100%;
  border-radius: 12px;
  transition: all 0.3s;
}
.attachment-upload :deep(.el-upload-dragger:hover) {
  border-color: #409eff;
  background-color: rgba(64, 158, 255, 0.02);
}
.attachment-link {
  color: #409eff;
  text-decoration: none;
}
.attachment-link:hover {
  text-decoration: underline;
}
.no-attachment {
  color: #909399;
}
</style>