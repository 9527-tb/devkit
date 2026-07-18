//! ToolchainDetector trait 占位。

pub trait ToolchainDetector: Send + Sync {
    fn id(&self) -> &'static str;
    fn detect(&self) -> Vec<crate::runtime::RuntimeEntry>;
}
