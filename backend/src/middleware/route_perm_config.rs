use actix_web::http::Method;
use regex::Regex;

/// 路由 → 权限映射配置（单独文件管理）
pub fn get_route_permission(
    method: &Method,
    path: &str
) -> (String, String) {
    let m = method.as_str();
    println!("{:#?}, {:#?}", m, path);
    let normalized = normalize_path(path);
    println!("{:#?}, {:#?}", m, normalized.clone());
    match (m, normalized.as_str()) {
        // ======================
        // 用户管理
        // ======================
        ("POST",   "/api/user/create")  => ("USER".into(),    "CREATE".into()),
        ("POST",   "/api/user/login")   => ("USER".into(),    "LOGIN".into()),
        ("GET",   "/api/user/info")   => ("USER".into(),    "VIEW".into()),
        ("GET",   "/api/user/{id}")   => ("USER".into(),    "VIEW".into()),

        ("PUT",    "/api/user/update")   => ("USER".into(),    "UPDATE".into()),
        ("PUT",    "/api/user/update-self")   => ("USER".into(),    "UPDATE".into()),
        ("DELETE",    "/api/user/delete/{id}")   => ("USER".into(),    "DELETE".into()),
        ("POST",   "/api/user/list")   => ("USER".into(),    "VIEW".into()),
        ("POST",   "/api/user")         => ("USER".into(),    "CREATE".into()),
        ("PUT",    "/api/user")         => ("USER".into(),    "UPDATE".into()),

        // ======================
        // ABAC 策略规则
        // ======================
        ("POST",   "/api/rule")             => ("RULE".into(),  "CREATE".into()),
        ("GET",    "/api/rule/list")            => ("RULE".into(),  "VIEW".into()),
        ("GET",    "/api/rule/query")      => ("RULE".into(),  "QUERY".into()),  // 条件查询
        ("PUT",    "/api/rule/{id}")        => ("RULE".into(),  "UPDATE".into()),
        ("DELETE", "/api/rule/{id}")        => ("RULE".into(),  "DELETE".into()),
        ("POST",   "/api/rule/find")        => ("RULE".into(),  "FIND".into()),

        // ======================
        // 主体属性管理
        // ======================
        ("POST",   "/api/subject-attribute")                         => ("SUBJECT_ATTRIBUTE".into(), "CREATE".into()),
        ("GET",    "/api/subject-attribute")                         => ("SUBJECT_ATTRIBUTE".into(), "LIST".into()),
        ("GET",    "/api/subject-attribute/{subject_type}")          => ("SUBJECT_ATTRIBUTE".into(), "VIEW".into()),
        ("GET",    "/api/subject-attribute/{subject_type}/{id}")     => ("SUBJECT_ATTRIBUTE".into(), "VIEW".into()),
        ("DELETE", "/api/subject-attribute/attr/{id}")                    => ("SUBJECT_ATTRIBUTE".into(), "DELETE".into()),
        ("DELETE", "/api/subject-attribute/{id}")                    => ("SUBJECT_ATTRIBUTE".into(), "DELETE".into()),
        ("GET",    "/api/subject-attribute/{subject_type}/{id}/{key}") => ("SUBJECT_ATTRIBUTE".into(), "VIEW".into()),
        ("DELETE", "/api/subject-attribute/{subject_type}/{id}/{key}") => ("SUBJECT_ATTRIBUTE".into(), "DELETE".into()),


        // ======================
        // 资源属性管理
        // ======================
        ("POST",   "/api/resource-attribute")                                      => ("RESOURCE_ATTRIBUTE".into(), "CREATE".into()),
        ("GET",    "/api/resource-attribute")                                      => ("RESOURCE_ATTRIBUTE".into(), "LIST".into()),
        ("GET",    "/api/resource-attribute/type/{resource_type}")                => ("RESOURCE_ATTRIBUTE".into(), "VIEW".into()),
        ("GET",    "/api/resource-attribute/record/{resource_type}/{resource_id}") => ("RESOURCE_ATTRIBUTE".into(), "VIEW".into()),
        ("GET",    "/api/resource-attribute/{resource_type}/{resource_id}")       => ("RESOURCE_ATTRIBUTE".into(), "VIEW".into()),
        ("DELETE", "/api/resource-attribute/res/{resource_type}/{resource_id}")       => ("RESOURCE_ATTRIBUTE".into(), "DELETE".into()),
        ("DELETE", "/api/resource-attribute/{resource_type}/{resource_id}")       => ("RESOURCE_ATTRIBUTE".into(), "DELETE".into()),
        ("GET",    "/api/resource-attribute/{resource_type}/{resource_id}/{key}") => ("RESOURCE_ATTRIBUTE".into(), "VIEW".into()),
        ("POST",   "/api/resource-attribute/{resource_type}/{resource_id}/{key}") => ("RESOURCE_ATTRIBUTE".into(), "UPDATE".into()),
        ("DELETE", "/api/resource-attribute/{resource_type}/{resource_id}/{key}") => ("RESOURCE_ATTRIBUTE".into(), "DELETE".into()),


        // ======================
        // 审批流程管理
        // ======================
        ("GET",    "/api/approval-manage/tree")          => ("APPROVAL_MANAGE".into(), "TREE".into()),
        ("GET",    "/api/approval-manage/tree/{id}")     => ("APPROVAL_MANAGE".into(), "VIEW".into()),
        ("GET",    "/api/approval-manage/active")          => ("APPROVAL_MANAGE".into(), "VIEW".into()),
        ("POST",   "/api/approval-manage/flow")          => ("APPROVAL_MANAGE".into(), "CREATE".into()),
        ("PUT",    "/api/approval-manage/flow/{id}")     => ("APPROVAL_MANAGE".into(), "UPDATE".into()),
        ("DELETE", "/api/approval-manage/flow/{id}")     => ("APPROVAL_MANAGE".into(), "DELETE".into()),
        ("GET",    "/api/approval-manage/flow/{id}")     => ("APPROVAL_MANAGE".into(), "VIEW".into()),
        ("POST",   "/api/approval-manage/node")          => ("APPROVAL_MANAGE".into(), "CREATE".into()),
        ("PUT",    "/api/approval-manage/node/{id}")     => ("APPROVAL_MANAGE".into(), "UPDATE".into()),
        ("DELETE", "/api/approval-manage/node/{id}")     => ("APPROVAL_MANAGE".into(), "DELETE".into()),
        ("GET",    "/api/approval-manage/node/{id}")     => ("APPROVAL_MANAGE".into(), "VIEW".into()),


        // ======================
        // 审批单据管理
        // ======================
        ("POST",   "/api/approval/submit")      => ("APPROVAL".into(), "SUBMIT".into()),
        ("POST",   "/api/approval/approve")     => ("APPROVAL".into(), "APPROVE".into()),
        ("POST",   "/api/approval/reject")      => ("APPROVAL".into(), "REJECT".into()),
        ("GET",    "/api/approval/todo")        => ("APPROVAL".into(), "TODO".into()),
        ("GET",    "/api/approval/my-orders")   => ("APPROVAL".into(), "MY_ORDERS".into()),
        ("GET",    "/api/approval/records/{id}") => ("APPROVAL".into(), "RECORDS".into()),
        // ======================
        // 飞行计划
        // ======================
        ("GET",    "/api/flight/plan")  => ("FLIGHT_PLAN".into(),  "QUERY".into()),
        ("POST",   "/api/flight/plan")  => ("FLIGHT_PLAN".into(),  "CREATE".into()),
        ("PUT",    "/api/flight/plan")  => ("FLIGHT_PLAN".into(),  "UPDATE".into()),
        ("DELETE", "/api/flight/plan")  => ("FLIGHT_PLAN".into(),  "DELETE".into()),


        // ======================
        // 菜单管理
        // ======================
        ("GET",    "/api/menu/tree")                    => ("MENU".into(),  "TREE".into()),
        ("GET",    "/api/menu/list")                    => ("MENU".into(),  "LIST".into()),
        ("POST",   "/api/menu")                         => ("MENU".into(),  "CREATE".into()),
        ("PUT",    "/api/menu/{id}")                    => ("MENU".into(),  "UPDATE".into()),
        ("DELETE", "/api/menu/{id}")                    => ("MENU".into(),  "DELETE".into()),
        ("GET",    "/api/menu/role/tree/{role}")   => ("MENU".into(),  "ROLE_TREE".into()),
        ("GET",    "/api/menu/role/list/{role}")   => ("MENU".into(),  "ROLE_LIST".into()),

        // ======================
        // 角色管理
        // ======================
        ("GET",    "/api/role/list")      => ("ROLE".into(), "LIST".into()),
        ("GET",    "/api/role/{id}")      => ("ROLE".into(), "VIEW".into()),
        ("POST",   "/api/role")           => ("ROLE".into(), "CREATE".into()),
        ("PUT",    "/api/role/{id}")      => ("ROLE".into(), "UPDATE".into()),
        ("DELETE", "/api/role/{id}")      => ("ROLE".into(), "DELETE".into()),
        ("GET",    "/api/role/all")       => ("ROLE".into(), "ALL".into()),

        // ======================
        // 角色菜单关联
        // ======================
        ("POST",   "/api/role-menu")                         => ("ROLE_MENU".into(), "CREATE".into()),
        ("PUT",    "/api/role-menu/{id}")                    => ("ROLE_MENU".into(), "UPDATE".into()),
        ("DELETE", "/api/role-menu/{id}")                    => ("ROLE_MENU".into(), "DELETE".into()),
        ("GET",    "/api/role-menu/role/menus/{role}")       => ("ROLE_MENU".into(), "QUERY".into()),
        ("POST",   "/api/role-menu/role/grant/{role}")       => ("ROLE_MENU".into(), "GRANT".into()),
        ("POST",   "/api/role-menu/role/replace/{role}")     => ("ROLE_MENU".into(), "REPLACE".into()),

        // ======================
        // 通知管理
        // ======================
        ("POST",   "/api/notification")                         => ("NOTIFICATION".into(), "CREATE".into()),
        ("POST",    "/api/notification/list")                    => ("NOTIFICATION".into(), "QUERY".into()),
        ("POST",    "/api/notification/list/{id}")          => ("NOTIFICATION".into(), "QUERY".into()),
        ("POST",   "/api/notification/mark-read")               => ("NOTIFICATION".into(), "MARK_READ".into()),
        ("DELETE", "/api/notification/{id}")                    => ("NOTIFICATION".into(), "DELETE".into()),
        ("GET",    "/api/notification/unread-count")            => ("NOTIFICATION".into(), "UNREAD_COUNT".into()),

        ("POST", "/api/milp/schedule")           => ("MILP".into(), "SCHEDULE".into()),

        // 未配置的路由 → 权限未知（默认拒绝）
        _ => ("UNKNOWN".into(), "UNKNOWN".into()),
    }
}


