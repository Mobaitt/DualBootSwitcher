use thiserror::Error;

#[derive(Debug, Error)]
pub enum BootError {
    #[error("当前平台暂不支持 UEFI 启动项管理")]
    UnsupportedPlatform,
    #[error("当前系统不是 UEFI 启动模式，BootNext 不可用")]
    NotUefi,
    #[error("权限不足，请以管理员权限运行此操作")]
    PermissionDenied,
    #[error("找不到系统命令：{0}")]
    CommandNotFound(String),
    #[error("系统命令执行失败：{0}")]
    CommandFailed(String),
    #[error("启动项输出解析失败：{0}")]
    ParseError(String),
    #[error("非法启动项 ID：{0}")]
    InvalidBootId(String),
    #[error("启动项不存在：{0}")]
    BootEntryNotFound(String),
    #[error("重启失败：{0}")]
    RebootFailed(String),
    #[error("{0}")]
    Unknown(String),
}

pub type BootResult<T> = Result<T, BootError>;
