<template>
  <template v-for="item in items" :key="item.id">
    <!-- 目录，有子菜单 -->
    <el-sub-menu
      v-if="item.menu_type === 0 && item.children && item.children.length > 0"
    >
      <template #title>{{ item.name }}</template>
      <MenuNode :items="item.children" />
    </el-sub-menu>

    <!-- 菜单项：类型为 1 且有路由 -->

    <el-menu-item
      v-else-if="item.menu_type === 1 && item.path"
      :index="item.path"
      @click="() => console.log('点击菜单:', item.name, 'path:', item.path)"
    >
      <span>{{ item.name }}</span>
    </el-menu-item>

    <!-- 菜单项：类型为 1 但无路由路径（不可点击） -->
    <el-menu-item v-else-if="item.menu_type === 1 && !item.path" disabled>
      <span>{{ item.name }}</span>
    </el-menu-item>

    <!-- 目录项（无子菜单）：类型为 0，不可点击 -->
    <el-menu-item
      v-else-if="item.menu_type === 0"
      :index="'disabled-' + item.id"
      disabled
    >
      <span>{{ item.name }}</span>
    </el-menu-item>

    <!-- 按钮类型不渲染（已在条件中排除） -->
  </template>
</template>

<script setup lang="ts">
import type { Menu } from "@/types/menu";

defineProps<{
  items: Menu[];
}>();
</script>