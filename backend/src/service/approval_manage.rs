use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::entity::{approval_flow, approval_node};
use crate::error::AppError;

// ========== DTO ==========

// 流程 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalFlowCreateDto {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalFlowUpdateDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
}

// 节点 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalNodeCreateDto {
    pub flow_id: i64,
    pub node_order: i32,
    pub node_name: String,
    pub approver_type: String,
    pub approver_value: Option<String>,
    pub approve_strategy: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalNodeUpdateDto {
    pub node_name: Option<String>,
    pub approver_type: Option<String>,
    pub approver_value: Option<String>,
    pub approve_strategy: Option<String>,
    pub status: Option<i16>,
}

// 树形节点（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: i64,
    pub name: String,
    pub r#type: String, // "flow" 或 "node"
    pub code: Option<String>,
    pub description: Option<String>,
    pub status: i16,
    pub node_order: Option<i32>,
    pub approver_type: Option<String>,
    pub approver_value: Option<String>,
    pub approve_strategy: Option<String>,
    pub children: Vec<TreeNode>,
}

// ========== 服务 ==========
#[derive(Clone)]
pub struct ApprovalManageService {
    pub db: DatabaseConnection,
}


impl ApprovalManageService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========== 流程操作 ==========
    pub async fn create_flow(&self, dto: ApprovalFlowCreateDto) -> Result<approval_flow::Model, AppError> {
        let now = Utc::now().naive_utc();
        let status = dto.status.unwrap_or(1);
        let active = approval_flow::ActiveModel {
            id: Default::default(),
            code: Set(dto.code),
            name: Set(dto.name),
            description: Set(dto.description),
            status: Set(status),
            create_time: Set(now),
            update_time: Set(now),
        };
        Ok(active.insert(&self.db).await?)
    }

    pub async fn update_flow(&self, id: i64, dto: ApprovalFlowUpdateDto) -> Result<approval_flow::Model, AppError> {
        let flow = approval_flow::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::BusinessError("流程不存在".into()))?;

        let mut active: approval_flow::ActiveModel = flow.into_active_model();
        if let Some(name) = dto.name {
            active.name = Set(name);
        }
        if let Some(desc) = dto.description {
            active.description = Set(Some(desc));
        }
        if let Some(status) = dto.status {
            active.status = Set(status);
        }
        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_flow(&self, id: i64) -> Result<(), AppError> {
        let res = approval_flow::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        if res.rows_affected == 0 {
            return Err(AppError::BusinessError("流程不存在".into()));
        }
        Ok(())
    }

    pub async fn get_flow(&self, id: i64) -> Result<Option<approval_flow::Model>, AppError> {
        Ok(approval_flow::Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn get_active_flow(&self) -> Result<Vec<approval_flow::Model>, AppError>{
        Ok(approval_flow::Entity::find().filter(approval_flow::Column::Status.eq(1)).
            all(&self.db).await?)
    }

    // ========== 节点操作 ==========
    pub async fn create_node(&self, dto: ApprovalNodeCreateDto) -> Result<approval_node::Model, AppError> {
        let now = Utc::now().naive_utc();
        let status = dto.status.unwrap_or(1);
        let active = approval_node::ActiveModel {
            id: Default::default(),
            flow_id: Set(dto.flow_id),
            node_order: Set(dto.node_order),
            node_name: Set(dto.node_name),
            approver_type: Set(dto.approver_type),
            approver_value: Set(dto.approver_value),
            approve_strategy: Set(dto.approve_strategy),
            status: Set(status),
            create_time: Set(now),
            update_time: Set(now),
        };
        Ok(active.insert(&self.db).await?)
    }

    pub async fn update_node(&self, id: i64, dto: ApprovalNodeUpdateDto) -> Result<approval_node::Model, AppError> {
        let node = approval_node::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::BusinessError("节点不存在".into()))?;

        let mut active: approval_node::ActiveModel = node.into_active_model();
        if let Some(name) = dto.node_name {
            active.node_name = Set(name);
        }
        if let Some(typ) = dto.approver_type {
            active.approver_type = Set(typ);
        }
        if let Some(val) = dto.approver_value {
            active.approver_value = Set(Some(val));
        }
        if let Some(strategy) = dto.approve_strategy {
            active.approve_strategy = Set(Option::from(strategy));
        }
        if let Some(status) = dto.status {
            active.status = Set(status);
        }
        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_node(&self, id: i64) -> Result<(), AppError> {
        let res = approval_node::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        if res.rows_affected == 0 {
            return Err(AppError::BusinessError("节点不存在".into()));
        }
        Ok(())
    }

    pub async fn get_node(&self, id: i64) -> Result<Option<approval_node::Model>, AppError> {
        Ok(approval_node::Entity::find_by_id(id).one(&self.db).await?)
    }

    // ========== 树形数据 ==========
    pub async fn get_flow_tree(&self) -> Result<Vec<TreeNode>, AppError> {
        // 获取所有流程
        let flows = approval_flow::Entity::find()
            .order_by_asc(approval_flow::Column::Id)
            .all(&self.db)
            .await?;

        let mut tree = Vec::new();
        for flow in flows {
            // 获取该流程下的节点
            let nodes = approval_node::Entity::find()
                .filter(approval_node::Column::FlowId.eq(flow.id))
                .order_by_asc(approval_node::Column::NodeOrder)
                .all(&self.db)
                .await?;

            let children: Vec<TreeNode> = nodes.into_iter().map(|node| TreeNode {
                id: node.id,
                name: node.node_name,
                r#type: "node".to_string(),
                code: None,
                description: None,
                status: node.status,
                node_order: Some(node.node_order),
                approver_type: Some(node.approver_type),
                approver_value: node.approver_value,
                approve_strategy: node.approve_strategy,
                children: vec![],
            }).collect();

            tree.push(TreeNode {
                id: flow.id,
                name: flow.name,
                r#type: "flow".to_string(),
                code: Some(flow.code),
                description: flow.description,
                status: flow.status,
                node_order: None,
                approver_type: None,
                approver_value: None,
                approve_strategy: None,
                children,
            });
        }
        Ok(tree)
    }

    // ========== 获取单个流程的树（可选） ==========
    pub async fn get_flow_tree_by_id(&self, flow_id: i64) -> Result<Option<TreeNode>, AppError> {
        let flow = approval_flow::Entity::find_by_id(flow_id)
            .one(&self.db)
            .await?;
        if let Some(flow) = flow {
            let nodes = approval_node::Entity::find()
                .filter(approval_node::Column::FlowId.eq(flow_id))
                .order_by_asc(approval_node::Column::NodeOrder)
                .all(&self.db)
                .await?;

            let children: Vec<TreeNode> = nodes.into_iter().map(|node| TreeNode {
                id: node.id,
                name: node.node_name,
                r#type: "node".to_string(),
                code: None,
                description: None,
                status: node.status,
                node_order: Some(node.node_order),
                approver_type: Some(node.approver_type),
                approver_value: node.approver_value,
                approve_strategy: node.approve_strategy,
                children: vec![],
            }).collect();

            Ok(Some(TreeNode {
                id: flow.id,
                name: flow.name,
                r#type: "flow".to_string(),
                code: Some(flow.code),
                description: flow.description,
                status: flow.status,
                node_order: None,
                approver_type: None,
                approver_value: None,
                approve_strategy: None,
                children,
            }))
        } else {
            Ok(None)
        }
    }
}