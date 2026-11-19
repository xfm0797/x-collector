use std::fmt::Display;

/// 将内部错误转换为前端可读字符串
pub fn to_err(e: impl Display) -> String {
    e.to_string()
}
