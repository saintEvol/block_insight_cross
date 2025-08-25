use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("客户端还未初始化")]
    ClientNotInitialized,
    #[error("reqwest error: {0}")]
    Reqwest(#[from]reqwest::Error),
    #[error("{0}")]
    LogicError(String),

}