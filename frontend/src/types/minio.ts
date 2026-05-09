/** 文件列表项 */
export interface FileItem {
    key: string;
    size: number;
    last_modified?: number | null; // Unix 秒级时间戳
}

/** 上传响应 */
export interface UploadResponse {
    key: string;
}

/** 获取文件URL响应 */
export interface FileUrlResponse {
    url: string;
}

/** 文件列表响应 */
export interface FileListResponse {
    files: FileItem[];
}

/** 删除文件参数（路径参数） */
export interface DeleteFileParams {
    key: string;
}