//! 自定义宏：自动关联 SeaORM 表名到 TS 类型名
#[macro_export]
macro_rules! ts_export_with_table_name {
    ($struct_name:ident, $table_name:expr) => {
        #[ts(
            export,
            rename = $table_name,
            rename_all = "camelCase"
        )]
        $struct_name
    };
}