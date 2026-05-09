use std::collections::HashSet;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::entity::{sys_menu, sys_role_menu};
use crate::utils::db::{DbError, DbResult};

// ---------- DTO ----------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMenuCreateDto {
    pub name: String,
    pub parent_id: Option<i64>,
    pub order_num: Option<i32>,
    pub path: Option<String>,
    pub query_params: Option<String>,
    pub menu_type: i16,
    pub permission: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMenuUpdateDto {
    pub name: Option<String>,
    pub parent_id: Option<i64>,
    pub order_num: Option<i32>,
    pub path: Option<String>,
    pub query_params: Option<String>,
    pub menu_type: Option<i16>,
    pub permission: Option<String>,
    pub remark: Option<String>,
}

// ---------- 树节点 ----------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysMenuTreeNode {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub order_num: i32,
    pub path: String,
    pub query_params: String,
    pub menu_type: i16,
    pub permission: String,
    pub remark: String,
    pub children: Vec<SysMenuTreeNode>,
}

// ---------- 服务 ----------
#[derive(Clone)]
pub struct SysMenuService(pub DatabaseConnection);

impl SysMenuService {
    /// 创建菜单
    pub async fn create(&self, dto: SysMenuCreateDto) -> DbResult<sys_menu::Model> {
        let now = Utc::now();
        let order_num = dto.order_num.unwrap_or(0);
        let path = dto.path.unwrap_or_default();
        let query_params = dto.query_params.unwrap_or_default();
        let permission = dto.permission.unwrap_or_default();
        let remark = dto.remark.unwrap_or_default();

        // 如果 parent_id 有值，需检查是否存在（外键会自动校验）
        let active = sys_menu::ActiveModel {
            id: Default::default(),
            name: Set(dto.name),
            parent_id: Set(dto.parent_id),
            order_num: Set(order_num),
            path: Set(path),
            query_params: Set(query_params),
            menu_type: Set(dto.menu_type),
            permission: Set(permission),

            create_by: Default::default(),
            create_time: Set(Option::from(now.naive_utc())),
            update_by: Default::default(),
            update_time: Set(Option::from(now.naive_utc())),
            remark: Set(Option::from(remark)),
        };
        Ok(active.insert(&self.0).await?)
    }

    /// 更新菜单
    pub async fn update(
        &self,
        id: i64,
        dto: SysMenuUpdateDto,
    ) -> DbResult<sys_menu::Model> {
        let menu = sys_menu::Entity::find_by_id(id)
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active: sys_menu::ActiveModel = menu.into_active_model();

        if let Some(name) = dto.name {
            active.name = Set(name);
        }

        active.parent_id = Set(None);
        if let Some(parent_id) = dto.parent_id {
            // // 防止循环引用：不能将自己或子孙设置为父级（前端）
            // if parent_id == id {
            //     return Err(DbError::InvalidParameter("不能将菜单设置为自己为父级".into()));
            // }
            active.parent_id = Set(Some(parent_id));
        }
        if let Some(order_num) = dto.order_num {
            active.order_num = Set(order_num);
        }
        if let Some(path) = dto.path {
            active.path = Set(path);
        }
        if let Some(query_params) = dto.query_params {
            active.query_params = Set(query_params);
        }
        if let Some(menu_type) = dto.menu_type {
            active.menu_type = Set(menu_type);
        }
        if let Some(permission) = dto.permission {
            active.permission = Set(permission);
        }
        if let Some(remark) = dto.remark {
            active.remark = Set(Option::from(remark));
        }

        //active.update_by = Set(operator_id);
        active.update_time = Set(Option::from(Utc::now().naive_utc()));

        Ok(active.update(&self.0).await?)
    }

    /// 删除菜单（物理删除）
    pub async fn delete(&self, id: i64) -> DbResult<()> {
        // 检查是否有子菜单
        let children = sys_menu::Entity::find()
            .filter(sys_menu::Column::ParentId.eq(id))
            .all(&self.0)
            .await?;
        if !children.is_empty() {
            return Err(DbError::InvalidParameter("请先删除子菜单".to_string()));
        }
        let res = sys_menu::Entity::delete_by_id(id)
            .exec(&self.0)
            .await?;
        if res.rows_affected == 0 {
            return Err(DbError::RecordNotFound);
        }
        Ok(())
    }

