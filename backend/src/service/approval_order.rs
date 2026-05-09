use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use chrono::Utc;
use sea_orm::TryGetError::DbErr;
use serde::{Deserialize, Serialize};
use crate::entity::{approval_flow, approval_node, approval_order, approval_record, notification, sys_user};
use crate::error::AppError;
use crate::service::minio::MinioService;
use crate::utils::db::DbError;
use crate::utils::ws_manager::WsManager;

// ---------- DTO ----------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitApprovalDto {
    pub flow_code: String,
    pub biz_id: i64,
    pub biz_type: String,
    pub attachment_url: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveDto {
    pub order_id: i64,
    pub opinion: Option<String>,
    pub attachment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectDto {
    pub order_id: i64,
    pub opinion: Option<String>,
    pub attachment_url: Option<String>,
}

// ---------- 服务 ----------
#[derive(Clone)]
pub struct ApprovalOrderService {
    pub db: DatabaseConnection,
    pub ws_manager: WsManager,
    pub minio: MinioService,
}

impl ApprovalOrderService {
    pub fn new(db: DatabaseConnection, ws_manager: WsManager,  minio: MinioService) -> Self {
        Self { db, ws_manager, minio }
    }

    // ========== 提交审批申请 ==========
    pub async fn submit(&self, dto: SubmitApprovalDto, applicant_id: i64) -> Result<approval_order::Model, AppError> {
        let txn = self.db.begin().await?;

        // 1. 获取流程定义
        let flow = approval_flow::Entity::find()
            .filter(approval_flow::Column::Code.eq(&dto.flow_code))
            .filter(approval_flow::Column::Status.eq(1))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::BusinessError("流程不存在或已禁用".into()))?;

        // 2. 获取节点列表
        let nodes = approval_node::Entity::find()
            .filter(approval_node::Column::FlowId.eq(flow.id))
            .filter(approval_node::Column::Status.eq(1))
            .order_by_asc(approval_node::Column::NodeOrder)
            .all(&txn)
            .await?;
        if nodes.is_empty() {
            return Err(AppError::BusinessError("流程未配置节点".into()));
        }
        let first_node = &nodes[0];

        // 3. 解析第一个节点的审批人
        let approver_ids = self.resolve_approvers(&txn, first_node, applicant_id).await?;
        if approver_ids.is_empty() {
            return Err(AppError::BusinessError("无法确定审批人".into()));
        }

        // 4. 获取申请人姓名
        let applicant_name = sys_user::Entity::find_by_id(applicant_id)
            .one(&txn)
            .await?
            .and_then(|u| u.real_name.or(Some(u.username)))
            .unwrap_or_default();

        // 5. 创建审批单据
        let now = Utc::now().naive_utc();
        let order = approval_order::ActiveModel {
            id: Default::default(),
            flow_code: Set(dto.flow_code.clone()),
            biz_id: Set(dto.biz_id),
            biz_type: Set(dto.biz_type.clone()),
            applicant_id: Set(applicant_id),
            applicant_name: Set(Some(applicant_name)),
            status: Set("PENDING".to_string()),
            current_node_order: Set(Some(first_node.node_order)),
            current_approver_ids: Set(Some(approver_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))),
            attachment_url: Set(dto.attachment_url.clone()),
            submitted_time: Set(Some(now)),
            finished_time: Set(None),
            remark: Set(dto.remark.clone()),
            create_time: Set(now),
            update_time: Set(now),
        }.insert(&txn).await?;

        txn.commit().await?;

        // 发送待办通知给当前审批人
        for approver_id in approver_ids {
            self.send_notification(
                approver_id,
                &format!("待办审批: {}", flow.name),
                &format!("您有一个{}需要审批", dto.biz_type),
                "TODO",
                None,
            ).await?;
        }

        Ok(order)
    }

    // ========== 审批通过 ==========
    pub async fn approve(&self, dto: ApproveDto, approver_id: i64) -> Result<approval_order::Model, AppError> {
        let txn = self.db.begin().await?;

        let order = approval_order::Entity::find_by_id(dto.order_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::BusinessError("审批单不存在".into()))?;

        if order.status != "PENDING" {
            return Err(AppError::BusinessError("审批单状态不是进行中".into()));
        }

        // 校验当前用户是否是当前节点的审批人
        let current_ids: Vec<i64> = order.current_approver_ids
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        if !current_ids.contains(&approver_id) {
            return Err(AppError::PermissionDenied);
        }

        // 记录审批记录
        let approver_name = sys_user::Entity::find_by_id(approver_id)
            .one(&txn)
            .await?
            .and_then(|u| u.real_name.or(Some(u.username)))
            .unwrap_or_default();

        let _record = approval_record::ActiveModel {
            id: Default::default(),
            order_id: Set(order.id),
            node_order: Set(order.current_node_order.unwrap_or(0)),
            approver_id: Set(approver_id),
            approver_name: Set(Some(approver_name)),
            action: Set("APPROVE".to_string()),
            opinion: Set(dto.opinion.clone()),
            attachment_url: Set(dto.attachment_url.clone()),
            create_time: Set(Utc::now().naive_utc()),
        }.insert(&txn).await?;

        // 从当前审批人列表中移除当前用户
        let remaining_ids: Vec<i64> = current_ids.into_iter().filter(|&id| id != approver_id).collect();
        if remaining_ids.is_empty() {
            // 当前节点所有审批人都已通过 -> 推进到下一节点
            self.advance_to_next_node(&txn, order.id).await?;
            let updated_order = approval_order::Entity::find_by_id(dto.order_id)
                .one(&txn)
                .await?
                .ok_or_else(|| AppError::BusinessError("审批单不存在".into()))?;
            txn.commit().await?;

            // 通知申请人审批完成（流程已结束）
            if updated_order.status == "APPROVED" {
                self.send_notification(
                    updated_order.applicant_id,
                    "审批完成",
                    &format!("您的{}申请已通过", updated_order.biz_type),
                    "DONE",
                    None,
                ).await?;
            }
            Ok(updated_order)
        } else {
            // 会签未完成，只更新当前审批人列表
            let mut active: approval_order::ActiveModel = order.into_active_model();
            active.current_approver_ids = Set(Some(remaining_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));
            active.update_time = Set(Utc::now().naive_utc());
            let updated = active.update(&txn).await?;
            txn.commit().await?;
            Ok(updated)
        }
    }

    // ========== 审批驳回 ==========
    pub async fn reject(&self, dto: RejectDto, approver_id: i64) -> Result<approval_order::Model, AppError> {
        let txn = self.db.begin().await?;

        let order = approval_order::Entity::find_by_id(dto.order_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::BusinessError("审批单不存在".into()))?;

        if order.status != "PENDING" {
            return Err(AppError::BusinessError("审批单状态不是进行中".into()));
        }

        // 校验当前用户是否是当前节点的审批人
        let current_ids: Vec<i64> = order.current_approver_ids
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        if !current_ids.contains(&approver_id) {
            return Err(AppError::PermissionDenied);
        }

        // 记录审批记录
        let approver_name = sys_user::Entity::find_by_id(approver_id)
            .one(&txn)
            .await?
            .and_then(|u| u.real_name.or(Some(u.username)))
            .unwrap_or_default();

        let _record = approval_record::ActiveModel {
            id: Default::default(),
            order_id: Set(order.id),
            node_order: Set(order.current_node_order.unwrap_or(0)),
            approver_id: Set(approver_id),
            approver_name: Set(Some(approver_name)),
            action: Set("REJECT".to_string()),
            opinion: Set(dto.opinion.clone()),
            attachment_url: Set(dto.attachment_url.clone()),
            create_time: Set(Utc::now().naive_utc()),
        }.insert(&txn).await?;

        // 驳回：流程终止
        let mut active: approval_order::ActiveModel = order.clone().into_active_model();
        active.status = Set("REJECTED".to_string());
        active.finished_time = Set(Some(Utc::now().naive_utc()));
        active.update_time = Set(Utc::now().naive_utc());


        // 日计划审批：重命名附件为“已驳回”
        if order.flow_code == "1" { // 根据实际flow_code调整
            if let Some(ref url) = order.attachment_url {
                let new_url = url.replace("-未审批", "-已驳回");
                active.attachment_url = Set(Some(new_url.clone()));
                if let Err(e) = self.minio.rename_file(url, &new_url).await {
                    // 记录日志，不影响审批结果
                    log::error!("重命名 MinIO 文件失败: {}", e);
                }
            }
        }
        let updated = active.update(&txn).await?;
        txn.commit().await?;

        // 通知申请人驳回
        self.send_notification(
            updated.applicant_id,
            "审批驳回",
            &format!("您的{}申请已被驳回", updated.biz_type),
            "REJECT",
            None,
        ).await?;

        Ok(updated)
    }

    // ========== 推进到下一节点 ==========
    async fn advance_to_next_node<C>(&self, conn: &C, order_id: i64) -> Result<(), AppError>
    where
        C: ConnectionTrait,
    {
        let order = approval_order::Entity::find_by_id(order_id)
            .one(conn)
            .await?
            .ok_or_else(|| AppError::BusinessError("审批单不存在".into()))?;

        let current_node_order = order.current_node_order.unwrap_or(0);
        let flow = approval_flow::Entity::find()
            .filter(approval_flow::Column::Code.eq(&order.flow_code))
            .one(conn)
            .await?
            .ok_or_else(|| AppError::BusinessError("流程不存在".into()))?;

        let nodes = approval_node::Entity::find()
            .filter(approval_node::Column::FlowId.eq(flow.id))
            .filter(approval_node::Column::Status.eq(1))
            .order_by_asc(approval_node::Column::NodeOrder)
            .all(conn)
            .await?;

        let next_index = nodes.iter().position(|n| n.node_order == current_node_order).map(|i| i + 1);
        if let Some(idx) = next_index {
            if idx < nodes.len() {
                // 有下一节点
                let next_node = &nodes[idx];
                let approver_ids = self.resolve_approvers(conn, next_node, order.applicant_id).await?;
                if approver_ids.is_empty() {
                    return Err(AppError::BusinessError("下一节点无法确定审批人".into()));
                }
                let mut active: approval_order::ActiveModel = order.clone().into_active_model();
                active.current_node_order = Set(Some(next_node.node_order));
                active.current_approver_ids = Set(Some(approver_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")));
                active.update_time = Set(Utc::now().naive_utc());
                active.update(conn).await?;

                // 通知新节点审批人
                for approver_id in approver_ids {
                    self.send_notification(
                        approver_id,
                        &format!("待办审批: {}", flow.name),
                        &format!("您有一个{}需要审批", order.biz_type),
                        "TODO",
                        None,
                    ).await?;
                }
                return Ok(());
            }
        }

        // 所有节点已通过，流程完成
        let mut active: approval_order::ActiveModel = order.clone().into_active_model();
        active.status = Set("APPROVED".to_string());
        active.finished_time = Set(Some(Utc::now().naive_utc()));
        active.update_time = Set(Utc::now().naive_utc());

        // 如果是日计划审批，将附件文件名从“未审批”改为“审批通过”
        if order.flow_code == "1" {
            if let Some(url) = order.attachment_url {
                let new_url = url.clone().replace("-未审批", "-审批通过");
                active.attachment_url = Set(Some(new_url.clone()));
                if let Err(e) = self.minio.rename_file(&*url, &new_url).await {
                    // 记录日志，不影响审批结果
                    log::error!("重命名 MinIO 文件失败: {}", e);
                }
            }
        }


        active.update(conn).await?;
        Ok(())
    }

    // ========== 解析审批人（泛型连接） ==========
    async fn resolve_approvers<C>(&self, conn: &C, node: &approval_node::Model, _applicant_id: i64) -> Result<Vec<i64>, AppError>
    where
        C: ConnectionTrait,
    {
        match node.approver_type.as_str() {
            "ROLE" => {
                let role_code = node.approver_value.as_deref().unwrap_or("");
                let users = sys_user::Entity::find()
                    .filter(sys_user::Column::Role.eq(role_code))
                    .filter(sys_user::Column::Status.eq(1))
                    .all(conn)
                    .await?;
                Ok(users.into_iter().map(|u| u.id).collect())
            }
            "USER" => {
                let raw = node.approver_value.as_deref().unwrap_or("");
                let usernames: Vec<&str> = raw.split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                if usernames.is_empty() {
                    return Ok(vec![]);
                }

                let mut user_ids = Vec::new();
                for username in usernames {
                    let user = sys_user::Entity::find()
                        .filter(sys_user::Column::Username.eq(username))
                        .one(conn)   // db 需传入数据库连接
                        .await
                        .map_err(|e| DbError::BusinessError("查询用户失败".to_string()))?;
                    let user_id = user
                        .ok_or_else(|| DbError::BusinessError("用户不存在: {}".to_string()))?
                        .id;
                    user_ids.push(user_id);
                }
                Ok(user_ids)
            }
            "DEPARTMENT_HEAD" => {
                // 根据申请人查询部门负责人，简化示例返回空
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    // ========== 发送通知（数据库持久化 + WebSocket 实时推送） ==========
    async fn send_notification(
        &self,
        user_id: i64,
        title: &str,
        content: &str,
        notif_type: &str,
        attachment_url: Option<String>,
    ) -> Result<(), AppError> {
        // 1. 持久化到数据库
        let now = Utc::now().naive_utc();
        let notif = notification::ActiveModel {
            id: Default::default(),
            user_id: Set(Some(user_id)),
            title: Set(Option::from(title.to_string())),
            content: Set(Option::from(content.to_string())),
            r#type: Set(notif_type.to_string()),
            attachment_url: Set(attachment_url.clone()),
            is_read: Set(0),
            create_time: Set(now),
        };
        notif.insert(&self.db).await?;

        // 2. WebSocket 实时推送
        let ws_msg = serde_json::json!({
            "type": notif_type,
            "title": title,
            "content": content,
            "attachment_url": attachment_url,
        }).to_string();
        self.ws_manager.send_to_user(user_id, ws_msg);

        Ok(())
    }

    // ========== 查询待办列表 ==========
    pub async fn get_todo_list(&self, user_id: i64) -> Result<Vec<approval_order::Model>, AppError> {
        let orders = approval_order::Entity::find()
            .filter(approval_order::Column::Status.eq("PENDING"))
            .filter(approval_order::Column::CurrentApproverIds.like(format!("%{}%", user_id)))
            .order_by_desc(approval_order::Column::CreateTime)
            .all(&self.db)
            .await?;
        Ok(orders)
    }

    pub async fn get_my_orders(&self, user_id: i64, status: Option<&str>) -> Result<Vec<approval_order::Model>, AppError> {
        let mut query = approval_order::Entity::find()
            .filter(approval_order::Column::ApplicantId.eq(user_id));
        if let Some(st) = status {
            query = query.filter(approval_order::Column::Status.eq(st));
        }
        query.order_by_desc(approval_order::Column::CreateTime)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_approval_records(&self, order_id: i64) -> Result<Vec<approval_record::Model>, AppError> {
        approval_record::Entity::find()
            .filter(approval_record::Column::OrderId.eq(order_id))
            .order_by_asc(approval_record::Column::CreateTime)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }
}