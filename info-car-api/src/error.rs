use std::collections::HashMap;

use regex::Regex;
use reqwest::{header::WWW_AUTHENTICATE, Response};
use thiserror::Error;
use url;

use crate::types::GenericEndpointError;

#[derive(Error, Debug)]
pub enum GenericClientError {
    #[error(transparent)]
    NoBearer(#[from] NoBearerError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JWTError(#[from] JWTError),
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    TokenGetError(#[from] RefreshTokenError),
    #[error(transparent)]
    CsrfTokenError(#[from] CsrfTokenError),
}

#[derive(Error, Debug)]
pub enum LogoutError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    NoBearer(#[from] NoBearerError),
}

#[derive(Error, Debug)]
pub enum RefreshTokenError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("No fragment was provided in the response URL")]
    NoFragmentProvided,
    #[error(transparent)]
    UrlFragmentParseError(#[from] serde_urlencoded::de::Error),
    #[error("The access token was not provided in the response. Response: {0:?}")]
    AccessTokenNotProvided(HashMap<String, String>),
    #[error("The expire time was not provided in the response")]
    ExpireTimeNotProvided,
    #[error("Could not parse the expire time as a number")]
    ExpireTimeParseError,
}

#[derive(Error, Debug)]
pub enum CsrfTokenError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("The element containing the CSRF token could not be found")]
    TokenNotFound,
    #[error("The CSRF token value could not be found on the input element")]
    TokenValueNotFound,
}

#[derive(Error, Debug)]
pub enum EnrollError {
    #[error(transparent)]
    NoBearer(#[from] NoBearerError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JWTError(#[from] JWTError),
    #[error(transparent)]
    GenericEndpointError(#[from] GenericEndpointError),
    #[error("The supplied argument ({0}) is empty")]
    EmptyArg(String),
}

#[derive(Error, Debug)]
#[error("Bearer token not found")]
pub struct NoBearerError;

#[derive(Error, Debug)]
#[error("Error ({}): {} ({})", .0.error_type, .0.description, .0.url)]
pub struct JWTError(JWTErrorMessage);

#[derive(Debug)]
pub struct JWTErrorMessage {
    pub error_type: String,
    pub description: String,
    pub url: String,
}

impl From<Vec<String>> for JWTErrorMessage {
    fn from(mut value: Vec<String>) -> Self {
        if value.len() < 3 {
            return JWTErrorMessage {
                error_type: "unknown_error".to_owned(),
                description: value.join(" "),
                url: "".to_owned(),
            };
        };

        // Pop for performance
        JWTErrorMessage {
            url: value.pop().unwrap(),
            description: value.pop().unwrap(),
            error_type: value.pop().unwrap(),
        }
    }
}

fn extract_all_quoted_strings(input: &str) -> Vec<String> {
    let re = Regex::new(r#""(.*?)""#).unwrap();

    re.captures_iter(input)
        .map(|cap| cap[1].to_string()) // Extract the matched group inside the quotes
        .collect()
}

pub fn handle_response(response: Response) -> Result<Response, JWTError> {
    if response.status() == 401 {
        let error_header = response
            .headers()
            .get(WWW_AUTHENTICATE)
            .expect("Header not found")
            .to_str()
            .unwrap_or("Failed to convert HeaderValue to string");
        let strings = extract_all_quoted_strings(error_header);
        return Err(JWTError(strings.into()));
    }
    Ok(response)
}
