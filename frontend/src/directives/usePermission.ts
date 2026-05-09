import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useUserStore } from '@/stores/user';

const getPermissionPrefix = (path: string): string => {
    let prefix = path.replace(/^\//, '').split('/')[0].toUpperCase();
    return prefix || '';
};

export function usePagePermission() {
    const route = useRoute();
    const userStore = useUserStore();
    const permissions = computed(() => userStore.permissions);

    const prefix = computed(() => getPermissionPrefix(route.path));

    const hasPagePermission = (operation: string): boolean => {
        if (!prefix.value) return false;
        const requiredPerm = `${prefix.value}:${operation.toUpperCase()}`;
        const has = permissions.value.some(p => p.toUpperCase() === requiredPerm);
        console.log(`[权限检查] 所需权限: ${requiredPerm}, 用户权限列表:`, permissions.value, `结果: ${has}`);
        return has;
    };

    const canCreate = computed(() => hasPagePermission('CREATE'));
    const canUpdate = computed(() => hasPagePermission('UPDATE'));
    const canDelete = computed(() => hasPagePermission('DELETE'));
    const canView = computed(() => hasPagePermission('VIEW'));
    const canQuery = computed(() => hasPagePermission('QUERY'));
    const canImport = computed(() => hasPagePermission('IMPORT'));
    const canExport = computed(() => hasPagePermission('EXPORT'));

    return {
        hasPagePermission,
        canCreate,
        canUpdate,
        canDelete,
        canView,
        canQuery,
        canImport,
        canExport,
        prefix,
    };
}