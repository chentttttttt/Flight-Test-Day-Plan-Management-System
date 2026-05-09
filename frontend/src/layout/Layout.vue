<template>
  <el-container class="layout-container">
    <el-aside width="220px" class="aside">
      <!-- 侧边栏内容不变 -->
      <div class="logo"><h3>管理系统</h3></div>
      <el-menu
        :default-active="activeMenu"
        router
        background-color="#304156"
        text-color="#bfcbd9"
        active-text-color="#409eff"
      >
        <MenuNode :items="menuTree" />
      </el-menu>
    </el-aside>
    <el-container>
      <el-header class="header">
        <div class="header-right">
          <!-- 待办通知图标（新增） -->
          <el-badge
            :value="todoStore.todoCount"
            :hidden="todoStore.todoCount === 0"
            class="notification-badge"
          >
            <el-button
              :icon="List"
              circle
              size="small"
              class="notification-btn"
              @click="goToTodo"
            />
          </el-badge>
          <!-- 原通知图标（可选） -->
          <el-badge
            :value="notificationStore.unreadCount"
            :hidden="notificationStore.unreadCount === 0"
            class="notification-badge"
          >
            <el-button
              :icon="Bell"
              circle
              size="small"
              class="notification-btn"
              @click="goToNotification"
            />
          </el-badge>
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              {{
                userStore.userInfo?.real_name ||
                userStore.userInfo?.username ||
                "管理员"
              }}
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="profile">个人中心</el-dropdown-item>
                <el-dropdown-item command="logout">退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>
      <el-main class="main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/user";
import { useNotificationStore } from "@/stores/notification";
import { useTodoStore } from "@/stores/todo";
import { ElMessage, ElMessageBox } from "element-plus";
import { getMenuTreeByRole } from "@/api/menu";
import type { Menu } from "@/types/menu";
import { ArrowDown, Bell, List } from "@element-plus/icons-vue";
import MenuNode from "@/components/MenuNode.vue";

const router = useRouter();
const userStore = useUserStore();
const notificationStore = useNotificationStore();
const todoStore = useTodoStore();

const activeMenu = computed(() => router.currentRoute.value.path);
const menuTree = ref<Menu[]>([]);
let pollTimer: number | null = null;

const extractPermissions = (menus: Menu[]): string[] => {
  let perms: string[] = [];
  for (const menu of menus) {
    if (menu.menu_type === 2 && menu.permission) perms.push(menu.permission);
    if (menu.children?.length) perms.push(...extractPermissions(menu.children));
  }
  return perms;
};

const loadMenuTree = async () => {
  const role = userStore.userInfo?.role;
  if (!role) {
    menuTree.value = [];
    userStore.setPermissions([]);
    return;
  }
  try {
    const res = await getMenuTreeByRole(role);
    menuTree.value = Array.isArray(res) ? res : [];
    const permissions = extractPermissions(menuTree.value);
    userStore.setPermissions(permissions);
  } catch (error) {
    console.error("菜单加载失败", error);
    ElMessage.error("菜单加载失败");
    menuTree.value = [];
    userStore.setPermissions([]);
  }
};

const startPolling = () => {
  if (pollTimer) clearInterval(pollTimer);
  pollTimer = window.setInterval(() => {
    notificationStore.fetchUnreadCount();
    todoStore.fetchTodoCount();
  }, 30000);
};

watch(
  () => userStore.userInfo,
  () => loadMenuTree(),
  { immediate: true, deep: true }
);

onMounted(() => {
  notificationStore.fetchUnreadCount();
  todoStore.fetchTodoCount();
  startPolling();
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});

const goToNotification = () => router.push("/notification");
const goToTodo = () => router.push("/approval?tab=todo"); // 跳转到审批管理的待办标签

const handleCommand = (command: string) => {
  if (command === "profile") router.push("/profile");
  else if (command === "logout") {
    ElMessageBox.confirm("确定退出登录吗？", "提示", { type: "warning" })
      .then(() => {
        userStore.logout();
        router.push("/login");
        ElMessage.success("退出成功");
      })
      .catch(() => {});
  }
};
</script>

<style scoped>
.layout-container {
  height: 100vh;
  margin: 0;
  padding: 0;
}
.aside {
  background-color: #304156;
  overflow: auto;
}
.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  background-color: #2b3a4b;
  margin-bottom: 20px;
}
.logo h3 {
  font-weight: normal;
  letter-spacing: 2px;
}
.header {
  background-color: #fff;
  border-bottom: 1px solid #e6e6e6;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 20px;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 20px;
}
.notification-badge {
  display: flex;
  align-items: center;
}
.notification-btn {
  background: transparent;
  border: none;
  font-size: 20px;
}
.notification-btn:hover {
  color: #409eff;
}
.user-info {
  cursor: pointer;
  color: #333;
  display: flex;
  align-items: center;
  gap: 5px;
}
.main {
  background-color: #f0f2f6;
  padding: 20px;
  overflow: auto;
}
.el-menu-item.is-active {
  background-color: #263445 !important;
  border-right: 3px solid #409eff;
}
.el-sub-menu .el-menu-item.is-active {
  background-color: #263445 !important;
}
.el-menu-item:hover {
  background-color: #2a3a4e !important;
}
</style>