fn normalize_path(path: &str) -> String {
    let mut path = path.to_string();

    // ==============================================
    // 1. 主体属性
    // ==============================================
    let re_subject_attr_id = Regex::new(r"/subject-attribute/attr/(\d+)$").unwrap();
    path = re_subject_attr_id.replace_all(&path, "/subject-attribute/attr/{id}").to_string();

    let re_subject_attr_key = Regex::new(r"/subject-attribute/([a-zA-Z0-9_]+)/(\d+)/([a-zA-Z0-9_]+)$").unwrap();
    path = re_subject_attr_key.replace_all(&path, "/subject-attribute/{subject_type}/{id}/{key}").to_string();

    let re_subject_attr_both = Regex::new(r"/subject-attribute/([a-zA-Z0-9_]+)/(\d+)$").unwrap();
    path = re_subject_attr_both.replace_all(&path, "/subject-attribute/{subject_type}/{id}").to_string();

    let re_subject_id = Regex::new(r"/subject-attribute/(\d+)$").unwrap();
    path = re_subject_id.replace_all(&path, "/subject-attribute/{id}").to_string();

    let re_subject_type = Regex::new(r"/subject-attribute/([a-zA-Z_]+)$").unwrap();
    path = re_subject_type.replace_all(&path, "/subject-attribute/{subject_type}").to_string();

    // ==============================================
    // 2. 资源属性
    // ==============================================
    // /resource-attribute/xxx/123/key
    let re_resource_key = Regex::new(r"/resource-attribute/([a-zA-Z0-9_]+)/(\d+)/([a-zA-Z0-9_]+)$").unwrap();
    path = re_resource_key.replace_all(&path, "/resource-attribute/{resource_type}/{resource_id}/{key}").to_string();

    // /resource-attribute/xxx/123
    let re_resource_both = Regex::new(r"/resource-attribute/([a-zA-Z0-9_]+)/(\d+)$").unwrap();
    path = re_resource_both.replace_all(&path, "/resource-attribute/{resource_type}/{resource_id}").to_string();

    // /resource-attribute/type/xxx
    let re_resource_type = Regex::new(r"/resource-attribute/type/([a-zA-Z0-9_]+)$").unwrap();
    path = re_resource_type.replace_all(&path, "/resource-attribute/type/{resource_type}").to_string();

    // /resource-attribute/record/xxx/123
    let re_resource_record = Regex::new(r"/resource-attribute/record/([a-zA-Z0-9_]+)/(\d+)$").unwrap();
    path = re_resource_record.replace_all(&path, "/resource-attribute/record/{resource_type}/{resource_id}").to_string();

    // /resource-attribute/res/xxx/123
    let re_resource_res = Regex::new(r"/resource-attribute/res/([a-zA-Z0-9_]+)/(\d+)$").unwrap();
    path = re_resource_res.replace_all(&path, "/resource-attribute/res/{resource_type}/{resource_id}").to_string();

    // ==============================================
    // 3. 角色相关
    // ==============================================
    let re_tree_list = Regex::new(r"/role/(tree|list)/([a-zA-Z0-9_]+)$").unwrap();
    path = re_tree_list.replace_all(&path, "/role/$1/{role}").to_string();

    let re_menus = Regex::new(r"/role/menus/([a-zA-Z0-9_]+)$").unwrap();
    path = re_menus.replace_all(&path, "/role/menus/{role}").to_string();

    let re_grant = Regex::new(r"/role/grant/([a-zA-Z0-9_]+)$").unwrap();
    path = re_grant.replace_all(&path, "/role/grant/{role}").to_string();

    let re_replace = Regex::new(r"/role/replace/([a-zA-Z0-9_]+)$").unwrap();
    path = re_replace.replace_all(&path, "/role/replace/{role}").to_string();

    // ==============================================
    // 4. 通用 ID 替换
    // ==============================================
    let re_id = Regex::new(r"/\d+$").unwrap();
    path = re_id.replace_all(&path, "/{id}").to_string();

    path
}