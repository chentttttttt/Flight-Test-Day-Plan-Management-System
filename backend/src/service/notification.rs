//! 通知服务模块（包含 DTO 与业务逻辑）
//! 支持向单个用户发送通知，或向某个角色下所有用户广播通知

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,PaginatorTrait,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::entity::{notification, sys_user};
use crate::utils::db::{DbError, DbResult};

// ========== DTO ==========

/// 创建通知请求
#[derive(Debug, Deserialize)]
pub struct CreateNotificationReq {
    /// 若指定 role，则向该角色所有用户发送通知（此时 user_id 可忽略）
    pub role: Option<String>,
    /// 若未指定 role，则向指定用户发送
    pub user_id: Option<i64>,
    pub title: String,
    pub content: String,
    /// 通知类型：SYSTEM, APPROVAL, TASK 等
    pub r#type: String,
    pub attachment_url: Option<String>,
}

/// 查询通知请求（分页、过滤）
#[derive(Debug, Deserialize)]
pub struct QueryNotificationReq {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub is_read: Option<i16>,   // 0:未读 1:已读
    pub r#type: Option<String>,
}

/// 批量标记已读请求
#[derive(Debug, Deserialize)]
pub struct MarkReadReq {
    pub notification_ids: Vec<i64>,
}

/// 通知响应体
#[derive(Debug, Serialize)]
pub struct NotificationResp {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub r#type: String,
    pub attachment_url: Option<String>,
    pub is_read: i16,
    pub create_time: chrono::NaiveDateTime,
}

/// 通知列表响应
#[derive(Debug, Serialize)]
pub struct NotificationListResp {
    pub list: Vec<NotificationResp>,
    pub total: u64,
}

// ========== Service ==========

#[derive(Clone)]
pub struct NotificationService(pub DatabaseConnection);

impl From<notification::Model> for NotificationResp {
    fn from(m: notification::Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id.unwrap_or(0),
            title: m.title.unwrap_or_default(),
            content: m.content.unwrap_or_default(),
            r#type: m.r#type,
            attachment_url: m.attachment_url,
            is_read: m.is_read,
            create_time: m.create_time,
        }
    }
}

impl NotificationService {
    /// 根据角色获取所有有效用户ID（未删除、启用）
    async fn get_user_ids_by_role(&self, role: &str) -> DbResult<Vec<i64>> {
        let users = sys_user::Entity::find()
            .filter(sys_user::Column::Role.eq(role))
            .filter(sys_user::Column::Deleted.eq(0))
            .filter(sys_user::Column::Status.eq(1))
            .all(&self.0)
            .await?;
        Ok(users.into_iter().map(|u| u.id).collect())
    }

    /// 创建通知（支持按角色广播）
    pub async fn create(&self, req: CreateNotificationReq) -> DbResult<Vec<notification::Model>> {
        let target_user_ids = if let Some(role) = req.role {
            let ids = self.get_user_ids_by_role(&role).await?;
            if ids.is_empty() {
                return Err(DbError::InvalidParameter(format!("角色 {} 下无有效用户", role)));
            }
            ids
        } else if let Some(user_id) = req.user_id {
            vec![user_id]
        } else {
            return Err(DbError::InvalidParameter("必须指定 role 或 user_id".to_string()));
        };

        let now = Utc::now().naive_utc();
        let txn = self.0.begin().await?;

        let mut created = Vec::new();
        for uid in target_user_ids {
            let active = notification::ActiveModel {
                id: Default::default(),
                user_id: Set(Some(uid)),
                title: Set(Some(req.title.clone())),
                content: Set(Some(req.content.clone())),
                r#type: Set(req.r#type.clone()),
                attachment_url: Set(req.attachment_url.clone()),
                is_read: Set(0i16),
                create_time: Set(now),
            };
            let model = active.insert(&txn).await?;
            created.push(model);
        }

        txn.commit().await?;
        Ok(created)
    }

    /// 获取用户通知列表（分页、过滤）
    pub async fn list_by_user(
        &self,
        user_id: i64,
        req: QueryNotificationReq,
    ) -> DbResult<(Vec<notification::Model>, u64)> {
        let page = req.page.unwrap_or(1);
        let page_size = req.page_size.unwrap_or(20);
        let mut query = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id));

        if let Some(is_read) = req.is_read {
            query = query.filter(notification::Column::IsRead.eq(is_read));
        }
        if let Some(typ) = req.r#type {
            query = query.filter(notification::Column::Type.eq(typ));
        }

        let total = query.clone().count(&self.0).await?;
        let list = query
            .order_by_desc(notification::Column::CreateTime)
            .offset(((page - 1) * page_size) as u64)
            .limit(page_size as u64)
            .all(&self.0)
            .await?;
        Ok((list, total))
    }

    /// 标记通知为已读（批量，仅限本人）
    pub async fn mark_as_read(&self, user_id: i64, ids: Vec<i64>) -> DbResult<()> {
        for id in ids {
            let record = notification::Entity::find_by_id(id)
                .one(&self.0)
                .await?;
            if let Some(mut model) = record {
                if model.user_id == Some(user_id) {
                    let mut active = model.into_active_model();
                    active.is_read = Set(1i16);
                    active.update(&self.0).await?;
                }
            }
        }
        Ok(())
    }

    /// 删除通知（仅限本人，使用 delete_by_id）
    pub async fn delete(&self, user_id: i64, id: i64) -> DbResult<bool> {
        let record = notification::Entity::find_by_id(id)
            .one(&self.0)
            .await?;
        if let Some(model) = record {
            if model.user_id == Some(user_id) {
                // 使用 Entity::delete_by_id 删除，避免 Model 的 delete 方法编译问题
                notification::Entity::delete_by_id(id)
                    .exec(&self.0)
                    .await?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 获取用户未读通知数量
    pub async fn unread_count(&self, user_id: i64) -> DbResult<u64> {
        let count = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::IsRead.eq(0i16))
            .count(&self.0)
            .await?;
        Ok(count)
    }
}