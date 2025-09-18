
use crate::model::model::{ApiResponse, ErrorResponse};


#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum UnifiedResponse<T> {
    Success(ApiResponse<T>),
    Error(ErrorResponse)
}

impl 