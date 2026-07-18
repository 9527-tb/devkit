//! 领域错误类型占位。

#[allow(dead_code)]
#[derive(Debug)]
pub enum DomainError {
    Message(String),
}

impl From<String> for DomainError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl DomainError {
    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        match self {
            Self::Message(m) => m,
        }
    }
}
