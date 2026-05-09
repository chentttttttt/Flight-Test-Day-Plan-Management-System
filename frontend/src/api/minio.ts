import request from "@/utils/request"; // 根据实际项目路径调整
import type {
    UploadResponse,
    FileUrlResponse,
    FileListResponse,
} from "@/types/minio";

/**
 * 上传文件到 MinIO
 * @param formData 包含文件的 FormData
 * @param filename 可选，指定自定义文件名（允许覆盖）
 */
export function uploadFile(
    formData: FormData,
    filename?: string
): Promise<UploadResponse> {
    return request.post("/minio/upload", formData, {
        headers: { "Content-Type": "multipart/form-data" },
        params: filename ? { filename } : {},
    });
}

/**
 * 获取文件临时签名下载链接
 * @param key 文件存储 key
 * @param expires 有效期秒数，默认 3600
 */
export function getFileUrl(
    key: string,
    expires = 3600
): Promise<FileUrlResponse> {
    return request.get(`/minio/url/${encodeURIComponent(key)}`, {
        params: { expires },
    });
}

/**
 * 删除文件
 * @param key 文件存储 key
 */
export function deleteFile(key: string): Promise<void> {
    return request.delete(`/minio/delete/${encodeURIComponent(key)}`);
}

/**
 * 列出指定前缀下的文件
 * @param prefix 前缀路径（如 "approval"）
 */
export function listFiles(prefix: string): Promise<FileListResponse> {
    return request.get("/minio/list", {
        params: { prefix },
    });
}

/**
 * 批量获取文件 URL（并发）
 * @param keys 文件 key 数组
 * @param expires 签名有效期（秒）
 */
export async function batchGetFileUrls(
    keys: string[],
    expires = 3600
): Promise<Map<string, string>> {
    const map = new Map<string, string>();
    const promises = keys.map(async (key) => {
        try {
            const res = await getFileUrl(key, expires);
            map.set(key, res.url);
        } catch (err) {
            console.error(`获取 ${key} URL 失败`, err);
        }
    });
    await Promise.allSettled(promises);
    return map;
}