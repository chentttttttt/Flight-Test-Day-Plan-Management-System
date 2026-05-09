<template>
  <div class="aircraft-management">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="header-title">试飞资源管理</span>
        </div>
      </template>

      <el-tabs v-model="activeTab">
        <!-- 机型管理 -->
        <el-tab-pane label="机型" name="model">
          <div class="toolbar">
            <el-button type="primary" @click="openModelDialog()"
              >新增机型</el-button
            >
          </div>
          <el-table :data="modelList" border stripe v-loading="modelLoading">
            <el-table-column prop="id" label="ID" width="80" />
            <el-table-column prop="name" label="机型名称" />
            <el-table-column prop="code" label="编码" />
            <el-table-column
              prop="description"
              label="描述"
              show-overflow-tooltip
            />
            <el-table-column label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === 1 ? 'success' : 'danger'">
                  {{ row.status === 1 ? "启用" : "禁用" }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="openModelDialog(row)"
                  >编辑</el-button
                >
                <el-button
                  size="small"
                  type="danger"
                  @click="handleDeleteModel(row)"
                  >删除</el-button
                >
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 科目管理 -->
        <el-tab-pane label="科目" name="subject">
          <div class="toolbar">
            <el-button type="primary" @click="openSubjectDialog()"
              >新增科目</el-button
            >
          </div>
          <el-table
            :data="subjectList"
            border
            stripe
            v-loading="subjectLoading"
          >
            <el-table-column prop="id" label="ID" width="80" />
            <el-table-column prop="name" label="科目名称" />
            <el-table-column prop="code" label="编码" />
            <el-table-column prop="danger_level" label="危险等级" width="100">
              <template #default="{ row }">
                <el-tag :type="row.danger_level === '高危' ? 'danger' : 'info'">
                  {{ row.danger_level }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column
              prop="default_duration"
              label="默认时长(h)"
              width="120"
            />
            <el-table-column prop="subject_type" label="科目类型" width="120" />
            <el-table-column
              prop="required_pilots"
              label="所需飞行员"
              width="120"
            />
            <el-table-column label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === 1 ? 'success' : 'danger'">
                  {{ row.status === 1 ? "启用" : "禁用" }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="openSubjectDialog(row)"
                  >编辑</el-button
                >
                <el-button
                  size="small"
                  type="danger"
                  @click="handleDeleteSubject(row)"
                  >删除</el-button
                >
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 飞机管理 -->
        <el-tab-pane label="飞机" name="aircraft">
          <div class="toolbar">
            <el-button type="primary" @click="openAircraftDialog()"
              >新增飞机</el-button
            >
          </div>
          <el-table
            :data="aircraftList"
            border
            stripe
            v-loading="aircraftLoading"
          >
            <el-table-column prop="id" label="ID" width="80" />
            <el-table-column prop="plane_no" label="飞机编号" />
            <el-table-column label="机型" width="150">
              <template #default="{ row }">
                {{ getModelName(row.model_id) }}
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '测试中' ? 'warning' : 'success'">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="remark" label="备注" show-overflow-tooltip />
            <el-table-column label="操作" width="180" fixed="right">
              <template #default="{ row }">
                <el-button size="small" @click="viewAircraftDetail(row)"
                  >详情</el-button
                >
                <el-button size="small" @click="openAircraftDialog(row)"
                  >编辑</el-button
                >
                <el-button
                  size="small"
                  type="danger"
                  @click="handleDeleteAircraft(row)"
                  >删除</el-button
                >
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 机型-科目关联管理 -->
        <el-tab-pane label="机型-科目关联" name="modelSubject">
          <div class="toolbar">
            <el-form :inline="true" size="default">
              <el-form-item label="机型">
                <el-select
                  v-model="filterModelId"
                  placeholder="全部机型"
                  clearable
                  filterable
                  @change="loadModelSubjects"
                >
                  <el-option
                    v-for="m in modelList"
                    :key="m.id"
                    :label="m.name"
                    :value="m.id"
                  />
                </el-select>
              </el-form-item>
              <el-form-item label="科目">
                <el-select
                  v-model="filterSubjectId"
                  placeholder="全部科目"
                  clearable
                  filterable
                  @change="loadModelSubjects"
                >
                  <el-option
                    v-for="s in subjectList"
                    :key="s.id"
                    :label="s.name"
                    :value="s.id"
                  />
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="loadModelSubjects"
                  >查询</el-button
                >
              </el-form-item>
            </el-form>
            <el-button type="success" @click="openAddAssociationDialog"
              >添加关联</el-button
            >
          </div>

          <el-table
            :data="modelSubjectList"
            border
            stripe
            v-loading="msLoading"
          >
            <el-table-column label="机型" width="200">
              <template #default="{ row }">
                {{ getModelName(row.model_id) }}
              </template>
            </el-table-column>
            <el-table-column label="科目" width="200">
              <template #default="{ row }">
                {{ getSubjectName(row.subject_id) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120">
              <template #default="{ row }">
                <el-button
                  size="small"
                  type="danger"
                  @click="handleDeleteAssociation(row)"
                  >解除关联</el-button
                >
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 机型/科目/飞机 通用弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="600px"
      @close="resetForm"
    >
      <el-form
        :model="formData"
        :rules="formRules"
        ref="formRef"
        label-width="100px"
      >
        <!-- 机型表单 -->
        <template v-if="dialogType === 'model'">
          <el-form-item label="机型名称" prop="name">
            <el-input v-model="formData.name" />
          </el-form-item>
          <el-form-item label="编码">
            <el-input v-model="formData.code" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input type="textarea" v-model="formData.description" />
          </el-form-item>
          <el-form-item label="状态">
            <el-switch
              v-model="formData.statusSwitch"
              :active-value="1"
              :inactive-value="0"
            />
          </el-form-item>
        </template>

        <!-- 科目表单 -->
        <template v-if="dialogType === 'subject'">
          <el-form-item label="科目名称" prop="name">
            <el-input v-model="formData.name" />
          </el-form-item>
          <el-form-item label="编码">
            <el-input v-model="formData.code" />
          </el-form-item>
          <el-form-item label="危险等级" prop="danger_level">
            <el-radio-group v-model="formData.danger_level">
              <el-radio label="普通">普通</el-radio>
              <el-radio label="高危">高危</el-radio>
            </el-radio-group>
          </el-form-item>
          <el-form-item label="默认时长(h)" prop="default_duration">
            <el-input-number
              v-model="formData.default_duration"
              :min="0.5"
              :step="0.5"
              :precision="1"
            />
          </el-form-item>
          <el-form-item label="所需飞行员数量" prop="required_pilots">
            <el-input-number
              v-model="formData.required_pilots"
              :min="1"
              :step="1"
            />
          </el-form-item>
          <el-form-item label="科目类型">
            <el-input v-model="formData.subject_type" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input type="textarea" v-model="formData.description" />
          </el-form-item>
          <el-form-item label="状态">
            <el-switch
              v-model="formData.statusSwitch"
              :active-value="1"
              :inactive-value="0"
            />
          </el-form-item>
        </template>

        <!-- 飞机表单 -->
        <template v-if="dialogType === 'aircraft'">
          <el-form-item label="飞机编号" prop="plane_no">
            <el-input v-model="formData.plane_no" />
          </el-form-item>
          <el-form-item label="机型" prop="model_id">
            <el-select v-model="formData.model_id" placeholder="请选择机型">
              <el-option
                v-for="m in modelList"
                :key="m.id"
                :label="m.name"
                :value="m.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="状态">
            <el-select v-model="formData.status">
              <el-option label="测试中" value="测试中" />
              <el-option label="完成" value="完成" />
            </el-select>
          </el-form-item>
          <el-form-item label="备注">
            <el-input type="textarea" v-model="formData.remark" />
          </el-form-item>
          <el-form-item label="扩展属性">
            <el-input
              type="textarea"
              v-model="extraAttrsStr"
              placeholder='{"key": "value"}'
              rows="2"
            />
          </el-form-item>
        </template>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm" :loading="submitLoading"
          >确定</el-button
        >
      </template>
    </el-dialog>

    <!-- 飞机详情弹窗 -->
    <el-dialog v-model="detailDialogVisible" title="飞机详情" width="800px">
      <div v-if="aircraftDetail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="飞机编号">{{
            aircraftDetail.aircraft.plane_no
          }}</el-descriptions-item>
          <el-descriptions-item label="机型">{{
            aircraftDetail.model?.name
          }}</el-descriptions-item>
          <el-descriptions-item label="状态">{{
            aircraftDetail.aircraft.status
          }}</el-descriptions-item>
          <el-descriptions-item label="总飞行时间(h)">{{
            aircraftDetail.aircraft.total_running_hours ?? "-"
          }}</el-descriptions-item>
          <el-descriptions-item label="备注">{{
            aircraftDetail.aircraft.remark || "-"
          }}</el-descriptions-item>
        </el-descriptions>
        <h4>科目完成情况</h4>
        <el-table :data="aircraftDetail.subjects" border>
          <el-table-column prop="subject.name" label="科目名称" />
          <el-table-column label="危险等级" width="100">
            <template #default="{ row }">
              <el-tag
                :type="row.subject.danger_level === '高危' ? 'danger' : 'info'"
                >{{ row.subject.danger_level }}</el-tag
              >
            </template>
          </el-table-column>
          <el-table-column label="已完成" width="80">
            <template #default="{ row }">
              <el-tag :type="row.completed ? 'success' : 'danger'">{{
                row.completed ? "是" : "否"
              }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column
            prop="operation_hours"
            label="累计运行(h)"
            width="120"
          />
          <el-table-column prop="start_time" label="开始时间" width="180" />
          <el-table-column prop="end_time" label="结束时间" width="180" />
          <!-- <el-table-column label="操作" width="150">
            <template #default="{ row }">
              <el-button
                v-if="!row.completed && row.start_time"
                size="small"
                @click="openEndFlightDialog(row.subject)"
                >结束试飞</el-button
              >
            </template>
          </el-table-column> -->
        </el-table>
      </div>
      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>

    <!-- 结束试飞弹窗 -->
    <el-dialog v-model="endFlightDialogVisible" title="结束试飞" width="400px">
      <el-form :model="endFlightForm" label-width="120px">
        <el-form-item label="空中飞行时长(h)">
          <el-input-number
            v-model="endFlightForm.flight_hours"
            :min="0"
            :step="0.5"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="endFlightDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          @click="confirmEndFlight"
          :loading="endFlightLoading"
          >确认结束</el-button
        >
      </template>
    </el-dialog>

    <!-- 新增关联弹窗 -->
    <el-dialog
      v-model="associationDialogVisible"
      title="添加机型-科目关联"
      width="400px"
      @close="resetAssociationForm"
    >
      <el-form
        :model="associationForm"
        :rules="associationRules"
        ref="associationFormRef"
        label-width="100px"
      >
        <el-form-item label="机型" prop="model_id">
          <el-select
            v-model="associationForm.model_id"
            placeholder="请选择机型"
            filterable
          >
            <el-option
              v-for="m in modelList"
              :key="m.id"
              :label="m.name"
              :value="m.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="科目" prop="subject_id">
          <el-select
            v-model="associationForm.subject_id"
            placeholder="请选择科目"
            filterable
          >
            <el-option
              v-for="s in subjectList"
              :key="s.id"
              :label="s.name"
              :value="s.id"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="associationDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          @click="confirmAddAssociation"
          :loading="associationLoading"
          >确定</el-button
        >
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
  listModels,
  createModel,
  updateModel,
  deleteModel,
  listSubjects,
  createSubject,
  updateSubject,
  deleteSubject,
  listAircraft,
  createAircraft,
  updateAircraft,
  deleteAircraft,
  getAircraftDetail,
  startFlight,
  endFlight,
  getIncompleteSubjects,
} from "@/api/aircraft";
import {
  createModelSubject,
  deleteModelSubject,
  listModelSubjects,
} from "@/api/modelSubject";
import type {
  AircraftModel,
  Subject,
  Aircraft,
  AircraftDetail,
  CreateAircraftDto,
} from "@/types/aircraft";

// ========== 机型 ==========
const modelList = ref<AircraftModel[]>([]);
const modelLoading = ref(false);
// ========== 科目 ==========
const subjectList = ref<Subject[]>([]);
const subjectLoading = ref(false);
// ========== 飞机 ==========
const aircraftList = ref<Aircraft[]>([]);
const aircraftLoading = ref(false);
const modelMap = ref<Record<number, string>>({});

// ========== 通用弹窗 ==========
const dialogVisible = ref(false);
const dialogTitle = ref("");
const dialogType = ref<"model" | "subject" | "aircraft">("model");
const formRef = ref<FormInstance>();
const submitLoading = ref(false);
const formData = reactive<any>({});
const formRules = ref({});
const extraAttrsStr = ref("");

// ========== 飞机详情 ==========
const detailDialogVisible = ref(false);
const aircraftDetail = ref<AircraftDetail | null>(null);

// ========== 结束试飞弹窗 ==========
const endFlightDialogVisible = ref(false);
const endFlightForm = reactive({ flight_hours: 0 });
const currentEndSubject = ref<Subject | null>(null);
const endFlightLoading = ref(false);

// ========== 机型-科目关联 ==========
const activeTab = ref("model");
const filterModelId = ref<number | undefined>();
const filterSubjectId = ref<number | undefined>();
const modelSubjectList = ref<any[]>([]);
const msLoading = ref(false);

// 新增关联弹窗
const associationDialogVisible = ref(false);
const associationFormRef = ref<FormInstance>();
const associationLoading = ref(false);
const associationForm = reactive({
  model_id: undefined as number | undefined,
  subject_id: undefined as number | undefined,
});
const associationRules: FormRules = {
  model_id: [{ required: true, message: "请选择机型", trigger: "change" }],
  subject_id: [{ required: true, message: "请选择科目", trigger: "change" }],
};

// 辅助函数
const getModelName = (modelId: number) => modelMap.value[modelId] || "-";
const getSubjectName = (subjectId: number) => {
  const s = subjectList.value.find((item) => item.id === subjectId);
  return s ? s.name : `科目${subjectId}`;
};

// 加载机型列表
const loadModels = async () => {
  modelLoading.value = true;
  try {
    modelList.value = await listModels();
    modelMap.value = {};
    modelList.value.forEach((m) => {
      modelMap.value[m.id] = m.name;
    });
  } finally {
    modelLoading.value = false;
  }
};

// 加载科目列表
const loadSubjects = async () => {
  subjectLoading.value = true;
  try {
    subjectList.value = await listSubjects();
  } finally {
    subjectLoading.value = false;
  }
};

// 加载飞机列表
const loadAircraft = async () => {
  aircraftLoading.value = true;
  try {
    aircraftList.value = await listAircraft();
  } finally {
    aircraftLoading.value = false;
  }
};

// 加载机型-科目关联
const loadModelSubjects = async () => {
  msLoading.value = true;
  try {
    const res = await listModelSubjects({
      model_id: filterModelId.value,
      subject_id: filterSubjectId.value,
    });
    modelSubjectList.value = res?.data || [];
  } catch (error) {
    ElMessage.error("加载关联数据失败");
    modelSubjectList.value = [];
  } finally {
    msLoading.value = false;
  }
};

// 添加关联弹窗
const openAddAssociationDialog = () => {
  associationForm.model_id = undefined;
  associationForm.subject_id = undefined;
  associationDialogVisible.value = true;
};

const resetAssociationForm = () => {
  associationFormRef.value?.resetFields();
};

const confirmAddAssociation = async () => {
  if (!associationFormRef.value) return;
  await associationFormRef.value.validate(async (valid) => {
    if (!valid) return;
    associationLoading.value = true;
    try {
      await createModelSubject(
        associationForm.model_id!,
        associationForm.subject_id!
      );
      ElMessage.success("关联已添加");
      associationDialogVisible.value = false;
      await loadModelSubjects();
    } catch (error: any) {
      ElMessage.error(error.message || "添加失败");
    } finally {
      associationLoading.value = false;
    }
  });
};

// 解除关联
const handleDeleteAssociation = async (row: any) => {
  try {
    await ElMessageBox.confirm(
      `确定解除“${getModelName(row.model_id)}”与“${getSubjectName(
        row.subject_id
      )}”的关联吗？`,
      "提示",
      { type: "warning" }
    );
    await deleteModelSubject(row.model_id, row.subject_id);
    ElMessage.success("关联已解除");
    await loadModelSubjects();
  } catch {
    // 用户取消
  }
};

// 飞机详情
const viewAircraftDetail = async (aircraft: Aircraft) => {
  try {
    aircraftDetail.value = await getAircraftDetail(aircraft.id);
    detailDialogVisible.value = true;
  } catch (error) {
    ElMessage.error("加载详情失败");
  }
};
const loadAircraftDetail = async (id: number) => {
  try {
    aircraftDetail.value = await getAircraftDetail(id);
  } catch {}
};

// 结束试飞
const openEndFlightDialog = (subject: Subject) => {
  currentEndSubject.value = subject;
  endFlightForm.flight_hours = subject.default_duration || 1;
  endFlightDialogVisible.value = true;
};

const confirmEndFlight = async () => {
  if (!aircraftDetail.value || !currentEndSubject.value) return;
  endFlightLoading.value = true;
  try {
    await endFlight({
      aircraft_id: aircraftDetail.value.aircraft.id,
      subject_id: currentEndSubject.value.id,
      flight_hours: endFlightForm.flight_hours,
    });
    ElMessage.success("试飞结束，数据已更新");
    endFlightDialogVisible.value = false;
    await loadAircraftDetail(aircraftDetail.value.aircraft.id);
  } catch (error: any) {
    ElMessage.error(error.message || "结束失败");
  } finally {
    endFlightLoading.value = false;
  }
};

// 通用CRUD操作
const openModelDialog = (row?: AircraftModel) => {
  dialogType.value = "model";
  if (row) {
    dialogTitle.value = "编辑机型";
    Object.assign(formData, {
      id: row.id,
      name: row.name,
      code: row.code,
      description: row.description,
      statusSwitch: row.status,
    });
  } else {
    dialogTitle.value = "新增机型";
    Object.assign(formData, {
      id: undefined,
      name: "",
      code: "",
      description: "",
      statusSwitch: 1,
    });
  }
  dialogVisible.value = true;
};

const openSubjectDialog = (row?: Subject) => {
  dialogType.value = "subject";
  if (row) {
    dialogTitle.value = "编辑科目";
    Object.assign(formData, {
      id: row.id,
      name: row.name,
      code: row.code,
      danger_level: row.danger_level,
      default_duration: row.default_duration,
      required_pilots: (row as any).required_pilots ?? 1, // 新增字段
      subject_type: row.subject_type,
      description: row.description,
      statusSwitch: row.status,
    });
  } else {
    dialogTitle.value = "新增科目";
    Object.assign(formData, {
      id: undefined,
      name: "",
      code: "",
      danger_level: "普通",
      default_duration: 2,
      required_pilots: 1, // 默认 1
      subject_type: "",
      description: "",
      statusSwitch: 1,
    });
  }
  dialogVisible.value = true;
};

const openAircraftDialog = (row?: Aircraft) => {
  dialogType.value = "aircraft";
  if (row) {
    dialogTitle.value = "编辑飞机";
    Object.assign(formData, {
      id: row.id,
      plane_no: row.plane_no,
      model_id: row.model_id,
      status: row.status,
      remark: row.remark,
    });
    extraAttrsStr.value = row.extra_attrs
      ? JSON.stringify(row.extra_attrs, null, 2)
      : "";
  } else {
    dialogTitle.value = "新增飞机";
    Object.assign(formData, {
      id: undefined,
      plane_no: "",
      model_id: undefined,
      status: "测试中",
      remark: "",
    });
    extraAttrsStr.value = "";
  }
  dialogVisible.value = true;
};

const resetForm = () => {
  formRef.value?.resetFields();
  extraAttrsStr.value = "";
};

const submitForm = async () => {
  submitLoading.value = true;
  try {
    const data = { ...formData };
    if (dialogType.value === "model") {
      const payload = {
        name: data.name,
        code: data.code || undefined,
        description: data.description || undefined,
        status: data.statusSwitch,
      };
      if (data.id) await updateModel(data.id, payload);
      else await createModel(payload);
      await loadModels();
    } else if (dialogType.value === "subject") {
      const payload = {
        name: data.name,
        code: data.code || undefined,
        danger_level: data.danger_level,
        default_duration: data.default_duration,
        required_pilots: data.required_pilots ?? 1, // 新增
        subject_type: data.subject_type || undefined,
        description: data.description || undefined,
        status: data.statusSwitch,
      };
      if (data.id) await updateSubject(data.id, payload);
      else await createSubject(payload);
      await loadSubjects();
    } else if (dialogType.value === "aircraft") {
      const payload: CreateAircraftDto = {
        model_id: data.model_id,
        plane_no: data.plane_no,
        status: data.status,
        remark: data.remark || undefined,
        extra_attrs: extraAttrsStr.value
          ? JSON.parse(extraAttrsStr.value)
          : undefined,
      };
      if (data.id) await updateAircraft(data.id, payload);
      else await createAircraft(payload);
      await loadAircraft();
    }
    ElMessage.success("操作成功");
    dialogVisible.value = false;
  } catch (error: any) {
    ElMessage.error(error.message || "操作失败");
  } finally {
    submitLoading.value = false;
  }
};

const handleDeleteModel = async (row: AircraftModel) => {
  await ElMessageBox.confirm("确认删除机型吗？", "提示", { type: "warning" });
  await deleteModel(row.id);
  await loadModels();
  ElMessage.success("删除成功");
};

const handleDeleteSubject = async (row: Subject) => {
  await ElMessageBox.confirm("确认删除科目吗？", "提示", { type: "warning" });
  await deleteSubject(row.id);
  await loadSubjects();
  ElMessage.success("删除成功");
};

const handleDeleteAircraft = async (row: Aircraft) => {
  await ElMessageBox.confirm("确认删除飞机吗？", "提示", { type: "warning" });
  await deleteAircraft(row.id);
  await loadAircraft();
  ElMessage.success("删除成功");
};

onMounted(async () => {
  await Promise.all([loadModels(), loadSubjects(), loadAircraft()]);
  loadModelSubjects();
});
</script>

<style scoped>
.aircraft-management {
  padding: 20px;
  height: 100%;
  background: #f5f7fa;
}
.card-header {
  font-size: 18px;
  font-weight: 600;
}
.toolbar {
  margin-bottom: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>