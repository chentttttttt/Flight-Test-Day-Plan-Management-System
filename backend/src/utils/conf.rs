// 全局通用：清理 Option<String> 空字符串
pub trait Normalize {
    fn normalize(self) -> Self;
}

// 自动把 Some("") 变成 None
impl Normalize for Option<String> {
    fn normalize(self) -> Self {
        self.filter(|s| !s.is_empty())
    }
}