use regex::Regex;
use reqwest::Response;
use thiserror::Error;
use url;

#[derive(Error, Debug)]
pub enum GenericClientError {
    #[error("Bearer token not provided")]
    NoBearer,
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
    #[error("The access token was not provided in the response")]
    AccessTokenNotProvided,
    #[error("The expire time was not provided in the response")]
    ExpireTimeNotProvided,
    #[error("Could not parse the expire time as a number")]
    ExpireTimeParseError,
}

#[derive(Error, Debug)]
pub enum CsrfTokenError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("The element containing the CSRF token could not be found")]
    TokenNotFound,
    #[error("The CSRF token value could not be found on the input element")]
    TokenValueNotFound,
}

#[derive(Error, Debug)]
#[error("Error ({0}): {1} ({2})")]
pub struct JWTError(String, String, String);

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
            .get("www-authenticate")
            .expect("Header not found")
            .to_str()
            .unwrap_or("Failed to convert HeaderValue to string");
        let strings = extract_all_quoted_strings(error_header);
        return Err(JWTError(
            strings[0].clone(),
            strings[1].clone(),
            strings[2].clone(),
        ));
    }
    Ok(response)
}
