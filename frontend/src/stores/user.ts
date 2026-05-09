import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserInfo } from '@/types/user'
import { getToken, setToken, removeToken, getUserInfo, setUserInfo, removeUserInfo } from '@/utils/auth'

import { getCurrentUserInfo } from '@/api/user'

export const useUserStore = defineStore('user', () => {
    const token = ref<string>(getToken() || '')
    const userInfo = ref<UserInfo | null>(getUserInfo())
    const permissions = ref<string[]>([])

    const isLoggedIn = computed(() => !!token.value)

    const setAuth = (newToken: string, user: UserInfo) => {
        token.value = newToken
        userInfo.value = user
        setToken(newToken)
        setUserInfo(user)
        console.log('[userStore] setAuth - 用户信息已保存:', user)
    }

    const setPermissions = (perms: string[]) => {
        permissions.value = perms
        console.log('[userStore] setPermissions - 权限列表已更新:', perms)
    }

    const logout = () => {
        token.value = ''
        userInfo.value = null
        permissions.value = []
        removeToken()
        removeUserInfo()
        console.log('[userStore] logout - 已清除所有状态')
    }
    const fetchUserInfo = async (): Promise<UserInfo> => {
        try {
            const user = await getCurrentUserInfo()   // 调用后端接口
            userInfo.value = user
            setUserInfo(user)                         // 同步到 localStorage
            console.log('[userStore] fetchUserInfo - 用户信息已刷新:', user)
            return user
        } catch (error) {
            console.error('[userStore] fetchUserInfo 失败:', error)
            throw error
        }
    }
    const currentUserId = computed(() => userInfo.value?.id || 0)

    return {
        token,
        userInfo,
        permissions,
        isLoggedIn,
        currentUserId,
        setAuth,
        setPermissions,
        logout,
        fetchUserInfo
    }
})