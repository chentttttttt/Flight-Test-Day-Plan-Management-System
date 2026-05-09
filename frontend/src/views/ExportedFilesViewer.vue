<template>
  <div class="exported-files-viewer">
    <el-card class="files-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon class="header-icon"><FolderOpened /></el-icon>
            <span class="header-title">MinIO 导出文件管理</span>
          </div>
        </div>
      </template>

      <div class="toolbar">
        <el-input
          v-model="prefix"
          placeholder="文件前缀，如 approval"
          style="width: 240px"
          clearable
          @keyup.enter="loadFiles"
        >
          <template #prepend>前缀</template>
        </el-input>
        <el-button type="primary" @click="loadFiles" :loading="loading">
          <el-icon><Search /></el-icon> 查询
        </el-button>
        <el-button
          type="success"
          @click="exportCurrentList"
          :disabled="fileList.length === 0"
        >
          <el-icon><Download /></el-icon> 导出清单
        </el-button>
      </div>

      <el-table
        :data="displayedFiles"
        v-loading="loading"
        border
        stripe
        class="files-table"
        empty-text="暂无文件"
      >
        <el-table-column
          prop="key"
          label="文件 Key"
          min-width="280"
          show-overflow-tooltip
        />
        <el-table-column label="大小" width="120" align="center">
          <template #default="{ row }">
            {{ formatFileSize(row.size) }}
          </template>
        </el-table-column>
        <el-table-column label="最后修改时间" width="180" align="center">
          <template #default="{ row }">
            {{ formatTimestamp(row.last_modified) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" align="center" fixed="right">
          <template #default="{ row }">
            <el-button
              type="primary"
              link
              :loading="row._viewLoading"
              @click="viewFile(row)"
            >
              <el-icon><View /></el-icon> 查看
            </el-button>
            <el-popconfirm
              title="确定删除此文件吗？"
              confirm-button-text="删除"
              cancel-button-text="取消"
              @confirm="deleteFileHandler(row)"
            >
              <template #reference>
                <el-button type="danger" link :loading="row._deleteLoading">
                  <el-icon><Delete /></el-icon> 删除
                </el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper" v-if="fileList.length > pageSize">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="fileList.length"
          layout="prev, pager, next"
          background
          small
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { ElMessage } from "element-plus";
import {
  FolderOpened,
  Search,
  View,
  Delete,
  Download,
} from "@element-plus/icons-vue";
import { listFiles, getFileUrl, deleteFile } from "@/api/minio";
import type { FileItem } from "@/types/minio"; // 全局类型，移除本地重复定义

// ========== 响应式数据 ==========
const prefix = ref("approval");
const fileList = ref<FileItem[]>([]);
const loading = ref(false);

// 前端分页
const currentPage = ref(1);
const pageSize = ref(10);
const displayedFiles = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return fileList.value.slice(start, start + pageSize.value);
});

// ========== 方法 ==========
const loadFiles = async () => {
  if (!prefix.value.trim()) {
    ElMessage.warning("请输入文件前缀");
    return;
  }
  loading.value = true;
  try {
    const res = await listFiles(prefix.value);
    // 后端返回 { files: FileItem[] }
    const list = res.files || [];
    fileList.value = list.map((item: FileItem) => ({
      ...item,
      _viewLoading: false,
      _deleteLoading: false,
    }));
    currentPage.value = 1;
  } catch (error) {
    ElMessage.error("加载文件列表失败");
    console.error(error);
  } finally {
    loading.value = false;
  }
};

const viewFile = async (row: FileItem) => {
  row._viewLoading = true;
  try {
    const res = await getFileUrl(row.key);
    // 后端返回 { url: string }
    const url = res.url;
    if (url) {
      window.open(url, "_blank");
    } else {
      ElMessage.error("获取链接失败");
    }
  } catch (error) {
    ElMessage.error("获取文件链接失败");
    console.error(error);
  } finally {
    row._viewLoading = false;
  }
};

const deleteFileHandler = async (row: FileItem) => {
  row._deleteLoading = true;
  try {
    await deleteFile(row.key);
    ElMessage.success("删除成功");
    await loadFiles();
  } catch (error) {
    ElMessage.error("删除失败");
    console.error(error);
  } finally {
    row._deleteLoading = false;
  }
};

const exportCurrentList = () => {
  const content = [
    "文件Key,大小(字节),最后修改时间",
    ...fileList.value.map(
      (item) =>
        `${item.key},${item.size},${
          item.last_modified
            ? new Date(item.last_modified * 1000).toISOString()
            : ""
        }`
    ),
  ].join("\n");
  const blob = new Blob(["\uFEFF" + content], {
    type: "text/csv;charset=utf-8",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${prefix.value}_文件清单_${new Date()
    .toISOString()
    .slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
};

const handlePageChange = (page: number) => {
  currentPage.value = page;
};

// ========== 工具函数 ==========
const formatFileSize = (bytes: number) => {
  if (bytes === undefined || bytes === null) return "-";
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

const formatTimestamp = (timestamp?: number | null) => {
  if (!timestamp) return "-";
  const date = new Date(timestamp * 1000);
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
};

// ========== 初始化 ==========
onMounted(() => {
  loadFiles();
});
</script>

<style scoped>
.exported-files-viewer {
  height: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #e9eef3 100%);
  padding: 20px;
  box-sizing: border-box;
}

.files-card {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.02);
  border: none;
  overflow: hidden;
}

.files-card :deep(.el-card__header) {
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

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.files-table {
  border-radius: 16px;
  overflow: hidden;
}

.files-table :deep(.el-table__inner-wrapper) {
  border-radius: 16px;
}

.files-table :deep(.el-table__header-wrapper) th {
  font-weight: 600;
  background-color: rgba(64, 158, 255, 0.05);
}

.files-table :deep(.el-table__row) {
  transition: background 0.2s;
}

.files-table :deep(.el-table__row:hover) {
  background-color: rgba(64, 158, 255, 0.04);
}

.pagination-wrapper {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

:deep(.el-button--primary) {
  background: linear-gradient(135deg, #409eff, #36d1dc);
  border: none;
  box-shadow: 0 4px 10px rgba(64, 158, 255, 0.3);
}

:deep(.el-button--primary:hover) {
  transform: translateY(-1px);
  box-shadow: 0 6px 14px rgba(64, 158, 255, 0.4);
}
</style>