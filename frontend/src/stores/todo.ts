import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getTodoList } from '@/api/approval';

export const useTodoStore = defineStore('todo', () => {
    const todoCount = ref(0);

    const fetchTodoCount = async () => {
        try {
            const list = await getTodoList();
            todoCount.value = list.length;
        } catch (error) {
            console.error('获取待办数量失败', error);
            todoCount.value = 0;
        }
    };

    const refresh = () => fetchTodoCount();

    return { todoCount, fetchTodoCount, refresh };
});