// stores/notification.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getUnreadCount } from '@/api/notification';

export const useNotificationStore = defineStore('notification', () => {
    const unreadCount = ref(0);

    const fetchUnreadCount = async () => {
        try {
            const count = await getUnreadCount();
            unreadCount.value = count;
        } catch (error) {
            console.error('获取未读数量失败', error);
        }
    };

    const decrement = (num = 1) => {
        unreadCount.value = Math.max(0, unreadCount.value - num);
    };

    return { unreadCount, fetchUnreadCount, decrement };
});