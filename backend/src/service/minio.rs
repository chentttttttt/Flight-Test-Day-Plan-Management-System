// minio_service.rs (原有文件追加)
use anyhow::Result;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Object, ObjectCannedAcl};
use bytes::Bytes;
use uuid::Uuid;

use crate::utils::minio::MinioConfig;

#[derive(Clone)]
pub struct MinioService {
    client: Client,
    bucket: String,
}

impl MinioService {
    pub async fn new(config: &MinioConfig) -> Result<Self> {
        let client = config.create_client().await?;
        // 确保bucket存在
        let buckets = client.list_buckets().send().await?;
        let exists = buckets.buckets().iter().any(|b| b.name().eq(&Some(&config.bucket)));
        if !exists {
            client.create_bucket().bucket(&config.bucket).send().await?;
        }
        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }

    /// 上传文件（自动生成唯一文件名，带UUID）
    pub async fn upload(&self, data: Bytes, original_name: &str, prefix: &str) -> Result<String> {
        let ext = std::path::Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let key = if prefix.is_empty() {
            format!("{}{}", Uuid::new_v4(), if ext.is_empty() { "".to_string() } else { format!(".{}", ext) })
        } else {
            format!("{}/{}{}", prefix, Uuid::new_v4(), if ext.is_empty() { "".to_string() } else { format!(".{}", ext) })
        };
        let body = ByteStream::from(data.to_vec());
        self.client.put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .send()
            .await?;
        Ok(key)
    }

    /// 上传文件，使用指定的完整key（覆盖已有同名文件）
    pub async fn upload_with_key(&self, data: Bytes, key: &str) -> Result<()> {
        let body = ByteStream::from(data.to_vec());
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    /// 获取临时下载URL（签名URL）
    pub async fn get_url(&self, key: &str, expires_in_secs: i64) -> Result<String> {
        let req = self.client.get_object()
            .bucket(&self.bucket)
            .key(key);
        let presigned = req.presigned(aws_sdk_s3::presigning::PresigningConfig::expires_in(
            std::time::Duration::from_secs(expires_in_secs as u64),
        )?).await?;
        Ok(presigned.uri().to_string())
    }

    /// 删除文件
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client.delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    /// 检查文件是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match self.client.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(e) => {
                if let Some(service_err) = e.as_service_error() {
                    if service_err.is_not_found() {
                        return Ok(false);
                    }
                }
                Err(e.into())
            }
        }
    }

    /// 列出指定前缀下的所有对象
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<Object>> {
        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await?;
        Ok(resp.contents().to_vec())
    }
    pub async fn rename_file(&self, source_key: &str, target_key: &str) -> Result<()> {
        let copy_source = format!("{}/{}", self.bucket, source_key);
        self.client.copy_object()
            .bucket(&self.bucket)
            .key(target_key)
            .copy_source(copy_source)
            .send()
            .await?;
        self.delete(source_key).await?;
        Ok(())
    }
}