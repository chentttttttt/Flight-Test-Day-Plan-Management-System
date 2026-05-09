const TOKEN_KEY = 'user_token'
const USER_INFO_KEY = 'user_info'

export function getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY)
}
export function setToken(token: string): void {
    localStorage.setItem(TOKEN_KEY, token)
}
export function removeToken(): void {
    localStorage.removeItem(TOKEN_KEY)
}

export function getUserInfo(): any | null {
    const info = localStorage.getItem(USER_INFO_KEY)
    if (!info) return null
    try {
        return JSON.parse(info)
    } catch {
        return null
    }
}
export function setUserInfo(userInfo: any): void {
    try {
        localStorage.setItem(USER_INFO_KEY, JSON.stringify(userInfo))
    } catch (e) {
        console.error('❌ 用户信息存储失败', e)
    }
}
export function removeUserInfo(): void {
    localStorage.removeItem(USER_INFO_KEY)
}