    /// 获取所有菜单（扁平列表）
    pub async fn list_all(&self) -> DbResult<Vec<sys_menu::Model>> {
        sys_menu::Entity::find()
            .order_by_asc(sys_menu::Column::ParentId)
            .order_by_asc(sys_menu::Column::OrderNum)
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 获取菜单树
    pub async fn tree(&self) -> DbResult<Vec<SysMenuTreeNode>> {
        let menus = self.list_all().await?;
        let mut roots = Vec::new();

        // 构建 id -> menu 映射
        let map: std::collections::HashMap<i64, &sys_menu::Model> = menus.iter().map(|m| (m.id, m)).collect();

        // 递归构建树
        fn build_node(menu: &sys_menu::Model, all: &[sys_menu::Model]) -> SysMenuTreeNode {
            let children: Vec<SysMenuTreeNode> = all
                .iter()
                .filter(|m| m.parent_id == Some(menu.id))
                .map(|m| build_node(m, all))
                .collect();
            SysMenuTreeNode {
                id: menu.id,
                name: menu.name.clone(),
                parent_id: menu.parent_id,
                order_num: menu.order_num,
                path: menu.path.clone(),
                query_params: menu.query_params.clone(),
                menu_type: menu.menu_type,
                permission: menu.permission.clone(),
                remark: menu.remark.clone().unwrap().clone(),
                children,
            }
        }

        for menu in &menus {
            if menu.parent_id.is_none() {
                roots.push(build_node(menu, &menus));
            }
        }
        Ok(roots)
    }

    /// 根据 ID 查询菜单
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<sys_menu::Model>> {
        sys_menu::Entity::find_by_id(id).one(&self.0).await.map_err(Into::into)
    }

    /// 批量创建按钮权限（用于初始化）
    pub async fn batch_create_permissions(
        &self,
        items: Vec<(String, String, i64)>, // (名称, 权限标识, 父菜单ID)
        operator_id: i64,
    ) -> DbResult<Vec<sys_menu::Model>> {
        let mut result = Vec::new();
        for (name, permission, parent_id) in items {
            let dto = SysMenuCreateDto {
                name,
                parent_id: Some(parent_id),
                order_num: Some(0),
                path: None,
                query_params: None,
                menu_type: 2, // 按钮
                permission: Some(permission),
                remark: None,
            };
            let menu = self.create(dto).await?;
            result.push(menu);
        }
        Ok(result)
    }


    /// 根据角色编码获取菜单列表（扁平，按 order_num 排序）
    pub async fn get_menus_by_role(&self, role_code: &str) -> DbResult<Vec<sys_menu::Model>> {
        // 1. 查询该角色拥有的菜单ID
        let role_menus = sys_role_menu::Entity::find()
            .filter(sys_role_menu::Column::Role.eq(role_code))
            .one(&self.0)
            .await?;
        let menu_ids: Vec<i64> = role_menus.unwrap().menu_ids.unwrap();
        if menu_ids.is_empty() {
            return Ok(vec![]);
        }

        // 2. 查询菜单详情，并排序
        sys_menu::Entity::find()
            .filter(sys_menu::Column::Id.is_in(menu_ids))
            .order_by_asc(sys_menu::Column::OrderNum)
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 根据角色编码获取菜单树（自动补全缺失的父菜单，保证树结构完整）
    pub async fn get_menu_tree_by_role(&self, role_code: &str) -> DbResult<Vec<SysMenuTreeNode>> {
        // 1. 获取角色直接拥有的菜单ID
        let role_menus = sys_role_menu::Entity::find()
            .filter(sys_role_menu::Column::Role.eq(role_code))
            .one(&self.0)
            .await?;
        let mut target_ids: HashSet<i64> = role_menus
            .into_iter()
            .flat_map(|rm| rm.menu_ids.unwrap())
            .collect();
        if target_ids.is_empty() {
            return Ok(vec![]);
        }

        // 2. 递归收集所有祖先ID（补全父菜单）
        let mut all_ids = target_ids.clone();
        let mut stack: Vec<i64> = target_ids.iter().cloned().collect();
        while let Some(id) = stack.pop() {
            // 查询该菜单的父ID
            let menu = sys_menu::Entity::find_by_id(id)
                .one(&self.0)
                .await?;
            if let Some(m) = menu {
                if let Some(parent_id) = m.parent_id {
                    if !all_ids.contains(&parent_id) {
                        all_ids.insert(parent_id);
                        stack.push(parent_id);
                    }
                }
            }
        }

        // 3. 获取所有需要显示的菜单（包含补全的父菜单）
        let menus = sys_menu::Entity::find()
            .filter(sys_menu::Column::Id.is_in(all_ids))
            .order_by_asc(sys_menu::Column::OrderNum)
            .all(&self.0)
            .await?;

        // 4. 构建菜单树（只保留有权限的菜单，但父菜单已补全）
        let map: std::collections::HashMap<i64, &sys_menu::Model> = menus.iter().map(|m| (m.id, m)).collect();

        fn build_node(menu: &sys_menu::Model, all: &[sys_menu::Model]) -> SysMenuTreeNode {
            let children: Vec<SysMenuTreeNode> = all
                .iter()
                .filter(|m| m.parent_id == Some(menu.id))
                .map(|m| build_node(m, all))
                .collect();
            SysMenuTreeNode {
                id: menu.id,
                name: menu.name.clone(),
                parent_id: menu.parent_id,
                order_num: menu.order_num,
                path: menu.path.clone(),
                query_params: menu.query_params.clone(),
                menu_type: menu.menu_type,
                permission: menu.permission.clone(),
                remark: menu.remark.clone().unwrap().clone(),
                children,
            }
        }

        let roots: Vec<SysMenuTreeNode> = menus
            .iter()
            .filter(|m| m.parent_id.is_none())
            .map(|m| build_node(m, &menus))
            .collect();
        Ok(roots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection, EntityTrait};
    use serial_test::serial;
    use std::env;

    async fn get_db() -> DatabaseConnection {
        dotenv().ok();
        let url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL not set");
        Database::connect(&url).await.unwrap()
    }

    async fn service() -> SysMenuService {
        SysMenuService(get_db().await)
    }

    async fn clean_db(db: &DatabaseConnection) {
        let _ = sys_role_menu::Entity::delete_many().exec(db).await;
        let _ = sys_menu::Entity::delete_many().exec(db).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_menu() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let dto = SysMenuCreateDto {
            name: "测试菜单".to_string(),
            parent_id: None,
            order_num: Some(1),
            path: Some("/test".to_string()),
            query_params: None,
            menu_type: 1,
            permission: None,
            remark: Some("测试".to_string()),
        };
        let menu = svc.create(dto).await.unwrap();
        assert_eq!(menu.name, "测试菜单");
        assert_eq!(menu.parent_id, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_tree() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let parent = svc.create(SysMenuCreateDto {
            name: "父菜单".to_string(),
            parent_id: None,
            order_num: Some(1),
            path: Some("/parent".to_string()),
            query_params: None,
            menu_type: 0,
            permission: None,
            remark: None,
        }).await.unwrap();

        svc.create(SysMenuCreateDto {
            name: "子菜单".to_string(),
            parent_id: Some(parent.id),
            order_num: Some(1),
            path: Some("/child".to_string()),
            query_params: None,
            menu_type: 1,
            permission: None,
            remark: None,
        }).await.unwrap();

        let tree = svc.tree().await.unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
    }

    async fn insert_menu_data(svc: &SysMenuService) -> (sys_menu::Model, sys_menu::Model, sys_menu::Model, sys_menu::Model) {
        let top = svc.create(SysMenuCreateDto {
            name: "系统管理".to_string(),
            parent_id: None,
            order_num: Some(1),
            path: Some("/system".to_string()),
            query_params: None,
            menu_type: 0,
            permission: None,
            remark: Some("顶级目录".to_string()),
        }).await.unwrap();

        let user_mgr = svc.create(SysMenuCreateDto {
            name: "用户管理".to_string(),
            parent_id: Some(top.id),
            order_num: Some(1),
            path: Some("/system/user".to_string()),
            query_params: None,
            menu_type: 1,
            permission: None,
            remark: Some("用户管理菜单".to_string()),
        }).await.unwrap();

        let btn_create = svc.create(SysMenuCreateDto {
            name: "新增用户".to_string(),
            parent_id: Some(user_mgr.id),
            order_num: Some(1),
            path: None,
            query_params: None,
            menu_type: 2,
            permission: Some("USER:CREATE".to_string()),
            remark: Some("创建用户按钮".to_string()),
        }).await.unwrap();

        let btn_update = svc.create(SysMenuCreateDto {
            name: "更新用户".to_string(),
            parent_id: Some(user_mgr.id),
            order_num: Some(2),
            path: None,
            query_params: None,
            menu_type: 2,
            permission: Some("USER:UPDATE".to_string()),
            remark: Some("更新用户按钮".to_string()),
        }).await.unwrap();

        (top, user_mgr, btn_create, btn_update)
    }

    #[tokio::test]
    #[serial]
    async fn test_get_menus_by_role() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let (top, user_mgr, btn_create, btn_update) = insert_menu_data(&svc).await;

        let role_code = "admin";

        // ==============================
        // 🔥 修复：使用数组 menu_ids
        // ==============================
        let role_menu = sys_role_menu::ActiveModel {
            id: Default::default(),
            role: Set(role_code.to_string()),
            menu_ids: Set(vec![btn_create.id, btn_update.id]), // ✅ 数组
            create_time: Set(Utc::now().naive_utc()),
            update_time: Set(Utc::now().naive_utc()),
        };
        role_menu.insert(&svc.0).await.unwrap();

        // 查询角色菜单
        let menus = svc.get_menus_by_role(role_code).await.unwrap();
        assert_eq!(menus.len(), 2);
        let ids: Vec<i64> = menus.iter().map(|m| m.id).collect();
        assert!(ids.contains(&btn_create.id));
        assert!(ids.contains(&btn_update.id));
        assert!(!ids.contains(&user_mgr.id));
        assert!(!ids.contains(&top.id));

        let empty = svc.get_menus_by_role("nonexist").await.unwrap();
        assert!(empty.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_menu_tree_by_role() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let (top, user_mgr, btn_create, btn_update) = insert_menu_data(&svc).await;

        let role_code = "admin";

        // ==============================
        // 🔥 修复：使用数组 menu_ids
        // ==============================
        let role_menu = sys_role_menu::ActiveModel {
            id: Default::default(),
            role: Set(role_code.to_string()),
            menu_ids: Set(vec![btn_create.id]), // ✅ 数组
            create_time: Set(Utc::now().naive_utc()),
            update_time: Set(Utc::now().naive_utc()),
        };
        role_menu.insert(&svc.0).await.unwrap();

        let tree = svc.get_menu_tree_by_role(role_code).await.unwrap();
        assert_eq!(tree.len(), 1);
        let top_node = &tree[0];
        assert_eq!(top_node.id, top.id);
        assert_eq!(top_node.children.len(), 1);

        let user_node = &top_node.children[0];
        assert_eq!(user_node.id, user_mgr.id);
        assert_eq!(user_node.children.len(), 1);

        let btn_node = &user_node.children[0];
        assert_eq!(btn_node.id, btn_create.id);

        // ==============================
        // 🔥 修复：更新为数组
        // ==============================
        let mut role = sys_role_menu::Entity::find()
            .filter(sys_role_menu::Column::Role.eq(role_code))
            .one(&svc.0)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();

        role.menu_ids = Set(vec![btn_create.id, btn_update.id]);
        role.update(&svc.0).await.unwrap();

        let tree2 = svc.get_menu_tree_by_role(role_code).await.unwrap();
        assert_eq!(tree2.len(), 1);
        let user_node2 = &tree2[0].children[0];
        assert_eq!(user_node2.children.len(), 2);
    }
}