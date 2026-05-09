use std::env;
use anyhow::Result;
use aws_sdk_s3::Client;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{Credentials, Region};

/// MinIO配置结构
#[derive(Clone, Debug)]
pub struct MinioConfig {
    pub endpoint: String,      // MinIO服务地址
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,        // 默认存储桶
    pub region: String,        // 通常为 "us-east-1"
    pub secure: bool,          // 是否使用HTTPS
}

impl MinioConfig {
    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9000".into()),
            access_key: env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into()),
            secret_key: env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".into()),
            bucket: env::var("MINIO_BUCKET").unwrap_or_else(|_| "exc".into()),
            region: env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".into()),
            secure: env::var("MINIO_SECURE").unwrap_or_else(|_| "false".parse().unwrap()).parse().unwrap(),
        }
    }

    /// 创建S3客户端
    pub async fn create_client(&self) -> Result<Client> {
        let credentials = Credentials::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            None,
            None,
            "minio",
        );
        let region = Region::new(self.region.clone());
        let endpoint = self.endpoint.clone();

        let provider = RegionProviderChain::first_try(region);
        let config = aws_config::from_env()
            .region(provider)
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);
        Ok(client)
    }
}