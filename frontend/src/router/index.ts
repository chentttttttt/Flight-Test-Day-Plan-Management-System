// src/router/index.ts
import {
    createRouter,
    createWebHistory,
} from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useUserStore } from '@/stores/user'

const routes: RouteRecordRaw[] = [
    {
        path: '/login',
        name: 'Login',
        component: () => import('@/views/Login.vue'),
        meta: { requiresAuth: false }
    },
    {
        path: '/',
        component: () => import('@/layout/Layout.vue'),
        //redirect: '/users',
        children: [
            {
                path: 'user/:role?',
                name: 'UserManage',
                component: () => import('@/views/UserManage.vue'),
                meta: { title: '用户管理', requiresAuth: true }
            },
            // 新增菜单管理
            {
                path: 'menu',
                name: 'MenuManage',
                component: () => import('@/views/MenuManage.vue'),
                meta: { title: '菜单管理', requiresAuth: true }
            },

            {
                path: 'role',
                name: 'RoleManage',
                component: () => import('@/views/RoleManage.vue'),
                meta: { title: '角色管理', requiresAuth: true }
            },
            {
                path: 'abac-rules/:role?',
                name: 'AbacPolicyRuleList',
                component: () => import('@/views/PolicyRuleManage.vue'),
                meta: { title: '策略规则', requiresAuth: true }
            },
            {
                path: 'subject-attribute/:type?',
                name: 'SubjectAttributeManage',
                component: () => import('@/views/SubjectAttributeManage.vue'),
                meta: { title: '主体属性管理', requiresAuth: true }
            },
            {
                path: 'resource-attribute/:type?',
                name: 'ResourceAttributeManage',
                component: () => import('@/views/ResourceAttributeManage.vue'),
                meta: { title: '资源属性管理', requiresAuth: true }
            },
            {
                path: 'flow',
                name: 'FlowManage',
                component: () => import('@/views/approval/FlowManage.vue'),
                meta: { title: '审批流程管理', requiresAuth: true }
            },
            {
                path: 'approval',
                name: 'Approval',
                component: () => import('@/views/approval/Approval.vue'),
                meta: { title: '审批管理', requiresAuth: true }
            },
            {
                path: 'profile',
                name: 'Profile',
                component: () => import('@/views/Profile.vue'),
                meta: { title: '个人中心' }
            },
            {
                path: '/notification',
                name: 'NotificationCenter',
                component: () => import('@/views/NotificationCenter.vue'),
                meta: { title: '消息中心', requiresAuth: true }
            },
            {
                path: '/aircraft',
                name: 'AircraftManage',
                component: () => import('@/views/FlightSchedule/DailyFlightPlan.vue'),
                meta: { title: '飞机管理', requiresAuth: true }
            },
            {
                path: '/flight-operation',
                name: 'FlightOperation',
                component: () => import('@/views/FlightSchedule/FlightOperation.vue'),
                meta: { title: '试飞管理' }
            },
            {
                path: '/aircraft-management',
                name: 'AircraftManagement',
                component: () => import('@/views/AircraftManagement.vue'),
                meta: { title: '试飞资源管理' }
            },
            {
                path: '/flight-schedule-new',
                name: 'FlightScheduleNew',
                component: () => import('@/views/FlightScheduleNew.vue'),
                meta: { title: '试飞计划管理' }
            },
            {
                path: '/pilot',
                name: 'PilotManagement',
                component: () => import('@/views/PilotManagement.vue'),
                meta: { title: '飞行员管理' }
            },
            {
                path: "/exported-files",
                name: "ExportedFiles",
                component: () => import("@/views/ExportedFilesViewer.vue"),
                meta: { title: "导出文件管理" }
            },
            {
                path: "/pilotModelFlightTime",
                name: "PilotModelFlightTime",
                component: () => import("@/views/PilotModelFlightTime.vue"),
                meta: { title: "飞行员飞行时间管理" }
            }
        ]
    },
    {
        path: '/:pathMatch(.*)*',
        redirect: '/'
    }
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

// 路由守卫：检查认证状态
router.beforeEach((to, _from, next) => {
    const userStore = useUserStore()
    const token = userStore.token

    if (to.meta.requiresAuth !== false && !token) {
        next({ name: 'Login', query: { redirect: to.fullPath } })
    } else if (to.name === 'Login' && token) {
        next({ path: '/' })
    } else {
        next()
    }
})

export default router
