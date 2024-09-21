use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EndpointResponse<T> {
    #[serde(rename = "errors")]
    Errors(Vec<GenericError>),
    #[serde(untagged)]
    Success(T),
}

#[derive(Error, Debug)]
#[error("{}", .0.iter().map(|v| format!("{} ({}). ", v.user_message, v.code)).collect::<String>())]
pub struct GenericEndpointError(Vec<GenericError>);

impl<T> EndpointResponse<T> {
    pub fn ok(self) -> Result<T, GenericEndpointError> {
        use EndpointResponse::*;
        match self {
            Success(v) => Ok(v),
            Errors(errs) => Err(GenericEndpointError(errs)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericError {
    pub code: String,
    pub path: Option<String>,
    pub user_message: String,
    pub timestamp: String,
}
