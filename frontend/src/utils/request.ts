// src/utils/request.ts
import axios, {
    type AxiosInstance,
    type AxiosResponse,
    type InternalAxiosRequestConfig
} from 'axios'
import { ElMessage } from 'element-plus'
import { useUserStore } from '@/stores/user'
import router from '@/router'

// 定义响应数据结构
export interface ApiResponse<T = any> {
    code: number
    message: string
    data: T
}

// 创建 axios 实例
const service: AxiosInstance = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
    timeout: 15000
})

// 请求拦截器
service.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
        const userStore = useUserStore()
        const token = userStore.token
        if (token) {
            config.headers.Authorization = `Bearer ${token}`
        }
        return config
    },
    (error) => {
        return Promise.reject(error)
    }
)

// 响应拦截器
service.interceptors.response.use(
    (response: AxiosResponse<ApiResponse>) => {
        const res = response.data
        // 根据后端统一返回的 code 判断
        if (res.code === 200) {
            return res.data // 直接返回 data 字段内容
        } else {
            // 业务错误
            ElMessage.error(res.message || '请求失败')
            return Promise.reject(new Error(res.message || 'Error'))
        }
    },
    (error) => {
        // HTTP 错误处理
        if (error.response) {
            const status = error.response.status
            if (status === 401) {
                ElMessage.error('登录已过期，请重新登录')
                const userStore = useUserStore()
                userStore.logout()
                router.push({ name: 'Login' })
            } else if (status === 403) {
                ElMessage.error('没有权限访问')
            } else if (status === 404) {
                ElMessage.error('请求资源不存在')
            } else {
                ElMessage.error(error.response.data?.message || '服务器错误')
            }
        } else {
            ElMessage.error('网络连接异常')
        }
        return Promise.reject(error)
    }
)

export default service
