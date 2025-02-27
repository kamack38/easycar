use serde::{Deserialize, Serialize};
use std::fmt::Write;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EndpointResponse<T> {
    #[serde(rename = "errors")]
    Errors(Vec<GenericError>),
    #[serde(untagged)]
    Success(T),
}

#[derive(Error, Debug)]
#[error("{}", .0.iter().fold(String::new(), |mut prev, v| {let _ = write!(prev, "{} ({}). ", v.user_message, v.code); prev}))]
pub struct GenericEndpointError(pub Vec<GenericError>);

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
